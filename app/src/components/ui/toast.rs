use leptos::prelude::*;
use std::sync::OnceLock;
use std::time::Duration;

#[derive(Clone, Copy)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

// 使用 OnceLock 安全初始化全局状态
static CURRENT_TOAST: OnceLock<RwSignal<Option<()>>> = OnceLock::new();

/// 显示Toast（自动替换前一个）
pub fn show_toast(message: String, toast_type: ToastType) {
    // 获取或初始化全局信号
    let toast_signal = CURRENT_TOAST.get_or_init(|| RwSignal::new(None));

    // 清理前一个Toast
    toast_signal.set(None);

    // 创建新Toast
    let toast_class = match toast_type {
        ToastType::Success => "alert alert-success",
        ToastType::Error => "alert alert-error",
        ToastType::Warning => "alert alert-warning",
        ToastType::Info => "alert alert-info",
    };

    mount_to_body(move || {
        let (visible, set_visible) = signal(true);

        // 自动消失逻辑
        set_timeout(
            move || { set_visible.set(false); }, // 修正这里
            Duration::from_secs(3)
        );

        view! {
            <Show
                when=move || visible.get()
                fallback=|| ()
            >
                <div class="toast toast-top toast-center z-50">
                    <div class=toast_class>
                        <span>{message.clone()}</span>
                    </div>
                </div>
            </Show>
        }
    });

    // 存储标记表示有Toast显示
    toast_signal.set(Some(()));
}
