use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{File, FileList};

#[derive(Debug, Clone)]
pub struct FileInfo {
    pub name: String,
    pub size: u64,
    pub file_type: String,
    // Store file data as bytes instead of File object for thread safety
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UploadStatus {
    Ready,
    Uploading,
    Success,
    Error(String),
}

#[component]
pub fn FileInput(
    accept: String,
    #[prop(optional, default = false)] multiple: bool,
    max_size: u64,                              // 最大文件大小（字节）
    #[prop(optional)] max_files: Option<usize>, // 最大文件数量
    #[prop(optional, default = false)] disabled: bool,
    #[prop(optional, default = true)] drag_drop: bool, // 是否支持拖拽上传
    #[prop(optional, default = true)] show_file_list: bool, // 是否显示文件列表
    #[prop(optional, default = false)] auto_upload: bool, // 是否自动上传
    #[prop(optional, default = String::new())] class: String,
    on_change: Callback<Vec<FileInfo>>,
    #[prop(optional, default = Callback::new(|_| {}))] on_upload: Callback<FileInfo>, // 改为非可选，提供默认值
    #[prop(optional)] on_remove: Option<Callback<usize>>,
    #[prop(optional)] on_error: Option<Callback<String>>,
) -> impl IntoView {
    let file_input_ref = NodeRef::<leptos::html::Input>::new();
    let files = RwSignal::new(Vec::<FileInfo>::new());
    let _upload_status = RwSignal::new(UploadStatus::Ready);
    let accept_attr = accept.clone();
    let is_dragging = RwSignal::new(false);

    // 验证文件
    let validate_file = {
        let accept = accept.clone();
        move |file: &File| -> Result<(), String> {
            // 检查文件大小
            if file.size() as u64 > max_size {
                return Err(format!("文件大小超过限制 ({} MB)", max_size / 1024 / 1024));
            }

            // 检查文件类型
            if !accept.is_empty() {
                let file_type = file.type_();
                let accepted_types: Vec<&str> = accept.split(',').map(|s| s.trim()).collect();

                let is_accepted = accepted_types.iter().any(|&accepted| {
                    if accepted.starts_with('.') {
                        // 扩展名匹配
                        file.name().ends_with(accepted)
                    } else if accepted.contains('/') {
                        // MIME 类型匹配
                        if accepted.ends_with("/*") {
                            let prefix = &accepted[..accepted.len() - 2];
                            file_type.starts_with(prefix)
                        } else {
                            file_type == accepted
                        }
                    } else {
                        false
                    }
                });

                if !is_accepted {
                    return Err(format!("不支持的文件类型: {}", file_type));
                }
            }

            Ok(())
        }
    };

    // 处理文件选择
    let handle_files = {
        let files = files.clone();
        let _on_change = on_change.clone();
        let on_upload = on_upload.clone();
        let on_error = on_error.clone();
        move |file_list: FileList| {
            leptos::logging::log!("handle_files被调用，文件数量: {}", file_list.length());
            let mut new_files = Vec::new();
            let current_files = files.get_untracked();

            for i in 0..file_list.length() {
                if let Some(file) = file_list.get(i) {
                    // 检查文件数量限制
                    if let Some(max_files) = max_files {
                        if current_files.len() + new_files.len() >= max_files {
                            if let Some(on_error) = &on_error {
                                on_error.run(format!("最多只能上传 {} 个文件", max_files));
                            }
                            break;
                        }
                    }

                    // 验证文件
                    match validate_file(&file) {
                        Ok(_) => {
                            // Create FileInfo without web_sys::File for thread safety
                            let file_name = file.name();
                            let file_size = file.size() as u64;
                            let file_type = file.type_();

                            let file_info = FileInfo {
                                name: file_name.clone(),
                                size: file_size,
                                file_type: file_type.clone(),
                                data: Vec::new(), // Will be populated when needed
                            };
                            new_files.push(file_info.clone());

                            // 处理文件上传
                            leptos::logging::log!(
                                "检测到上传回调，开始异步读取文件: {}",
                                file_name
                            );
                            #[cfg(target_arch = "wasm32")]
                            {
                                let upload_callback = on_upload.clone();
                                let web_file = file.clone();

                                // 异步读取文件数据并上传
                                leptos::task::spawn_local(async move {
                                    leptos::logging::log!("开始读取文件数据: {}", file_name);
                                    // 读取文件数据
                                    match crate::utils::file_upload::read_file_as_bytes(&web_file)
                                        .await
                                    {
                                        Ok(file_data) => {
                                            leptos::logging::log!(
                                                "文件读取成功: {} ({}字节)",
                                                file_name,
                                                file_data.len()
                                            );
                                            // 创建包含实际数据的 FileInfo
                                            let file_info_with_data = FileInfo {
                                                name: file_name,
                                                size: file_size,
                                                file_type,
                                                data: file_data,
                                            };
                                            leptos::logging::log!("调用上传回调");
                                            upload_callback.run(file_info_with_data);
                                        }
                                        Err(e) => {
                                            leptos::logging::error!("读取文件失败: {}", e);
                                            if let Some(on_error) = &on_error {
                                                on_error.run(format!("读取文件失败: {}", e));
                                            }
                                        }
                                    }
                                });
                            }
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                on_upload.run(file_info.clone());
                            }
                        }
                        Err(error) => {
                            if let Some(on_error) = &on_error {
                                on_error.run(error);
                            }
                        }
                    }
                }
            }

            // if !new_files.is_empty() {
            //     let mut all_files = current_files;
            //     all_files.extend(new_files.clone());
            //     files.set(all_files);

            //     on_change.run(new_files.clone());

            //     // 自动上传 - 只有在auto_upload=true且没有on_upload回调时才执行
            //     if auto_upload && on_upload.is_none() {
            //         for file_info in new_files {
            //             // 这里需要实际的上传逻辑，但由于没有on_upload回调，暂时跳过
            //             leptos::logging::log!("自动上传文件: {}", file_info.name);
            //         }
            //     }
            // }
        }
    };

