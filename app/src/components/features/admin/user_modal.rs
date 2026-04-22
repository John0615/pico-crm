use crate::components::ui::file_input::FileInfo;
use crate::components::ui::form::{
    CustomValidator, DaisyForm, FieldType, FormContainer, FormField, ValidationRule,
};
use crate::components::ui::modal::{Modal, DETAIL_MODAL_BOX_CLASS};
use crate::components::ui::toast::{error, success};
use crate::server::user_handlers::create_user;
use crate::utils::api::call_api;
use leptos::prelude::*;
use shared::user::CreateUserRequest;

#[component]
pub fn UserModal<F>(show: RwSignal<bool>, on_finish: F) -> impl IntoView
where
    F: Fn() + Copy + Send + 'static,
{
    let avatar_url = RwSignal::new(String::new());

    Effect::new(move |_| {
        if !show.get() {
            avatar_url.set(String::new());
        }
    });

    let initial_fields = build_user_form_fields(None, avatar_url);

    let submit = move |fields: Vec<FormField>| async move {
        let avatar = avatar_url.get_untracked();
        let request = CreateUserRequest {
            user_name: field_value(&fields, "user_name"),
            password: field_value(&fields, "password"),
            email: optional_field_value(&fields, "email"),
            phone_number: optional_field_value(&fields, "phone_number"),
            employment_status: optional_field_value(&fields, "employment_status"),
            skills: parse_list_input(&field_value(&fields, "skills")),
            service_areas: parse_list_input(&field_value(&fields, "service_areas")),
            training_records: parse_list_input(&field_value(&fields, "training_records")),
            certificates: parse_list_input(&field_value(&fields, "certificates")),
            health_status: optional_field_value(&fields, "health_status"),
            employee_note: optional_field_value(&fields, "employee_note"),
            joined_at: optional_field_value(&fields, "joined_at"),
            avatar_url: (!avatar.is_empty()).then_some(avatar),
            merchant_uuid: None,
            role: Some("user".to_string()),
        };

        match call_api(create_user(request)).await {
            Ok(_) => {
                show.set(false);
                success("操作成功".to_string());
                on_finish();
                Ok(())
            }
            Err(e) => {
                error(e.to_string());
                Err(vec![e.to_string()])
            }
        }
    };

    view! {
        <Modal show=show box_class=DETAIL_MODAL_BOX_CLASS>
            <FormContainer title="新建员工" class="max-w-4xl">
                <DaisyForm
                    initial_fields
                    on_submit=submit
                    submit_text="提交".to_string()
                    reset_text="取消".to_string()
                    form_class="max-w-none".to_string()
                />
            </FormContainer>
        </Modal>
    }
}

