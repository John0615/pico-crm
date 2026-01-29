use leptos::prelude::*;

#[component]
pub fn Modal(
    show: RwSignal<bool>,
    #[prop(optional)] box_class: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <dialog open=move || *show.read() class="modal modal-bottom sm:modal-middle">
            <div class=move || format!("modal-box w-11/12 {}", box_class)>
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
