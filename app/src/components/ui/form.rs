use crate::components::ui::file_input::{FileInfo, SimpleFileInput, UploadStatus};
use leptos::ev::SubmitEvent;
use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::future::Future;
use std::sync::Arc;

// 表单字段定义
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FormField {
    pub name: String,
    pub label: String,
    pub field_type: FieldType,
    pub required: bool,
    pub value: ArcRwSignal<String>,
    pub placeholder: Option<String>,
    pub error_message: ArcRwSignal<Option<String>>,
    #[serde(skip)]
    pub validation: Option<ValidationRule>,
}

// 字段类型枚举
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum FieldType {
    Text,
    Email,
    Password,
    Number,
    TextArea,
    Select(Vec<(String, String)>),
    Checkbox,
    File {
        accept: Option<String>,
        multiple: bool,
        max_size: Option<u64>,
        #[serde(skip)]
        on_upload: Option<Callback<FileInfo>>, // 上传回调，跳过序列化
    },
}

impl FieldType {
    fn to_string(&self) -> String {
        match self {
            FieldType::Text => "text".to_string(),
            FieldType::Email => "email".to_string(),
            FieldType::Password => "password".to_string(),
            FieldType::Number => "number".to_string(),
            _ => "text".to_string(),
        }
    }
}

// 验证规则
#[derive(Debug, Clone)]
pub enum ValidationRule {
    MinLength(usize),
    MaxLength(usize),
    Email,
    CnMobile,
    Custom(CustomValidator),
}

#[derive(Clone)]
pub struct CustomValidator(pub Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync + 'static>);

impl fmt::Debug for CustomValidator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<custom_validator>")
    }
}

impl CustomValidator {
    pub fn new(f: impl Fn(&str) -> Result<(), String> + Send + Sync + 'static) -> Self {
        Self(Arc::new(f))
    }
    pub fn validate(&self, value: &str) -> Result<(), String> {
        (self.0)(value)
    }
}

fn is_valid_email(value: &str) -> bool {
    let value = value.trim();
    let mut parts = value.split('@');
    let local = match parts.next() {
        Some(part) if !part.is_empty() => part,
        _ => return false,
    };
    let domain = match parts.next() {
        Some(part) if !part.is_empty() => part,
        _ => return false,
    };
    if parts.next().is_some() {
        return false;
    }
    if local.chars().any(|c| c.is_whitespace()) || domain.chars().any(|c| c.is_whitespace()) {
        return false;
    }
    let mut domain_parts = domain.split('.');
    let first = match domain_parts.next() {
        Some(part) if !part.is_empty() => part,
        _ => return false,
    };
    let mut has_dot = false;
    for part in domain_parts {
        if part.is_empty() {
            return false;
        }
        has_dot = true;
    }
    !first.is_empty() && has_dot
}

fn is_valid_cn_mobile(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 11 {
        return false;
    }
    if bytes[0] != b'1' {
        return false;
    }
    if !(b'3'..=b'9').contains(&bytes[1]) {
        return false;
    }
    bytes.iter().all(|b| b.is_ascii_digit())
}

