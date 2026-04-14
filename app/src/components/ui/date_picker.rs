use leptos::prelude::*;

#[component]
pub fn FlyonDatePicker(
    value: RwSignal<String>,
    #[prop(optional)] placeholder: String,
    #[prop(optional)] class: String,
) -> impl IntoView {
    picker_input(value, placeholder, class, false, "Y-m-d".to_string())
}

#[component]
pub fn FlyonDateTimePicker(
    value: RwSignal<String>,
    #[prop(optional)] placeholder: String,
    #[prop(optional)] class: String,
) -> impl IntoView {
    picker_input(value, placeholder, class, true, "Y-m-d H:i".to_string())
}

fn picker_input(
    value: RwSignal<String>,
    placeholder: String,
    class: String,
    enable_time: bool,
    format: String,
) -> impl IntoView {
    let input_ref = NodeRef::<leptos::html::Input>::new();
    let class = if class.trim().is_empty() {
        "input input-bordered".to_string()
    } else {
        class
    };

    #[cfg(not(target_arch = "wasm32"))]
    let _ = (&enable_time, &format);

    #[cfg(target_arch = "wasm32")]
    {
        let input_ref = input_ref.clone();
        let format_clone = format.clone();
        let initialized = RwSignal::new(false);
        Effect::new(move |_| {
            if initialized.get() {
                return;
            }
            if let Some(input) = input_ref.get() {
                init_flatpickr_with_retry(input, format_clone.clone(), enable_time);
                initialized.set(true);
            }
        });

        let input_ref = input_ref.clone();
        let value = value.clone();
        let format_clone = format.clone();
        Effect::new(move |_| {
            let current = value.get();
            if let Some(input) = input_ref.get() {
                sync_flatpickr_value(&input, current.as_str(), format_clone.as_str());
            }
        });
    }

    let value_clone = value.clone();
    let value_for_input = value.clone();
    let value_for_change = value.clone();

    view! {
        <input
            node_ref=input_ref
            type="text"
            class=class
            placeholder=placeholder
            prop:value=move || value_clone.with(|value| value.clone())
            on:input=move |ev| {
                let new_value = event_target_value(&ev);
                value_for_input.set(new_value.clone());
            }
            on:change=move |ev| {
                let new_value = event_target_value(&ev);
                value_for_change.set(new_value.clone());
            }
        />
    }
}

#[cfg(target_arch = "wasm32")]
fn init_flatpickr_with_retry(input: web_sys::HtmlInputElement, format: String, enable_time: bool) {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    if init_flatpickr(&input, &format, enable_time) {
        return;
    }

    let Some(window) = web_sys::window() else {
        return;
    };

    let input_clone = input.clone();
    let format_clone = format.clone();
    let cb_1 = Closure::once_into_js(move || {
        let _ = init_flatpickr(&input_clone, &format_clone, enable_time);
    });
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(cb_1.unchecked_ref(), 100);

    let input_clone = input.clone();
    let cb_2 = Closure::once_into_js(move || {
        let _ = init_flatpickr(&input_clone, &format, enable_time);
    });
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(cb_2.unchecked_ref(), 400);
}

#[cfg(target_arch = "wasm32")]
fn init_flatpickr(input: &web_sys::HtmlInputElement, format: &str, enable_time: bool) -> bool {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;
    use web_sys::Event;

    let Some(window) = web_sys::window() else {
        return false;
    };
    let Ok(flatpickr) = js_sys::Reflect::get(&window, &JsValue::from_str("flatpickr")) else {
        return false;
    };
    if flatpickr.is_null() || flatpickr.is_undefined() {
        return false;
    }
    if let Ok(existing) = js_sys::Reflect::get(input, &JsValue::from_str("_flatpickr")) {
        if !existing.is_null() && !existing.is_undefined() {
            return true;
        }
    }

    let options = js_sys::Object::new();
    let _ = js_sys::Reflect::set(
        &options,
        &JsValue::from_str("enableTime"),
        &JsValue::from_bool(enable_time),
    );
    if enable_time {
        let _ = js_sys::Reflect::set(
            &options,
            &JsValue::from_str("time_24hr"),
            &JsValue::from_bool(true),
        );
        let _ = js_sys::Reflect::set(
            &options,
            &JsValue::from_str("minuteIncrement"),
            &JsValue::from_f64(5.0),
        );
    }
    let _ = js_sys::Reflect::set(
        &options,
        &JsValue::from_str("allowInput"),
        &JsValue::from_bool(true),
    );
    let _ = js_sys::Reflect::set(
        &options,
        &JsValue::from_str("dateFormat"),
        &JsValue::from_str(format),
    );

    // Use zh locale if it has been loaded (public/vendor/zh.js).
    if let Ok(l10ns) = js_sys::Reflect::get(&flatpickr, &JsValue::from_str("l10ns")) {
        if let Ok(zh) = js_sys::Reflect::get(&l10ns, &JsValue::from_str("zh")) {
            if !zh.is_undefined() && !zh.is_null() {
                let _ = js_sys::Reflect::set(&options, &JsValue::from_str("locale"), &zh);
            }
        }
    }
    // flatpickr updates the input programmatically; dispatch events so Leptos picks up changes.
    let input_clone = input.clone();
    let on_change = Closure::wrap(Box::new(move |_selected_dates: JsValue| {
        if let Ok(event) = Event::new("input") {
            let _ = input_clone.dispatch_event(&event);
        }
        if let Ok(event) = Event::new("change") {
            let _ = input_clone.dispatch_event(&event);
        }
    }) as Box<dyn FnMut(JsValue)>);
    let _ = js_sys::Reflect::set(
        &options,
        &JsValue::from_str("onChange"),
        on_change.as_ref().unchecked_ref(),
    );
    on_change.forget();

    if let Ok(flatpickr) = flatpickr.dyn_into::<js_sys::Function>() {
        let input_value: JsValue = input.clone().into();
        let options_value: JsValue = options.into();
        let _ = flatpickr.call2(&JsValue::NULL, &input_value, &options_value);
        return true;
    }

    false
}

#[cfg(target_arch = "wasm32")]
fn sync_flatpickr_value(input: &web_sys::HtmlInputElement, value: &str, format: &str) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;

    let trimmed = value.trim();
    if input.value() == trimmed {
        return;
    }

    if let Ok(instance) = js_sys::Reflect::get(input, &JsValue::from_str("_flatpickr")) {
        if instance.is_null() || instance.is_undefined() {
            input.set_value(trimmed);
            return;
        }

        if trimmed.is_empty() {
            if let Ok(clear) = js_sys::Reflect::get(&instance, &JsValue::from_str("clear")) {
                if let Ok(clear_fn) = clear.dyn_into::<js_sys::Function>() {
                    let _ = clear_fn.call0(&instance);
                    return;
                }
            }
            input.set_value("");
            return;
        }

        if let Ok(set_date) = js_sys::Reflect::get(&instance, &JsValue::from_str("setDate")) {
            if let Ok(set_date_fn) = set_date.dyn_into::<js_sys::Function>() {
                let _ = set_date_fn.call3(
                    &instance,
                    &JsValue::from_str(trimmed),
                    &JsValue::from_bool(false),
                    &JsValue::from_str(format),
                );
                return;
            }
        }
    }

    input.set_value(trimmed);
}
