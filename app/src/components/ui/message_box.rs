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

// <dialog open id="delete-confirm-modal" class="modal">
//   <div class="modal-box">
//     <h3 class="font-bold text-lg flex items-center gap-2">
//       <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor">
//         <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
//       </svg>
//       "删除确认"
//     </h3>
//     <p class="py-4">"您确定要删除此项吗？删除后数据将无法恢复！"</p>
//     <div class="modal-action">
//       <form method="dialog" class="flex gap-2">
//         <button class="btn btn-ghost">"取消"</button>
//         <button id="confirm-delete-btn" class="btn btn-error">"确认删除"</button>
//       </form>
//     </div>
//   </div>
//   <form method="dialog" class="modal-backdrop">
//     <button>close</button>
//   </form>
// </dialog>
