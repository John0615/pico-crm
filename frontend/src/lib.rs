#![recursion_limit = "1024"]

#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    #[cfg(debug_assertions)]
    {
        _ = console_log::init_with_level(log::Level::Debug);
        console_error_panic_hook::set_once();
    }

    leptos::mount::hydrate_lazy(App);
}
