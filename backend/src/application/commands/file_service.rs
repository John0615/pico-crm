use crate::domain::gateways::file_storage::FileStorageGateway;
use crate::domain::models::file::{
    FileDeleteRequest, FileDownloadRequest, FileDownloadResponse, FileListRequest,
    FileListResponse, FileUploadRequest, FileUploadResponse, FileValidationRules,
};
use std::sync::Arc;

// 存储桶常量
const BUCKET_NAME: &str = "pico-crm";

pub struct FileCommandService {
    file_gateway: Arc<dyn FileStorageGateway>,
}

impl FileCommandService {
    pub fn new(file_gateway: Arc<dyn FileStorageGateway>) -> Self {
        Self { file_gateway }
    }

    /// 内部上传文件方法
    async fn upload_file_internal(
        &self,
        key: String,
        file_name: String,
        data: Vec<u8>,
        content_type: Option<String>,
    ) -> Result<FileUploadResponse, String> {
        let request = FileUploadRequest {
            bucket: BUCKET_NAME.to_string(),
            key,
            file_name,
            data,
            content_type,
        };

        self.file_gateway.upload_file(request).await
    }

    /// 上传用户头像
    pub async fn upload_avatar(
        &self,
        user_id: &str,
        file_name: &str,
        data: Vec<u8>,
        file_extension: &str,
    ) -> Result<FileUploadResponse, String> {
        // 验证文件
        let validation_rules = FileValidationRules::for_images();
        let content_type = Self::get_content_type_by_extension(file_extension)
            .unwrap_or_else(|| "application/octet-stream".to_string());

        validation_rules.validate(file_name, &content_type, data.len() as u64)?;

        let key = format!("avatars/{}.{}", user_id, file_extension);
        let content_type = match file_extension {
            "jpg" | "jpeg" => Some("image/jpeg".to_string()),
            "png" => Some("image/png".to_string()),
            "gif" => Some("image/gif".to_string()),
            "webp" => Some("image/webp".to_string()),
            _ => None,
        };

        self.upload_file_internal(key, file_name.to_string(), data, content_type)
            .await
    }

    /// 上传文档文件
    pub async fn upload_document(
        &self,
        document_id: &str,
        filename: &str,
        data: Vec<u8>,
    ) -> Result<FileUploadResponse, String> {
        // 验证文件
        let validation_rules = FileValidationRules::for_documents();
        let content_type = Self::get_content_type_by_filename(filename)
            .unwrap_or_else(|| "application/octet-stream".to_string());

        validation_rules.validate(filename, &content_type, data.len() as u64)?;

        let key = format!("documents/{}/{}", document_id, filename);
        let content_type = Self::get_content_type_by_filename(filename);

        self.upload_file_internal(key, filename.to_string(), data, content_type)
            .await
    }

    /// 上传临时文件
    pub async fn upload_temp_file(
        &self,
        temp_id: &str,
        filename: &str,
        data: Vec<u8>,
    ) -> Result<FileUploadResponse, String> {
        // 对临时文件使用图片验证规则（因为主要用于头像上传）
        let validation_rules = FileValidationRules::for_images();
        let content_type = Self::get_content_type_by_filename(filename)
            .unwrap_or_else(|| "application/octet-stream".to_string());

        validation_rules.validate(filename, &content_type, data.len() as u64)?;

        let key = format!("temp/{}/{}", temp_id, filename);
        let content_type = Self::get_content_type_by_filename(filename);

        self.upload_file_internal(key, filename.to_string(), data, content_type)
            .await
    }

    /// 从文件路径上传头像
    pub async fn upload_avatar_from_path(
        &self,
        user_id: &str,
        file_path: &std::path::Path,
    ) -> Result<FileUploadResponse, String> {
        // 读取文件数据
        let data = std::fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        // 获取文件名和扩展名
        let file_name = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("avatar")
            .to_string();

        let file_extension = file_path
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("png");

        self.upload_avatar(user_id, &file_name, data, file_extension)
            .await
    }

    /// 从文件路径上传文档
    pub async fn upload_document_from_path(
        &self,
        document_id: &str,
        file_path: &std::path::Path,
    ) -> Result<FileUploadResponse, String> {
        // 读取文件数据
        let data = std::fs::read(file_path).map_err(|e| format!("Failed to read file: {}", e))?;

        // 获取文件名
        let filename = file_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("document")
            .to_string();

        self.upload_document(document_id, &filename, data).await
    }

    /// 下载用户头像
    pub async fn download_avatar(
        &self,
        user_id: &str,
        file_extension: &str,
    ) -> Result<FileDownloadResponse, String> {
        let key = format!("avatars/{}.{}", user_id, file_extension);
        let request = FileDownloadRequest {
            bucket: BUCKET_NAME.to_string(),
            key,
        };

        self.file_gateway.download_file(request).await
    }

    /// 下载文档
    pub async fn download_document(
        &self,
        document_id: &str,
        filename: &str,
    ) -> Result<FileDownloadResponse, String> {
        let key = format!("documents/{}/{}", document_id, filename);
        let request = FileDownloadRequest {
            bucket: BUCKET_NAME.to_string(),
            key,
        };

        self.file_gateway.download_file(request).await
    }

