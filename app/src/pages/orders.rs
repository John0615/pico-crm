use crate::components::ui::date_picker::FlyonDatePicker;
use crate::components::ui::modal::Modal;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable};
use crate::components::ui::toast::{error, success};
use crate::server::contact_handlers::{fetch_contacts, get_contact};
use crate::server::order_handlers::{fetch_orders, update_order_status};
use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use shared::contact::{Contact, ContactQuery};
use shared::order::{Order, OrderQuery, UpdateOrderStatus};
use shared::ListResult;
use std::collections::{HashMap, HashSet};

impl Identifiable for Order {
    fn id(&self) -> String {
        format!("{}-{}-{}", self.uuid, self.status, self.updated_at)
    }
}

#[component]
pub fn OrdersPage() -> impl IntoView {
    let query = use_query_map();
    let refresh_count = RwSignal::new(0);
    let (status_filter, set_status_filter) = signal(String::new());
    let (contact_filter, set_contact_filter) = signal(String::new());
    let date_start = RwSignal::new(String::new());
    let date_end = RwSignal::new(String::new());

    let show_detail_modal = RwSignal::new(false);
    let detail_order: RwSignal<Option<Order>> = RwSignal::new(None);
    let contact_labels: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let pending_contacts: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());

    let data = Resource::new(
        move || {
            (
                status_filter.get(),
                contact_filter.get(),
                date_start.get(),
                date_end.get(),
                refresh_count.get(),
                query.with(|value| value.clone()),
            )
        },
        |(status, contact, start, end, _, query)| async move {
            let page = query
                .get("page")
                .unwrap_or_default()
                .parse::<u64>()
                .unwrap_or(1);
            let page_size = query
                .get("page_size")
                .unwrap_or_default()
                .parse::<u64>()
                .unwrap_or(10);

            let params = OrderQuery {
                page,
                page_size,
                status: (!status.is_empty()).then_some(status),
                customer_uuid: (!contact.is_empty()).then_some(contact),
                start_date: (!start.is_empty()).then_some(start),
                end_date: (!end.is_empty()).then_some(end),
            };

            let result = call_api(fetch_orders(params)).await.unwrap_or_else(|e| {
                logging::error!("Error loading orders: {e}");
                ListResult {
                    items: Vec::new(),
                    total: 0,
                }
            });
            (result.items, result.total)
        },
    );

    Effect::new(move || {
        let map = query.with(|value| value.clone());
        if let Some(status) = map.get("status") {
            if status_filter.get() != status {
                set_status_filter.set(status);
            }
        }
        if let Some(start) = map.get("start_date") {
            if date_start.get() != start {
                date_start.set(start);
            }
        }
        if let Some(end) = map.get("end_date") {
            if date_end.get() != end {
                date_end.set(end);
            }
        }
    });

    let contacts = Resource::new(
        move || (),
        |_| async move {
            let params = ContactQuery {
                page: 1,
                page_size: 100,
                sort: None,
                filters: None,
            };

            match call_api(fetch_contacts(params)).await {
                Ok(result) => result.items,
                Err(err) => {
                    logging::error!("Error loading contacts: {err}");
                    Vec::new()
                }
            }
        },
    );

    Effect::new(move || {
        if let Some(items) = contacts.get() {
            let mut map = HashMap::new();
            for contact in items {
                map.insert(
                    contact.contact_uuid.clone(),
                    contact_display_label(&contact),
                );
            }
            contact_labels.set(map);
        }
    });

    Effect::new(move || {
        if cfg!(feature = "ssr") {
            return;
        }
        let Some((items, _)) = data.get() else {
            return;
        };

        let existing = contact_labels.get();
        let mut pending = pending_contacts.get();
        let mut missing_ids: Vec<String> = Vec::new();
        for order in &items {
            let Some(contact_id) = order.customer_uuid.clone() else {
                continue;
            };
            if contact_id.is_empty() {
                continue;
            }
            if existing.contains_key(&contact_id) || pending.contains(&contact_id) {
                continue;
            }
            pending.insert(contact_id.clone());
            missing_ids.push(contact_id);
        }
        if missing_ids.is_empty() {
            return;
        }
        pending_contacts.set(pending);

        for contact_id in missing_ids {
            let contact_labels = contact_labels;
            let pending_contacts = pending_contacts;
            spawn_local(async move {
                let label = match call_api(get_contact(contact_id.clone())).await {
                    Ok(Some(contact)) => contact_display_label(&contact),
                    _ => "未知客户".to_string(),
                };
                contact_labels.update(|map| {
                    map.insert(contact_id.clone(), label);
                });
                pending_contacts.update(|set| {
                    set.remove(&contact_id);
                });
            });
        }
    });

    let open_detail = move |order: Order| {
        detail_order.set(Some(order));
        show_detail_modal.set(true);
    };

    let confirm_order = move |uuid: String| {
        spawn_local(async move {
            let payload = UpdateOrderStatus {
                status: "confirmed".to_string(),
            };
            let result = call_api(update_order_status(uuid, payload)).await;
            match result {
                Ok(_) => {
                    success("订单已确认".to_string());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("确认失败: {}", err)),
            }
        });
    };

    view! {
        <Title text="订单管理 - PicoCRM"/>
        <div class="space-y-4">
            <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <h1 class="text-2xl font-semibold">"订单管理"</h1>
            </div>

            <div class="card bg-base-100 shadow-sm">
                <div class="card-body p-4 flex flex-col gap-3 md:flex-row md:items-end">
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"状态"</span>
                        <select
                            class="select select-bordered min-w-[160px]"
                            prop:value=move || status_filter.get()
                            on:change=move |ev| set_status_filter.set(event_target_value(&ev))
                        >
                            <option value="">"全部"</option>
                            <option value="pending">"待确认"</option>
                            <option value="confirmed">"已确认"</option>
                            <option value="dispatching">"派工中"</option>
                            <option value="in_service">"服务中"</option>
                            <option value="completed">"已完成"</option>
                            <option value="cancelled">"已取消"</option>
                        </select>
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"客户ID"</span>
                        <input
                            class="input input-bordered"
                            placeholder="输入客户UUID"
                            on:input=move |ev| set_contact_filter.set(event_target_value(&ev))
                        />
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"服务开始"</span>
                        <FlyonDatePicker value=date_start class="input input-bordered".to_string() />
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"服务结束"</span>
                        <FlyonDatePicker value=date_end class="input input-bordered".to_string() />
                    </div>
                </div>
            </div>

            <div class="overflow-x-auto overflow-y-auto h-[calc(100vh-260px)] bg-base-100 rounded-lg shadow">
                <DaisyTable data=data>
                    <Column
                        slot:columns
                        prop="uuid".to_string()
                        label="订单ID".to_string()
                        class="font-semibold"
                    >
                        {
                            let item: Option<Order> = use_context::<Order>();
                            view! { <span class="text-xs">{item.as_ref().map(|v| v.uuid.clone()).unwrap_or_default()}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="customer_uuid".to_string() label="客户".to_string()>
                        {
                            let item: Option<Order> = use_context::<Order>();
                            let contact_id = item.as_ref().and_then(|v| v.customer_uuid.clone());
                            let fallback = item.as_ref().and_then(|v| v.customer_name.clone());
                            let label = contact_id
                                .as_ref()
                                .map(|id| {
                                    contact_labels
                                        .get()
                                        .get(id)
                                        .cloned()
                                        .or(fallback.clone())
                                        .unwrap_or_else(|| {
                                            if pending_contacts.get().contains(id) {
                                                "加载中...".to_string()
                                            } else {
                                                "未知客户".to_string()
                                            }
                                        })
                                })
                                .unwrap_or_else(|| "-".to_string());
                            view! { <span class="text-xs">{label}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="status".to_string() label="状态".to_string()>
                        {
                            let item: Option<Order> = use_context::<Order>();
                            let status_value = item.as_ref().map(|v| v.status.clone()).unwrap_or_default();
                            let label = order_status_label(&status_value);
                            let badge_class = order_status_badge_class(&status_value);
                            view! {
                                <span class=format!("badge {}", badge_class)>{label}</span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        prop="settlement".to_string()
                        label="结算".to_string()
                    >
                        {
                            let item: Option<Order> = use_context::<Order>();
                            let settlement_value = item.as_ref().map(|v| v.settlement_status.clone()).unwrap_or_default();
                            let label = settlement_status_label(&settlement_value);
                            let badge_class = settlement_status_badge_class(&settlement_value);
                            view! {
                                <span class=format!("badge {}", badge_class)>{label}</span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        prop="actions".to_string()
                        label="操作".to_string()
                        class="text-right"
                    >
                        <div class="flex justify-end gap-1">
                            {
                                let item: Option<Order> = use_context::<Order>();
                                let order = StoredValue::new(item);
                                let can_confirm = order
                                    .with_value(|value| {
                                        value
                                            .as_ref()
                                            .map(|value| value.status == "pending")
                                            .unwrap_or(false)
                                    });
                                view! {
                                    <Show when=move || can_confirm>
                                        <button class="btn btn-soft btn-primary btn-xs" on:click=move |_| {
                                            if let Some(value) = order.with_value(|value| value.clone()) {
                                                confirm_order(value.uuid.clone());
                                            }
                                        }>
                                            "确认"
                                        </button>
                                    </Show>
                                    <button class="btn btn-ghost btn-xs" on:click=move |_| {
                                        if let Some(value) = order.with_value(|value| value.clone()) {
                                            open_detail(value);
                                        }
                                    }>
                                        "详情"
                                    </button>
                                }
                            }
                        </div>
                    </Column>
                </DaisyTable>
            </div>

            <Transition>
                {move || {
                    data.with(|data| {
                        data.as_ref().map(|data| view! { <Pagination total_items=data.1 /> })
                    })
                }}
            </Transition>
        </div>

        <Modal show=show_detail_modal>
            <div class="space-y-4">
                <h3 class="text-lg font-semibold">"订单详情"</h3>
                {move || {
                    if let Some(order) = detail_order.get() {
                        let customer_name = order
                            .customer_name
                            .clone()
                            .filter(|value| !value.trim().is_empty())
                            .unwrap_or_else(|| "未知客户".to_string());
                        view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                {detail_item("订单ID", order.uuid.clone())}
                                {detail_item("需求单ID", display_optional(order.request_id.clone()))}
                                {detail_item("客户", customer_name)}
                                {detail_item("客户UUID", display_optional(order.customer_uuid.clone()))}
                                {detail_item("状态", order_status_label(&order.status).to_string())}
                                {detail_item("结算状态", settlement_status_label(&order.settlement_status).to_string())}
                                {detail_item("服务开始", display_optional(order.scheduled_start_at.clone()))}
                                {detail_item("服务结束", display_optional(order.scheduled_end_at.clone()))}
                                {detail_item("派工备注", display_optional(order.dispatch_note.clone()))}
                                {detail_item("备注", display_optional(order.notes.clone()))}
                                {detail_item("结算备注", display_optional(order.settlement_note.clone()))}
                                {detail_item("创建时间", order.inserted_at.clone())}
                                {detail_item("更新时间", order.updated_at.clone())}
                            </div>
                        }.into_any()
                    } else {
                        view! { <div class="text-sm text-base-content/60">"暂无详情"</div> }.into_any()
                    }
                }}
                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_detail_modal.set(false)>
                        "关闭"
                    </button>
                </div>
            </div>
        </Modal>
    }
}

fn contact_display_label(contact: &Contact) -> String {
    let name = contact.user_name.trim();
    let mut label = String::new();
    if !name.is_empty() {
        label.push_str(name);
    }
    if label.is_empty() {
        label = "未命名客户".to_string();
    }
    let extra = contact.phone_number.trim();
    if !extra.is_empty() {
        format!("{} ({})", label, extra)
    } else {
        label
    }
}

fn display_optional(value: Option<String>) -> String {
    value
        .and_then(|v| {
            let trimmed = v.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .unwrap_or_else(|| "-".to_string())
}

fn detail_item(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-1">
            <span class="text-xs text-base-content/60">{label}</span>
            <span class="text-sm break-all">{value}</span>
        </div>
    }
}

fn order_status_label(status: &str) -> &'static str {
    match status {
        "pending" => "待确认",
        "confirmed" => "已确认",
        "dispatching" => "派工中",
        "in_service" => "服务中",
        "completed" => "已完成",
        "cancelled" => "已取消",
        _ => "未知",
    }
}

fn settlement_status_label(status: &str) -> &'static str {
    match status {
        "unsettled" => "未结算",
        "settled" => "已结算",
        _ => "未知",
    }
}

fn order_status_badge_class(status: &str) -> &'static str {
    match status {
        "pending" => "badge-warning",
        "confirmed" => "badge-info",
        "dispatching" => "badge-warning",
        "in_service" => "badge-warning",
        "completed" => "badge-success",
        "cancelled" => "badge-error",
        _ => "badge-info",
    }
}

fn settlement_status_badge_class(status: &str) -> &'static str {
    match status {
        "unsettled" => "badge-warning",
        "settled" => "badge-success",
        _ => "badge-info",
    }
}
