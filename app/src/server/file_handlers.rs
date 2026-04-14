use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::file::{FileDeleteRequest, FileInfoRequest, FileUploadRequest, FileUploadResponse};

/// 上传文件到服务器
#[server(
    name = UploadFile,
    prefix = "/api",
    endpoint = "/upload_file",
)]
pub async fn upload_file(request: FileUploadRequest) -> Result<FileUploadResponse, ServerFnError> {
    use backend::application::commands::shared::file_service::FileCommandService;
    use backend::infrastructure::gateways::rustfs_gateway::RustFSGateway;
    use leptos::prelude::*;
    use std::sync::Arc;

    leptos::logging::log!("收到文件上传请求: {:?}", request.file_name);

    let file_name = request.file_name;
    let file_data = request.file_data;
    let content_type = request.content_type;

    // 验证请求数据
    if file_name.is_empty() {
        leptos::logging::error!("文件名为空");
        return Err(ServerFnError::Args("文件名不能为空".to_string()));
    }

    if file_data.is_empty() {
        leptos::logging::error!("文件数据为空");
        return Err(ServerFnError::Args("文件数据不能为空".to_string()));
    }

    leptos::logging::log!(
        "开始上传文件: {} ({}字节, {})",
        file_name,
        file_data.len(),
        content_type
    );

    // 验证文件类型（仅允许图片）
    if !content_type.starts_with("image/") {
        leptos::logging::error!("不支持的文件类型: {}", content_type);
        return Err(ServerFnError::Args("只允许上传图片文件".to_string()));
    }

    // 验证文件大小（最大5MB）
    const MAX_FILE_SIZE: usize = 5 * 1024 * 1024; // 5MB
    if file_data.len() > MAX_FILE_SIZE {
        leptos::logging::error!("文件大小超限: {} > {}", file_data.len(), MAX_FILE_SIZE);
        return Err(ServerFnError::Args("文件大小不能超过5MB".to_string()));
    }

    // 创建 RustFS Gateway
    let gateway = match RustFSGateway::from_env().await {
        Ok(gateway) => Arc::new(gateway),
        Err(e) => {
            leptos::logging::error!("创建 RustFS Gateway 失败: {}", e);
            return Err(ServerFnError::ServerError(format!(
                "文件存储服务初始化失败: {}",
                e
            )));
        }
    };

    // 创建文件服务
    let file_service = FileCommandService::new(gateway);

    // 生成临时ID用于上传
    let temp_id = uuid::Uuid::new_v4().to_string();

    // 使用文件服务上传临时文件
    match file_service
        .upload_temp_file(&temp_id, &file_name, file_data)
        .await
    {
        Ok(response) => {
            leptos::logging::log!("文件上传成功: {}", response.file_url);

            // 转换为前端期望的响应格式
            Ok(FileUploadResponse {
                file_url: response.file_url,
                file_name: response.file_name,
                file_size: response.file_size,
            })
        }
        Err(e) => {
            leptos::logging::error!("文件上传失败: {}", e);
            Err(ServerFnError::ServerError(format!("文件上传失败: {}", e)))
        }
    }
}

/// 删除文件
#[server(
    name = DeleteFile,
    prefix = "/api",
    endpoint = "/delete_file",
)]
pub async fn delete_file(request: FileDeleteRequest) -> Result<(), ServerFnError> {
    use std::fs;

    // 从URL提取文件路径
    let file_path = if request.file_url.starts_with("/uploads/") {
        format!("public{}", request.file_url)
    } else {
        return Err(ServerFnError::new("无效的文件URL".to_string()));
    };

    // 删除文件
    match fs::remove_file(&file_path) {
        Ok(_) => {
            leptos::logging::log!("文件删除成功: {}", file_path);
            Ok(())
        }
        Err(e) => {
            leptos::logging::error!("删除文件失败: {} - {}", file_path, e);
            Err(ServerFnError::new("删除文件失败".to_string()))
        }
    }
}

/// 获取文件信息
#[server(
    name = GetFileInfo,
    prefix = "/api",
    endpoint = "/get_file_info",
)]
pub async fn get_file_info(
    request: FileInfoRequest,
) -> Result<Option<FileUploadResponse>, ServerFnError> {
    use std::fs;
    use std::path::Path;

    // 从URL提取文件路径
    let file_path = if request.file_url.starts_with("/uploads/") {
        format!("public{}", request.file_url)
    } else {
        return Err(ServerFnError::new("无效的文件URL".to_string()));
    };

    // 检查文件是否存在
    match fs::metadata(&file_path) {
        Ok(metadata) => {
            let file_name = Path::new(&file_path)
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown")
                .to_string();

            Ok(Some(FileUploadResponse {
                file_url: request.file_url,
                file_name,
                file_size: metadata.len(),
            }))
        }
        Err(_) => Ok(None),
    }
}
