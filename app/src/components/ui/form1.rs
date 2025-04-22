use leptos::prelude::*;
use std::collections::HashMap;

/// 表单字段类型
#[derive(Debug, Clone, PartialEq)]
pub enum FieldType {
    Text,
    Email,
    Password,
    TextArea,
    Checkbox,
    Select,
    // 可扩展其他类型
}

/// 表单字段配置
#[derive(Debug, Clone)]
pub struct FieldConfig {
    pub field_type: FieldType,
    pub name: String,
    pub label: String,
    pub placeholder: Option<String>,
    pub required: bool,
    pub options: Option<Vec<(String, String)>>, // 用于Select
    pub validation: Option<FieldValidation>,
}

/// 字段验证配置
#[derive(Debug, Clone)]
pub struct FieldValidation {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub custom_validator: Option<fn(&str) -> Result<(), String>>,
}

/// 表单状态
#[derive(Debug, Clone)]
pub struct FormState {
    pub values: RwSignal<HashMap<String, String>>,
    pub errors: RwSignal<HashMap<String, String>>,
    pub is_submitting: RwSignal<bool>,
    pub is_dirty: RwSignal<bool>,
}

impl FormState {
    pub fn new(fields: &[FieldConfig]) -> Self {
        let initial_values = fields.iter()
            .map(|f| (f.name.clone(), String::new()))
            .collect();

        Self {
            values: RwSignal::new(initial_values),
            errors: RwSignal::new(HashMap::new()),
            is_submitting: RwSignal::new(false),
            is_dirty: RwSignal::new(false),
        }
    }

    pub fn validate(&self, fields: &[FieldConfig]) -> bool {
        let mut has_errors = false;
        let mut new_errors = HashMap::new();

        for field in fields {
            let value = self.values.with(|v| v.get(&field.name).cloned().unwrap_or_default());

            if field.required && value.is_empty() {
                new_errors.insert(field.name.clone(), format!("{} 是必填项", field.label));
                has_errors = true;
                continue;
            }

            if let Some(validation) = &field.validation {
                if let Some(min) = validation.min_length {
                    if value.len() < min {
                        new_errors.insert(
                            field.name.clone(),
                            format!("{} 至少需要 {} 个字符", field.label, min)
                        );
                        has_errors = true;
                    }
                }

                if let Some(max) = validation.max_length {
                    if value.len() > max {
                        new_errors.insert(
                            field.name.clone(),
                            format!("{} 最多不能超过 {} 个字符", field.label, max)
                        );
                        has_errors = true;
                    }
                }

                if let Some(pattern) = &validation.pattern {
                    let regex = regex::Regex::new(pattern).unwrap();
                    if !regex.is_match(&value) {
                        new_errors.insert(
                            field.name.clone(),
                            format!("{} 格式不正确", field.label)
                        );
                        has_errors = true;
                    }
                }

                if let Some(validator) = validation.custom_validator {
                    if let Err(msg) = validator(&value) {
                        new_errors.insert(field.name.clone(), msg);
                        has_errors = true;
                    }
                }
            }
        }

        self.errors.set(new_errors);
        !has_errors
    }

    pub fn reset(&self) {
        self.values.update(|v| {
            for value in v.values_mut() {
                *value = String::new();
            }
        });
        self.errors.set(HashMap::new());
        self.is_dirty.set(false);
    }
}