    // 文件输入变化处理
    let on_file_change = {
        let handle_files = handle_files.clone();
        move |ev: leptos::ev::Event| {
            ev.prevent_default(); // 阻止默认行为
            ev.stop_propagation(); // 阻止事件冒泡到父元素（避免触发表单的其他提交逻辑）
            leptos::logging::log!("文件输入变化事件被触发");
            let input = ev
                .target()
                .unwrap()
                .dyn_into::<web_sys::HtmlInputElement>()
                .unwrap();
            if let Some(file_list) = input.files() {
                leptos::logging::log!("检测到{}个文件", file_list.length());
                handle_files(file_list);
            } else {
                leptos::logging::log!("没有检测到文件");
            }
        }
    };

    // 点击上传区域
    let trigger_file_input = move |ev: leptos::ev::MouseEvent| {
        ev.prevent_default(); // 阻止默认行为
        ev.stop_propagation(); // 阻止事件冒泡
        leptos::logging::log!("上传区域被点击");
        if !disabled {
            if let Some(input) = file_input_ref.get_untracked() {
                leptos::logging::log!("触发文件选择对话框");
                input.click();
            }
        }
    };

    // 拖拽处理
    let on_drag_over = move |ev: leptos::ev::DragEvent| {
        if drag_drop && !disabled {
            ev.prevent_default();
            is_dragging.set(true);
        }
    };

    let on_drag_leave = move |_: leptos::ev::DragEvent| {
        if drag_drop && !disabled {
            is_dragging.set(false);
        }
    };

    let on_drop = {
        let handle_files = handle_files.clone();
        move |ev: leptos::ev::DragEvent| {
            if drag_drop && !disabled {
                ev.prevent_default();
                is_dragging.set(false);

                if let Some(data_transfer) = ev.data_transfer() {
                    if let Some(file_list) = data_transfer.files() {
                        handle_files(file_list);
                    }
                }
            }
        }
    };

    // 移除文件
    let remove_file = move |index: usize| {
        let mut current_files = files.get_untracked();
        if index < current_files.len() {
            current_files.remove(index);
            files.set(current_files);

            if let Some(on_remove) = on_remove {
                on_remove.run(index);
            }
        }
    };

    // 清空文件
    let clear_files = move |_| {
        files.set(Vec::new());
        if let Some(input) = file_input_ref.get_untracked() {
            input.set_value("");
        }
    };

