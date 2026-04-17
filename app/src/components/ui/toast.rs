// use leptos::logging::log;
use leptos::prelude::*;
use std::cell::RefCell;
use std::time::Duration;

thread_local! {
    static TOAST_SIGNAL: RefCell<Option<RwSignal<ToastState>>> = const { RefCell::new(None) };
}

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
    TOAST_SIGNAL.with(|slot| {
        *slot.borrow_mut() = Some(toast_state);
    });
    let state = toast_state;
    // 自动隐藏效果
    Effect::new(move |_| {
        // log!("state11: {:?}", state.get());
        if state.with(|state| state.visible) {
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
            <div class=move || if state.with(|state| state.visible) { "block" } else { "hidden" }>
                {move || {
                    state.with(|state| {
                        let msg = state.message.clone()?;
                        let class = match state.toast_type.unwrap_or(ToastType::Info) {
                            ToastType::Success => "alert alert-success",
                            ToastType::Error => "alert alert-error",
                            ToastType::Warning => "alert alert-warning",
                            ToastType::Info => "alert alert-info",
                        };
                        Some(view! { <div class=class>{msg}</div> })
                    })
                }}
            </div>
        </div>
    }
}

// 显示Toast的接口
fn show_toast(message: String, toast_type: ToastType) {
    TOAST_SIGNAL.with(|slot| {
        if let Some(state) = *slot.borrow() {
            state.set(ToastState {
                message: Some(message),
                toast_type: Some(toast_type),
                visible: true,
                ..Default::default()
            });
        } else {
            eprintln!("toast signal not initialized yet");
        }
    });
}

pub fn success(message: String) {
    show_toast(message, ToastType::Success);
}

pub fn error(message: String) {
    show_toast(message, ToastType::Error);
}

pub fn warning(message: String) {
    show_toast(message, ToastType::Warning);
}

pub fn info(message: String) {
    show_toast(message, ToastType::Info);
}
