use crate::components::ui::file_input::FileInfo;
use crate::components::ui::form::{
    CustomValidator, DaisyForm, FieldType, FormContainer, FormField, ValidationRule,
};
use crate::components::ui::modal::Modal;
use crate::components::ui::toast::success;
use crate::server::user_handlers::{get_user, update_user};
use crate::utils::api::call_api;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::user::{CreateUserRequest, User};

// 更新用户模态框
#[component]
pub fn UpdateUserModal<F>(
    show: RwSignal<bool>,
    user_uuid: ReadSignal<String>,
    on_finish: F,
) -> impl IntoView
where
    F: Fn() + Copy + Send + 'static,
{
    let (user_data, set_user_data) = signal::<Option<User>>(None);
    let (loading, set_loading) = signal(false);

    // 创建一个信号来存储上传的头像URL
    let avatar_url = RwSignal::new(String::new());

    // 当模态框打开且有uuid时，加载用户数据
    Effect::new(move |_| {
        let show_modal = show.read();
        let uuid = user_uuid.read();

        if *show_modal && !(*uuid).is_empty() {
            set_loading.set(true);

            spawn_local(async move {
                match call_api(get_user((*uuid.clone()).to_string())).await {
                    Ok(user) => {
                        // 设置当前头像URL
                        avatar_url.set(user.avatar_url.clone().unwrap_or_default());
                        set_user_data.set(Some(user));
                    }
                    Err(_e) => {
                        set_user_data.set(None);
                    }
                }
                set_loading.set(false);
            });
        } else if !*show_modal {
            // 模态框关闭时清空数据
            set_user_data.set(None);
            avatar_url.set(String::new());
        }
    });

    let create_initial_fields = move || {
        let user = user_data.read();
        vec![
            FormField {
                name: "user_name".to_string(),
                label: "用户名".to_string(),
                field_type: FieldType::Text,
                required: true,
                value: ArcRwSignal::new(
                    user.as_ref()
                        .map(|u| u.user_name.clone())
                        .unwrap_or_default(),
                ),
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
                required: false, // 更新时密码可选
                value: ArcRwSignal::new(String::new()),
                placeholder: Some("留空则不修改密码".into()),
                error_message: ArcRwSignal::new(None),
                validation: Some(ValidationRule::Custom(CustomValidator::new(|val: &str| {
                    if val.is_empty() {
                        Ok(()) // 空密码表示不修改
                    } else {
                        let len = val.len();
                        if len < 6 {
                            Err("密码至少6个字符".into())
                        } else if len > 100 {
                            Err("密码超出100个字符".into())
                        } else {
                            Ok(())
                        }
                    }
                }))),
            },
            FormField {
                name: "email".to_string(),
                label: "邮箱".to_string(),
                field_type: FieldType::Email,
                required: false,
                value: ArcRwSignal::new(
                    user.as_ref()
                        .and_then(|u| u.email.clone())
                        .unwrap_or_default(),
                ),
                placeholder: Some("输入邮箱".into()),
                error_message: ArcRwSignal::new(None),
                validation: Some(ValidationRule::Regex(r"^[^@\s]+@[^@\s]+\.[^@\s]+$".into())),
            },
            FormField {
                name: "phone_number".to_string(),
                label: "手机号".to_string(),
                field_type: FieldType::Text,
                required: false,
                value: ArcRwSignal::new(
                    user.as_ref()
                        .and_then(|u| u.phone_number.clone())
                        .unwrap_or_default(),
                ),
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
        ]
    };

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
            avatar_url: if avatar.is_empty() { None } else { Some(avatar) },
        };
        let uuid = user_uuid.with_untracked(|value| value.clone());
        // 调用API并处理结果
        match call_api(update_user(uuid, request)).await {
            Ok(_) => {
                show.set(false);
                success("更新成功".to_string());
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
        <Modal show=show>
            <FormContainer title="修改用户">
                {move || {
                    if *loading.read() {
                        view! {
                            <div class="flex justify-center items-center p-8">
                                <span class="loading loading-spinner loading-md"></span>
                                <span class="ml-2">"加载中..."</span>
                            </div>
                        }.into_any()
                    } else {
                        let initial_fields = create_initial_fields();
                        let _current_user = user_data.read();

                        view! {
                            <div class="space-y-4">
                                <DaisyForm
                                    initial_fields
                                    on_submit=submit
                                    submit_text="更新".to_string()
                                    reset_text="取消".to_string()
                                />
                            </div>
                        }.into_any()
                    }
                }}
            </FormContainer>
        </Modal>
    }
}
