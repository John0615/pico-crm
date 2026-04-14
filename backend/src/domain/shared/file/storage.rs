use super::model::{
    FileDeleteRequest, FileDownloadRequest, FileDownloadResponse, FileListRequest,
    FileListResponse, FileUploadRequest, FileUploadResponse,
};
use async_trait::async_trait;

#[async_trait]
pub trait FileStorageGateway: Send + Sync {
    async fn upload_file(&self, request: FileUploadRequest) -> Result<FileUploadResponse, String>;

    async fn download_file(
        &self,
        request: FileDownloadRequest,
    ) -> Result<FileDownloadResponse, String>;

    async fn delete_file(&self, request: FileDeleteRequest) -> Result<(), String>;

    async fn list_files(&self, request: FileListRequest) -> Result<FileListResponse, String>;

    async fn create_bucket(&self, bucket: &str) -> Result<(), String>;

    async fn delete_bucket(&self, bucket: &str) -> Result<(), String>;

    async fn list_buckets(&self) -> Result<Vec<String>, String>;
}
