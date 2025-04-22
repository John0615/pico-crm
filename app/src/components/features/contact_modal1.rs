use leptos::prelude::*;
use crate::components::ui::form1::*;
use crate::components::ui::modal::Modal;
use leptos::logging::log;

#[component]
pub fn ContactModal1(show: RwSignal<bool>) -> impl IntoView {
    let submit_form = Callback::new(move |form_state: RwSignal<FormState>| {
        // 处理表单提交
        let current_state = form_state.get();
        log!("提交数据1: {:?}", current_state);
    });
    view! {
        <Modal show=show>
            <ContactForm submit_form=submit_form  />
        </Modal>
    }
}

#[component]
pub fn ContactForm(#[prop(into)] submit_form: Callback<RwSignal<FormState>>) -> impl IntoView {
    // 定义表单字段
    let fields = vec![
        FieldConfig {
            field_type: FieldType::Text,
            name: "user_name".to_string(),
            label: "客户姓名".to_string(),
            placeholder: Some("输入客户姓名".to_string()),
            required: true,
            options: None,
            validation: Some(FieldValidation {
                min_length: Some(1),
                max_length: Some(20),
                pattern: None,
                custom_validator: None,
            }),
        },
        FieldConfig {
            field_type: FieldType::Text,
            name: "company".to_string(),
            label: "公司名称".to_string(),
            placeholder: Some("输入公司名称".to_string()),
            required: true,
            options: None,
            validation: Some(FieldValidation {
                min_length: Some(1),
                max_length: Some(100),
                pattern: None,
                custom_validator: None,
            }),
        },
        FieldConfig {
            field_type: FieldType::Text,
            name: "position".to_string(),
            label: "职位".to_string(),
            placeholder: Some("输入职位".to_string()),
            required: true,
            options: None,
            validation: Some(FieldValidation {
                min_length: Some(1),
                max_length: Some(50),
                pattern: None,
                custom_validator: None,
            }),
        },
        FieldConfig {
            field_type: FieldType::Text,
            name: "phone".to_string(),
            label: "联系电话".to_string(),
            placeholder: Some("输入联系电话".to_string()),
            required: true,
            options: None,
            validation: Some(FieldValidation {
                min_length: Some(1),
                max_length: Some(50),
                pattern: None,
                custom_validator: None,
            }),
        },
        FieldConfig {
            field_type: FieldType::Text,
            name: "email".to_string(),
            label: "电子邮箱".to_string(),
            placeholder: Some("输入电子邮箱".to_string()),
            required: true,
            options: None,
            validation: Some(FieldValidation {
                min_length: Some(1),
                max_length: Some(100),
                pattern: Some(r"^[^@\s]+@[^@\s]+\.[^@\s]+$".to_string()),
                custom_validator: None,
            }),
        },
        FieldConfig {
            field_type: FieldType::Select,
            name: "value_level".to_string(),
            label: "客户价值".to_string(),
            placeholder: None,
            required: true,
            options: Some(vec![
                ("".to_string(), "请选择".to_string()),
                ("一级".to_string(), "1".to_string()),
                ("二级".to_string(), "2".to_string()),
                ("三级".to_string(), "3".to_string()),
                ("四级".to_string(), "4".to_string()),
                ("五级".to_string(), "5".to_string()),
            ]),
            validation: None,
        },
        FieldConfig {
            field_type: FieldType::Select,
            name: "status".to_string(),
            label: "客户状态".to_string(),
            placeholder: None,
            required: true,
            options: Some(vec![
                ("".to_string(), "请选择".to_string()),
                ("活跃".to_string(), "2".to_string()),
                ("潜在".to_string(), "1".to_string()),
                ("不活跃".to_string(), "0".to_string()),
            ]),
            validation: None,
        },
    ];

    let form_state = RwSignal::new(FormState::new(&fields));

    let submit = {
        let fields = fields.clone();
        move |_| {
            if form_state.get().validate(&fields) {
                form_state.get().is_submitting.set(true);
                // 提交逻辑...
                log!("提交数据: {:?}", form_state.get().values.get());
                submit_form.run(form_state);
            }
        }
    };

    view! {
        <FormContainer title="新建客户" class="p-6 rounded-lg shadow">
            <For
                each=move || fields.clone()
                key=|field| field.name.clone()
                children={
                    move |field| {
                        view! {
                            <FormField field=field form_state=form_state.get()/>
                        }
                    }
                }
            />

            <FormActions justify_end=true>
                <button
                    type="button"
                    class="btn btn-ghost"
                    on:click={
                        move |_| form_state.get().reset()
                    }
                >
                    "重置"
                </button>
                <button
                    type="button"
                    class="btn btn-primary"
                    on:click=submit
                    disabled= move || form_state.get().is_submitting.get()
                >
                    {
                        move || if form_state.get().is_submitting.get() {
                            "提交中..."
                        } else {
                            "提交"
                        }
                    }
                </button>
            </FormActions>
        </FormContainer>
    }
}
