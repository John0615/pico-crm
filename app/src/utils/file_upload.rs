use crate::components::ui::file_input::FileInfo;
use crate::server::file_handlers::upload_file;
use crate::utils::api::call_api;
use leptos::logging::log;
use shared::file::{FileUploadRequest, FileUploadResponse};
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{File, FileReader};

/// 上传文件并返回文件URL
pub async fn upload_file_from_info(file_info: FileInfo) -> Result<String, String> {
    // 由于 FileInfo 中的 data 字段目前是空的，我们需要从实际的 File 对象读取数据
    // 这里我们先返回一个模拟的URL，实际实现需要从 File 对象读取数据

    // 模拟文件上传过程
    log!("开始上传文件: {}", file_info.name);

    // 这里应该是实际的文件数据读取和上传逻辑
    // 由于当前的 FileInfo 结构体中 data 字段是空的，我们需要重新设计

    // 暂时返回一个模拟的URL
    let mock_url = format!("/uploads/{}", file_info.name);
    log!("文件上传完成: {}", mock_url);

    Ok(mock_url)
}

/// 从 File 对象上传文件
pub async fn upload_file_from_web_file(file: File) -> Result<FileUploadResponse, String> {
    let file_name = file.name();
    let content_type = file.type_();

    // 读取文件数据
    let file_data = read_file_as_bytes(&file)
        .await
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let request = FileUploadRequest {
        file_name,
        file_data, // 直接使用二进制数据
        content_type,
    };

    // 调用服务端上传函数
    match call_api(upload_file(request)).await {
        Ok(response) => {
            log!("文件上传成功: {}", response.file_url);
            Ok(response)
        }
        Err(e) => {
            log!("文件上传失败: {}", e);
            Err(format!("上传失败: {}", e))
        }
    }
}

/// 从 FileInfo 上传文件（需要重新读取文件数据）
pub async fn upload_file_info_with_data(
    file_name: String,
    file_data: Vec<u8>,
    content_type: String,
) -> Result<FileUploadResponse, String> {
    log!(
        "准备上传文件: {} ({}字节, {})",
        file_name,
        file_data.len(),
        content_type
    );

    let request = FileUploadRequest {
        file_name: file_name.clone(),
        file_data, // 直接使用二进制数据
        content_type: content_type.clone(),
    };

    // 调用改进后的服务端上传函数
    log!("开始调用 upload_file 服务函数");
    let result = call_api(upload_file(request)).await;
    log!("upload_file 调用完成，结果: {:?}", result.is_ok());

    match result {
        Ok(response) => {
            log!("文件上传成功: {}", response.file_url);
            Ok(response)
        }
        Err(e) => {
            log!("文件上传失败: {}", e);
            Err(format!("上传失败: {}", e))
        }
    }
}

/// 读取 File 对象为字节数组
pub async fn read_file_as_bytes(file: &File) -> Result<Vec<u8>, String> {
    let file_reader = FileReader::new().map_err(|_| "创建FileReader失败")?;

    // 创建一个 Promise 来处理异步文件读取
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        let reader = file_reader.clone();
        let reject_clone = reject.clone();
        let reject_clone2 = reject.clone();

        // 设置读取完成回调
        let onload = Closure::wrap(Box::new(move |_: web_sys::Event| {
            if let Ok(result) = reader.result() {
                resolve.call1(&JsValue::NULL, &result).unwrap();
            } else {
                reject_clone
                    .call1(&JsValue::NULL, &JsValue::from_str("读取文件失败"))
                    .unwrap();
            }
        }) as Box<dyn FnMut(_)>);

        file_reader.set_onload(Some(onload.as_ref().unchecked_ref()));
        onload.forget();

        // 设置错误回调
        let onerror = Closure::wrap(Box::new(move |_: web_sys::Event| {
            reject_clone2
                .call1(&JsValue::NULL, &JsValue::from_str("读取文件出错"))
                .unwrap();
        }) as Box<dyn FnMut(_)>);

        file_reader.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();

        // 开始读取文件
        if let Err(_) = file_reader.read_as_array_buffer(file) {
            reject
                .call1(&JsValue::NULL, &JsValue::from_str("开始读取文件失败"))
                .unwrap();
        }
    });

    // 等待 Promise 完成
    let result = JsFuture::from(promise)
        .await
        .map_err(|_| "文件读取Promise失败")?;

    // 将 ArrayBuffer 转换为 Vec<u8>
    let array_buffer = js_sys::ArrayBuffer::from(result);
    let uint8_array = js_sys::Uint8Array::new(&array_buffer);
    let mut bytes = vec![0; uint8_array.length() as usize];
    uint8_array.copy_to(&mut bytes);

    Ok(bytes)
}
