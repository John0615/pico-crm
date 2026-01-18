use crate::components::ui::form::{
    CustomValidator, DaisyForm, FieldType, FormContainer, FormField, ValidationRule,
};
use crate::components::ui::modal::Modal;
use crate::components::ui::toast::success;
use crate::server::user_handlers::{update_user, get_user};
use crate::utils::api::call_api;
use leptos::logging::log;
use leptos::prelude::*;
use leptos::task::spawn_local;
use shared::user::{User, CreateUserRequest};

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

    // 当模态框打开且有uuid时，加载用户数据
    Effect::new(move |_| {
        let show_modal = show.get();
        let uuid = user_uuid.get();
        
        if show_modal && !uuid.is_empty() {
            log!("Loading user data for uuid: {}", uuid);
            set_loading.set(true);
            
            spawn_local(async move {
                match call_api(get_user(uuid)).await {
                    Ok(user) => {
                        log!("Successfully loaded user: {:?}", user);
                        set_user_data.set(Some(user));
                    }
                    Err(e) => {
                        log!("Failed to load user: {:?}", e);
                        set_user_data.set(None);
                    }
                }
                set_loading.set(false);
            });
        } else if !show_modal {
            // 模态框关闭时清空数据
            set_user_data.set(None);
        }
    });

    let create_initial_fields = move || {
        let user = user_data.get();
        vec![
            FormField {
                name: "user_name".to_string(),
                label: "用户名".to_string(),
                field_type: FieldType::Text,
                required: true,
                value: ArcRwSignal::new(user.as_ref().map(|u| u.user_name.clone()).unwrap_or_default()),
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
                value: ArcRwSignal::new(user.as_ref().and_then(|u| u.email.clone()).unwrap_or_default()),
                placeholder: Some("输入邮箱".into()),
                error_message: ArcRwSignal::new(None),
                validation: Some(ValidationRule::Regex(r"^[^@\s]+@[^@\s]+\.[^@\s]+$".into())),
            },
            FormField {
                name: "phone_number".to_string(),
                label: "手机号".to_string(),
                field_type: FieldType::Text,
                required: false,
                value: ArcRwSignal::new(user.as_ref().and_then(|u| u.phone_number.clone()).unwrap_or_default()),
                placeholder: Some("输入手机号".into()),
                error_message: ArcRwSignal::new(None),
                validation: None,
            },
        ]
    };

    let submit = move |fields: Vec<FormField>| async move {
        let request = CreateUserRequest {
            user_name: fields[0].value.get_untracked().clone(),
            password: fields[1].value.get_untracked().clone(),
            email: if fields[2].value.get_untracked().is_empty() {
                None
            } else {
                Some(fields[2].value.get_untracked().clone())
            },
            phone_number: if fields[3].value.get_untracked().is_empty() {
                None
            } else {
                Some(fields[3].value.get_untracked().clone())
            },
        };
        let uuid = user_uuid.get_untracked();
        log!("Updating user: {} with {:?}", uuid, request);
        // 调用API并处理结果
        match call_api(update_user(uuid, request)).await {
            Ok(_) => {
                log!("更新成功");
                show.set(false);
                success("更新成功".to_string());
                on_finish();
                Ok(())
            }
            Err(e) => {
                log!("API错误: {:?}", e);
                // 根据错误类型转换
                Err(vec![e.to_string()])
            }
        }
    };

    view! {
        <Modal show=show>
            <FormContainer title="修改用户">
                {move || {
                    if loading.get() {
                        view! {
                            <div class="flex justify-center items-center p-8">
                                <span class="loading loading-spinner loading-md"></span>
                                <span class="ml-2">"加载中..."</span>
                            </div>
                        }.into_any()
                    } else {
                        let initial_fields = create_initial_fields();
                        view! {
                            <DaisyForm
                                initial_fields
                                on_submit=submit
                                submit_text="更新".to_string()
                                reset_text="取消".to_string()
                            />
                        }.into_any()
                    }
                }}
            </FormContainer>
        </Modal>
    }
}