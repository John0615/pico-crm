use crate::components::ui::form::{
    CustomValidator, DaisyForm, FieldType, FormContainer, FormField, ValidationRule,
};
use crate::components::ui::modal::Modal;
use crate::components::ui::toast::success;
use crate::server::contact_handlers::{get_contact, update_contact};
use crate::utils::api::call_api;
use leptos::logging::log;
use leptos::prelude::*;
use shared::contact::{Contact, UpdateContact};

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
        move || contact_uuid.with(|value| value.clone()),
        |uuid| async move {
            let init_contact = if uuid.is_empty() {
                Contact {
                    ..Default::default()
                }
            } else {
                match call_api(get_contact(uuid)).await {
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
                    name: "phone".to_string(),
                    label: "联系电话".to_string(),
                    field_type: FieldType::Text,
                    required: true,
                    value: ArcRwSignal::new(init_contact.phone_number.clone()),
                    placeholder: Some("输入联系电话".into()),
                    error_message: ArcRwSignal::new(None),
                    validation: Some(ValidationRule::CnMobile),
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
        let uuid = contact_uuid.with_untracked(|value| value.clone());
        let user_name = fields[0].value.with_untracked(|value| value.clone());
        let phone_number = fields[1].value.with_untracked(|value| value.clone());
        let value_level = fields[2]
            .value
            .with_untracked(|value| value.parse::<i32>().unwrap_or(1));
        let status = fields[3]
            .value
            .with_untracked(|value| value.parse::<i32>().unwrap_or(1));

        let contact = UpdateContact {
            contact_uuid: uuid,
            user_name,
            phone_number,
            value_level,
            status,
        };
        log!("Submitting: {:?}", contact);
        // 调用API并处理结果
        match call_api(update_contact(contact)).await {
            Ok(_) => {
                log!("修改成功");
                show.set(false);
                success("操作成功".to_string());
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
                            initial_fields.with(|fields| {
                                fields.as_ref().map(|fields| {
                                    view! {
                                        <DaisyForm
                                            initial_fields=fields.clone()
                                            on_submit=submit
                                            submit_text="提交".to_string()
                                            reset_text="取消".to_string()
                                        />
                                    }
                                })
                            })
                        }}

                </Transition>
            </FormContainer>
        </Modal>
    }
}
