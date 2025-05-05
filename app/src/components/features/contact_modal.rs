use crate::components::ui::form::{
    CustomValidator, DaisyForm, FieldType, FormContainer, FormField, ValidationRule,
};
use crate::components::ui::modal::Modal;
use crate::components::ui::toast::{show_toast, ToastType};
use leptos::logging::log;
use leptos::prelude::*;
use shared::contact::Contact;

#[server]
pub async fn add_contact(contact: Contact) -> Result<(), ServerFnError> {
    use backend::application::services::contact_service;
    use backend::infrastructure::db::Database;
    let pool = expect_context::<Database>();

    println!("Adding contact: {:?}", contact);
    let result = contact_service::create_contact(&pool.connection, contact)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    println!("Adding contact results: {:?}", result);

    Ok(())
}

#[component]
pub fn ContactModal<F>(show: RwSignal<bool>, on_finish: F) -> impl IntoView
where
    F: Fn() + Copy + Send + 'static,
{
    let initial_fields = vec![
        FormField {
            name: "name".to_string(),
            label: "客户姓名".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: RwSignal::new("".to_string()),
            placeholder: Some("输入客户姓名".into()),
            error_message: RwSignal::new(None),
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
            value: RwSignal::new("".to_string()),
            placeholder: Some("输入公司名称".into()),
            error_message: RwSignal::new(None),
            validation: Some(ValidationRule::MinLength(2)),
        },
        FormField {
            name: "position".to_string(),
            label: "职位".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: RwSignal::new("".to_string()),
            placeholder: Some("输入职位".into()),
            error_message: RwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "phone".to_string(),
            label: "联系电话".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: RwSignal::new("".to_string()),
            placeholder: Some("输入联系电话".into()),
            error_message: RwSignal::new(None),
            validation: Some(ValidationRule::Regex(
                r"^1[3-9]\d{9}$".into(), // 中国手机号正则
            )),
        },
        FormField {
            name: "email".to_string(),
            label: "电子邮箱".to_string(),
            field_type: FieldType::Email,
            required: true,
            value: RwSignal::new("".to_string()),
            placeholder: Some("输入电子邮箱".to_string()),
            error_message: RwSignal::new(None),
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
            value: RwSignal::new("".to_string()), // 默认选中3星
            placeholder: None,
            error_message: RwSignal::new(None),
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
            value: RwSignal::new("".to_string()), // 默认值
            placeholder: None,
            error_message: RwSignal::new(None),
            validation: None,
        },
    ];
    let submit = move |fields: Vec<FormField>| async move {
        let contact = Contact {
            user_name: fields[0].value.get_untracked().clone(),
            company: fields[1].value.get_untracked().clone(),
            position: fields[2].value.get_untracked().clone(),
            phone_number: fields[3].value.get_untracked().clone(),
            email: fields[4].value.get_untracked().clone(),
            value_level: fields[5].value.get_untracked().parse::<i32>().unwrap_or(1),
            status: fields[6].value.get_untracked().parse::<i32>().unwrap_or(1),
            ..Default::default()
        };
        log!("Submitting: {:?}", contact);
        // 调用API并处理结果
        match add_contact(contact).await {
            Ok(_) => {
                log!("添加成功");
                show.set(false);
                show_toast("操作成功".to_string(), ToastType::Success);
                on_finish();
                Ok(())
            }
            Err(e) => {
                log!("API错误: {:?}", e);
                show_toast("操作失败".to_string(), ToastType::Success);
                // 根据错误类型转换
                Err(vec![e.to_string()])
            }
        }
    };

    view! {
        <Modal show=show>
            <FormContainer title="新建客户">
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