#[component]
pub fn FormContainer(
    #[prop(optional)] title: Option<&'static str>,
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
fn FieldContainer<F, IV>(
    label: String,
    children: Children,
    error: F,
) -> impl IntoView
where
    F: Fn() -> IV,
    IV: IntoView,
{
    view! {
        <fieldset class="form-control">
            <label class="label">
                <span class="label-text">{label}</span>
            </label>
            {children()}
            {error()}
        </fieldset>
    }
}


#[component]
pub fn FormField(
    field: FieldConfig,
    form_state: FormState,
) -> impl IntoView {
    // 1. 统一错误处理（使用闭包捕获）
    let error_view = {
        let field_name = field.name.clone();
        move || {
            form_state.errors.with(|errors| {
                errors.get(&field_name)
                    .map(|e| view! { <p class="error">{e.clone()}</p> })
            })
        }
    };

    let field_value = {
        let field_name = field.name.clone();
        move || {
            form_state.values.with(|v| v.get(&field_name).cloned().unwrap_or_default())
        }
    };

    let has_error = {
        let field_name = field.name.clone();
        move || {
            form_state.errors.with(|e| e.contains_key(&field_name))
        }
    };

    let on_input = {
        let field_name = field.name.clone();
        move |ev| {
            let value = event_target_value(&ev);
            form_state.values.update(|v| {
                v.insert(field_name.clone(), value);
            });
            form_state.is_dirty.set(true);
        }
    };

    match field.field_type {
        FieldType::Text | FieldType::Email | FieldType::Password => {
            let input_type = match field.field_type {
                FieldType::Email => "email",
                FieldType::Password => "password",
                _ => "text"
            };
            let field_name = field.name.clone();

            view! {
                <FieldContainer
                    label=field.label.clone()
                    error=error_view
                >
                    <input
                        type=input_type
                        name=field_name.clone()
                        class=move || format!(
                            "input input-bordered {}",
                            if has_error() { "input-error" } else { "" }
                        )
                        placeholder=field.placeholder.clone()
                        required=field.required
                        prop:value=field_value()
                        on:input=on_input
                    />
                </FieldContainer>
            }.into_any()
        },

        FieldType::TextArea => {
            let field_name = field.name.clone();
            view! {
                <FieldContainer
                    label=field.label.clone()
                    error=error_view
                >
                    <textarea
                        name=field_name.clone()
                        class=move || format!(
                            "textarea textarea-bordered {}",
                            if has_error() { "textarea-error" } else { "" }
                        )
                        placeholder=field.placeholder.clone()
                        required=field.required
                        prop:value=field_value()
                        on:input=on_input
                    />
                </FieldContainer>
            }.into_any()
        },

        FieldType::Checkbox => {
            let field_name = field.name.clone();
            view! {
                <FieldContainer
                    label=field.label.clone()
                    error=error_view
                >
                    <label class="label cursor-pointer justify-start gap-2">
                        <input
                            type="checkbox"
                            name=field_name.clone()
                            class="checkbox checkbox-primary"
                            checked={
                                let field_name = field_name.clone();
                                move || {
                                    form_state.values.with(|v|
                                        v.get(&field_name).map(|s| s == "true").unwrap_or(false)
                                    )
                                }
                            }
                            on:change=move |ev| {
                                let checked = event_target_checked(&ev);
                                form_state.values.update(|v| {
                                    v.insert(field_name.clone(), checked.to_string());
                                });
                                form_state.is_dirty.set(true);
                            }
                        />
                        <span class="label-text">{field.label.clone()}</span>
                    </label>
               </FieldContainer>
            }.into_any()
        },

        FieldType::Select => {
            let field_name = field.name.clone();
            view! {
                <FieldContainer
                    label=field.label.clone()
                    error=error_view
                >
                    <select
                        name=field_name.clone()
                        class="select select-bordered w-full"
                        required=field.required
                        prop:value=field_value()
                        on:change=on_input
                    >
                        {field.options.unwrap_or_default().into_iter().map(|option| {
                            view! {
                                <option value={option.0.clone()}>
                                    {option.1.clone()}
                                </option>
                            }
                        }).collect::<Vec<_>>()}
                    </select>
                </FieldContainer>
            }.into_any()
        },

    }
}


#[component]
pub fn FormActions(
    #[prop(optional)] justify_end: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!(
            "flex {} gap-2 mt-6",
            if justify_end { "justify-end" } else { "justify-start" }
        )>
            {children()}
        </div>
    }
}
