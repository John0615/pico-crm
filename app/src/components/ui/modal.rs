use leptos::prelude::*;


#[component]
pub fn Modal(
    show: RwSignal<bool>,
    children: Children
) -> impl IntoView {
    view! {
        <dialog open=show.get() class="modal modal-bottom sm:modal-middle">
            <div class="modal-box w-11/12 max-w-5xl">
                <button
                    class="btn btn-sm btn-circle absolute right-2 top-2"
                    on:click=move |_| show.set(false)
                >
                    "✕"
                </button>
                {children()}
                <div class="modal-action">
                    <button class="btn">Close</button>
                </div>
            </div>
            <button on:click=move |_| show.set(false)>"close"</button>
        </dialog>
    }
}
