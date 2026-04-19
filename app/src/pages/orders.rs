use crate::components::ui::date_picker::FlyonDatePicker;
use crate::components::ui::modal::{Modal, DETAIL_MODAL_BOX_CLASS};
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable};
use crate::components::ui::toast::{error, success};
use crate::server::contact_handlers::{fetch_contacts, get_contact};
use crate::server::order_handlers::{
    cancel_order, fetch_orders, get_order_change_logs, update_order, update_order_settlement,
    update_order_status,
};
use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use shared::contact::{Contact, ContactQuery};
use shared::order::{
    CancelOrderRequest, Order, OrderChangeLogDto, OrderQuery, UpdateOrderRequest,
    UpdateOrderSettlement, UpdateOrderStatus,
};
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
    let show_edit_modal = RwSignal::new(false);
    let show_cancel_modal = RwSignal::new(false);
    let detail_order: RwSignal<Option<Order>> = RwSignal::new(None);
    let editing_order_uuid = RwSignal::new(String::new());
    let cancelling_order_uuid = RwSignal::new(String::new());

    let form_customer_uuid = RwSignal::new(String::new());
    let form_amount_cents = RwSignal::new(String::new());
    let form_notes = RwSignal::new(String::new());
    let cancel_reason = RwSignal::new(String::new());
    let settlement_status = RwSignal::new(String::new());
    let settlement_note = RwSignal::new(String::new());

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

    let order_logs = Resource::new(
        move || detail_order.get().map(|order| order.uuid.clone()),
        |uuid| async move {
            match uuid {
                Some(uuid) => call_api(get_order_change_logs(uuid))
                    .await
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        },
    );

    let open_edit = move |order: Order| {
        editing_order_uuid.set(order.uuid.clone());
        form_customer_uuid.set(order.customer_uuid.unwrap_or_default());
        form_amount_cents.set(order.amount_cents.to_string());
        form_notes.set(order.notes.unwrap_or_default());
        show_edit_modal.set(true);
    };

    let open_cancel = move |order: Order| {
        cancelling_order_uuid.set(order.uuid.clone());
        cancel_reason.set(order.cancellation_reason.unwrap_or_default());
        show_cancel_modal.set(true);
    };

    let open_detail = move |order: Order| {
        settlement_status.set(order.settlement_status.clone());
        settlement_note.set(order.settlement_note.clone().unwrap_or_default());
        detail_order.set(Some(order));
        show_detail_modal.set(true);
    };

    let save_order_details = move |_| {
        let uuid = editing_order_uuid.get();
        if uuid.trim().is_empty() {
            error("缺少订单ID".to_string());
            return;
        }
        let amount_cents = match form_amount_cents.get().trim().parse::<i64>() {
            Ok(value) if value >= 0 => value,
            _ => {
                error("金额必须是非负整数".to_string());
                return;
            }
        };

        let payload = UpdateOrderRequest {
            customer_uuid: form_customer_uuid.get().trim().to_string(),
            amount_cents,
            notes: normalize_optional(&form_notes.get()),
        };

        spawn_local(async move {
            match call_api(update_order(uuid, payload)).await {
                Ok(_) => {
                    success("订单已更新".to_string());
                    show_edit_modal.set(false);
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("更新失败: {}", err)),
            }
        });
    };

    let submit_cancel_order = move |_| {
        let uuid = cancelling_order_uuid.get();
        if uuid.trim().is_empty() {
            error("缺少订单ID".to_string());
            return;
        }
        let reason = cancel_reason.get();
        if reason.trim().is_empty() {
            error("取消订单必须填写原因".to_string());
            return;
        }

        spawn_local(async move {
            match call_api(cancel_order(
                uuid,
                CancelOrderRequest {
                    reason: reason.trim().to_string(),
                },
            ))
            .await
            {
                Ok(_) => {
                    success("订单已取消".to_string());
                    show_cancel_modal.set(false);
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("取消失败: {}", err)),
            }
        });
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

    let save_settlement = move |_| {
        let Some(order) = detail_order.get() else {
            return;
        };

        let payload = UpdateOrderSettlement {
            settlement_status: settlement_status.get(),
            settlement_note: normalize_optional(&settlement_note.get()),
        };

        spawn_local(async move {
            match call_api(update_order_settlement(order.uuid.clone(), payload)).await {
                Ok(updated) => {
                    success("结算状态已更新".to_string());
                    detail_order.set(Some(updated));
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("更新结算失败: {}", err)),
            }
        });
    };

    view! {
        <Title text="订单管理 - PicoCRM"/>
        <div class="space-y-4">
            <div class="flex flex-col items-start gap-3 text-left md:flex-row md:items-center md:justify-between">
                <div class="text-left">
                    <h1 class="text-2xl font-semibold">"订单管理"</h1>
                    <p class="mt-1 text-sm text-base-content/60">"订单只能从已确认的需求单转化生成，这里仅支持查看、编辑、取消、收款更新和变更记录"</p>
                </div>
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
                        freeze=true
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
                            view! { <span class=format!("badge {}", badge_class)>{label}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="settlement".to_string() label="结算".to_string()>
                        {
                            let item: Option<Order> = use_context::<Order>();
                            let settlement_value = item.as_ref().map(|v| v.settlement_status.clone()).unwrap_or_default();
                            let label = settlement_status_label(&settlement_value);
                            let badge_class = settlement_status_badge_class(&settlement_value);
                            view! { <span class=format!("badge {}", badge_class)>{label}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="completed_at".to_string() label="完成时间".to_string()>
                        {
                            let item: Option<Order> = use_context::<Order>();
                            view! { <span class="text-xs">{item.as_ref().and_then(|v| v.completed_at.clone()).unwrap_or_else(|| "-".to_string())}</span> }
                        }
                    </Column>
                    <Column
                        slot:columns
                        freeze=true
                        prop="actions".to_string()
                        label="操作".to_string()
                        class="text-right"
                    >
                        <div class="flex justify-end gap-1">
                            {
                                let item: Option<Order> = use_context::<Order>();
                                let order = StoredValue::new(item);
                                let can_confirm = order
                                    .with_value(|value| value.as_ref().map(|value| value.status == "pending").unwrap_or(false));
                                let can_edit = order
                                    .with_value(|value| value.as_ref().map(|value| value.status != "completed" && value.status != "cancelled").unwrap_or(false));
                                let can_cancel = order
                                    .with_value(|value| value.as_ref().map(|value| value.status != "completed" && value.status != "cancelled").unwrap_or(false));
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
                                    <Show when=move || can_edit>
                                        <button class="btn btn-soft btn-warning btn-xs" on:click=move |_| {
                                            if let Some(value) = order.with_value(|value| value.clone()) {
                                                open_edit(value);
                                            }
                                        }>
                                            "编辑"
                                        </button>
                                    </Show>
                                    <Show when=move || can_cancel>
                                        <button class="btn btn-soft btn-error btn-xs" on:click=move |_| {
                                            if let Some(value) = order.with_value(|value| value.clone()) {
                                                open_cancel(value);
                                            }
                                        }>
                                            "取消"
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

        <Modal show=show_edit_modal box_class=DETAIL_MODAL_BOX_CLASS>
            <div class="space-y-4">
                <h3 class="text-lg font-semibold">"编辑订单"</h3>
                <OrderForm
                    contacts=contacts
                    customer_uuid=form_customer_uuid
                    amount_cents=form_amount_cents
                    notes=form_notes
                />
                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_edit_modal.set(false)>"取消"</button>
                    <button class="btn btn-primary" on:click=save_order_details>"保存修改"</button>
                </div>
            </div>
        </Modal>

        <Modal show=show_cancel_modal>
            <div class="space-y-4">
                <h3 class="text-lg font-semibold">"取消订单"</h3>
                <label class="form-control">
                    <span class="label-text text-sm">"取消原因"</span>
                    <textarea
                        class="textarea textarea-bordered min-h-32"
                        prop:value=move || cancel_reason.get()
                        on:input=move |ev| cancel_reason.set(event_target_value(&ev))
                        placeholder="必须填写取消原因"
                    />
                </label>
                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_cancel_modal.set(false)>"返回"</button>
                    <button class="btn btn-error" on:click=submit_cancel_order>"确认取消"</button>
                </div>
            </div>
        </Modal>

        <Modal show=show_detail_modal box_class=DETAIL_MODAL_BOX_CLASS>
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
                                {detail_item("取消原因", display_optional(order.cancellation_reason.clone()))}
                                {detail_item("完成时间", display_optional(order.completed_at.clone()))}
                                {detail_item("结算状态", settlement_status_label(&order.settlement_status).to_string())}
                                {detail_item("订单金额(分)", order.amount_cents.to_string())}
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

                <div class="rounded-box border border-base-200 p-4 space-y-3">
                    <div class="font-medium">"收款更新"</div>
                    <div class="grid gap-3 md:grid-cols-[180px,1fr]">
                        <select
                            class="select select-bordered"
                            prop:value=move || settlement_status.get()
                            on:change=move |ev| settlement_status.set(event_target_value(&ev))
                        >
                            <option value="unsettled">"未结算"</option>
                            <option value="settled">"已结算"</option>
                        </select>
                        <input
                            class="input input-bordered"
                            prop:value=move || settlement_note.get()
                            on:input=move |ev| settlement_note.set(event_target_value(&ev))
                            placeholder="填写结算备注"
                        />
                    </div>
                    <div class="flex justify-end">
                        <button class="btn btn-sm btn-primary" on:click=save_settlement>
                            "保存结算状态"
                        </button>
                    </div>
                </div>

                <div class="rounded-box border border-base-200 p-4 space-y-3">
                    <div class="font-medium">"变更记录"</div>
                    <Transition fallback=move || view! { <div class="text-sm text-base-content/60">"加载中..."</div> }>
                        {move || {
                            order_logs.get().map(|logs| {
                                if logs.is_empty() {
                                    view! { <div class="text-sm text-base-content/60">"暂无变更记录"</div> }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-3">
                                            <For
                                            each=move || logs.clone().into_iter().enumerate()
                                            key=|(idx, item)| format!("{}-{}", idx, item.uuid)
                                            children=move |(_, item)| {
                                                let diff_items = order_log_diff_items(&item);
                                                let has_diff = !diff_items.is_empty();
                                                let diff_rows = diff_items
                                                    .iter()
                                                    .cloned()
                                                    .map(|field| {
                                                        view! {
                                                            <div class="grid gap-2 rounded bg-base-100 p-2 text-xs md:grid-cols-[120px,1fr,1fr]">
                                                                <div class="font-medium text-base-content/70">{field.0}</div>
                                                                <div>
                                                                    <div class="text-[11px] text-base-content/50">"变更前"</div>
                                                                    <div class="whitespace-pre-wrap break-all">{field.1}</div>
                                                                </div>
                                                                <div>
                                                                    <div class="text-[11px] text-base-content/50">"变更后"</div>
                                                                    <div class="whitespace-pre-wrap break-all">{field.2}</div>
                                                                </div>
                                                            </div>
                                                        }
                                                    })
                                                    .collect_view();
                                                view! {
                                                    <div class="rounded-box bg-base-200/50 p-3 space-y-2">
                                                        <div class="flex items-center justify-between gap-2">
                                                            <span class="font-medium text-sm">{order_log_action_label(&item.action)}</span>
                                                            <span class="text-xs text-base-content/60">{item.created_at.clone()}</span>
                                                        </div>
                                                        <div class="text-xs text-base-content/60">
                                                            {format!(
                                                                "操作人UUID: {}",
                                                                item.operator_uuid.clone().unwrap_or_else(|| "-".to_string())
                                                            )}
                                                        </div>
                                                        {if has_diff {
                                                            view! {
                                                                <div class="space-y-2">
                                                                    {diff_rows}
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="text-xs text-base-content/60">
                                                                    "该操作没有记录到具体字段差异"
                                                                </div>
                                                            }.into_any()
                                                        }}
                                                    </div>
                                                }
                                            }
                                            />
                                        </div>
                                    }.into_any()
                                }
                            })
                        }}
                    </Transition>
                </div>

                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_detail_modal.set(false)>
                        "关闭"
                    </button>
                </div>
            </div>
        </Modal>
    }
}

#[component]
fn OrderForm(
    contacts: Resource<Vec<Contact>>,
    customer_uuid: RwSignal<String>,
    amount_cents: RwSignal<String>,
    notes: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
            <label class="form-control">
                <span class="label-text text-sm">"客户"</span>
                <select
                    class="select select-bordered"
                    prop:value=move || customer_uuid.get()
                    on:change=move |ev| customer_uuid.set(event_target_value(&ev))
                >
                    <option value="">"请选择客户"</option>
                    <Transition fallback=move || view! { <option value="">"加载中..."</option> }>
                        {move || contacts.get().map(|items| {
                            view! {
                                <For
                                    each=move || items.clone().into_iter()
                                    key=|contact| contact.contact_uuid.clone()
                                    children=move |contact| {
                                        let label = contact_display_label(&contact);
                                        view! { <option value={contact.contact_uuid}>{label}</option> }
                                    }
                                />
                            }
                        })}
                    </Transition>
                </select>
            </label>
            <label class="form-control">
                <span class="label-text text-sm">"订单金额(分)"</span>
                <input
                    class="input input-bordered"
                    type="number"
                    min="0"
                    prop:value=move || amount_cents.get()
                    on:input=move |ev| amount_cents.set(event_target_value(&ev))
                    placeholder="例如：29900"
                />
            </label>
            <label class="form-control md:col-span-2">
                <span class="label-text text-sm">"订单备注"</span>
                <textarea
                    class="textarea textarea-bordered min-h-24"
                    prop:value=move || notes.get()
                    on:input=move |ev| notes.set(event_target_value(&ev))
                    placeholder="补充订单说明"
                />
            </label>
        </div>
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

fn normalize_optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
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

fn order_log_action_label(action: &str) -> &'static str {
    match action {
        "created" => "创建订单",
        "details_updated" => "编辑订单",
        "status_changed" => "状态变更",
        "cancelled" => "取消订单",
        "assignment_updated" => "派工更新",
        "settlement_updated" => "结算更新",
        _ => "订单变更",
    }
}

fn order_log_diff_items(item: &OrderChangeLogDto) -> Vec<(String, String, String)> {
    let mut result = Vec::new();
    let before = item
        .before_data
        .as_ref()
        .and_then(|value| value.as_object());
    let after = item.after_data.as_ref().and_then(|value| value.as_object());

    let keys = [
        "customer_uuid",
        "status",
        "cancellation_reason",
        "completed_at",
        "settlement_status",
        "amount_cents",
        "notes",
        "dispatch_note",
        "settlement_note",
        "scheduled_start_at",
        "scheduled_end_at",
    ];

    for key in keys {
        let before_value = before.and_then(|map| map.get(key));
        let after_value = after.and_then(|map| map.get(key));
        if before_value == after_value {
            continue;
        }
        result.push((
            order_log_field_label(key),
            format_json_field(before_value),
            format_json_field(after_value),
        ));
    }

    result
}

fn order_log_field_label(key: &str) -> String {
    match key {
        "customer_uuid" => "客户UUID".to_string(),
        "status" => "订单状态".to_string(),
        "cancellation_reason" => "取消原因".to_string(),
        "completed_at" => "完成时间".to_string(),
        "settlement_status" => "结算状态".to_string(),
        "amount_cents" => "订单金额(分)".to_string(),
        "notes" => "订单备注".to_string(),
        "dispatch_note" => "派工备注".to_string(),
        "settlement_note" => "结算备注".to_string(),
        "scheduled_start_at" => "服务开始".to_string(),
        "scheduled_end_at" => "服务结束".to_string(),
        _ => key.to_string(),
    }
}

fn format_json_field(value: Option<&serde_json::Value>) -> String {
    match value {
        None | Some(serde_json::Value::Null) => "-".to_string(),
        Some(serde_json::Value::String(value)) => {
            if value.trim().is_empty() {
                "-".to_string()
            } else {
                value.clone()
            }
        }
        Some(serde_json::Value::Number(value)) => value.to_string(),
        Some(serde_json::Value::Bool(value)) => {
            if *value {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Some(other) => serde_json::to_string_pretty(other).unwrap_or_else(|_| other.to_string()),
    }
}
