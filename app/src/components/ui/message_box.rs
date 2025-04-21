use leptos::prelude::*;

#[component]
pub fn MessageBox(
    title: String,
    message: String,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let (is_visible, set_is_visible) = signal(true);

    view! {
        <Show
            when=move || is_visible.get()
            fallback=|| ()
        >
            <div class="fixed inset-0 flex items-center justify-center bg-black bg-opacity-50 z-50">
                <div class="modal modal-open">
                    <div class="modal-box">
                        <h3 class="font-bold text-lg">{title.clone()}</h3>
                        <p class="py-4">{message.clone()}</p>
                        <div class="modal-action">
                            <button
                                class="btn btn-primary"
                                on:click=move |_| {
                                    on_confirm.run(());
                                    set_is_visible.set(false);
                                }
                            >
                                "确认"
                            </button>
                            <button
                                class="btn btn-ghost"
                                on:click=move |_| {
                                    on_cancel.run(());
                                    set_is_visible.set(false);
                                }
                            >
                                "取消"
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

pub fn confirm(
    title: &str,
    message: &str,
    on_result: impl Fn(bool) + Clone + Send + Sync + 'static,
) {
    let title = title.to_string();
    let message = message.to_string();

    mount_to_body(move || {
        let on_confirm = {
            let on_result = on_result.clone();
            Callback::new(move |_| {
                on_result(true);
            })
        };

        let on_cancel = {
            let on_result = on_result;
            Callback::new(move |_| {
                on_result(false);
            })
        };

        view! {
            <MessageBox
                title
                message
                on_confirm
                on_cancel
            />
        }
    });
}
