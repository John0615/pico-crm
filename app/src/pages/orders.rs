use crate::components::ui::date_picker::{FlyonDatePicker, FlyonDateTimePicker};
use crate::components::ui::modal::Modal;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable};
use crate::components::ui::toast::{error, success};
use crate::server::contact_handlers::{fetch_contacts, get_contact};
use crate::server::order_handlers::{
    fetch_orders, update_order_assignment, update_order_settlement, update_order_status,
};
use crate::server::user_handlers::{fetch_users, get_user};
use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use shared::contact::{Contact, ContactQuery};
use shared::order::{
    Order, OrderQuery, UpdateOrderAssignment, UpdateOrderSettlement, UpdateOrderStatus,
};
use shared::user::{User, UserListQuery};
use shared::ListResult;
use std::collections::{HashMap, HashSet};

impl Identifiable for Order {
    fn id(&self) -> String {
        self.uuid.clone()
    }
}

#[component]
pub fn OrdersPage() -> impl IntoView {
    let query = use_query_map();
    let refresh_count = RwSignal::new(0);
    let (status_filter, set_status_filter) = signal(String::new());
    let (contact_filter, set_contact_filter) = signal(String::new());
    let (user_filter, set_user_filter) = signal(String::new());
    let date_start = RwSignal::new(String::new());
    let date_end = RwSignal::new(String::new());

    let show_assignment_modal = RwSignal::new(false);
    let show_detail_modal = RwSignal::new(false);
    let assignment_order_uuid = RwSignal::new(String::new());
    let assigned_user_uuid = RwSignal::new(String::new());
    let scheduled_start_at = RwSignal::new(String::new());
    let scheduled_end_at = RwSignal::new(String::new());
    let dispatch_note = RwSignal::new(String::new());
    let detail_order: RwSignal<Option<Order>> = RwSignal::new(None);
    let contact_labels: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let user_labels: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let pending_contacts: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
    let pending_users: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());

    let data = Resource::new(
        move || {
            (
                status_filter.get(),
                contact_filter.get(),
                user_filter.get(),
                date_start.get(),
                date_end.get(),
                *refresh_count.read(),
                query.with(|value| value.clone()),
            )
        },
        |(status, contact, user, start, end, _, query)| async move {
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
                contact_uuid: (!contact.is_empty()).then_some(contact),
                assigned_user_uuid: (!user.is_empty()).then_some(user),
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

    let users = Resource::new(
        move || (),
        |_| async move {
            let params = UserListQuery {
                page: 1,
                page_size: 200,
                name: None,
                role: Some("user".to_string()),
                status: None,
            };

            match call_api(fetch_users(params)).await {
                Ok(result) => result.items,
                Err(err) => {
                    logging::error!("Error loading users: {err}");
                    Vec::new()
                }
            }
        },
    );

    Effect::new(move || {
        if let Some(items) = contacts.get() {
            let mut map = HashMap::new();
            for contact in items {
                map.insert(contact.contact_uuid.clone(), contact_display_label(&contact));
            }
            contact_labels.set(map);
        }
    });

    Effect::new(move || {
        if let Some(items) = users.get() {
            let mut map = HashMap::new();
            for user in items {
                map.insert(user.uuid.clone(), user_display_label(&user));
            }
            user_labels.set(map);
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
            let Some(contact_id) = order.contact_uuid.clone() else { continue };
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

    Effect::new(move || {
        if cfg!(feature = "ssr") {
            return;
        }
        let Some((items, _)) = data.get() else {
            return;
        };

        let existing = user_labels.get();
        let mut pending = pending_users.get();
        let mut missing_ids: Vec<String> = Vec::new();
        for order in &items {
            let Some(user_id) = order.assigned_user_uuid.clone() else { continue };
            if user_id.is_empty() {
                continue;
            }
            if existing.contains_key(&user_id) || pending.contains(&user_id) {
                continue;
            }
            pending.insert(user_id.clone());
            missing_ids.push(user_id);
        }
        if missing_ids.is_empty() {
            return;
        }
        pending_users.set(pending);

        for user_id in missing_ids {
            let user_labels = user_labels;
            let pending_users = pending_users;
            spawn_local(async move {
                let label = match call_api(get_user(user_id.clone())).await {
                    Ok(user) => user_display_label(&user),
                    _ => "未知员工".to_string(),
                };
                user_labels.update(|map| {
                    map.insert(user_id.clone(), label);
                });
                pending_users.update(|set| {
                    set.remove(&user_id);
                });
            });
        }
    });

    let open_assignment = move |order: Order| {
        assignment_order_uuid.set(order.uuid.clone());
        assigned_user_uuid.set(order.assigned_user_uuid.clone().unwrap_or_default());
        scheduled_start_at.set(to_datetime_local(order.scheduled_start_at.clone()));
        scheduled_end_at.set(to_datetime_local(order.scheduled_end_at.clone()));
        dispatch_note.set(order.dispatch_note.clone().unwrap_or_default());
        show_assignment_modal.set(true);
    };

    let open_detail = move |order: Order| {
        detail_order.set(Some(order));
        show_detail_modal.set(true);
    };

    let submit_assignment = move |_| {
        if is_end_before_start(&scheduled_start_at.get(), &scheduled_end_at.get()) {
            error("结束时间必须晚于开始时间".to_string());
            return;
        }

        let uuid = assignment_order_uuid.get();
        let payload = UpdateOrderAssignment {
            assigned_user_uuid: normalize_optional(&assigned_user_uuid.get()),
            scheduled_start_at: normalize_datetime_local(&scheduled_start_at.get()),
            scheduled_end_at: normalize_datetime_local(&scheduled_end_at.get()),
            dispatch_note: normalize_optional(&dispatch_note.get()),
        };
        spawn_local(async move {
            let result = call_api(update_order_assignment(uuid, payload)).await;
            match result {
                Ok(_) => {
                    success("派工已更新".to_string());
                    show_assignment_modal.set(false);
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("更新失败: {}", err)),
            }
        });
    };

    let change_status = move |uuid: String, status: String| {
        spawn_local(async move {
            let payload = UpdateOrderStatus { status };
            let result = call_api(update_order_status(uuid, payload)).await;
            match result {
                Ok(_) => {
                    success("状态已更新".to_string());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("更新失败: {}", err)),
            }
        });
    };

    let change_settlement = move |uuid: String, settlement: String| {
        spawn_local(async move {
            let payload = UpdateOrderSettlement {
                settlement_status: settlement,
                settlement_note: None,
            };
            let result = call_api(update_order_settlement(uuid, payload)).await;
            match result {
                Ok(_) => {
                    success("结算状态已更新".to_string());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("更新失败: {}", err)),
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
                        <span class="text-xs text-base-content/60">"员工"</span>
                        <Transition fallback=move || view! {
                            <select
                                class="select select-bordered min-w-[200px]"
                                prop:value=move || user_filter.get()
                                on:change=move |ev| set_user_filter.set(event_target_value(&ev))
                            >
                                <option value="">"全部"</option>
                            </select>
                        }>
                            {move || {
                                let items = users.get().unwrap_or_default();
                                let options = items
                                    .into_iter()
                                    .map(|user| {
                                        let label = user_display_label(&user);
                                        view! { <option value={user.uuid}>{label}</option> }
                                    })
                                    .collect::<Vec<_>>();
                                view! {
                                    <select
                                        class="select select-bordered min-w-[200px]"
                                        prop:value=move || user_filter.get()
                                        on:change=move |ev| set_user_filter.set(event_target_value(&ev))
                                    >
                                        <option value="">"全部"</option>
                                        {options}
                                    </select>
                                }
                            }}
                        </Transition>
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
                    <Column slot:columns prop="contact_uuid".to_string() label="客户".to_string()>
                        {
                            let item: Option<Order> = use_context::<Order>();
                            let contact_id = item.as_ref().and_then(|v| v.contact_uuid.clone());
                            let label = contact_id
                                .as_ref()
                                .map(|id| {
                                    contact_labels
                                        .get()
                                        .get(id)
                                        .cloned()
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
                    <Column
                        slot:columns
                        prop="assigned_user_uuid".to_string()
                        label="员工".to_string()
                    >
                        {
                            let item: Option<Order> = use_context::<Order>();
                            let user_id = item.as_ref().and_then(|v| v.assigned_user_uuid.clone());
                            let label = user_id
                                .as_ref()
                                .map(|id| {
                                    user_labels
                                        .get()
                                        .get(id)
                                        .cloned()
                                        .unwrap_or_else(|| {
                                            if pending_users.get().contains(id) {
                                                "加载中...".to_string()
                                            } else {
                                                "未知员工".to_string()
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
                            let uuid = item.as_ref().map(|v| v.uuid.clone()).unwrap_or_default();
                            let status_value = item.as_ref().map(|v| v.status.clone()).unwrap_or_default();
                            view! {
                                <select
                                    class="select select-bordered select-xs"
                                    prop:value=move || status_value.clone()
                                    on:change=move |ev| change_status(uuid.clone(), event_target_value(&ev))
                                >
                                    <option value="pending">"待确认"</option>
                                    <option value="confirmed">"已确认"</option>
                                    <option value="dispatching">"派工中"</option>
                                    <option value="in_service">"服务中"</option>
                                    <option value="completed">"已完成"</option>
                                    <option value="cancelled">"已取消"</option>
                                </select>
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
                            let uuid = item.as_ref().map(|v| v.uuid.clone()).unwrap_or_default();
                            let settlement_value = item.as_ref().map(|v| v.settlement_status.clone()).unwrap_or_default();
                            view! {
                                <select
                                    class="select select-bordered select-xs"
                                    prop:value=move || settlement_value.clone()
                                    on:change=move |ev| change_settlement(uuid.clone(), event_target_value(&ev))
                                >
                                    <option value="unsettled">"未结算"</option>
                                    <option value="settled">"已结算"</option>
                                </select>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        prop="actions".to_string()
                        label="操作".to_string()
                        class="text-right"
                    >
                        <div class="flex items-center justify-end gap-2">
                            {
                                let item: Option<Order> = use_context::<Order>();
                                let order = item.clone();
                                let detail = item.clone();
                                view! {
                                    <button class="btn btn-ghost btn-xs" on:click=move |_| {
                                        if let Some(value) = order.clone() {
                                            open_assignment(value);
                                        }
                                    }>
                                        "派工"
                                    </button>
                                    <button class="btn btn-outline btn-xs" on:click=move |_| {
                                        if let Some(value) = detail.clone() {
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

        <Modal show=show_assignment_modal>
            <h3 class="text-lg font-semibold mb-4">"派工信息"</h3>
            <div class="space-y-3">
                <label class="form-control w-full">
                    <div class="label"><span class="label-text">"员工"</span></div>
                    <Transition fallback=move || view! {
                        <select
                            class="select select-bordered w-full"
                            prop:value=move || assigned_user_uuid.get()
                            on:change=move |ev| assigned_user_uuid.set(event_target_value(&ev))
                        >
                            <option value="">"未分配"</option>
                        </select>
                    }>
                        {move || {
                            let items = users.get().unwrap_or_default();
                            let options = items
                                .into_iter()
                                .map(|user| {
                                    let label = user_display_label(&user);
                                    view! { <option value={user.uuid}>{label}</option> }
                                })
                                .collect::<Vec<_>>();
                            view! {
                                <select
                                    class="select select-bordered w-full"
                                    prop:value=move || assigned_user_uuid.get()
                                    on:change=move |ev| assigned_user_uuid.set(event_target_value(&ev))
                                >
                                    <option value="">"未分配"</option>
                                    {options}
                                </select>
                            }
                        }}
                    </Transition>
                </label>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <label class="form-control w-full">
                        <div class="label"><span class="label-text">"开始时间"</span></div>
                        <FlyonDateTimePicker
                            value=scheduled_start_at
                            class="input input-bordered".to_string()
                        />
                    </label>
                    <label class="form-control w-full">
                        <div class="label"><span class="label-text">"结束时间"</span></div>
                        <FlyonDateTimePicker
                            value=scheduled_end_at
                            class="input input-bordered".to_string()
                        />
                    </label>
                </div>
                <label class="form-control w-full">
                    <div class="label"><span class="label-text">"派工备注"</span></div>
                    <textarea
                        class="textarea textarea-bordered w-full"
                        rows="2"
                        prop:value=move || dispatch_note.get()
                        on:input=move |ev| dispatch_note.set(event_target_value(&ev))
                    ></textarea>
                </label>
                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_assignment_modal.set(false)>
                        "取消"
                    </button>
                    <button class="btn btn-primary" on:click=submit_assignment>
                        "保存"
                    </button>
                </div>
            </div>
        </Modal>

        <Modal show=show_detail_modal>
            <div class="space-y-4">
                <h3 class="text-lg font-semibold">"订单详情"</h3>
                {move || {
                    if let Some(order) = detail_order.get() {
                        view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                {detail_item("订单ID", order.uuid.clone())}
                                {detail_item("需求单ID", display_optional(order.request_id.clone()))}
                                {detail_item("客户UUID", display_optional(order.contact_uuid.clone()))}
                                {detail_item("员工UUID", display_optional(order.assigned_user_uuid.clone()))}
                                {detail_item("状态", order.status.clone())}
                                {detail_item("结算状态", order.settlement_status.clone())}
                                {detail_item("服务开始", display_optional(order.scheduled_start_at.clone()))}
                                {detail_item("服务结束", display_optional(order.scheduled_end_at.clone()))}
                                {detail_item("派工备注", display_optional(order.dispatch_note.clone()))}
                                {detail_item("备注", display_optional(order.notes.clone()))}
                                {detail_item("结算备注", display_optional(order.settlement_note.clone()))}
                                {detail_item("创建时间", order.inserted_at.clone())}
                                {detail_item("更新时间", order.updated_at.clone())}
                            </div>
                        }.into_view()
                    } else {
                        view! { <div class="text-sm text-base-content/60">"暂无详情"</div> }.into_view()
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
    let company = contact.company.trim();
    let mut label = String::new();
    if !name.is_empty() {
        label.push_str(name);
    }
    if !company.is_empty() {
        if !label.is_empty() {
            label.push_str(" / ");
        }
        label.push_str(company);
    }
    if label.is_empty() {
        label = "未命名客户".to_string();
    }
    let extra = contact.phone_number.trim();
    if !extra.is_empty() {
        format!("{} ({})", label, extra)
    } else {
        let email = contact.email.trim();
        if !email.is_empty() {
            format!("{} ({})", label, email)
        } else {
            label
        }
    }
}

fn user_display_label(user: &User) -> String {
    if let Some(phone) = user.phone_number.clone().filter(|value| !value.is_empty()) {
        format!("{} ({})", user.user_name, phone)
    } else if let Some(email) = user.email.clone().filter(|value| !value.is_empty()) {
        format!("{} ({})", user.user_name, email)
    } else {
        user.user_name.clone()
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

fn normalize_datetime_local(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = trimmed.replace('T', " ");
    if normalized.len() == 16 {
        Some(format!("{}:00", normalized))
    } else {
        Some(normalized)
    }
}

fn to_datetime_local(value: Option<String>) -> String {
    let Some(value) = value else { return String::new() };
    if value.trim().is_empty() {
        return String::new();
    }
    let replaced = value.replace('T', " ");
    if replaced.len() >= 16 {
        replaced[..16].to_string()
    } else {
        replaced
    }
}

fn is_end_before_start(start: &str, end: &str) -> bool {
    match (normalize_datetime_local(start), normalize_datetime_local(end)) {
        (Some(start), Some(end)) => end <= start,
        _ => false,
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