#[component]
pub fn DaisyForm<F, T>(
    initial_fields: Vec<FormField>,
    on_submit: F,
    #[prop(optional)] submit_text: Option<String>,
    #[prop(optional)] reset_text: Option<String>,
) -> impl IntoView
where
    F: Fn(Vec<FormField>) -> T + Copy + 'static,
    T: Future<Output = Result<(), Vec<String>>> + 'static,
{
    let fields = RwSignal::new(initial_fields);

    let is_submitting = RwSignal::new(false);

    // 验证单个字段
    let validate_field = move |field: &FormField| -> Option<String> {
        let value = field.value.read();
        if field.required && value.is_empty() {
            return Some(format!("{}不能为空", field.label));
        }

        if let Some(validation) = &field.validation {
            match validation {
                ValidationRule::MinLength(min) if value.len() < *min => {
                    return Some(format!("{}长度至少{}", field.label, min));
                }
                ValidationRule::MaxLength(max) if value.len() > *max => {
                    return Some(format!("{}长度最多{}", field.label, max));
                }
                ValidationRule::Email => {
                    if !value.is_empty() && !is_valid_email(value.as_str()) {
                        return Some(format!("{}格式不正确", field.label));
                    }
                }
                ValidationRule::CnMobile => {
                    if !value.is_empty() && !is_valid_cn_mobile(value.as_str()) {
                        return Some(format!("{}格式不正确", field.label));
                    }
                }
                ValidationRule::Custom(validator) => {
                    if let Err(msg) = validator.validate(value.as_str()) {
                        return Some(msg);
                    }
                }
                _ => {}
            }
        }
        None
    };

    // 验证整个表单
    let validate_form = move || {
        let mut is_valid = true;
        fields.with_untracked(|fields| {
            for field in fields.iter() {
                let error = validate_field(field);
                let error = error.clone();
                field.error_message.set(error.clone());
                if error.is_some() {
                    is_valid = false;
                }
            }
        });
        is_valid
    };

    // 重置表单
    let reset = move || {
        fields.with_untracked(|fields| {
            for field in fields.iter() {
                field.value.set(String::new());
                field.error_message.set(None);
            }
        });
    };

    // 提交表单
    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        if !validate_form() {
            return;
        }

        is_submitting.set(true);

        spawn_local(async move {
            let result = on_submit(fields.get_untracked().clone()).await;
            is_submitting.set(false);
            match result {
                Ok(_) => reset(),
                Err(_errors) => {
                    // Handle errors silently or show user feedback
                }
            };
        });
    };

    view! {
        <form class="form-control w-full max-w-md mx-auto" on:submit=submit>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <For
                    each=move || fields.with(|fields| fields.clone())
                    key=|field| field.name.clone()
                    children=move |field| {
                        view! {
                            {match field.field_type.clone() {
                                FieldType::Text | FieldType::Email | FieldType::Password | FieldType::Number =>
                                    view! {
                                        <TextInput
                                            field_type=field.field_type.to_string()
                                            name=field.name
                                            label=field.label
                                            value=field.value
                                            required=field.required
                                            placeholder=field.placeholder.unwrap_or_default()
                                            error_message=field.error_message
                                        />
                                    }.into_any(),
                                FieldType::TextArea =>
                                    view! {
                                        <TextAreaInput
                                            name=field.name
                                            label=field.label
                                            value=field.value
                                            required=field.required
                                            placeholder=field.placeholder.unwrap_or_default()
                                            error_message=field.error_message
                                        />
                                    }.into_any(),
                                FieldType::Select(options) =>
                                    view! {
                                        <SelectInput
                                            name=field.name
                                            label=field.label
                                            value=field.value
                                            required=true
                                            options=options
                                            error_message=field.error_message
                                        />
                                    }.into_any(),
                                FieldType::Checkbox =>
                                    view! {
                                        <CheckboxInput
                                            name=field.name
                                            label=field.label
                                            checked=ArcRwSignal::new(false)
                                            error_message=field.error_message
                                        />
                                    }.into_any(),
                                FieldType::File { accept, multiple, max_size, on_upload } => {
                                    let field_name = field.name.clone();
                                    let field_label = field.label.clone();
                                    let field_error = field.error_message.clone();
                                    let field_value = field.value.clone();
                                    let accept_str = accept.unwrap_or_default();

                                    view! {
                                        <FileFieldInput
                                            _name=field_name
                                            label=field_label
                                            accept=accept_str
                                            multiple=multiple
                                            max_size=max_size.unwrap_or(10 * 1024 * 1024) // Default 10MB
                                            value=field_value
                                            on_upload=on_upload.unwrap_or_else(|| Callback::new(|_| {}))
                                            error_message=field_error
                                        />
                                    }.into_any()
                                },
                            }}
                        }
                    }
                />
            </div>

            <div class="flex gap-2 justify-end mt-6">
                <button
                    type="button"
                    class="btn btn-ghost"
                    on:click=move |_|reset()
                    disabled=move || *is_submitting.read()
                >
                    {reset_text.unwrap_or("重置".into())}
                </button>

                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled=move || *is_submitting.read()
                >
                    {move || if *is_submitting.read() {
                        view! {
                            <span class="loading loading-spinner"></span>
                        }.into_view().into_any()
                    } else {
                        view! {
                            {submit_text.clone().unwrap_or("提交".into())}
                        }.into_view().into_any()
                    }}
                </button>
            </div>
        </form>
    }
}

