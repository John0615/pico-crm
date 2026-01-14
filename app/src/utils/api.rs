use crate::components::ui::toast::error as show_error_toast;
use leptos::logging;
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;
use std::future::Future;

/// API 调用统一封装函数
///
/// 为所有 server function 调用提供统一的错误处理、日志记录和用户提示
///
/// # 参数
/// * `server_fn_call` - server function 的 Future
///
/// # 返回
/// * `Ok(T)` - 成功时返回数据
/// * `Err(String)` - 失败时返回格式化的错误信息
///
/// # 示例
/// ```rust
/// let result = call_api(fetch_contacts(params)).await;
/// match result {
///     Ok(data) => {
///         // 处理数据
///     }
///     Err(err_msg) => {
///         // 错误已自动处理（日志、提示），这里可做额外处理
///     }
/// }
/// ```
pub async fn call_api<T, F>(server_fn_call: F) -> Result<T, String>
where
    F: Future<Output = Result<T, ServerFnError>>,
{
    match server_fn_call.await {
        Ok(data) => Ok(data),
        Err(e) => {
            logging::log!("API Error: {:?}", e);
            handle_api_error(&e);
            Err(format_api_error(&e))
        }
    }
}

/// 错误处理函数
///
/// 负责记录日志、显示用户提示和处理未授权跳转
fn handle_api_error(error: &ServerFnError) {
    // 1. 记录详细的错误日志
    logging::error!("API Error: {:?}", error);

    // 2. 检查是否是 401 未授权错误
    if is_unauthorized_error(error) {
        // 跳转到登录页
        let navigate = use_navigate();
        navigate("/login", Default::default());
        return;
    }

    // 3. 显示用户友好的提示
    let message = get_user_friendly_message(error);
    show_error_toast(message.to_string());
}

/// 判断是否是 401 未授权错误
fn is_unauthorized_error(error: &ServerFnError) -> bool {
    logging::log!("Checking if error is unauthorized: {:?}", error);
    match error {
        // MiddlewareError 专门用于中间件层面的错误，包括认证失败
        ServerFnError::MiddlewareError(_) => true,
        ServerFnError::ServerError(msg) => {
            // 检查错误信息中是否包含 401 或 unauthorized 关键字
            msg.contains("401") || msg.to_lowercase().contains("unauthorized")
        }
        ServerFnError::Deserialization(msg) => {
            // 反序列化错误也可能是 401 响应导致的
            msg.contains("401") || msg.to_lowercase().contains("unauthorized")
        }
        ServerFnError::Request(msg) => {
            // 请求错误也检查 401
            msg.contains("401") || msg.to_lowercase().contains("unauthorized")
        }
        _ => false,
    }
}

/// 获取用户友好的错误提示信息
fn get_user_friendly_message(error: &ServerFnError) -> &str {
    match error {
        ServerFnError::MiddlewareError(_) => "认证失败，请重新登录",
        ServerFnError::Request(_) => "网络连接失败，请检查网络设置",
        ServerFnError::ServerError(msg) => {
            // 如果是自定义的服务器错误，直接显示
            if !msg.is_empty() {
                return "操作失败";
            }
            "服务器暂时不可用，请稍后重试"
        }
        ServerFnError::Deserialization(_) => "数据格式错误，请稍后重试",
        ServerFnError::Serialization(_) => "数据序列化失败",
        _ => "操作失败，请重试",
    }
}

/// 格式化 API 错误为字符串
///
/// 用于开发调试或详细错误记录
fn format_api_error(error: &ServerFnError) -> String {
    let error_str = error.to_string();

    // 清理错误信息，移除 "error running server function: " 前缀
    if error_str.starts_with("error running server function: ") {
        error_str.replace("error running server function: ", "")
    } else {
        match error {
            ServerFnError::MiddlewareError(msg) => format!("认证错误: {}", msg),
            ServerFnError::Request(msg) => format!("网络错误: {}", msg),
            ServerFnError::ServerError(msg) => {
                if msg.is_empty() {
                    "服务器错误".to_string()
                } else {
                    msg.clone()
                }
            }
            ServerFnError::Deserialization(msg) => format!("数据解析失败: {}", msg),
            ServerFnError::Serialization(msg) => format!("数据序列化失败: {}", msg),
            _ => "未知错误".to_string(),
        }
    }
}
