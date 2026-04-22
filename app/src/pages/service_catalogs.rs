use crate::components::ui::modal::{Modal, DETAIL_MODAL_BOX_CLASS};
use crate::components::ui::table::{Column, DaisyTable, Identifiable};
use crate::components::ui::toast::{error, success};
use crate::server::service_catalog_handlers::{
    create_service_catalog, fetch_service_catalogs, update_service_catalog,
};
use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use shared::service_catalog::{
    CreateServiceCatalogRequest, ServiceCatalog, UpdateServiceCatalogRequest,
};

impl Identifiable for ServiceCatalog {
    fn id(&self) -> String {
        format!("{}-{}", self.uuid, self.updated_at)
    }
}

#[component]
pub fn ServiceCatalogsPage() -> impl IntoView {
    let refresh_count = RwSignal::new(0);
    let show_modal = RwSignal::new(false);
    let editing_item: RwSignal<Option<ServiceCatalog>> = RwSignal::new(None);

    let name = RwSignal::new(String::new());
    let description = RwSignal::new(String::new());
    let base_price_cents = RwSignal::new(String::new());
    let default_duration_minutes = RwSignal::new(String::new());
    let sort_order = RwSignal::new(String::new());
    let is_active = RwSignal::new(true);

    let data = Resource::new(
        move || refresh_count.get(),
        |_| async move {
            let items = call_api(fetch_service_catalogs(None))
                .await
                .unwrap_or_else(|e| {
                    logging::error!("Error loading service catalogs: {e}");
                    Vec::new()
                });
            let total = items.len() as u64;
            (items, total)
        },
    );

    let open_create = move |_| {
        editing_item.set(None);
        name.set(String::new());
        description.set(String::new());
        base_price_cents.set(String::new());
        default_duration_minutes.set(String::new());
        sort_order.set("0".to_string());
        is_active.set(true);
        show_modal.set(true);
    };

    let open_edit = move |item: ServiceCatalog| {
        name.set(item.name.clone());
        description.set(item.description.clone().unwrap_or_default());
        base_price_cents.set(item.base_price_cents.to_string());
        default_duration_minutes.set(
            item.default_duration_minutes
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        sort_order.set(item.sort_order.to_string());
        is_active.set(item.is_active);
        editing_item.set(Some(item));
        show_modal.set(true);
    };

    let save = move |_| {
        let parsed_price = match base_price_cents.get().trim().parse::<i64>() {
            Ok(value) if value >= 0 => value,
            _ => {
                error("基础价格必须是非负整数".to_string());
                return;
            }
        };
        let parsed_duration = if default_duration_minutes.get().trim().is_empty() {
            None
        } else {
            match default_duration_minutes.get().trim().parse::<i32>() {
                Ok(value) if value > 0 => Some(value),
                _ => {
                    error("默认时长必须是正整数".to_string());
                    return;
                }
            }
        };
        let parsed_sort = if sort_order.get().trim().is_empty() {
            0
        } else {
            match sort_order.get().trim().parse::<i32>() {
                Ok(value) => value,
                Err(_) => {
                    error("排序值必须是整数".to_string());
                    return;
                }
            }
        };

        let create_payload = CreateServiceCatalogRequest {
            name: name.get(),
            description: normalize_optional(&description.get()),
            base_price_cents: parsed_price,
            default_duration_minutes: parsed_duration,
            is_active: is_active.get(),
            sort_order: Some(parsed_sort),
        };

        if let Some(editing) = editing_item.get() {
            let payload = UpdateServiceCatalogRequest {
                name: create_payload.name,
                description: create_payload.description,
                base_price_cents: create_payload.base_price_cents,
                default_duration_minutes: create_payload.default_duration_minutes,
                is_active: create_payload.is_active,
                sort_order: create_payload.sort_order,
            };
            spawn_local(async move {
                match call_api(update_service_catalog(editing.uuid, payload)).await {
                    Ok(_) => {
                        success("服务项目已更新".to_string());
                        show_modal.set(false);
                        refresh_count.update(|value| *value += 1);
                    }
                    Err(err) => error(format!("更新失败: {}", err)),
                }
            });
        } else {
            spawn_local(async move {
                match call_api(create_service_catalog(create_payload)).await {
                    Ok(_) => {
                        success("服务项目已创建".to_string());
                        show_modal.set(false);
                        refresh_count.update(|value| *value += 1);
                    }
                    Err(err) => error(format!("创建失败: {}", err)),
                }
            });
        }
    };

    view! {
        <Title text="服务项目 - PicoCRM"/>
        <div class="space-y-4">
            <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <div>
                    <h1 class="text-2xl font-semibold text-left">"服务项目"</h1>
                    <p class="mt-1 text-sm text-base-content/60">
                        "维护服务项目、基础价格和默认时长，供排班和后续报价使用"
                    </p>
                </div>
                <button class="btn btn-primary" on:click=open_create>"新增服务项目"</button>
            </div>

            <div class="overflow-x-auto bg-base-100 rounded-lg shadow">
                <DaisyTable data=data>
                    <Column slot:columns prop="name".to_string() label="名称".to_string() class="font-semibold">
                        {
                            let item: Option<ServiceCatalog> = use_context::<ServiceCatalog>();
                            view! { <span>{item.as_ref().map(|value| value.name.clone()).unwrap_or_default()}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="base_price_cents".to_string() label="基础价格(分)".to_string()>
                        {
                            let item: Option<ServiceCatalog> = use_context::<ServiceCatalog>();
                            view! { <span>{item.as_ref().map(|value| value.base_price_cents.to_string()).unwrap_or_else(|| "-".to_string())}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="default_duration_minutes".to_string() label="默认时长(分钟)".to_string()>
                        {
                            let item: Option<ServiceCatalog> = use_context::<ServiceCatalog>();
                            view! {
                                <span>
                                    {item.as_ref().and_then(|value| value.default_duration_minutes.map(|v| v.to_string())).unwrap_or_else(|| "-".to_string())}
                                </span>
                            }
                        }
                    </Column>
                    <Column slot:columns prop="status".to_string() label="状态".to_string()>
                        {
                            let item: Option<ServiceCatalog> = use_context::<ServiceCatalog>();
                            let active = item.as_ref().map(|value| value.is_active).unwrap_or(false);
                            view! {
                                <span class=move || if active { "badge badge-success" } else { "badge badge-ghost" }>
                                    {if active { "启用" } else { "停用" }}
                                </span>
                            }
                        }
                    </Column>
                    <Column slot:columns prop="description".to_string() label="说明".to_string()>
                        {
                            let item: Option<ServiceCatalog> = use_context::<ServiceCatalog>();
                            view! {
                                <div class="max-w-72 whitespace-normal text-sm opacity-80">
                                    {item.as_ref().and_then(|value| value.description.clone()).unwrap_or_else(|| "-".to_string())}
                                </div>
                            }
                        }
                    </Column>
                    <Column slot:columns prop="sort_order".to_string() label="排序".to_string()>
                        {
                            let item: Option<ServiceCatalog> = use_context::<ServiceCatalog>();
                            view! { <span>{item.as_ref().map(|value| value.sort_order.to_string()).unwrap_or_else(|| "0".to_string())}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="actions".to_string() label="操作".to_string() class="text-right">
                        <div class="flex justify-end gap-1">
                            {
                                let item: Option<ServiceCatalog> = use_context::<ServiceCatalog>();
                                let item = StoredValue::new(item);
                                view! {
                                    <button class="btn btn-soft btn-warning btn-xs" on:click=move |_| {
                                        if let Some(value) = item.with_value(|value| value.clone()) {
                                            open_edit(value);
                                        }
                                    }>
                                        "编辑"
                                    </button>
                                }
                            }
                        </div>
                    </Column>
                </DaisyTable>
            </div>
        </div>

        <Modal show=show_modal box_class=DETAIL_MODAL_BOX_CLASS>
            <div class="space-y-4">
                <h3 class="text-lg font-semibold">
                    {move || if editing_item.get().is_some() { "编辑服务项目" } else { "新增服务项目" }}
                </h3>
                <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
                    <label class="form-control">
                        <span class="label-text text-sm">"名称"</span>
                        <input class="input input-bordered" prop:value=move || name.get() on:input=move |ev| name.set(event_target_value(&ev)) placeholder="例如：深度保洁" />
                    </label>
                    <label class="form-control">
                        <span class="label-text text-sm">"基础价格(分)"</span>
                        <input class="input input-bordered" type="number" min="0" prop:value=move || base_price_cents.get() on:input=move |ev| base_price_cents.set(event_target_value(&ev)) placeholder="例如：29900" />
                    </label>
                    <label class="form-control">
                        <span class="label-text text-sm">"默认时长(分钟)"</span>
                        <input class="input input-bordered" type="number" min="1" prop:value=move || default_duration_minutes.get() on:input=move |ev| default_duration_minutes.set(event_target_value(&ev)) placeholder="例如：120" />
                    </label>
                    <label class="form-control">
                        <span class="label-text text-sm">"排序"</span>
                        <input class="input input-bordered" type="number" prop:value=move || sort_order.get() on:input=move |ev| sort_order.set(event_target_value(&ev)) placeholder="默认 0" />
                    </label>
                    <label class="form-control md:col-span-2">
                        <span class="label-text text-sm">"说明"</span>
                        <textarea class="textarea textarea-bordered min-h-24" prop:value=move || description.get() on:input=move |ev| description.set(event_target_value(&ev)) placeholder="补充服务项目说明" />
                    </label>
                    <label class="form-control md:col-span-2">
                        <div class="label"><span class="label-text text-sm">"启用状态"</span></div>
                        <input type="checkbox" class="toggle toggle-success" prop:checked=move || is_active.get() on:change=move |ev| is_active.set(event_target_checked(&ev)) />
                    </label>
                </div>
                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_modal.set(false)>"取消"</button>
                    <button class="btn btn-primary" on:click=save>"保存"</button>
                </div>
            </div>
        </Modal>
    }
}

fn normalize_optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