// Option<impl IntoView + 'static> 在 Leptos 中表示 可选的、任何能够转换为视图的类型：

// Option<T>：表示值可以是 Some(T) 或 None
// impl IntoView：Leptos 的 trait，表示任何可以转换为视图的类型，包括：
// 基本类型（&str, String, i32 等）
// Leptos 视图（View）
// 信号（Signal, Memo）
// 闭包返回视图（Fn() -> impl IntoView）
// 'static：生命周期标记，表示类型不包含非静态引用

#[component]
pub fn FormContainer(
    #[prop(optional)] title: Option<impl IntoView + 'static>,
    #[prop(optional)] description: Option<&'static str>,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("bg-base-100 {}", class)>
            {title.map(|t| view! { <h2 class="card-title text-2xl mb-2">{t}</h2> })}
            {description.map(|d| view! { <p class="text-gray-500 mb-4">{d}</p> })}
            <div class="space-y-4">
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn FormActions(#[prop(optional)] justify_end: bool, children: Children) -> impl IntoView {
    view! {
        <div class=format!(
            "flex {} gap-2 mt-6",
            if justify_end { "justify-end" } else { "justify-start" }
        )>
            {children()}
        </div>
    }
}

#[component]
pub fn TextInput(
    name: String,
    field_type: String,
    label: String,
    value: ArcRwSignal<String>,
    #[prop(optional)] placeholder: String,
    #[prop(optional)] required: bool,
    #[prop(optional)] class: String,
    #[prop(optional)] error_message: ArcRwSignal<Option<String>>,
) -> impl IntoView {
    let value_clone = value.clone();
    view! {
        <fieldset class=format!("fieldset form-control {}", class)>
            <label class="label">
                <span class="label-text">{label}</span>
                {required.then(|| view! { <span class="text-error">*</span> })}
            </label>
            <input
                type=field_type
                name=name
                class="input input-bordered"
                placeholder=placeholder
                required=required
                prop:value=move || value_clone.with(|value| value.clone())
                on:input=move |ev| {
                    let new_value = event_target_value(&ev);
                    value.set(new_value.clone());
                }
            />
            <p class="label">
                <span class="label-text-alt text-error h-2">
                    {move || error_message.with(|msg| msg.clone().unwrap_or_default())}
                </span>
            </p>
        </fieldset>
    }
}

#[component]
pub fn CheckboxInput(
    name: String,
    label: String,
    checked: ArcRwSignal<bool>,
    #[prop(optional)] class: String,
    #[prop(optional)] error_message: ArcRwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <fieldset class=format!("fieldset form-control {}", class)>
            <label class="label cursor-pointer justify-start gap-2">
                <input
                    type="checkbox"
                    name=name
                    class="checkbox checkbox-primary"
                    checked={
                        let checked = checked.clone();
                        move || *checked.read()
                    }
                    on:change={
                        let checked = checked.clone();
                        move |ev| checked.set(event_target_checked(&ev))
                    }
                />
                <span class="label-text">{label}</span>
            </label>
            <p class="label">
                <span class="label-text-alt text-error h-2">
                    {move || error_message.with(|msg| msg.clone().unwrap_or_default())}
                </span>
            </p>
        </fieldset>
    }
}

