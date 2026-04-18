use crate::components::ui::form::{
    CustomValidator, DaisyForm, FieldType, FormContainer, FormField, ValidationRule,
};
use crate::components::ui::modal::{Modal, DETAIL_MODAL_BOX_CLASS};
use crate::components::ui::toast::{error, success};
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
                Contact::default()
            } else {
                match call_api(get_contact(uuid)).await {
                    Ok(Some(contact)) => contact,
                    _ => Contact::default(),
                }
            };

            build_contact_form_fields(&init_contact)
        },
    );

    let submit = move |fields: Vec<FormField>| async move {
        let uuid = contact_uuid.with_untracked(|value| value.clone());
        let contact = UpdateContact {
            contact_uuid: uuid,
            user_name: field_value(&fields, "name"),
            phone_number: field_value(&fields, "phone"),
            address: optional_field_value(&fields, "address"),
            community: optional_field_value(&fields, "community"),
            building: optional_field_value(&fields, "building"),
            house_area_sqm: optional_field_value(&fields, "house_area_sqm")
                .and_then(|value| value.parse::<i32>().ok()),
            service_need: optional_field_value(&fields, "service_need"),
            tags: parse_tags_input(&field_value(&fields, "tags")),
            last_service_at: optional_field_value(&fields, "last_service_at"),
            follow_up_status: optional_field_value(&fields, "follow_up_status"),
        };

        log!("Submitting: {:?}", contact);
        match call_api(update_contact(contact)).await {
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
            <FormContainer title="修改客户" class="max-w-4xl">
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
                                        form_class="max-w-none".to_string()
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

fn build_contact_form_fields(contact: &Contact) -> Vec<FormField> {
    vec![
        FormField {
            name: "name".to_string(),
            label: "客户姓名".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: ArcRwSignal::new(contact.user_name.clone()),
            placeholder: Some("输入客户姓名".into()),
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
            name: "phone".to_string(),
            label: "联系电话".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: ArcRwSignal::new(contact.phone_number.clone()),
            placeholder: Some("输入联系电话".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::CnMobile),
        },
        FormField {
            name: "community".to_string(),
            label: "小区/社区".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: ArcRwSignal::new(contact.community.clone().unwrap_or_default()),
            placeholder: Some("例如：万科城市花园".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::MaxLength(64)),
        },
        FormField {
            name: "building".to_string(),
            label: "楼栋/门牌".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: ArcRwSignal::new(contact.building.clone().unwrap_or_default()),
            placeholder: Some("例如：3号楼1202".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::MaxLength(64)),
        },
        FormField {
            name: "address".to_string(),
            label: "详细地址".to_string(),
            field_type: FieldType::TextArea,
            required: false,
            value: ArcRwSignal::new(contact.address.clone().unwrap_or_default()),
            placeholder: Some("输入详细服务地址".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::MaxLength(255)),
        },
        FormField {
            name: "house_area_sqm".to_string(),
            label: "房屋面积(㎡)".to_string(),
            field_type: FieldType::Number,
            required: false,
            value: ArcRwSignal::new(
                contact
                    .house_area_sqm
                    .map(|value| value.to_string())
                    .unwrap_or_default(),
            ),
            placeholder: Some("例如：90".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::Custom(CustomValidator::new(|val: &str| {
                let trimmed = val.trim();
                if trimmed.is_empty() {
                    return Ok(());
                }
                match trimmed.parse::<i32>() {
                    Ok(value) if value >= 0 => Ok(()),
                    _ => Err("请输入非负整数".into()),
                }
            }))),
        },
        FormField {
            name: "follow_up_status".to_string(),
            label: "跟进状态".to_string(),
            field_type: FieldType::Select(follow_up_status_options()),
            required: false,
            value: ArcRwSignal::new(
                contact
                    .follow_up_status
                    .clone()
                    .unwrap_or_else(|| "pending".to_string()),
            ),
            placeholder: None,
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "tags".to_string(),
            label: "标签".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: ArcRwSignal::new(contact.tags.join(", ")),
            placeholder: Some("多个标签用逗号分隔".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::Custom(CustomValidator::new(|val: &str| {
                let tags = parse_tags_input(val);
                if tags.len() > 8 {
                    Err("最多输入8个标签".into())
                } else {
                    Ok(())
                }
            }))),
        },
        FormField {
            name: "last_service_at".to_string(),
            label: "最近服务时间".to_string(),
            field_type: FieldType::DateTimePicker,
            required: false,
            value: ArcRwSignal::new(contact.last_service_at.clone().unwrap_or_default()),
            placeholder: Some("选择最近一次服务时间".into()),
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "service_need".to_string(),
            label: "服务需求".to_string(),
            field_type: FieldType::TextArea,
            required: false,
            value: ArcRwSignal::new(contact.service_need.clone().unwrap_or_default()),
            placeholder: Some("例如：深度保洁、每周一次".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::MaxLength(500)),
        },
    ]
}

fn follow_up_status_options() -> Vec<(String, String)> {
    vec![
        ("pending".to_string(), "待跟进".to_string()),
        ("contacted".to_string(), "已联系".to_string()),
        ("quoted".to_string(), "已报价".to_string()),
        ("scheduled".to_string(), "已预约".to_string()),
        ("completed".to_string(), "已完成".to_string()),
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

fn parse_tags_input(value: &str) -> Vec<String> {
    let mut tags = Vec::new();
    for item in value.split([',', '，']) {
        let trimmed = item.trim();
        if trimmed.is_empty() {
            continue;
        }
        if tags.iter().any(|tag| tag == trimmed) {
            continue;
        }
        tags.push(trimmed.to_string());
    }
    tags
}
