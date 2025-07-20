use crate::components::ui::form::{
    CustomValidator, DaisyForm, FieldType, FormContainer, FormField, ValidationRule,
};
use crate::components::ui::modal::Modal;
use crate::components::ui::toast::success;
use leptos::logging::log;
use leptos::prelude::*;
use shared::contact::{Contact, UpdateContact};

#[cfg(feature = "ssr")]
mod ssr {
    pub use backend::application::services::contact_service::ContactAppService;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::repositories::contact_repository_impl::SeaOrmContactRepository;
}

#[server]
pub async fn get_contact(uuid: String) -> Result<Option<Contact>, ServerFnError> {
    use self::ssr::*;

    let pool = expect_context::<Database>();

    let contact_repository = SeaOrmContactRepository::new(pool.connection.clone());
    let app_service = ContactAppService::new(contact_repository);

    println!("fetch contact uuid: {:?}", uuid);
    let result = app_service
        .fetch_contact(uuid)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    println!("fetch contact result: {:?}", result);

    Ok(result)
}

#[server]
pub async fn edit_contact(contact: UpdateContact) -> Result<(), ServerFnError> {
    use self::ssr::*;

    let pool = expect_context::<Database>();

    let contact_repository = SeaOrmContactRepository::new(pool.connection.clone());
    let app_service = ContactAppService::new(contact_repository);

    println!("editing contact: {:?}", contact);
    let result = app_service
        .update_contact(contact)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    println!("editing contact results: {:?}", result);

    Ok(())
}

#[component]
pub fn UpdateContactModal<F>(
    show: RwSignal<bool>,
    contact_uuid: ReadSignal<String>,
    on_finish: F,
) -> impl IntoView
where
    F: Fn() + Copy + Send + 'static,
{
    let initial_fields = Resource::new(
        move || (contact_uuid.get()),
        |uuid| async move {
            let init_contact = if uuid.is_empty() {
                Contact {
                    ..Default::default()
                }
            } else {
                match get_contact(uuid).await {
                    Ok(Some(contact)) => contact,
                    _ => Contact {
                        ..Default::default()
                    },
                }
            };

            vec![
                FormField {
                    name: "name".to_string(),
                    label: "客户姓名".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    value: ArcRwSignal::new(init_contact.user_name.clone()),
                    placeholder: Some("输入客户姓名".into()),
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
                    name: "company".to_string(),
                    label: "公司名称".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    value: ArcRwSignal::new(init_contact.company.clone()),
                    placeholder: Some("输入公司名称".into()),
                    error_message: ArcRwSignal::new(None),
                    validation: Some(ValidationRule::MinLength(2)),
                },
                FormField {
                    name: "position".to_string(),
                    label: "职位".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    value: ArcRwSignal::new(init_contact.position.clone()),
                    placeholder: Some("输入职位".into()),
                    error_message: ArcRwSignal::new(None),
                    validation: None,
                },
                FormField {
                    name: "phone".to_string(),
                    label: "联系电话".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    value: ArcRwSignal::new(init_contact.phone_number.clone()),
                    placeholder: Some("输入联系电话".into()),
                    error_message: ArcRwSignal::new(None),
                    validation: Some(ValidationRule::Regex(
                        r"^1[3-9]\d{9}$".into(), // 中国手机号正则
                    )),
                },
                FormField {
                    name: "email".to_string(),
                    label: "电子邮箱".to_string(),
                    field_type: FieldType::Email,
                    required: true,
                    value: ArcRwSignal::new(init_contact.email.clone()),
                    placeholder: Some("输入电子邮箱".to_string()),
                    error_message: ArcRwSignal::new(None),
                    validation: Some(ValidationRule::Regex(
                        r"^[^@\s]+@[^@\s]+\.[^@\s]+$".into(), // 基础邮箱验证
                    )),
                },
                FormField {
                    name: "value_level".to_string(),
                    label: "客户价值".to_string(),
                    field_type: FieldType::Select(vec![
                        ("".to_string(), "请选择客户价值".to_string()),
                        ("1".to_string(), "活跃客户".to_string()),
                        ("2".to_string(), "潜在客户".to_string()),
                        ("3".to_string(), "不活跃客户".to_string()),
                    ]),
                    required: true,
                    value: ArcRwSignal::new(init_contact.value_level.to_string().clone()), // 默认选中3星
                    placeholder: None,
                    error_message: ArcRwSignal::new(None),
                    validation: None,
                },
                FormField {
                    name: "status".to_string(),
                    label: "客户状态".to_string(),
                    field_type: FieldType::Select(vec![
                        ("".to_string(), "请选择客户状态".to_string()),
                        ("1".to_string(), "已签约".to_string()),
                        ("2".to_string(), "待跟进".to_string()),
                        ("3".to_string(), "已流失".to_string()),
                    ]),
                    required: true,
                    value: ArcRwSignal::new(init_contact.status.to_string().clone()),
                    placeholder: None,
                    error_message: ArcRwSignal::new(None),
                    validation: None,
                },
            ]
        },
    );

    let submit = move |fields: Vec<FormField>| async move {
        let contact = UpdateContact {
            contact_uuid: contact_uuid.get_untracked(),
            user_name: fields[0].value.get_untracked(),
            company: fields[1].value.get_untracked(),
            position: fields[2].value.get_untracked(),
            phone_number: fields[3].value.get_untracked(),
            email: fields[4].value.get_untracked(),
            value_level: fields[5].value.get_untracked().parse::<i32>().unwrap_or(1),
            status: fields[6].value.get_untracked().parse::<i32>().unwrap_or(1),
        };
        log!("Submitting: {:?}", contact);
        // 调用API并处理结果
        match edit_contact(contact).await {
            Ok(_) => {
                log!("修改成功");
                show.set(false);
                success("操作成功".to_string());
                on_finish();
                Ok(())
            }
            Err(e) => {
                log!("API错误: {:?}", e);
                success("操作失败".to_string());
                // 根据错误类型转换
                Err(vec![e.to_string()])
            }
        }
    };

    view! {
        <Modal show=show>
            <FormContainer title="修改客户">
                <Transition
                    fallback=move || view! {
                        <tr class="h-[calc(100vh-300px)]">
                            <td colspan="9" class="h-32 text-center align-middle">
                                <span class="loading loading-bars loading-xl"></span>
                            </td>
                        </tr>
                    }
                >
                    {move || {
                            initial_fields.get().map(|fields| {
                                view! {
                                    <DaisyForm
                                        initial_fields=fields
                                        on_submit=submit
                                        submit_text="提交".to_string()
                                        reset_text="取消".to_string()
                                    />
                                }
                            })
                        }}

                </Transition>
            </FormContainer>
        </Modal>
    }
}
