use leptos::prelude::*;
use crate::components::ui::form::{FormContainer, TextInput, SelectInput};
use crate::components::ui::toast::{show_toast, ToastType};
use crate::components::ui::modal::Modal;
use shared::contact::Contact;
use leptos::ev::SubmitEvent;
use leptos::logging::log;

#[server]
pub async fn add_contact(contact: Contact) -> Result<(), ServerFnError> {
    use backend::presentation::handlers::contacts;
    use backend::infrastructure::db::Database;
    let pool = expect_context::<Database>();

    println!("Adding contact: {:?}", contact);
    let result = contacts::create_contact(&pool.connection, contact).await.map_err(|e| ServerFnError::new(e))?;
    println!("Adding contact results: {:?}", result);

    Ok(())
}


#[component]
pub fn ContactModal(
    show: RwSignal<bool>,
) -> impl IntoView {
    let name = RwSignal::new("".to_string());
    let company = RwSignal::new("".to_string());
    let position = RwSignal::new("".to_string());
    let phone = RwSignal::new("".to_string());
    let email = RwSignal::new("".to_string());
    let value_level = RwSignal::new(3); // 1-5级
    let status = RwSignal::new(1); // 1:活跃 2:潜在 3:不活跃
    let contact_act = ServerAction::<AddContact>::new();
    let pending = contact_act.pending();
    let result = contact_act.value();

    Effect::new(move |_| {
        if let Some(Ok(())) = result.get() {
            show.set(false);
            show_toast("操作成功".to_string(), ToastType::Success);
        } else if let Some(Ok(_)) = result.get() {
            show_toast("操作失败".to_string(), ToastType::Success);
        }
    });

    let handle_submit = move |ev: SubmitEvent| {
        ev.prevent_default();
        log!(
            "新增客户: name={}, company={}, position={}, phone={}, email={}, value={}, status={}",
            name.get(),
            company.get(),
            position.get(),
            phone.get(),
            email.get(),
            value_level.get(),
            status.get()
        );
        let contact = Contact {
            contact_uuid: "".to_string(),
            user_name: name.get(),
            company: company.get(),
            position: position.get(),
            phone_number: phone.get(),
            email: email.get(),
            last_contact: "".to_string(),
            value_level: value_level.get(),
            status: status.get(),
            ..Default::default()
        };

        contact_act.dispatch(AddContact{contact});
        log!("result: {:?}", result.get());
    };

    view! {
        <Modal show=show>
            <FormContainer title="新建客户">

                <form on:submit=handle_submit class="mt-4">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <TextInput
                            field_type="text".to_string()
                            name="name".to_string()
                            label="客户姓名".to_string()
                            value=name
                            required=true
                            placeholder="输入客户姓名".to_string()
                        />

                        <TextInput
                            field_type="text".to_string()
                            name="company".to_string()
                            label="公司名称".to_string()
                            value=company
                            required=true
                            placeholder="输入公司名称".to_string()
                        />
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <TextInput
                            field_type="text".to_string()
                            name="position".to_string()
                            label="职位".to_string()
                            value=position
                            required=true
                            placeholder="输入职位".to_string()
                        />

                        <TextInput
                            field_type="text".to_string()
                            name="phone".to_string()
                            label="联系电话".to_string()
                            value=phone
                            required=true
                            placeholder="输入联系电话".to_string()
                        />
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <TextInput
                            field_type="email".to_string()
                            name="email".to_string()
                            label="电子邮箱".to_string()
                            value=email
                            required=true
                            placeholder="输入电子邮箱".to_string()
                        />

                        <fieldset class="fieldset form-control">
                            <label class="label">
                                <span class="label-text">"客户价值"</span>
                                <span class="text-error">*</span>
                            </label>
                            <div class="rating rating-md">
                                {(1..=5).map(|level| view! {
                                    <input
                                        type="radio"
                                        name="value_level"
                                        class="mask mask-star-2 bg-orange-400"
                                        checked=move || value_level.get() == level
                                        on:click=move |_| value_level.set(level)
                                    />
                                }).collect::<Vec<_>>()}
                            </div>
                        </fieldset>
                    </div>

                    <SelectInput
                        name="status".to_string()
                        label="客户状态".to_string()
                        value=status
                        required=true
                        options=vec![
                            (1, "活跃客户".to_string()),
                            (2, "潜在客户".to_string()),
                            (3, "不活跃客户".to_string()),
                        ]
                    />

                    <div class="modal-action">
                        <button on:click= move |_| show.set(false) type="button" class="btn btn-ghost">
                            "取消"
                        </button>
                        <button type="submit" disabled={pending.get()} class="btn btn-primary">
                            "保存客户"
                        </button>
                    </div>
                </form>
            </FormContainer>
        </Modal>
    }
}
