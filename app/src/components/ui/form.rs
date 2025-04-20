use leptos::prelude::*;


#[component]
pub fn FormContainer(
    #[prop(optional)] title: Option<&'static str>,
    #[prop(optional)] description: Option<&'static str>,
    #[prop(optional)] class: &'static str,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!("bg-base-100 {}", class)>
            {title.map(|t| view! { <h2 class="card-title text-2xl mb-2">{t}</h2> })}
            {description.map(|d| view! { <p class="text-gray-500 mb-4">{d}</p> })}
            <div class="space-y-4">
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn FormActions(
    #[prop(optional)] justify_end: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div class=format!(
            "flex {} gap-2 mt-6",
            if justify_end { "justify-end" } else { "justify-start" }
        )>
            {children()}
        </div>
    }
}


#[derive(Debug, Clone, Default)]
pub struct ValidationState {
    pub is_valid: bool,
    pub message: String,
}

#[component]
pub fn TextInput(
    name: &'static str,
    label: &'static str,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] required: bool,
    #[prop(optional)] class: &'static str,
    #[prop(default=RwSignal::new(ValidationState{
        is_valid: true,
        message: String::new(),
    }))] validation_state: RwSignal<ValidationState>,
) -> impl IntoView {

    view! {
        <fieldset class=format!("fieldset form-control {}", class)>
            <label class="label">
                <span class="label-text">{label}</span>
                {required.then(|| view! { <span class="text-error">*</span> })}
            </label>
            <input
                type="text"
                name=name
                class=move || {format!(
                    "input input-bordered {}",
                    if !validation_state.get().is_valid { "input-error" } else { "" }
                )}
                placeholder=placeholder
                required=required
                prop:value=move || value.get()
                on:input=move |ev| {
                    let new_value = event_target_value(&ev);
                    value.set(new_value.clone());
                }
            />
            <p class="label">
                <span class="label-text-alt text-error h-4">
                    {move || if !validation_state.get().is_valid {
                        validation_state.get().message.clone()
                    } else {
                        "".to_string()
                    }}
                </span>
            </p>
        </fieldset>
    }
}

#[component]
pub fn CheckboxInput(
    name: &'static str,
    label: &'static str,
    checked: RwSignal<bool>,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    view! {
        <fieldset class=format!("fieldset form-control {}", class)>
            <label class="label cursor-pointer justify-start gap-2">
                <input
                    type="checkbox"
                    name=name
                    class="checkbox checkbox-primary"
                    checked=move || checked.get()
                    on:change=move |ev| checked.set(event_target_checked(&ev))
                />
                <span class="label-text">{label}</span>
            </label>
        </fieldset>
    }
}

#[component]
pub fn TextAreaInput(
    name: &'static str,
    label: &'static str,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] rows: usize,
    #[prop(optional)] class: &'static str,
) -> impl IntoView {
    view! {
        <fieldset class=format!("fieldset form-control {}", class)>
            <label class="label">
                <span class="label-text">{label}</span>
            </label>
            <textarea
                name=name
                class="textarea textarea-bordered"
                placeholder=placeholder
                rows=rows
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
            ></textarea>
        </fieldset>
    }
}

#[component]
pub fn SelectInput<T>(
    name: &'static str,
    label: &'static str,
    value: RwSignal<T>,
    options: Vec<(T, String)>,
    #[prop(optional)] required: bool,
    #[prop(optional)] class: &'static str,
) -> impl IntoView
where
    T: Clone + PartialEq + Send +  Sync + 'static,  // 移除了 ToString 约束
{

    view! {
        <fieldset class=format!("fieldset form-control {}", class)>
            <label class="label">
                <span class="label-text">{label}</span>
                {required.then(|| view! { <span class="text-error">*</span> })}
            </label>
            <select
                name=name
                class="select select-bordered w-full"
                required=required
            >
                <For
                    each=move || options.clone()
                    key=|(_, text)| text.clone()
                    children=move |(val, text)| {
                        let is_selected = move || value.get() == val;
                        view! {
                            <option
                                value=text
                                selected=is_selected
                            >
                                {text.clone()}
                            </option>
                        }
                    }
                />
            </select>
        </fieldset>
    }
}

#[component]
pub fn EmailInput(
    name: &'static str,
    label: &'static str,
    value: RwSignal<String>,
    #[prop(optional)] placeholder: &'static str,
    #[prop(optional)] required: bool,
    #[prop(optional)] class: &'static str,
    #[prop(default=RwSignal::new(ValidationState{
        is_valid: true,
        message: String::new(),
    }))] validation_state: RwSignal<ValidationState>,
) -> impl IntoView {
    view! {
        <fieldset class=format!("fieldset form-control {}", class)>
            <label class="label">
                <span class="label-text">{label}</span>
                {required.then(|| view! { <span class="text-error">*</span> })}
            </label>
            <input
                type="email"
                name=name
                class=move || {format!(
                    "input input-bordered {}",
                    if !validation_state.get().is_valid { "input-error" } else { "" }
                )}
                placeholder=placeholder
                required=required
                prop:value=move || value.get()
                on:input=move |ev| {
                    let new_value = event_target_value(&ev);
                    value.set(new_value.clone());
                }
            />
            <p class="label">
                <span class="label-text-alt text-error h-4">
                    {move || if !validation_state.get().is_valid {
                        validation_state.get().message.clone()
                    } else {
                        "".to_string()
                    }}
                </span>
            </p>
        </fieldset>
    }
}
