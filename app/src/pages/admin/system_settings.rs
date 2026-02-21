use std::collections::HashMap;

use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use serde_json::Value;

use crate::components::ui::toast::{error, success};
use crate::utils::api::call_api;
use shared::system_config::{SystemConfigItemDto, SystemConfigUpdateItem, SystemConfigUpdateRequest};

pub use crate::server::system_config_handlers::{fetch_system_config, update_system_config};

const MASKED_VALUE: &str = "******";

#[component]
pub fn SystemSettings() -> impl IntoView {
    let refresh_count = RwSignal::new(0);
    let active_tab = RwSignal::new(String::new());
    let draft_values: RwSignal<HashMap<String, Value>> = RwSignal::new(HashMap::new());
    let original_values: RwSignal<HashMap<String, Value>> = RwSignal::new(HashMap::new());

    let data = Resource::new(
        move || *refresh_count.read(),
        |_| async move {
            call_api(fetch_system_config()).await.unwrap_or_else(|e| {
                logging::error!("Error loading system config: {e}");
                Vec::new()
            })
        },
    );

    Effect::new(move |_| {
        if let Some(categories) = data.get() {
            if active_tab.get().is_empty() {
                if let Some(first) = categories.first() {
                    active_tab.set(first.code.clone());
                }
            }

            let mut map = HashMap::new();
            for item in categories.iter().flat_map(|c| c.items.iter()) {
                map.insert(item.key.clone(), item.value.clone());
            }
            draft_values.set(map.clone());
            original_values.set(map);
        }
    });

    let has_changes = Memo::new(move |_| {
        draft_values.with(|draft| original_values.with(|orig| draft != orig))
    });

    let on_save = move |_| {
        let updates = draft_values.with(|draft| {
            original_values.with(|orig| build_updates(draft, orig))
        });
        if updates.is_empty() {
            return;
        }

        spawn_local(async move {
            let request = SystemConfigUpdateRequest { items: updates };
            match call_api(update_system_config(request)).await {
                Ok(_) => {
                    success("系统配置已保存".to_string());
                    refresh_count.update(|value| *value += 1);
                }
                Err(e) => {
                    error(format!("保存失败: {}", e));
                }
            }
        });
    };

    let on_reset = move |_| {
        let updates = original_values.with(|orig| {
            orig.keys()
                .map(|key| SystemConfigUpdateItem {
                    key: key.clone(),
                    value: None,
                    reset_to_default: Some(true),
                })
                .collect::<Vec<_>>()
        });
        if updates.is_empty() {
            return;
        }

        spawn_local(async move {
            let request = SystemConfigUpdateRequest { items: updates };
            match call_api(update_system_config(request)).await {
                Ok(_) => {
                    success("系统配置已重置为默认值".to_string());
                    refresh_count.update(|value| *value += 1);
                }
                Err(e) => {
                    error(format!("重置失败: {}", e));
                }
            }
        });
    };

    view! {
        <Title text="系统设置 - PicoCRM"/>
        <div class="">
            <Suspense fallback=move || view! {
                <div class="p-6 text-sm text-base-content/60">"加载中..."</div>
            }>
                {move || data.get().map(|categories| {
                    let active_code = active_tab.get();
                    let active_category = categories
                        .iter()
                        .find(|c| c.code == active_code)
                        .cloned()
                        .or_else(|| categories.first().cloned());

                    view! {
                        <div class="tabs tabs-boxed mb-4 flex flex-wrap gap-2">
                            <For
                                each=move || categories.clone()
                                key=|category| category.code.clone()
                                children=move |category| {
                                    let code = category.code.clone();
                                    let active_code = code.clone();
                                    let is_active = move || active_tab.get() == active_code;
                                    view! {
                                        <a
                                            class=move || {
                                                if is_active() { "tab tab-active" } else { "tab" }
                                            }
                                            on:click=move |_| {
                                                active_tab.set(code.clone());
                                            }
                                        >
                                            {category.name}
                                        </a>
                                    }
                                }
                            />
                        </div>

                        {active_category.map(|category| {
                            view! {
                                <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-6">
                                    <For
                                        each=move || category.items.clone()
                                        key=|item| item.key.clone()
                                        children=move |item| {
                                            let key = item.key.clone();
                                            let widget = widget_for(&item);
                                            let options = options_for(&item);
                                            let placeholder = placeholder_for(&item);
                                            let key_for_value = key.clone();
                                            let value_signal = move || {
                                                draft_values.with(|draft| {
                                                    draft
                                                        .get(&key_for_value)
                                                        .cloned()
                                                        .unwrap_or_else(|| item.value.clone())
                                                })
                                            };
                                            let disabled = !item.is_editable;

                                            view! {
                                                <div class="card bg-base-100 shadow-xl">
                                                    <div class="card-body">
                                                        <fieldset class="border border-base-300 rounded-lg p-4">
                                                            <legend class="text-lg font-semibold px-2 flex items-center gap-2">
                                                                <span>{item.label.clone()}</span>
                                                                {item.is_required.then(|| view! {
                                                                    <span class="badge badge-sm badge-primary">"必填"</span>
                                                                })}
                                                                {item.is_sensitive.then(|| view! {
                                                                    <span class="badge badge-sm badge-warning">"敏感"</span>
                                                                })}
                                                            </legend>
                                                            <div class="space-y-3">
                                                                {match widget.as_str() {
                                                                    "toggle" => {
                                                                        let key = key.clone();
                                                                        view! {
                                                                        <label class="flex items-center gap-3 cursor-pointer">
                                                                            <input
                                                                                type="checkbox"
                                                                                class="checkbox checkbox-primary"
                                                                                prop:checked=move || value_signal().as_bool().unwrap_or(false)
                                                                                on:change=move |ev| {
                                                                                    let checked = event_target_checked(&ev);
                                                                                    draft_values.update(|draft| {
                                                                                        draft.insert(key.clone(), Value::Bool(checked));
                                                                                    });
                                                                                }
                                                                                disabled=disabled
                                                                            />
                                                                            <span class="text-sm leading-tight">{placeholder.unwrap_or_else(|| "启用".to_string())}</span>
                                                                        </label>
                                                                        }.into_any()
                                                                    }
                                                                    "select" => {
                                                                        let key = key.clone();
                                                                        view! {
                                                                        <select
                                                                            class="select select-bordered w-full"
                                                                            prop:value=move || value_to_string(&value_signal())
                                                                            on:change=move |ev| {
                                                                                let raw = event_target_value(&ev);
                                                                                draft_values.update(|draft| {
                                                                                    draft.insert(key.clone(), Value::String(raw));
                                                                                });
                                                                            }
                                                                            disabled=disabled
                                                                        >
                                                                            <For
                                                                                each=move || options.clone()
                                                                                key=|opt| opt.clone()
                                                                                children=move |opt| view! {
                                                                                    <option value={opt.clone()}>{opt.clone()}</option>
                                                                                }
                                                                            />
                                                                        </select>
                                                                        }.into_any()
                                                                    }
                                                                    "textarea" => {
                                                                        let key = key.clone();
                                                                        view! {
                                                                        <textarea
                                                                            class="textarea textarea-bordered w-full"
                                                                            rows="4"
                                                                            prop:value=move || value_to_string(&value_signal())
                                                                            placeholder=placeholder.clone().unwrap_or_default()
                                                                            on:input=move |ev| {
                                                                                let raw = event_target_value(&ev);
                                                                                draft_values.update(|draft| {
                                                                                    draft.insert(key.clone(), Value::String(raw));
                                                                                });
                                                                            }
                                                                            disabled=disabled
                                                                        ></textarea>
                                                                        }.into_any()
                                                                    }
                                                                    "number" => {
                                                                        let key = key.clone();
                                                                        view! {
                                                                        <input
                                                                            type="number"
                                                                            class="input input-bordered w-full"
                                                                            prop:value=move || value_to_string(&value_signal())
                                                                            placeholder=placeholder.clone().unwrap_or_default()
                                                                            on:input=move |ev| {
                                                                                let raw = event_target_value(&ev);
                                                                                let value = parse_number_value(&raw);
                                                                                draft_values.update(|draft| {
                                                                                    draft.insert(key.clone(), value);
                                                                                });
                                                                            }
                                                                            disabled=disabled
                                                                        />
                                                                        }.into_any()
                                                                    }
                                                                    "password" => {
                                                                        let key = key.clone();
                                                                        view! {
                                                                        <input
                                                                            type="password"
                                                                            class="input input-bordered w-full"
                                                                            prop:value=move || value_to_string(&value_signal())
                                                                            placeholder=placeholder.clone().unwrap_or_else(|| MASKED_VALUE.to_string())
                                                                            on:input=move |ev| {
                                                                                let raw = event_target_value(&ev);
                                                                                draft_values.update(|draft| {
                                                                                    draft.insert(key.clone(), Value::String(raw));
                                                                                });
                                                                            }
                                                                            disabled=disabled
                                                                        />
                                                                        }.into_any()
                                                                    }
                                                                    _ => {
                                                                        let key = key.clone();
                                                                        view! {
                                                                        <input
                                                                            type="text"
                                                                            class="input input-bordered w-full"
                                                                            prop:value=move || value_to_string(&value_signal())
                                                                            placeholder=placeholder.clone().unwrap_or_default()
                                                                            on:input=move |ev| {
                                                                                let raw = event_target_value(&ev);
                                                                                draft_values.update(|draft| {
                                                                                    draft.insert(key.clone(), Value::String(raw));
                                                                                });
                                                                            }
                                                                            disabled=disabled
                                                                        />
                                                                        }.into_any()
                                                                    }
                                                                }}
                                                                {item.description.clone().map(|desc| view! {
                                                                    <p class="label text-sm text-base-content/60">{desc}</p>
                                                                })}
                                                            </div>
                                                        </fieldset>
                                                    </div>
                                                </div>
                                            }
                                        }
                                    />
                                </div>
                            }
                        })}

                        <div class="flex justify-end space-x-4">
                            <button class="btn btn-outline" on:click=on_reset>
                                "重置为默认值"
                            </button>
                            <button
                                class="btn btn-primary"
                                on:click=on_save
                                disabled=move || !has_changes.get()
                            >
                                "保存设置"
                            </button>
                        </div>
                    }
                })}
            </Suspense>
        </div>
    }
}