#[component]
pub fn TextAreaInput(
    name: String,
    label: String,
    value: ArcRwSignal<String>,
    #[prop(optional)] placeholder: String,
    #[prop(optional)] required: bool,
    #[prop(optional)] rows: usize,
    #[prop(optional)] class: String,
    #[prop(optional)] error_message: ArcRwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <fieldset class=format!("fieldset form-control {}", class)>
            <label class="label">
                <span class="label-text">{label}</span>
            </label>
            <textarea
                name=name
                required=required
                class="textarea textarea-bordered"
                placeholder=placeholder
                rows=rows
                prop:value={
                    let value = value.clone();
                    move || value.with(|value| value.clone())
                }
                on:input={
                    let value = value.clone();
                    move |ev| value.set(event_target_value(&ev))
                }
            ></textarea>
            <p class="label">
                <span class="label-text-alt text-error h-2">
                    {move || error_message.with(|msg| msg.clone().unwrap_or_default())}
                </span>
            </p>
        </fieldset>
    }
}

#[component]
pub fn SelectInput<T>(
    name: String,
    label: String,
    value: ArcRwSignal<T>,
    options: Vec<(T, String)>,
    #[prop(optional)] required: bool,
    #[prop(optional)] class: String,
    #[prop(optional)] error_message: ArcRwSignal<Option<String>>,
) -> impl IntoView
where
    T: Clone + PartialEq + Send + Sync + 'static, // 移除了 ToString 约束
{
    let cloned_value1 = value.clone();
    let cloned_value2 = value.clone();

    view! {
        <fieldset class=format!("fieldset form-control {}", class)>
            <label class="label">
                <span class="label-text">{label}</span>
                {required.then(|| view! { <span class="text-error">*</span> })}
            </label>
            <select
                name=name
                class="select select-bordered w-full"
                required=required
                on:input={
                    let options = options.clone();
                    move |ev| {
                        let new_value = event_target_value(&ev);
                        if let Some((val, _)) = options.clone().iter().find(|(_, display)| *display == new_value) {
                            cloned_value1.set(val.clone());
                        }
                    }
                }
            >
                <For
                    each={
                        let options = options.clone();
                        move || options.clone()
                    }
                    key=|(_, text)| text.clone()
                    children=move |(val, text)| {
                        let value = cloned_value2.clone();
                        let val = val.clone();
                        let is_selected = move || value.with(|current| current == &val);
                        view! {
                            <option
                                value=text
                                selected=is_selected
                            >
                                {text.clone()}
                            </option>
                        }
                    }
                />
            </select>
            <p class="label">
                <span class="label-text-alt text-error h-2">
                    {move || error_message.with(|msg| msg.clone().unwrap_or_default())}
                </span>
            </p>
        </fieldset>
    }
}

fn looks_like_image_url(value: &str) -> bool {
    if value.starts_with("data:image/") {
        return true;
    }
    let value = value.split('?').next().unwrap_or(value).to_ascii_lowercase();
    [".png", ".jpg", ".jpeg", ".gif", ".webp", ".bmp", ".svg"]
        .iter()
        .any(|ext| value.ends_with(ext))
}

