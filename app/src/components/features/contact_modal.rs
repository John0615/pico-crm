use leptos::prelude::*;
use crate::components::ui::form::{FormContainer, TextInput, EmailInput, SelectInput};
use crate::components::ui::toast::{show_toast, ToastType};
use crate::components::ui::modal::Modal;
use leptos::ev::SubmitEvent;
use leptos::logging::log;


#[component]
pub fn ContactModal() -> impl IntoView {
    let show_modal = RwSignal::new(true);
    let name = RwSignal::new("".to_string());
    let company = RwSignal::new("".to_string());
    let position = RwSignal::new("".to_string());
    let phone = RwSignal::new("".to_string());
    let email = RwSignal::new("".to_string());
    let value_level = RwSignal::new(3); // 1-5级
    let status = RwSignal::new(1); // 1:活跃 2:潜在 3:不活跃

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
        show_toast("操作成功".to_string(), ToastType::Success);
    };

    view! {
        <Modal show=show_modal>
            <FormContainer>

                <form on:submit=handle_submit class="mt-4">
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <TextInput
                            name="name"
                            label="客户姓名"
                            value=name
                            required=true
                            placeholder="输入客户姓名"
                        />

                        <TextInput
                            name="company"
                            label="公司名称"
                            value=company
                            required=true
                            placeholder="输入公司名称"
                        />
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <TextInput
                            name="position"
                            label="职位"
                            value=position
                            required=true
                            placeholder="输入职位"
                        />

                        <TextInput
                            name="phone"
                            label="联系电话"
                            value=phone
                            required=true
                            placeholder="输入联系电话"
                        />
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <EmailInput
                            name="email"
                            label="电子邮箱"
                            value=email
                            required=true
                            placeholder="输入电子邮箱"
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
                        name="status"
                        label="客户状态"
                        value=status
                        required=true
                        options=vec![
                            (1, "活跃客户".to_string()),
                            (2, "潜在客户".to_string()),
                            (3, "不活跃客户".to_string()),
                        ]
                    />

                    <div class="modal-action">
                        <button type="button" class="btn btn-ghost">
                            "取消"
                        </button>
                        <button type="submit" class="btn btn-primary">
                            "保存客户"
                        </button>
                    </div>
                </form>
            </FormContainer>
        </Modal>
    }
}
