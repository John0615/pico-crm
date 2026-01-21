use async_trait::async_trait;
use crate::domain::models::file::{
    FileUploadRequest, FileUploadResponse, FileDownloadRequest, FileDownloadResponse,
    FileDeleteRequest, FileListRequest, FileListResponse
};

/// 文件存储网关接口
#[async_trait]
pub trait FileStorageGateway: Send + Sync {
    /// 上传文件
    async fn upload_file(&self, request: FileUploadRequest) -> Result<FileUploadResponse, String>;
    
    /// 下载文件
    async fn download_file(&self, request: FileDownloadRequest) -> Result<FileDownloadResponse, String>;
    
    /// 删除文件
    async fn delete_file(&self, request: FileDeleteRequest) -> Result<(), String>;
    
    /// 列出文件
    async fn list_files(&self, request: FileListRequest) -> Result<FileListResponse, String>;
    
    /// 创建存储桶
    async fn create_bucket(&self, bucket: &str) -> Result<(), String>;
    
    /// 删除存储桶
    async fn delete_bucket(&self, bucket: &str) -> Result<(), String>;
    
    /// 列出存储桶
    async fn list_buckets(&self) -> Result<Vec<String>, String>;
}