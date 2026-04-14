use leptos::prelude::*;

#[component]
pub fn Modal(
    show: RwSignal<bool>,
    #[prop(optional)] box_class: &'static str,
    children: Children,
) -> impl IntoView {
    // 默认类：modal-box w-11/12 max-h-[80vh] overflow-y-auto
    // 如果 box_class 中包含 max-h-none 和 overflow-visible，则使用这些自定义类
    // 否则保持默认的 max-h-[80vh] overflow-y-auto

    let modal_box_class: String =
        if box_class.contains("max-h-none") && box_class.contains("overflow-visible") {
            // 用户希望禁用滚动限制，使用自定义类
            if box_class.is_empty() {
                "modal-box w-11/12 max-h-none overflow-visible".to_string()
            } else {
                format!(
                    "{} modal-box w-11/12 max-h-none overflow-visible",
                    box_class
                )
            }
        } else if box_class.is_empty() {
            // 默认类
            "modal-box w-11/12 max-h-[80vh] overflow-y-auto".to_string()
        } else {
            // 有自定义类但没有 max-h-none 和 overflow-visible
            format!(
                "{} modal-box w-11/12 max-h-[80vh] overflow-y-auto",
                box_class
            )
        };

    view! {
        <dialog open=move || *show.read() class="modal modal-bottom sm:modal-middle">
            <div class=modal_box_class>
                <button
                    class="btn btn-sm btn-circle absolute right-2 top-2"
                    on:click=move |_| show.set(false)
                >
                    "✕"
                </button>
                {children()}
            </div>
        </dialog>
    }
}
