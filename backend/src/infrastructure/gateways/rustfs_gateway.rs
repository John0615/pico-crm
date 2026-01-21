use crate::domain::gateways::file_storage::FileStorageGateway;
use crate::domain::models::file::{
    FileUploadRequest, FileUploadResponse, FileDownloadRequest, FileDownloadResponse,
    FileDeleteRequest, FileListRequest, FileListResponse, FileInfo
};
use async_trait::async_trait;
use aws_config::BehaviorVersion;
use aws_credential_types::Credentials;
use aws_sdk_s3::{primitives::ByteStream, config::Region, Client};
use std::env;

#[derive(Debug, Clone)]
pub struct RustFSConfig {
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub endpoint_url: String,
}

impl RustFSConfig {
    pub fn from_env() -> Result<Self, String> {
        let region = env::var("RUSTFS_REGION")
            .map_err(|_| "RUSTFS_REGION environment variable not set".to_string())?;
        let access_key_id = env::var("RUSTFS_ACCESS_KEY_ID")
            .map_err(|_| "RUSTFS_ACCESS_KEY_ID environment variable not set".to_string())?;
        let secret_access_key = env::var("RUSTFS_SECRET_ACCESS_KEY")
            .map_err(|_| "RUSTFS_SECRET_ACCESS_KEY environment variable not set".to_string())?;
        let endpoint_url = env::var("RUSTFS_ENDPOINT_URL")
            .map_err(|_| "RUSTFS_ENDPOINT_URL environment variable not set".to_string())?;

        Ok(RustFSConfig {
            region,
            access_key_id,
            secret_access_key,
            endpoint_url,
        })
    }
}

pub struct RustFSGateway {
    client: Client,
}

impl RustFSGateway {
    pub async fn new(config: RustFSConfig) -> Result<Self, String> {
        let credentials = Credentials::new(
            config.access_key_id,
            config.secret_access_key,
            None,
            None,
            "rustfs",
        );

        let region = Region::new(config.region);
        let endpoint_url = config.endpoint_url;

        let shared_config = aws_config::defaults(BehaviorVersion::latest())
            .region(region)
            .credentials_provider(credentials)
            .endpoint_url(endpoint_url)
            .load()
            .await;

        let client = Client::new(&shared_config);

        Ok(RustFSGateway { client })
    }

    pub async fn from_env() -> Result<Self, String> {
        let config = RustFSConfig::from_env()?;
        Self::new(config).await
    }
}

#[async_trait]
impl FileStorageGateway for RustFSGateway {
    async fn upload_file(&self, request: FileUploadRequest) -> Result<FileUploadResponse, String> {
        let file_size = request.data.len() as u64;
        
        let mut put_object = self
            .client
            .put_object()
            .bucket(&request.bucket)
            .key(&request.key)
            .body(ByteStream::from(request.data));

        if let Some(content_type) = &request.content_type {
            put_object = put_object.content_type(content_type);
        }

        match put_object.send().await {
            Ok(res) => {
                let file_path = format!("{}/{}", request.bucket, request.key);
                let file_url = format!("{}/{}/{}", 
                    env::var("RUSTFS_ENDPOINT_URL").unwrap_or_default(),
                    request.bucket,
                    request.key
                );
                
                Ok(FileUploadResponse {
                    file_path,
                    file_url,
                    file_name: request.file_name,
                    file_size,
                    etag: res.e_tag().map(|s| s.to_string()),
                    content_type: request.content_type,
                })
            }
            Err(e) => {
                eprintln!("Error uploading file: {:?}", e);
                Err(format!("Failed to upload file: {}", e))
            }
        }
    }

    async fn download_file(&self, request: FileDownloadRequest) -> Result<FileDownloadResponse, String> {
        match self
            .client
            .get_object()
            .bucket(&request.bucket)
            .key(&request.key)
            .send()
            .await
        {
            Ok(res) => {
                let content_type = res.content_type().map(|s| s.to_string());
                let data = res
                    .body
                    .collect()
                    .await
                    .map_err(|e| format!("Failed to read file data: {}", e))?
                    .into_bytes()
                    .to_vec();

                Ok(FileDownloadResponse {
                    data,
                    content_type,
                })
            }
            Err(e) => {
                eprintln!("Error downloading file: {:?}", e);
                Err(format!("Failed to download file: {}", e))
            }
        }
    }

    async fn delete_file(&self, request: FileDeleteRequest) -> Result<(), String> {
        match self
            .client
            .delete_object()
            .bucket(&request.bucket)
            .key(&request.key)
            .send()
            .await
        {
            Ok(_) => {
                println!("File deleted successfully: {}/{}", request.bucket, request.key);
                Ok(())
            }
            Err(e) => {
                eprintln!("Error deleting file: {:?}", e);
                Err(format!("Failed to delete file: {}", e))
            }
        }
    }

    async fn list_files(&self, request: FileListRequest) -> Result<FileListResponse, String> {
        let mut list_objects = self
            .client
            .list_objects_v2()
            .bucket(&request.bucket);

        if let Some(prefix) = request.prefix {
            list_objects = list_objects.prefix(prefix);
        }

        match list_objects.send().await {
            Ok(res) => {
                let files: Vec<FileInfo> = res
                    .contents()
                    .iter()
                    .map(|object| FileInfo {
                        key: object.key().unwrap_or_default().to_string(),
                        size: object.size(),
                        last_modified: object.last_modified().map(|dt| {
                            chrono::DateTime::from_timestamp(dt.secs(), dt.subsec_nanos())
                                .unwrap_or_default()
                        }),
                        etag: object.e_tag().map(|s| s.to_string()),
                    })
                    .collect();

                println!("Total files found: {}", files.len());
                Ok(FileListResponse { files })
            }
            Err(e) => {
                eprintln!("Error listing files: {:?}", e);
                Err(format!("Failed to list files: {}", e))
            }
        }
    }

    async fn create_bucket(&self, bucket: &str) -> Result<(), String> {
        match self
            .client
            .create_bucket()
            .bucket(bucket)
            .send()
            .await
        {
            Ok(_) => {
                println!("Bucket created successfully: {}", bucket);
                Ok(())
            }
            Err(e) => {
                eprintln!("Error creating bucket: {:?}", e);
                Err(format!("Failed to create bucket: {}", e))
            }
        }
    }

    async fn delete_bucket(&self, bucket: &str) -> Result<(), String> {
        match self
            .client
            .delete_bucket()
            .bucket(bucket)
            .send()
            .await
        {
            Ok(_) => {
                println!("Bucket deleted successfully: {}", bucket);
                Ok(())
            }
            Err(e) => {
                eprintln!("Error deleting bucket: {:?}", e);
                Err(format!("Failed to delete bucket: {}", e))
            }
        }
    }

    async fn list_buckets(&self) -> Result<Vec<String>, String> {
        match self.client.list_buckets().send().await {
            Ok(res) => {
                let buckets = res
                    .buckets()
                    .iter()
                    .filter_map(|bucket| bucket.name().map(|s| s.to_string()))
                    .collect();

                println!("Total buckets found: {}", res.buckets().len());
                Ok(buckets)
            }
            Err(e) => {
                eprintln!("Error listing buckets: {:?}", e);
                Err(format!("Failed to list buckets: {}", e))
            }
        }
    }
}