    // 格式化文件大小
    let format_file_size = |size: u64| -> String {
        if size < 1024 {
            format!("{} B", size)
        } else if size < 1024 * 1024 {
            format!("{:.1} KB", size as f64 / 1024.0)
        } else if size < 1024 * 1024 * 1024 {
            format!("{:.1} MB", size as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.1} GB", size as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    };

    view! {
        <div class=format!("file-upload-container {}", class)>
            // 隐藏的文件输入
            <input
                node_ref=file_input_ref
                type="file"
                class="hidden"
                accept=accept_attr
                multiple=multiple
                disabled=disabled
                on:change=on_file_change
                on:click=move |ev: leptos::ev::MouseEvent| {
                    ev.stop_propagation(); // 阻止事件冒泡
                }
            />

            // 上传区域
            <div
                class=move || format!(
                    "upload-area border-2 border-dashed rounded-lg p-6 text-center cursor-pointer transition-all duration-200 w-full max-w-xs mx-auto {}",
                    if disabled {
                        "border-gray-300 bg-gray-50 cursor-not-allowed"
                    } else if is_dragging.get() {
                        "border-primary bg-primary/10"
                    } else {
                        "border-gray-300 hover:border-primary hover:bg-base-200"
                    }
                )
                on:click=trigger_file_input
                on:dragover=on_drag_over
                on:dragleave=on_drag_leave
                on:drop=on_drop
            >
                <div class="flex flex-col items-center gap-2">
                    // 上传图标
                    <div class="text-4xl text-gray-400">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-12 h-12">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M3 16.5v2.25A2.25 2.25 0 0 0 5.25 21h13.5A2.25 2.25 0 0 0 21 18.75V16.5m-13.5-9L12 3m0 0 4.5 4.5M12 3v13.5" />
                        </svg>
                    </div>

                    // 上传文本
                    <div class="text-sm">
                        {if drag_drop && !disabled {
                            view! {
                                <div>
                                    <span class="text-primary font-medium">点击上传</span>
                                    <span class="text-gray-500"> 或拖拽文件到此处</span>
                                </div>
                            }
                        } else {
                            view! {
                                <div>
                                    <span class="text-primary font-medium">点击选择文件</span>
                                </div>
                            }
                        }}
                    </div>

                    // 文件限制提示
                    <div class="text-xs text-gray-400">
                        {if !accept.is_empty() {
                            format!("支持格式: {}", accept)
                        } else {
                            String::new()
                        }}
                        {if max_size > 0 {
                            format!(" 最大大小: {}", format_file_size(max_size))
                        } else {
                            String::new()
                        }}
                    </div>
                </div>
            </div>

            // 文件列表
            {move || if show_file_list && !files.get().is_empty() {
                view! {
                    <div class="file-list mt-4">
                        <div class="flex justify-between items-center mb-2">
                            <span class="text-sm font-medium">已选择文件 ({files.get().len()})</span>
                            <button
                                type="button"
                                class="btn btn-ghost btn-xs"
                                on:click=clear_files
                            >
                                清空
                            </button>
                        </div>

                        <div class="space-y-2">
                            {files.get().into_iter().enumerate().map(|(index, file_info)| {
                                view! {
                                    <div class="flex items-center justify-between p-3 bg-base-200 rounded-lg">
                                        <div class="flex items-center gap-3 flex-1 min-w-0">
                                            // 文件图标
                                            <div class="text-primary">
                                                {if file_info.file_type.starts_with("image/") {
                                                    view! {
                                                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                                            <path stroke-linecap="round" stroke-linejoin="round" d="m2.25 15.75 5.159-5.159a2.25 2.25 0 0 1 3.182 0l5.159 5.159m-1.5-4.5 1.409-1.409a2.25 2.25 0 0 1 3.182 0l2.909 2.909m-18 3.75h16.5a1.5 1.5 0 0 0 1.5-1.5V6a1.5 1.5 0 0 0-1.5-1.5H3.75A1.5 1.5 0 0 0 2.25 6v12a1.5 1.5 0 0 0 1.5 1.5Zm10.5-11.25h.008v.008h-.008V8.25Zm.375 0a.375.375 0 1 1-.75 0 .375.375 0 0 1 .75 0Z" />
                                                        </svg>
                                                    }
                                                } else {
                                                    view! {
                                                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-5 h-5">
                                                            <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
                                                        </svg>
                                                    }
                                                }}
                                            </div>

                                            // 文件信息
                                            <div class="flex-1 min-w-0">
                                                <div class="text-sm font-medium truncate">{file_info.name.clone()}</div>
                                                <div class="text-xs text-gray-500">
                                                    {format_file_size(file_info.size)} - {file_info.file_type.clone()}
                                                </div>
                                            </div>
                                        </div>

                                        // 操作按钮
                                        <div class="flex items-center gap-2">
                                            {if !auto_upload {
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="btn btn-primary btn-xs"
                                                        on:click={
                                                            let file_info = file_info.clone();
                                                            let upload_callback = on_upload.clone();
                                                            move |_| {
                                                                upload_callback.run(file_info.clone());
                                                            }
                                                        }
                                                    >
                                                        上传
                                                    </button>
                                                }
                                            } else {
                                                view! {
                                                    <button
                                                        type="button"
                                                        class="btn btn-primary btn-xs"
                                                        style="display: none;"
                                                    >
                                                        上传
                                                    </button>
                                                }
                                            }}

                                            <button
                                                type="button"
                                                class="btn btn-ghost btn-xs text-error"
                                                on:click={
                                                    move |_| remove_file(index)
                                                }
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-4 h-4">
                                                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18 18 6M6 6l12 12" />
                                                </svg>
                                            </button>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()}
                        </div>
                    </div>
                }
            } else {
                view! {
                    <div class="file-list mt-4" style="display: none;">
                        <div class="flex justify-between items-center mb-2">
                            <span class="text-sm font-medium">已选择文件 (0)</span>
                        </div>
                    </div>
                }
            }}
        </div>
    }
}

// 简化版文件上传组件
#[component]
pub fn SimpleFileInput(
    accept: String,
    #[prop(optional, default = false)] multiple: bool,
    #[prop(optional, default = false)] disabled: bool,
    max_size: u64,
    #[prop(optional, default = String::new())] class: String,
    on_change: Callback<Vec<FileInfo>>,
    #[prop(optional, default = Callback::new(|_| {}))] on_upload: Callback<FileInfo>, // 改为非可选
) -> impl IntoView {
    view! {
        <FileInput
            accept=if accept.is_empty() { "*/*".to_string() } else { accept }
            multiple=multiple
            disabled=disabled
            max_size=max_size
            class=class
            show_file_list=false
            drag_drop=false
            auto_upload=false
            on_change=on_change
            on_upload=on_upload
        />
    }
}

// 头像上传组件
#[component]
pub fn AvatarUpload(
    #[prop(optional)] current_avatar: Option<String>,
    #[prop(optional)] size: Option<String>, // "sm", "md", "lg"
    #[prop(optional, default = false)] disabled: bool,
    #[prop(optional, default = String::new())] class: String,
    on_upload: Callback<FileInfo>,
) -> impl IntoView {
    let avatar_preview = RwSignal::new(current_avatar);
    let size_class = match size.as_deref() {
        Some("sm") => "w-16 h-16",
        Some("lg") => "w-32 h-32",
        _ => "w-24 h-24", // md (default)
    };

    let handle_file_change = move |files: Vec<FileInfo>| {
        if let Some(file_info) = files.first() {
            // For now, we'll use a placeholder URL since we don't have the File object
            // In a real implementation, you'd create a blob URL from the file data
            avatar_preview.set(Some("data:image/svg+xml;base64,PHN2ZyB3aWR0aD0iMjQiIGhlaWdodD0iMjQiIHZpZXdCb3g9IjAgMCAyNCAyNCIgZmlsbD0ibm9uZSIgeG1sbnM9Imh0dHA6Ly93d3cudzMub3JnLzIwMDAvc3ZnIj4KPHBhdGggZD0iTTEyIDJDNi40OCAyIDIgNi40OCAyIDEyUzYuNDggMjIgMTIgMjJTMjIgMTcuNTIgMjIgMTJTMTcuNTIgMiAxMiAyWk0xMiA2QzEzLjY2IDYgMTUgNy4zNCAxNSA5UzEzLjY2IDEyIDEyIDEyUzkgMTAuNjYgOSA5UzEwLjM0IDYgMTIgNlpNMTIgMjBDOS43NCAyMCA3Ljc5IDE4Ljg1IDYuNzggMTcuMUM3LjY4IDE1LjgxIDkuNzUgMTUgMTIgMTVTMTYuMzIgMTUuODEgMTcuMjIgMTcuMUMxNi4yMSAxOC44NSAxNC4yNiAyMCAxMiAyMFoiIGZpbGw9IiM5Q0EzQUYiLz4KPHN2Zz4K".to_string()));

            on_upload.run(file_info.clone());
        }
    };

    view! {
        <div class=format!("avatar-upload {}", class)>
            <div class="flex items-center gap-4">
                // 头像预览
                <div class=format!("avatar {}", size_class)>
                    <div class="rounded-full overflow-hidden border-2 border-gray-200">
                        {move || if let Some(src) = avatar_preview.get() {
                            view! {
                                <img src=src alt="Avatar" class="w-full h-full object-cover" />
                            }.into_any()
                        } else {
                            view! {
                                <div class="w-full h-full bg-gray-100 flex items-center justify-center">
                                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-8 h-8 text-gray-400">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 6a3.75 3.75 0 1 1-7.5 0 3.75 3.75 0 0 1 7.5 0ZM4.501 20.118a7.5 7.5 0 0 1 14.998 0A17.933 17.933 0 0 1 12 21.75c-2.676 0-5.216-.584-7.499-1.632Z" />
                                    </svg>
                                </div>
                            }.into_any()
                        }}
                    </div>
                </div>

                // 上传按钮
                <SimpleFileInput
                    accept="image/*".to_string()
                    multiple=false
                    disabled=disabled
                    max_size=5 * 1024 * 1024 // 5MB for avatar
                    class="flex-1".to_string()
                    on_change=Callback::new(handle_file_change)
                    on_upload=on_upload
                />
            </div>
        </div>
    }
}