    /// 删除用户头像
    pub async fn delete_avatar(&self, user_id: &str, file_extension: &str) -> Result<(), String> {
        let key = format!("avatars/{}.{}", user_id, file_extension);
        let request = FileDeleteRequest {
            bucket: BUCKET_NAME.to_string(),
            key,
        };

        self.file_gateway.delete_file(request).await
    }

    /// 删除文档
    pub async fn delete_document(&self, document_id: &str, filename: &str) -> Result<(), String> {
        let key = format!("documents/{}/{}", document_id, filename);
        let request = FileDeleteRequest {
            bucket: BUCKET_NAME.to_string(),
            key,
        };

        self.file_gateway.delete_file(request).await
    }

    /// 列出用户的所有头像
    pub async fn list_user_avatars(&self, user_id: &str) -> Result<FileListResponse, String> {
        let prefix = Some(format!("avatars/{}.", user_id));
        let request = FileListRequest {
            bucket: BUCKET_NAME.to_string(),
            prefix,
        };

        self.file_gateway.list_files(request).await
    }

    /// 列出文档
    pub async fn list_documents(
        &self,
        document_id: Option<&str>,
    ) -> Result<FileListResponse, String> {
        let prefix = match document_id {
            Some(id) => Some(format!("documents/{}/", id)),
            None => Some("documents/".to_string()),
        };
        let request = FileListRequest {
            bucket: BUCKET_NAME.to_string(),
            prefix,
        };

        self.file_gateway.list_files(request).await
    }

    /// 列出临时文件
    pub async fn list_temp_files(&self, temp_id: Option<&str>) -> Result<FileListResponse, String> {
        let prefix = match temp_id {
            Some(id) => Some(format!("temp/{}/", id)),
            None => Some("temp/".to_string()),
        };
        let request = FileListRequest {
            bucket: BUCKET_NAME.to_string(),
            prefix,
        };

        self.file_gateway.list_files(request).await
    }

    /// 根据文件名获取 Content-Type
    fn get_content_type_by_filename(filename: &str) -> Option<String> {
        if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
            Some("image/jpeg".to_string())
        } else if filename.ends_with(".png") {
            Some("image/png".to_string())
        } else if filename.ends_with(".gif") {
            Some("image/gif".to_string())
        } else if filename.ends_with(".webp") {
            Some("image/webp".to_string())
        } else if filename.ends_with(".pdf") {
            Some("application/pdf".to_string())
        } else if filename.ends_with(".doc") {
            Some("application/msword".to_string())
        } else if filename.ends_with(".docx") {
            Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .to_string(),
            )
        } else if filename.ends_with(".xls") {
            Some("application/vnd.ms-excel".to_string())
        } else if filename.ends_with(".xlsx") {
            Some("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string())
        } else if filename.ends_with(".txt") {
            Some("text/plain".to_string())
        } else if filename.ends_with(".json") {
            Some("application/json".to_string())
        } else if filename.ends_with(".xml") {
            Some("application/xml".to_string())
        } else if filename.ends_with(".html") {
            Some("text/html".to_string())
        } else if filename.ends_with(".css") {
            Some("text/css".to_string())
        } else if filename.ends_with(".js") {
            Some("application/javascript".to_string())
        } else {
            Some("application/octet-stream".to_string())
        }
    }

    /// 根据文件扩展名获取 Content-Type
    fn get_content_type_by_extension(extension: &str) -> Option<String> {
        match extension.to_lowercase().as_str() {
            "jpg" | "jpeg" => Some("image/jpeg".to_string()),
            "png" => Some("image/png".to_string()),
            "gif" => Some("image/gif".to_string()),
            "webp" => Some("image/webp".to_string()),
            "pdf" => Some("application/pdf".to_string()),
            "doc" => Some("application/msword".to_string()),
            "docx" => Some(
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .to_string(),
            ),
            "xls" => Some("application/vnd.ms-excel".to_string()),
            "xlsx" => Some(
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
            ),
            "txt" => Some("text/plain".to_string()),
            "json" => Some("application/json".to_string()),
            "xml" => Some("application/xml".to_string()),
            "html" => Some("text/html".to_string()),
            "css" => Some("text/css".to_string()),
            "js" => Some("application/javascript".to_string()),
            _ => Some("application/octet-stream".to_string()),
        }
    }

    // 管理员方法 - 用于存储桶管理
    /// 创建存储桶（管理员功能）
    pub async fn create_bucket(&self) -> Result<(), String> {
        self.file_gateway.create_bucket(BUCKET_NAME).await
    }

    /// 删除存储桶（管理员功能）
    pub async fn delete_bucket(&self) -> Result<(), String> {
        self.file_gateway.delete_bucket(BUCKET_NAME).await
    }

    /// 列出存储桶（管理员功能）
    pub async fn list_buckets(&self) -> Result<Vec<String>, String> {
        self.file_gateway.list_buckets().await
    }

    /// 获取存储桶名称
    pub fn get_bucket_name() -> &'static str {
        BUCKET_NAME
    }
}