fn build_user_form_fields(
    user: Option<&shared::user::User>,
    avatar_url: RwSignal<String>,
) -> Vec<FormField> {
    vec![
        FormField {
            name: "user_name".to_string(),
            label: "员工姓名".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: ArcRwSignal::new(user.map(|u| u.user_name.clone()).unwrap_or_default()),
            placeholder: Some("输入员工姓名".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::Custom(CustomValidator::new(|val: &str| {
                let len = val.trim().chars().count();
                if len < 2 {
                    Err("至少2个字符".into())
                } else if len > 50 {
                    Err("超出50个字符".into())
                } else {
                    Ok(())
                }
            }))),
        },
        FormField {
            name: "password".to_string(),
            label: "登录密码".to_string(),
            field_type: FieldType::Password,
            required: true,
            value: ArcRwSignal::new(String::new()),
            placeholder: Some("输入登录密码".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::Custom(CustomValidator::new(|val: &str| {
                let len = val.len();
                if len < 6 {
                    Err("密码至少6个字符".into())
                } else if len > 100 {
                    Err("密码超出100个字符".into())
                } else {
                    Ok(())
                }
            }))),
        },
        FormField {
            name: "phone_number".to_string(),
            label: "联系电话".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: ArcRwSignal::new(
                user.and_then(|u| u.phone_number.clone())
                    .unwrap_or_default(),
            ),
            placeholder: Some("输入联系电话".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::CnMobile),
        },
        FormField {
            name: "email".to_string(),
            label: "邮箱".to_string(),
            field_type: FieldType::Email,
            required: false,
            value: ArcRwSignal::new(user.and_then(|u| u.email.clone()).unwrap_or_default()),
            placeholder: Some("输入邮箱".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::Email),
        },
        FormField {
            name: "employment_status".to_string(),
            label: "员工状态".to_string(),
            field_type: FieldType::Select(employment_status_options()),
            required: true,
            value: ArcRwSignal::new(
                user.map(|u| u.employment_status.clone())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "active".to_string()),
            ),
            placeholder: None,
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "joined_at".to_string(),
            label: "入职时间".to_string(),
            field_type: FieldType::DateTimePicker,
            required: false,
            value: ArcRwSignal::new(user.and_then(|u| u.joined_at.clone()).unwrap_or_default()),
            placeholder: Some("选择入职时间".into()),
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "skills".to_string(),
            label: "技能".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: ArcRwSignal::new(user.map(|u| u.skills.join(", ")).unwrap_or_default()),
            placeholder: Some("多个技能用逗号分隔".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::Custom(CustomValidator::new(|val: &str| {
                if parse_list_input(val).len() > 12 {
                    Err("最多输入12个技能".into())
                } else {
                    Ok(())
                }
            }))),
        },
        FormField {
            name: "training_records".to_string(),
            label: "培训记录".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: ArcRwSignal::new(
                user.map(|u| u.training_records.join(", "))
                    .unwrap_or_default(),
            ),
            placeholder: Some("多个培训记录用逗号分隔".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::Custom(CustomValidator::new(|val: &str| {
                if parse_list_input(val).len() > 12 {
                    Err("最多输入12条培训记录".into())
                } else {
                    Ok(())
                }
            }))),
        },
        FormField {
            name: "certificates".to_string(),
            label: "证书".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: ArcRwSignal::new(user.map(|u| u.certificates.join(", ")).unwrap_or_default()),
            placeholder: Some("多个证书用逗号分隔".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::Custom(CustomValidator::new(|val: &str| {
                if parse_list_input(val).len() > 12 {
                    Err("最多输入12个证书".into())
                } else {
                    Ok(())
                }
            }))),
        },
        FormField {
            name: "service_areas".to_string(),
            label: "服务范围".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: ArcRwSignal::new(user.map(|u| u.service_areas.join(", ")).unwrap_or_default()),
            placeholder: Some("多个服务范围用逗号分隔".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::Custom(CustomValidator::new(|val: &str| {
                if parse_list_input(val).len() > 12 {
                    Err("最多输入12个服务范围".into())
                } else {
                    Ok(())
                }
            }))),
        },
        FormField {
            name: "health_status".to_string(),
            label: "健康状态".to_string(),
            field_type: FieldType::Select(health_status_options()),
            required: true,
            value: ArcRwSignal::new(
                user.map(|u| u.health_status.clone())
                    .filter(|value| !value.is_empty())
                    .unwrap_or_else(|| "healthy".to_string()),
            ),
            placeholder: None,
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "employee_note".to_string(),
            label: "员工备注".to_string(),
            field_type: FieldType::TextArea,
            required: false,
            value: ArcRwSignal::new(
                user.and_then(|u| u.employee_note.clone())
                    .unwrap_or_default(),
            ),
            placeholder: Some("补充员工说明，例如擅长项目、时间限制".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::MaxLength(500)),
        },
        FormField {
            name: "avatar".to_string(),
            label: "头像".to_string(),
            field_type: FieldType::File {
                accept: Some("image/*".to_string()),
                multiple: false,
                max_size: Some(5 * 1024 * 1024),
                on_upload: Some(Callback::new({
                    let avatar_url = avatar_url;
                    move |file_info: FileInfo| {
                        let avatar_url = avatar_url;
                        let file_name = file_info.name.clone();
                        let file_data = file_info.data.clone();
                        let content_type = file_info.file_type.clone();

                        leptos::task::spawn_local(async move {
                            if let Ok(response) =
                                crate::utils::file_upload::upload_file_info_with_data(
                                    file_name,
                                    file_data,
                                    content_type,
                                )
                                .await
                            {
                                avatar_url.set(response.file_url);
                            }
                        });
                    }
                })),
            },
            required: false,
            value: {
                let avatar_field_value =
                    ArcRwSignal::new(user.and_then(|u| u.avatar_url.clone()).unwrap_or_default());
                Effect::new({
                    let avatar_field_value = avatar_field_value.clone();
                    move |_| {
                        avatar_field_value.set(avatar_url.get());
                    }
                });
                avatar_field_value
            },
            placeholder: None,
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
    ]
}

fn employment_status_options() -> Vec<(String, String)> {
    vec![
        ("active".to_string(), "在岗".to_string()),
        ("on_leave".to_string(), "休假".to_string()),
        ("resigned".to_string(), "离职".to_string()),
    ]
}

fn health_status_options() -> Vec<(String, String)> {
    vec![
        ("healthy".to_string(), "健康".to_string()),
        ("attention".to_string(), "需关注".to_string()),
        ("expired".to_string(), "已过期".to_string()),
    ]
}

fn field_value(fields: &[FormField], name: &str) -> String {
    fields
        .iter()
        .find(|field| field.name == name)
        .map(|field| field.value.get_untracked())
        .unwrap_or_default()
}

fn optional_field_value(fields: &[FormField], name: &str) -> Option<String> {
    let value = field_value(fields, name);
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn parse_list_input(value: &str) -> Vec<String> {
    let mut items = Vec::new();
    for item in value.split([',', '，']) {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }
        if items.iter().any(|existing| existing == trimmed) {
            continue;
        }
        items.push(trimmed.to_string());
    }
    items
}
