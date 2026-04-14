use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct File {
    pub id: String,
    pub original_name: String,
    pub storage_path: String,
    pub access_url: String,
    pub size: u64,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl File {
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

    pub fn update(&mut self, size: u64, content_type: Option<String>, etag: Option<String>) {
        self.size = size;
        self.content_type = content_type;
        self.etag = etag;
        self.updated_at = chrono::Utc::now();
    }

    pub fn is_image(&self) -> bool {
        self.content_type
            .as_ref()
            .map(|ct| ct.starts_with("image/"))
            .unwrap_or(false)
    }

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

    pub fn get_extension(&self) -> Option<String> {
        std::path::Path::new(&self.original_name)
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_lowercase())
    }
}

#[derive(Debug, Clone)]
pub struct FileUploadRequest {
    pub bucket: String,
    pub key: String,
    pub file_name: String,
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileUploadResponse {
    pub file_path: String,
    pub file_url: String,
    pub file_name: String,
    pub file_size: u64,
    pub etag: Option<String>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileDownloadRequest {
    pub bucket: String,
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct FileDownloadResponse {
    pub data: Vec<u8>,
    pub content_type: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileDeleteRequest {
    pub bucket: String,
    pub key: String,
}

#[derive(Debug, Clone)]
pub struct FileListRequest {
    pub bucket: String,
    pub prefix: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub key: String,
    pub size: Option<i64>,
    pub last_modified: Option<chrono::DateTime<chrono::Utc>>,
    pub etag: Option<String>,
}

#[derive(Debug, Clone)]
pub struct FileListResponse {
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileType {
    Avatar,
    Document,
    Temporary,
    Other,
}

impl FileType {
    pub fn get_path_prefix(&self) -> &'static str {
        match self {
            FileType::Avatar => "avatars",
            FileType::Document => "documents",
            FileType::Temporary => "temp",
            FileType::Other => "files",
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileValidationRules {
    pub max_size: u64,
    pub allowed_mime_types: Vec<String>,
    pub allowed_extensions: Vec<String>,
}

impl FileValidationRules {
    pub fn for_images() -> Self {
        Self {
            max_size: 5 * 1024 * 1024,
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

    pub fn for_documents() -> Self {
        Self {
            max_size: 10 * 1024 * 1024,
            allowed_mime_types: vec![
                "application/pdf".to_string(),
                "application/msword".to_string(),
                "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    .to_string(),
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

    pub fn validate(&self, file_name: &str, content_type: &str, size: u64) -> Result<(), String> {
        if size > self.max_size {
            return Err(format!(
                "文件大小超过限制: {} > {} bytes",
                size, self.max_size
            ));
        }

        if !self.allowed_mime_types.is_empty()
            && !self.allowed_mime_types.contains(&content_type.to_string())
        {
            return Err(format!("不支持的文件类型: {}", content_type));
        }

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
