use leptos::ev::SubmitEvent;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use regex::Regex;
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
    Regex(String),
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
    let fields = StoredValue::new(initial_fields.clone());

    let is_submitting = RwSignal::new(false);

    // 验证单个字段
    let validate_field = move |field: &FormField| -> Option<String> {
        let value = field.value.get();
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
                ValidationRule::Regex(pattern) => {
                    if let Ok(re) = Regex::new(pattern) {
                        if !re.is_match(&value) {
                            return Some(format!("{}格式不正确", field.label));
                        }
                    }
                }
                ValidationRule::Custom(validator) => {
                    if let Err(msg) = validator.validate(&value) {
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
        for field in fields.read_value().iter() {
            let error = validate_field(field);
            let error = error.clone();
            field.error_message.set(error.clone());
            if error.is_some() {
                is_valid = false;
            }
        }
        is_valid
    };

    // 重置表单
    let reset = move || {
        for field in fields.read_value().iter() {
            field.value.set(String::new());
            field.error_message.set(None);
        }
    };

    // 提交表单
    let submit = move |ev: SubmitEvent| {
        ev.prevent_default();

        if !validate_form() {
            return;
        }

        is_submitting.set(true);

        spawn_local(async move {
            let result = on_submit(fields.read_value().to_vec().clone()).await;
            is_submitting.set(false);
            match result {
                Ok(_) => reset(),
                Err(errors) => {
                    for error in errors {
                        log!("Form error: {}", error);
                    }
                }
            };
        });
    };

    view! {
        <form class="form-control w-full max-w-md mx-auto" on:submit=submit>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <For
                    each=move || fields.read_value().to_vec()
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
                    disabled=move || is_submitting.get()
                >
                    {reset_text.unwrap_or("重置".into())}
                </button>

                <button
                    type="submit"
                    class="btn btn-primary"
                    disabled=move || is_submitting.get()
                >
                    {move || if is_submitting.get() {
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
                prop:value=value.get_untracked()
                on:input=move |ev| {
                    let new_value = event_target_value(&ev);
                    value.set(new_value.clone());
                }
            />
            <p class="label">
                <span class="label-text-alt text-error h-2">
                    {move || error_message.get().unwrap_or_default()}
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
                    checked=checked.get()
                    on:change=move |ev| checked.set(event_target_checked(&ev))
                />
                <span class="label-text">{label}</span>
            </label>
            <p class="label">
                <span class="label-text-alt text-error h-2">
                    {move || error_message.get().unwrap_or_default()}
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
                prop:value=value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
            ></textarea>
            <p class="label">
                <span class="label-text-alt text-error h-2">
                    {move || error_message.get().unwrap_or_default()}
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
                        let is_selected = move || value.get() == val;
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
                    {move || error_message.get().unwrap_or_default()}
                </span>
            </p>
        </fieldset>
    }
}