fn build_updates(
    draft: &HashMap<String, Value>,
    original: &HashMap<String, Value>,
) -> Vec<SystemConfigUpdateItem> {
    draft
        .iter()
        .filter_map(|(key, value)| {
            original.get(key).and_then(|orig| {
                if orig != value {
                    Some(SystemConfigUpdateItem {
                        key: key.clone(),
                        value: Some(value.clone()),
                        reset_to_default: None,
                    })
                } else {
                    None
                }
            })
        })
        .collect()
}

fn value_to_string(value: &Value) -> String {
    match value {
        Value::String(v) => v.clone(),
        Value::Number(v) => v.to_string(),
        Value::Bool(v) => v.to_string(),
        Value::Null => "".to_string(),
        other => other.to_string(),
    }
}

fn parse_number_value(raw: &str) -> Value {
    if raw.trim().is_empty() {
        return Value::Null;
    }
    if let Ok(int_val) = raw.parse::<i64>() {
        return Value::Number(int_val.into());
    }
    if let Ok(float_val) = raw.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(float_val) {
            return Value::Number(num);
        }
    }
    Value::Null
}

fn widget_for(item: &SystemConfigItemDto) -> String {
    if let Some(schema) = item.ui_schema.as_ref() {
        if let Some(widget) = schema.get("widget").and_then(|v| v.as_str()) {
            return widget.to_string();
        }
    }
    match item.value_type.as_str() {
        "bool" => "toggle".to_string(),
        "number" => "number".to_string(),
        "enum" => "select".to_string(),
        "json" => "textarea".to_string(),
        _ => "text".to_string(),
    }
}

fn options_for(item: &SystemConfigItemDto) -> Vec<String> {
    if let Some(schema) = item.ui_schema.as_ref() {
        if let Some(options) = schema.get("options").and_then(|v| v.as_array()) {
            return options
                .iter()
                .filter_map(|opt| opt.as_str().map(|s| s.to_string()))
                .collect();
        }
    }
    if let Some(validation) = item.validation.as_ref() {
        if let Some(options) = validation.get("options").and_then(|v| v.as_array()) {
            return options
                .iter()
                .filter_map(|opt| opt.as_str().map(|s| s.to_string()))
                .collect();
        }
    }
    Vec::new()
}

fn placeholder_for(item: &SystemConfigItemDto) -> Option<String> {
    item.ui_schema
        .as_ref()
        .and_then(|schema| schema.get("placeholder"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
}
