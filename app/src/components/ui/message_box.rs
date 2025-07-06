use leptos::prelude::*;

#[derive(Clone, Copy, Debug)]
pub enum MessageBoxType {
    Confirm,
    Delete,
    // 可以添加更多类型
}

#[derive(Debug, Clone)]
pub struct MessageBoxState {
    title: String,
    message: String,
    visible: bool,
    message_type: MessageBoxType,
    on_confirm: Option<Callback<()>>,
    on_cancel: Option<Callback<()>>,
}

impl Default for MessageBoxState {
    fn default() -> Self {
        Self {
            title: String::new(),
            message: String::new(),
            visible: false,
            message_type: MessageBoxType::Confirm,
            on_confirm: None,
            on_cancel: None,
        }
    }
}

#[component]
pub fn MessageBox() -> impl IntoView {
    let message_state = RwSignal::new(MessageBoxState::default());
    provide_context(message_state);

    let state = use_context::<RwSignal<MessageBoxState>>()
        .expect("there to be a `message_state` signal provided");

    view! {
        <Show when=move || state.get().visible fallback=|| ()>
            <div class="modal modal-open">
                <div class="modal-box">
                    <button on:click=move |_| {
                        if let Some(cb) = state.get().on_cancel {
                            cb.run(());
                        }
                        state.update(|s| s.visible = false);
                    } class="btn btn-sm btn-circle btn-ghost absolute right-2 top-2">"✕"</button>
                    <h3 class="font-bold text-lg flex items-center gap-2">
                        {move || state.get().title.clone()}
                    </h3>
                    <p class="py-2 flex items-center">
                        {move || match state.get().message_type {
                            MessageBoxType::Delete => view! {
                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4 mr-4 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                </svg>
                            }.into_any(),
                            _ => view! {}.into_any(),
                        }}
                        {move || state.get().message.clone()}
                    </p>
                    <div class="modal-action">
                        <button
                            class="btn btn-sm btn-ghost"
                            on:click=move |_| {
                                if let Some(cb) = state.get().on_cancel {
                                    cb.run(());
                                }
                                state.update(|s| s.visible = false);
                            }
                        >
                            "取消"
                        </button>
                        <button
                            class=move || match state.get().message_type {
                                MessageBoxType::Delete => "btn btn-sm btn-error",
                                _ => "btn btn-sm btn-primary",
                            }
                            on:click=move |_| {
                                if let Some(cb) = state.get().on_confirm {
                                    cb.run(());
                                }
                                state.update(|s| s.visible = false);
                            }
                        >
                            {move || match state.get().message_type {
                                MessageBoxType::Delete => "确认删除",
                                _ => "确认",
                            }}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

pub fn show_message_box(
    title: &str,
    message: &str,
    message_type: MessageBoxType,
    on_confirm: Option<Callback<()>>,
    on_cancel: Option<Callback<()>>,
) {
    let state = use_context::<RwSignal<MessageBoxState>>()
        .expect("there to be a `message_state` signal provided");

    state.set(MessageBoxState {
        title: title.to_string(),
        message: message.to_string(),
        visible: true,
        message_type,
        on_confirm,
        on_cancel,
    });
}

pub fn confirm(
    title: &str,
    message: &str,
    on_result: impl Fn(bool) + Clone + Send + Sync + 'static,
) {
    let on_confirm = Callback::new({
        let on_result = on_result.clone();
        move |_| on_result(true)
    });

    let on_cancel = Callback::new(move |_| on_result(false));

    show_message_box(
        title,
        message,
        MessageBoxType::Confirm,
        Some(on_confirm),
        Some(on_cancel),
    );
}

pub fn delete_confirm(
    title: &str,
    message: &str,
    on_result: impl Fn(bool) + Clone + Send + Sync + 'static,
) {
    let on_confirm = Callback::new({
        let on_result = on_result.clone();
        move |_| on_result(true)
    });

    let on_cancel = Callback::new(move |_| on_result(false));

    show_message_box(
        title,
        message,
        MessageBoxType::Delete,
        Some(on_confirm),
        Some(on_cancel),
    );
}
