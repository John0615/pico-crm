use crate::components::ui::file_input::FileInfo;
use crate::components::ui::form::{
    CustomValidator, DaisyForm, FieldType, FormContainer, FormField, ValidationRule,
};
use crate::components::ui::modal::Modal;
use crate::components::ui::toast::success;
use crate::server::user_handlers::create_user;
use crate::utils::api::call_api;
use leptos::prelude::*;
use shared::user::CreateUserRequest;

// 新建员工模态框
#[component]
pub fn UserModal<F>(show: RwSignal<bool>, on_finish: F) -> impl IntoView
where
    F: Fn() + Copy + Send + 'static,
{
    // 创建一个信号来存储上传的头像URL
    let avatar_url = RwSignal::new(String::new());

    // 当模态框关闭时清理头像URL
    Effect::new(move |_| {
        let show_modal = *show.read();
        if !show_modal {
            avatar_url.set(String::new());
        }
    });

    let initial_fields = vec![
        FormField {
            name: "user_name".to_string(),
            label: "用户名".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: ArcRwSignal::new(String::new()),
            placeholder: Some("输入用户名".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::Custom(CustomValidator::new(|val: &str| {
                let len = val.len();
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
            label: "密码".to_string(),
            field_type: FieldType::Password,
            required: true,
            value: ArcRwSignal::new(String::new()),
            placeholder: Some("输入密码".into()),
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
            name: "email".to_string(),
            label: "邮箱".to_string(),
            field_type: FieldType::Email,
            required: false,
            value: ArcRwSignal::new(String::new()),
            placeholder: Some("输入邮箱".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::Email),
        },
        FormField {
            name: "phone_number".to_string(),
            label: "手机号".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: ArcRwSignal::new(String::new()),
            placeholder: Some("输入手机号".into()),
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "avatar".to_string(),
            label: "头像".to_string(),
            field_type: FieldType::File {
                accept: Some("image/*".to_string()),
                multiple: false,
                max_size: Some(5 * 1024 * 1024), // 5MB
                on_upload: Some(Callback::new({
                    let avatar_url = avatar_url.clone();
                    move |file_info: FileInfo| {
                        // 异步上传文件到服务器
                        let avatar_url = avatar_url.clone();
                        let file_name = file_info.name.clone();
                        let file_data = file_info.data.clone();
                        let content_type = file_info.file_type.clone();

                        leptos::task::spawn_local(async move {
                            match crate::utils::file_upload::upload_file_info_with_data(
                                file_name,
                                file_data,
                                content_type,
                            )
                            .await
                            {
                                Ok(response) => {
                                    avatar_url.set(response.file_url);
                                }
                                Err(_e) => {
                                    // Handle error silently or show user feedback
                                }
                            }
                        });
                    }
                })),
            },
            required: false,
            value: {
                let avatar_field_value = ArcRwSignal::new(String::new());
                // 创建一个响应式效果，当avatar_url改变时更新字段值
                Effect::new({
                    let avatar_url = avatar_url.clone();
                    let avatar_field_value = avatar_field_value.clone();
                    move |_| {
                        avatar_field_value.set(avatar_url.with(|url| url.clone()));
                    }
                });
                avatar_field_value
            },
            placeholder: None,
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
    ];

    let submit = move |fields: Vec<FormField>| async move {
        let user_name = fields[0].value.with_untracked(|value| value.clone());
        let password = fields[1].value.with_untracked(|value| value.clone());
        let email = fields[2].value.with_untracked(|value| value.clone());
        let phone_number = fields[3].value.with_untracked(|value| value.clone());
        let avatar = avatar_url.with_untracked(|value| value.clone());

        let request = CreateUserRequest {
            user_name,
            password,
            email: if email.is_empty() { None } else { Some(email) },
            phone_number: if phone_number.is_empty() {
                None
            } else {
                Some(phone_number)
            },
            avatar_url: if avatar.is_empty() {
                None
            } else {
                Some(avatar)
            },
            merchant_uuid: None,
            role: None,
        };
        // 调用API并处理结果
        match call_api(create_user(request)).await {
            Ok(_) => {
                show.set(false);
                success("操作成功".to_string());
                on_finish();
                Ok(())
            }
            Err(e) => {
                // 根据错误类型转换
                Err(vec![e.to_string()])
            }
        }
    };

    view! {
        <Modal show=show box_class="max-h-none overflow-visible">
            <FormContainer title="新建员工">
                <DaisyForm
                    initial_fields
                    on_submit=submit
                    submit_text="提交".to_string()
                    reset_text="取消".to_string()
                />
            </FormContainer>
        </Modal>
    }
}
