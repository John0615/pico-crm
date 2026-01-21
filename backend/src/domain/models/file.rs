use serde::{Deserialize, Serialize};

/// 文件实体
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct File {
    /// 文件ID
    pub id: String,
    /// 原始文件名
    pub original_name: String,
    /// 存储路径（bucket/key）
    pub storage_path: String,
    /// 访问URL
    pub access_url: String,
    /// 文件大小（字节）
    pub size: u64,
    /// MIME类型
    pub content_type: Option<String>,
    /// ETag（用于完整性验证）
    pub etag: Option<String>,
    /// 创建时间
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// 更新时间
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl File {
    /// 创建新文件实体
    pub fn new(
        id: String,
        original_name: String,
        storage_path: String,
        access_url: String,
        size: u64,
        content_type: Option<String>,
        etag: Option<String>,
    ) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            original_name,
            storage_path,
            access_url,
            size,
            content_type,
            etag,
            created_at: now,
            updated_at: now,
        }
    }

    /// 更新文件信息
    pub fn update(&mut self, size: u64, content_type: Option<String>, etag: Option<String>) {
        self.size = size;
        self.content_type = content_type;
        self.etag = etag;
        self.updated_at = chrono::Utc::now();
    }

    /// 检查文件是否为图片
    pub fn is_image(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.starts_with("image/"))
            .unwrap_or(false)
    }

    /// 检查文件是否为文档
    pub fn is_document(&self) -> bool {
        if let Some(content_type) = &self.content_type {
            matches!(
                content_type.as_str(),
                "application/pdf"
                    | "application/msword"
                    | "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    | "application/vnd.ms-excel"
                    | "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                    | "text/plain"
            )
        } else {
            false
        }
    }

    /// 获取文件扩展名
    pub fn get_extension(&self) -> Option<String> {
        std::path::Path::new(&self.original_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
    }
}

/// 文件上传请求
#[derive(Debug, Clone)]
pub struct FileUploadRequest {
    pub bucket: String,
    pub key: String,
    pub file_name: String,
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}

/// 文件上传响应
#[derive(Debug, Clone)]
pub struct FileUploadResponse {
    pub file_path: String,
    pub file_url: String,
    pub file_name: String,
    pub file_size: u64,
    pub etag: Option<String>,
    pub content_type: Option<String>,
}

/// 文件下载请求
#[derive(Debug, Clone)]
pub struct FileDownloadRequest {
    pub bucket: String,
    pub key: String,
}

/// 文件下载响应
#[derive(Debug, Clone)]
pub struct FileDownloadResponse {
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}

/// 文件删除请求
#[derive(Debug, Clone)]
pub struct FileDeleteRequest {
    pub bucket: String,
    pub key: String,
}

/// 文件列表请求
#[derive(Debug, Clone)]
pub struct FileListRequest {
    pub bucket: String,
    pub prefix: Option<String>,
}

/// 文件信息
#[derive(Debug, Clone)]
pub struct FileInfo {
    pub key: String,
    pub size: Option<i64>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub etag: Option<String>,
}

/// 文件列表响应
#[derive(Debug, Clone)]
pub struct FileListResponse {
    pub files: Vec<FileInfo>,
}

/// 文件类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    /// 头像
    Avatar,
    /// 文档
    Document,
    /// 临时文件
    Temporary,
    /// 其他
    Other,
}

impl FileType {
    /// 获取存储路径前缀
    pub fn get_path_prefix(&self) -> &'static str {
        match self {
            FileType::Avatar => "avatars",
            FileType::Document => "documents",
            FileType::Temporary => "temp",
            FileType::Other => "files",
        }
    }
}

/// 文件验证规则
#[derive(Debug, Clone)]
pub struct FileValidationRules {
    /// 最大文件大小（字节）
    pub max_size: u64,
    /// 允许的MIME类型
    pub allowed_mime_types: Vec<String>,
    /// 允许的文件扩展名
    pub allowed_extensions: Vec<String>,
}

impl FileValidationRules {
    /// 创建图片文件验证规则
    pub fn for_images() -> Self {
        Self {
            max_size: 5 * 1024 * 1024, // 5MB
            allowed_mime_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "image/gif".to_string(),
                "image/webp".to_string(),
            ],
            allowed_extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "webp".to_string(),
            ],
        }
    }

    /// 创建文档文件验证规则
    pub fn for_documents() -> Self {
        Self {
            max_size: 10 * 1024 * 1024, // 10MB
            allowed_mime_types: vec![
                "application/pdf".to_string(),
                "application/msword".to_string(),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document".to_string(),
                "application/vnd.ms-excel".to_string(),
                "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string(),
                "text/plain".to_string(),
            ],
            allowed_extensions: vec![
                "pdf".to_string(),
                "doc".to_string(),
                "docx".to_string(),
                "xls".to_string(),
                "xlsx".to_string(),
                "txt".to_string(),
            ],
        }
    }

    /// 验证文件
    pub fn validate(&self, file_name: &str, content_type: &str, size: u64) -> Result<(), String> {
        // 检查文件大小
        if size > self.max_size {
            return Err(format!(
                "文件大小超过限制: {} > {} bytes",
                size, self.max_size
            ));
        }

        // 检查MIME类型
        if !self.allowed_mime_types.is_empty() && !self.allowed_mime_types.contains(&content_type.to_string()) {
            return Err(format!("不支持的文件类型: {}", content_type));
        }

        // 检查文件扩展名
        if !self.allowed_extensions.is_empty() {
            let extension = std::path::Path::new(file_name)
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|s| s.to_lowercase())
                .unwrap_or_default();

            if !self.allowed_extensions.contains(&extension) {
                return Err(format!("不支持的文件扩展名: {}", extension));
            }
        }

        Ok(())
    }
}