use serde::{Deserialize, Serialize};

/// 文件上传请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileUploadRequest {
    pub file_name: String,
    pub file_data: Vec<u8>, // 直接使用二进制数据
    pub content_type: String,
}

/// 文件上传响应
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileUploadResponse {
    pub file_url: String,
    pub file_name: String,
    pub file_size: u64,
}

/// 文件删除请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileDeleteRequest {
    pub file_url: String,
}

/// 文件信息查询请求
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileInfoRequest {
    pub file_url: String,
}
