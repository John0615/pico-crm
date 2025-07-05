use leptos::logging::log;
use leptos::prelude::*;
use std::time::Duration;

#[derive(Clone, Copy, Debug)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

#[derive(Debug, Clone, Default)]
pub struct ToastState {
    message: Option<String>,
    toast_type: Option<ToastType>,
    visible: bool,
}

#[component]
pub fn Toast() -> impl IntoView {
    let toast_state = RwSignal::new(ToastState::default());
    provide_context(toast_state);
    // 获取或初始化Store
    let state =
        use_context::<RwSignal<ToastState>>().expect("there to be a `toast_state` signal provided");
    // 自动隐藏效果
    Effect::new(move |_| {
        // log!("state11: {:?}", state.get());
        if state.get().visible {
            set_timeout(
                move || {
                    state.set(ToastState {
                        visible: false,
                        ..Default::default()
                    })
                },
                Duration::from_secs(3),
            );
        }
    });

    view! {
        <div class="toast toast-top toast-center z-[100]">
            <Show when=move || state.get().visible fallback=move || view!{<div></div>}>
                {move || state.get().message.map(|msg| {
                    let class = match state.get().toast_type.unwrap() {
                        ToastType::Success => "alert alert-success",
                        ToastType::Error => "alert alert-error",
                        ToastType::Warning => "alert alert-warning",
                        ToastType::Info => "alert alert-info",
                    };
                    view! { <div class=class>{msg}</div> }
                })}
            </Show>
        </div>
    }
}

// 显示Toast的接口
pub fn show_toast(message: String, toast_type: ToastType) {
    let state =
        use_context::<RwSignal<ToastState>>().expect("there to be a `toast_state` signal provided");

    // log!("state: {:?}", state);

    state.set(ToastState {
        message: Some(message),
        toast_type: Some(toast_type),
        visible: true,
        ..Default::default()
    });
}
