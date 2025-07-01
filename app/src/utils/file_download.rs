use js_sys::{Array, Uint8Array};
use wasm_bindgen::JsCast;
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};

pub fn download_file(data: &[u8], filename: &str) -> Result<(), String> {
    // 1. 创建Uint8Array并填充数据
    let js_array = Uint8Array::new_with_length(data.len() as u32);
    js_array.copy_from(data);

    // 2. 创建包含Uint8Array的JS数组
    let array = Array::new();
    array.push(&js_array);

    // 3. 配置Blob属性
    let blob_properties = BlobPropertyBag::new();
    blob_properties.set_type("application/vnd.openxmlformats-officedocument.spreadsheetml.sheet");

    // 4. 创建Blob（关键修正）
    let blob = Blob::new_with_u8_array_sequence_and_options(&array, &blob_properties)
        .map_err(|e| format!("Blob creation failed: {:?}", e))?;

    // 5. 生成对象URL
    let url = Url::create_object_url_with_blob(&blob)
        .map_err(|e| format!("URL creation failed: {:?}", e))?;

    // 6. 创建并触发下载链接
    let window = web_sys::window().ok_or("No window object")?;
    let document = window.document().ok_or("No document object")?;
    let a = document
        .create_element("a")
        .map_err(|e| format!("Anchor creation failed: {:?}", e))?
        .dyn_into::<HtmlAnchorElement>()
        .map_err(|e| format!("Anchor cast failed: {:?}", e))?;

    a.set_href(&url);
    a.set_download(filename);
    a.style()
        .set_property("display", "none")
        .map_err(|e| format!("Style set failed: {:?}", e))?;

    document
        .body()
        .ok_or("No body element")?
        .append_child(&a)
        .map_err(|e| format!("Append failed: {:?}", e))?;
    a.click();

    // 7. 清理
    let _ = document.body().unwrap().remove_child(&a);
    let _ = Url::revoke_object_url(&url);

    Ok(())
}