#[component]
pub fn FileFieldInput(
    _name: String,
    label: String,
    accept: String,
    #[prop(optional, default = false)] multiple: bool,
    max_size: u64,
    value: ArcRwSignal<String>,    // 存储文件路径
    on_upload: Callback<FileInfo>, // 上传回调
    #[prop(optional, default = String::new())] class: String,
    #[prop(optional)] error_message: ArcRwSignal<Option<String>>,
) -> impl IntoView {
    let upload_status = RwSignal::new(UploadStatus::Ready);
    let accept_is_image = accept.to_lowercase().contains("image");

    let handle_file_change = {
        let upload_status = upload_status.clone();
        move |files: Vec<FileInfo>| {
            if !files.is_empty() {
                upload_status.set(UploadStatus::Uploading);
            }
        }
    };

    let handle_upload = {
        let upload_status = upload_status.clone();
        let on_upload = on_upload.clone();
        Callback::new(move |file_info: FileInfo| {
            upload_status.set(UploadStatus::Uploading);
            on_upload.run(file_info);
        })
    };

    // 监听 value 变化：有值 -> 成功；清空 -> 重置
    Effect::new({
        let upload_status = upload_status.clone();
        let value = value.clone();
        move |_| {
            let is_empty = value.with(|current_value| current_value.is_empty());
            if is_empty {
                let should_reset =
                    upload_status.with(|status| !matches!(status, UploadStatus::Ready));
                if should_reset {
                    upload_status.set(UploadStatus::Ready);
                }
                return;
            }

            let should_mark_success = upload_status
                .with(|status| matches!(status, UploadStatus::Uploading | UploadStatus::Ready));
            if should_mark_success {
                upload_status.set(UploadStatus::Success);
            }
        }
    });

    let value_for_upload_area = value.clone();
    let value_for_preview_area = value.clone();
    let value_for_image = value.clone();
    let value_for_image_src = value.clone();
    let value_for_image_url = value.clone();
    let value_for_file = value.clone();
    let value_for_file_url = value.clone();

    view! {
        <fieldset class=format!("fieldset form-control {}", class)>
            <label class="label">
                <span class="label-text font-medium">{label}</span>
            </label>

            <div class="space-y-2">
                // 文件上传区域 - 始终渲染，但根据状态显示不同内容
                <div class="relative">
                    // 上传区域 - 当没有文件且不在上传时显示
                    <div class=move || {
                        let is_empty = value_for_upload_area.with(|value| value.is_empty());
                        let is_uploading =
                            upload_status.with(|status| matches!(status, UploadStatus::Uploading));
                        if is_empty && !is_uploading {
                            "block"
                        } else {
                            "hidden"
                        }
                    }>
                        <SimpleFileInput
                            accept=accept.clone()
                            multiple=multiple
                            max_size=max_size
                            class="w-full".to_string()
                            on_change=Callback::new(handle_file_change)
                            on_upload=handle_upload
                        />
                    </div>

                    // 预览区域 - 当有文件或正在上传时显示
                    <div class=move || {
                        let has_value = value_for_preview_area.with(|value| !value.is_empty());
                        let is_uploading =
                            upload_status.with(|status| matches!(status, UploadStatus::Uploading));
                        if is_uploading || has_value {
                            "block"
                        } else {
                            "hidden"
                        }
                    }>
                        <div class="relative border-2 border-dashed border-gray-300 rounded-lg p-3 text-center bg-base-100 transition-all duration-200 w-full max-w-xs mx-auto min-h-[148px]">
                            // Loading状态显示
                            <div class=move || {
                                let is_uploading =
                                    upload_status.with(|status| matches!(status, UploadStatus::Uploading));
                                if is_uploading {
                                    "flex flex-col items-center gap-3"
                                } else {
                                    "hidden"
                                }
                            }>
                                <div class="loading loading-spinner loading-lg text-primary"></div>
                                <div class="text-sm text-info font-medium">上传中...</div>
                                <div class="text-xs text-gray-400">请稍候</div>
                            </div>

                            // 图片预览区域
                            <div class=move || {
                                let is_image = value_for_image.with(|current_value| {
                                    !current_value.is_empty()
                                        && !current_value.contains(',')
                                        && (accept_is_image || looks_like_image_url(current_value))
                                });
                                let is_uploading =
                                    upload_status.with(|status| matches!(status, UploadStatus::Uploading));
                                if is_image && !is_uploading {
                                    "flex flex-col items-center gap-1"
                                } else {
                                    "hidden"
                                }
                            }>
                                <div class="w-20 h-20 mx-auto">
                                    <img src=move || value_for_image_src.with(|value| value.clone()) alt="预览" class="w-full h-full object-cover rounded-lg border border-gray-200" />
                                </div>
                                <div class="text-sm text-success font-medium">上传成功</div>
                                <div class="text-xs text-gray-500 break-words max-w-full px-1 leading-tight">{move || {
                                    value_for_image_url.with(|url| {
                                        if url.len() > 30 {
                                            format!("{}...{}", &url[..15], &url[url.len()-10..])
                                        } else {
                                            url.clone()
                                        }
                                    })
                                }}</div>
                                // 删除按钮
                                <button
                                    type="button"
                                    class="btn btn-xs btn-outline btn-error absolute top-2 right-2"
                                    on:click={
                                        let value = value.clone();
                                        let upload_status = upload_status.clone();
                                        move |ev: leptos::ev::MouseEvent| {
                                            ev.prevent_default();
                                            ev.stop_propagation();
                                            value.set(String::new());
                                            upload_status.set(UploadStatus::Ready);
                                        }
                                    }
                                    title="删除文件"
                                >
                                    "删除"
                                </button>
                            </div>

                            // 非图片文件预览区域
                            <div class=move || {
                                let (has_value, is_image) = value_for_file.with(|current_value| {
                                    let is_image = !current_value.is_empty()
                                        && !current_value.contains(',')
                                        && (accept_is_image || looks_like_image_url(current_value));
                                    (!current_value.is_empty(), is_image)
                                });
                                let is_uploading =
                                    upload_status.with(|status| matches!(status, UploadStatus::Uploading));
                                if has_value && !is_uploading && !is_image {
                                    "flex flex-col items-center gap-1"
                                } else {
                                    "hidden"
                                }
                            }>
                                <div class="text-3xl text-primary">
                                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-10 h-10">
                                        <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 0 0-3.375-3.375h-1.5A1.125 1.125 0 0 1 13.5 7.125v-1.5a3.375 3.375 0 0 0-3.375-3.375H8.25m2.25 0H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 0 0-9-9Z" />
                                    </svg>
                                </div>
                                <div class="text-sm text-success font-medium">上传成功</div>
                                <div class="text-xs text-gray-500 break-words max-w-full px-1 leading-tight">{move || {
                                    value_for_file_url.with(|url| {
                                        if url.len() > 30 {
                                            format!("{}...{}", &url[..15], &url[url.len()-10..])
                                        } else {
                                            url.clone()
                                        }
                                    })
                                }}</div>
                                // 删除按钮
                                <button
                                    type="button"
                                    class="btn btn-xs btn-outline btn-error absolute top-2 right-2"
                                    on:click={
                                        let value = value.clone();
                                        let upload_status = upload_status.clone();
                                        move |ev: leptos::ev::MouseEvent| {
                                            ev.prevent_default();
                                            ev.stop_propagation();
                                            value.set(String::new());
                                            upload_status.set(UploadStatus::Ready);
                                        }
                                    }
                                    title="删除文件"
                                >
                                    "删除"
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                // 上传状态显示
                {move || upload_status.with(|status| match status {
                    UploadStatus::Uploading => view! {
                        <div class="flex items-center gap-2 text-sm text-info">
                            <span class="loading loading-spinner loading-sm"></span>
                            <span>上传中...</span>
                        </div>
                    }.into_any(),
                    UploadStatus::Error(_) => view! {
                        <div class="text-sm text-error">
                            <span>"上传失败"</span>
                        </div>
                    }.into_any(),
                    _ => view! { <div></div> }.into_any()
                })}
            </div>

            <p class="label">
                <span class="label-text-alt text-error h-2">
                    {move || error_message.with(|msg| msg.clone().unwrap_or_default())}
                </span>
            </p>
        </fieldset>
    }
}
