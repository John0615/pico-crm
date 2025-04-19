use leptos::prelude::*;

#[component]
pub fn ContactModal() -> impl IntoView {

    view! {
        <dialog  class="modal">
          <div class="modal-box w-11/12 max-w-5xl">
            <h3 class="text-lg font-bold">Hello!</h3>
            <p class="py-4">Click the button below to close</p>
            <div class="modal-action">
                // <button on:click=move |_| set_open.set(false) class="btn">Close</button>
                <button class="btn">Close</button>
            </div>
          </div>
        </dialog>
    }
}
