use crate::components::ui::date_picker::{FlyonDatePicker, FlyonDateTimePicker};
use crate::components::ui::modal::{Modal, DETAIL_MODAL_BOX_CLASS};
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable};
use chrono::{Duration, NaiveDateTime};
use leptos::prelude::*;
use shared::order::{Order, OrderFeedback};
use shared::schedule::Schedule;
use shared::service_catalog::ServiceCatalog;
use shared::user::User;
use std::collections::{HashMap, HashSet};

use super::support::*;

#[component]
pub fn SchedulesFiltersCard(
    view_mode: RwSignal<String>,
    status_filter: RwSignal<String>,
    user_filter: RwSignal<String>,
    list_date_start: RwSignal<String>,
    list_date_end: RwSignal<String>,
    calendar_date_start: RwSignal<String>,
    calendar_date_end: RwSignal<String>,
    users: Resource<Vec<User>>,
    #[prop(into)] is_worker: Signal<bool>,
) -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow-sm">
            <div class="card-body p-4 flex flex-col gap-3 md:flex-row md:items-end">
                <div class="flex flex-col gap-1">
                    <span class="text-xs text-base-content/60">"状态"</span>
                    <select
                        class="select select-bordered min-w-[160px]"
                        prop:value=move || status_filter.get()
                        on:change=move |ev| status_filter.set(event_target_value(&ev))
                    >
                        <option value="">"全部"</option>
                        <option value="planned">"已排班"</option>
                        <option value="in_service">"服务中"</option>
                        <option value="done">"已完成"</option>
                        <option value="cancelled">"已取消"</option>
                    </select>
                </div>
                <Show when=move || !is_worker.get() fallback=|| ()>
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"员工"</span>
                        <Transition fallback=move || view! {
                            <select
                                class="select select-bordered min-w-[200px]"
                                prop:value=move || user_filter.get()
                                on:change=move |ev| user_filter.set(event_target_value(&ev))
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
                                        on:change=move |ev| user_filter.set(event_target_value(&ev))
                                    >
                                        <option value="">"全部"</option>
                                        {options}
                                    </select>
                                }
                            }}
                        </Transition>
                    </div>
                </Show>
                <Show
                    when=move || view_mode.get() == "calendar"
                    fallback=move || view! {
                        <>
                            <div class="flex flex-col gap-1">
                                <span class="text-xs text-base-content/60">"服务开始"</span>
                                <div class="relative">
                                    <FlyonDatePicker
                                        value=list_date_start
                                        class="input input-bordered pr-9 w-full".to_string()
                                    />
                                    <Show when=move || !list_date_start.get().trim().is_empty()>
                                        <button
                                            type="button"
                                            class="absolute right-2 top-1/2 -translate-y-1/2 text-base-content/40 hover:text-base-content text-lg leading-none"
                                            on:click=move |_| list_date_start.set(String::new())
                                        >
                                            "×"
                                        </button>
                                    </Show>
                                </div>
                            </div>
                            <div class="flex flex-col gap-1">
                                <span class="text-xs text-base-content/60">"服务结束"</span>
                                <div class="relative">
                                    <FlyonDatePicker
                                        value=list_date_end
                                        class="input input-bordered pr-9 w-full".to_string()
                                    />
                                    <Show when=move || !list_date_end.get().trim().is_empty()>
                                        <button
                                            type="button"
                                            class="absolute right-2 top-1/2 -translate-y-1/2 text-base-content/40 hover:text-base-content text-lg leading-none"
                                            on:click=move |_| list_date_end.set(String::new())
                                        >
                                            "×"
                                        </button>
                                    </Show>
                                </div>
                            </div>
                        </>
                    }
                >
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"周开始"</span>
                        <FlyonDatePicker
                            value=calendar_date_start
                            class="input input-bordered".to_string()
                        />
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"周结束"</span>
                        <FlyonDatePicker
                            value=calendar_date_end
                            class="input input-bordered".to_string()
                        />
                    </div>
                </Show>
            </div>
        </div>
    }
}

#[component]
pub fn SchedulesBacklogPanel(
    available_orders: Resource<(Vec<Order>, u64)>,
    contact_labels: RwSignal<HashMap<String, String>>,
    pending_contacts: RwSignal<HashSet<String>>,
    open_new_schedule_for_order: Callback<String>,
) -> impl IntoView {
    view! {
        <div class="overflow-x-auto overflow-y-auto h-[calc(100vh-260px)] bg-base-100 rounded-lg shadow">
            <Transition fallback=move || view! {
                <div class="p-6 text-sm text-base-content/60">"加载中..."</div>
            }>
                {move || {
                    let (orders, _) = available_orders.get().unwrap_or_default();

                    if orders.is_empty() {
                        return view! {
                            <div class="p-6 text-sm text-base-content/60 space-y-2">
                                <div class="font-medium text-base-content">"暂无待排班订单"</div>
                                <div>"当前可派工订单都已处理，可以切换到列表或日历查看已排班结果。"</div>
                            </div>
                        }
                        .into_any();
                    }

                    let contact_labels_store = StoredValue::new(contact_labels.get());
                    let pending_contacts_store = StoredValue::new(pending_contacts.get());
                    let open_new_schedule_for_order = open_new_schedule_for_order.clone();

                    view! {
                        <table class="table table-sm">
                            <thead>
                                <tr>
                                    <th>"订单"</th>
                                    <th>"客户"</th>
                                    <th>"状态"</th>
                                    <th>"金额"</th>
                                    <th>"创建时间"</th>
                                    <th>"备注"</th>
                                    <th class="text-right">"操作"</th>
                                </tr>
                            </thead>
                            <tbody>
                                <For
                                    each=move || orders.clone()
                                    key=|order| order.uuid.clone()
                                    children=move |order| {
                                        let order_uuid = order.uuid.clone();
                                        let short_id = shorten_uuid(&order.uuid);
                                        let contact_label = contact_labels_store.with_value(|labels| {
                                            pending_contacts_store.with_value(|pending| {
                                                order_contact_label(&order, labels, pending)
                                            })
                                        });
                                        let status_label = order_status_label(&order.status).to_string();
                                        let status_class = order_status_badge_class(&order.status).to_string();
                                        let amount_label = if order.amount_cents > 0 {
                                            format!("{} 分", order.amount_cents)
                                        } else {
                                            "-".to_string()
                                        };
                                        let created_at = display_optional(Some(order.inserted_at.clone()));
                                        let notes = display_optional(order.notes.clone());
                                        let open_new_schedule_for_order =
                                            open_new_schedule_for_order.clone();

                                        view! {
                                            <tr class="hover">
                                                <td>
                                                    <div class="font-medium">{format!("订单 {}", short_id)}</div>
                                                    <div class="text-xs text-base-content/50 break-all">{order.uuid}</div>
                                                </td>
                                                <td class="max-w-[220px]">
                                                    <div class="font-medium break-words">{contact_label}</div>
                                                </td>
                                                <td>
                                                    <span class=format!("badge {}", status_class)>{status_label}</span>
                                                </td>
                                                <td class="text-xs">{amount_label}</td>
                                                <td class="text-xs">{created_at}</td>
                                                <td class="max-w-[260px] text-xs whitespace-pre-wrap break-words">
                                                    {notes}
                                                </td>
                                                <td class="text-right">
                                                    <button
                                                        class="btn btn-primary btn-xs"
                                                        on:click=move |_| {
                                                            open_new_schedule_for_order
                                                                .run(order_uuid.clone())
                                                        }
                                                    >
                                                        "去排班"
                                                    </button>
                                                </td>
                                            </tr>
                                        }
                                    }
                                />
                            </tbody>
                        </table>
                    }
                    .into_any()
                }}
            </Transition>
        </div>

        <Transition>
            {move || {
                available_orders.with(|data| {
                    data.as_ref()
                        .map(|(_, total)| view! { <Pagination total_items=*total /> })
                })
            }}
        </Transition>
    }
}

#[component]
pub fn SchedulesCalendarPanel(
    data: Resource<(Vec<Schedule>, u64)>,
    calendar_date_start: RwSignal<String>,
    calendar_date_end: RwSignal<String>,
    user_labels: RwSignal<HashMap<String, String>>,
    contact_labels: RwSignal<HashMap<String, String>>,
    pending_contacts: RwSignal<HashSet<String>>,
    user_filter: RwSignal<String>,
    #[prop(into)] is_worker: Signal<bool>,
    open_assignment: Callback<Schedule>,
    open_detail: Callback<Schedule>,
) -> impl IntoView {
    view! {
        {move || {
            let (week_start, week_end) =
                calendar_week_range(&calendar_date_start.get(), &calendar_date_end.get());
            let week_label = format!("{} ~ {}", format_date(week_start), format_date(week_end));
            let row_height: i32 = 48;

            view! {
                <div class="space-y-3">
                    <div class="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
                        <div class="flex items-center gap-2">
                            <button
                                class="btn btn-sm"
                                on:click=move |_| {
                                    shift_week_range(-1, calendar_date_start, calendar_date_end)
                                }
                            >
                                "上一周"
                            </button>
                            <button
                                class="btn btn-sm"
                                on:click=move |_| {
                                    shift_week_range(1, calendar_date_start, calendar_date_end)
                                }
                            >
                                "下一周"
                            </button>
                        </div>
                        <div class="text-sm font-semibold">{week_label}</div>
                    </div>
                    <div class="overflow-auto h-[calc(100vh-260px)] bg-base-100 rounded-lg shadow">
                        <Transition fallback=move || view! {
                            <div class="p-6 text-sm text-base-content/60">"加载中..."</div>
                        }>
                            {move || {
                                let Some((items, total)) = data.get() else {
                                    return view! { <div class="p-6 text-sm text-base-content/60">"加载中..."</div> }
                                        .into_any();
                                };
                                let days = (0..7)
                                    .map(|offset| week_start + Duration::days(offset as i64))
                                    .collect::<Vec<_>>();
                                let user_labels_snapshot = user_labels.get();
                                let contact_labels_snapshot = contact_labels.get();
                                let pending_contacts_snapshot = pending_contacts.get();
                                let user_filter_value = user_filter.get();
                                let calendar_items = build_calendar_items(&items, week_start, week_end);
                                if calendar_items.is_empty() {
                                    return view! {
                                        <div class="p-6 text-sm text-base-content/60 space-y-3">
                                            <div>"本周暂无排班"</div>
                                        </div>
                                    }
                                    .into_any();
                                }
                                let (start_hour, end_hour) = calendar_time_bounds(&calendar_items);
                                let columns_store = StoredValue::new(calendar_columns(
                                    &calendar_items,
                                    &user_labels_snapshot,
                                    &user_filter_value,
                                ));
                                let grouped_store =
                                    StoredValue::new(group_calendar_items(calendar_items));
                                let hours_store =
                                    StoredValue::new((start_hour..end_hour).collect::<Vec<_>>());
                                let contact_labels_store = StoredValue::new(contact_labels_snapshot);
                                let pending_contacts_store =
                                    StoredValue::new(pending_contacts_snapshot);
                                let total_height = ((end_hour - start_hour) as i32) * row_height;
                                let grid_style = format!(
                                    "grid-template-columns: 84px repeat({}, minmax(180px, 1fr));",
                                    columns_store.with_value(|cols| cols.len())
                                );
                                let truncated = total > items.len() as u64;
                                let open_assignment = open_assignment.clone();
                                let open_detail = open_detail.clone();

                                view! {
                                    <div class="min-w-max p-4 space-y-4">
                                        <div
                                            class="grid sticky top-0 z-20 bg-base-100 border-b border-base-200"
                                            style=grid_style.clone()
                                        >
                                            <div class="px-3 py-2 text-xs font-semibold text-base-content/70">
                                                "时间"
                                            </div>
                                            <For
                                                each=move || columns_store.with_value(|cols| cols.clone())
                                                key=|column| column.id.clone()
                                                children=move |column| {
                                                    view! {
                                                        <div class="px-3 py-2 text-xs font-semibold text-base-content/70 border-l border-base-200">
                                                            {column.label}
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>
                                        {truncated.then(|| view! {
                                            <div class="text-xs text-warning px-2">
                                                "当前周排班较多，仅展示前 500 条。"
                                            </div>
                                        })}
                                        <For
                                            each=move || days.clone()
                                            key=|day| format_date(*day)
                                            children=move |day| {
                                                let day_label = format!(
                                                    "{} {}",
                                                    format_date(day),
                                                    weekday_label(day)
                                                );
                                                let day_items = StoredValue::new(
                                                    grouped_store
                                                        .with_value(|map| map.get(&day).cloned().unwrap_or_default()),
                                                );
                                                let columns_for_day = columns_store;
                                                let hours_for_day = hours_store;
                                                let contact_labels_for_day = contact_labels_store;
                                                let pending_contacts_for_day = pending_contacts_store;
                                                let open_assignment = open_assignment.clone();
                                                let open_detail = open_detail.clone();
                                                view! {
                                                    <div class="rounded-lg border border-base-200 overflow-hidden">
                                                        <div class="px-3 py-2 bg-base-200 text-sm font-semibold">
                                                            {day_label}
                                                        </div>
                                                        <div class="grid" style=grid_style.clone()>
                                                            <div class="border-r border-base-200">
                                                                <div class="flex flex-col text-xs text-base-content/60" style=format!("height: {}px;", total_height)>
                                                                    <For
                                                                        each=move || {
                                                                            hours_for_day.with_value(|hours| hours.clone())
                                                                        }
                                                                        key=|hour| *hour
                                                                        children=move |hour| {
                                                                            view! {
                                                                                <div
                                                                                    class="border-b border-base-200 px-2 py-1"
                                                                                    style=format!("height: {}px;", row_height)
                                                                                >
                                                                                    {format!("{:02}:00", hour)}
                                                                                </div>
                                                                            }
                                                                        }
                                                                    />
                                                                </div>
                                                            </div>
                                                            <For
                                                                each=move || {
                                                                    columns_for_day.with_value(|cols| cols.clone())
                                                                }
                                                                key=|column| column.id.clone()
                                                                children=move |column| {
                                                                    let schedule_items = StoredValue::new(
                                                                        day_items
                                                                            .with_value(|map| {
                                                                                map.get(&column.id)
                                                                                    .cloned()
                                                                                    .unwrap_or_default()
                                                                            }),
                                                                    );
                                                                    let hours_for_column = hours_for_day;
                                                                    let open_assignment =
                                                                        open_assignment.clone();
                                                                    let open_detail =
                                                                        open_detail.clone();
                                                                    let is_worker = is_worker.clone();
                                                                    view! {
                                                                        <div
                                                                            class="relative border-l border-base-200"
                                                                            style=format!("height: {}px;", total_height)
                                                                        >
                                                                            <div class="absolute inset-0">
                                                                                <For
                                                                                    each=move || {
                                                                                        hours_for_column
                                                                                            .with_value(|hours| hours.clone())
                                                                                    }
                                                                                    key=|hour| *hour
                                                                                    children=move |_| {
                                                                                        view! {
                                                                                            <div
                                                                                                class="border-b border-base-200"
                                                                                                style=format!("height: {}px;", row_height)
                                                                                            ></div>
                                                                                        }
                                                                                    }
                                                                                />
                                                                            </div>
                                                                            <div class="absolute inset-0">
                                                                                <For
                                                                                    each=move || {
                                                                                        schedule_items
                                                                                            .with_value(|items| items.clone())
                                                                                    }
                                                                                    key=|item| item.schedule.order_uuid.clone()
                                                                                    children=move |item| {
                                                                                        let schedule = item.schedule.clone();
                                                                                        let status_label =
                                                                                            schedule_status_label(&schedule).to_string();
                                                                                        let status_class =
                                                                                            schedule_card_classes(&schedule);
                                                                                        let contact_label =
                                                                                            contact_labels_for_day
                                                                                                .with_value(|labels| {
                                                                                                    pending_contacts_for_day
                                                                                                        .with_value(|pending| {
                                                                                                            schedule_contact_label(
                                                                                                                &schedule,
                                                                                                                labels,
                                                                                                                pending,
                                                                                                            )
                                                                                                        })
                                                                                                });
                                                                                        let time_label =
                                                                                            format_time_range(item.start, item.end);
                                                                                        let contact_label_title =
                                                                                            contact_label.clone();
                                                                                        let time_label_title =
                                                                                            time_label.clone();
                                                                                        let can_edit = !is_worker.get()
                                                                                            && schedule.schedule_status == "planned";
                                                                                        let schedule_for_click =
                                                                                            schedule.clone();
                                                                                        let open_assignment =
                                                                                            open_assignment.clone();
                                                                                        let open_detail =
                                                                                            open_detail.clone();
                                                                                        {calendar_event_position(
                                                                                            item.start,
                                                                                            item.end,
                                                                                            start_hour,
                                                                                            end_hour,
                                                                                            row_height,
                                                                                        )
                                                                                        .map(|(top, height)| {
                                                                                            view! {
                                                                                                <div
                                                                                                    class=format!(
                                                                                                        "absolute left-2 right-2 rounded-md border px-2 py-1 text-xs shadow-sm cursor-pointer hover:shadow {}",
                                                                                                        status_class
                                                                                                    )
                                                                                                    style=format!("top: {}px; height: {}px;", top, height)
                                                                                                    title=format!("{} | {}", contact_label_title, time_label_title)
                                                                                                    on:click=move |_| {
                                                                                                        if can_edit {
                                                                                                            open_assignment
                                                                                                                .run(schedule_for_click.clone());
                                                                                                        } else {
                                                                                                            open_detail
                                                                                                                .run(schedule_for_click.clone());
                                                                                                        }
                                                                                                    }
                                                                                                >
                                                                                                    <div class="font-semibold truncate">{contact_label}</div>
                                                                                                    <div class="text-[11px] opacity-80">{time_label}</div>
                                                                                                    <div class="text-[11px] opacity-70">{status_label}</div>
                                                                                                </div>
                                                                                            }
                                                                                        })}
                                                                                    }
                                                                                />
                                                                            </div>
                                                                        </div>
                                                                    }
                                                                }
                                                            />
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                }
                                .into_any()
                            }}
                        </Transition>
                    </div>
                </div>
            }
            .into_view()
        }}
    }
}

#[component]
pub fn SchedulesListPanel(
    data: Resource<(Vec<Schedule>, u64)>,
    contact_labels: RwSignal<HashMap<String, String>>,
    pending_contacts: RwSignal<HashSet<String>>,
    user_labels: RwSignal<HashMap<String, String>>,
    pending_users: RwSignal<HashSet<String>>,
    #[prop(into)] is_worker: Signal<bool>,
    open_assignment: Callback<Schedule>,
    cancel_assignment: Callback<String>,
    update_status: Callback<(String, String)>,
    open_detail: Callback<Schedule>,
) -> impl IntoView {
    view! {
        <div class="overflow-x-auto overflow-y-auto h-[calc(100vh-260px)] bg-base-100 rounded-lg shadow">
            <DaisyTable data=data.clone()>
                <Column
                    slot:columns
                    freeze=true
                    prop="order_uuid".to_string()
                    label="订单ID".to_string()
                    class="font-semibold"
                >
                    {
                        let item: Option<Schedule> = use_context::<Schedule>();
                        view! {
                            <span class="text-xs">
                                {item.as_ref().map(|v| v.order_uuid.clone()).unwrap_or_default()}
                            </span>
                        }
                    }
                </Column>
                <Column slot:columns prop="customer_uuid".to_string() label="客户".to_string()>
                    {
                        let item: Option<Schedule> = use_context::<Schedule>();
                        let contact_id = item.as_ref().and_then(|v| v.customer_uuid.clone());
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
                        let item: Option<Schedule> = use_context::<Schedule>();
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
                <Column slot:columns prop="time".to_string() label="服务时间".to_string()>
                    {
                        let item: Option<Schedule> = use_context::<Schedule>();
                        let label = item
                            .as_ref()
                            .map(|schedule| {
                                format_time_window(
                                    schedule.scheduled_start_at.clone(),
                                    schedule.scheduled_end_at.clone(),
                                )
                            })
                            .unwrap_or_else(|| "-".to_string());
                        view! { <span class="text-xs">{label}</span> }
                    }
                </Column>
                <Column slot:columns prop="schedule_status".to_string() label="排班状态".to_string()>
                    {
                        let item: Option<Schedule> = use_context::<Schedule>();
                        let status_value = item
                            .as_ref()
                            .map(|v| v.schedule_status.clone())
                            .unwrap_or_default();
                        let status_label = item
                            .as_ref()
                            .map(schedule_status_label)
                            .unwrap_or("未知");
                        let badge_class = schedule_status_badge_class(&status_value);
                        view! {
                            <span class=format!("badge {}", badge_class)>
                                {status_label}
                            </span>
                        }
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
                            let item: Option<Schedule> = use_context::<Schedule>();
                            let schedule = StoredValue::new(item);
                            let has_schedule = schedule.with_value(|value| value.is_some());
                            let can_assign = schedule.with_value(|value| {
                                value
                                    .as_ref()
                                    .map(|value| value.schedule_status == "planned")
                                    .unwrap_or(false)
                            });
                            let can_cancel = schedule.with_value(|value| {
                                value
                                    .as_ref()
                                    .map(|value| {
                                        value.schedule_status == "planned"
                                            || value.schedule_status == "in_service"
                                    })
                                    .unwrap_or(false)
                            });
                            let can_start = schedule.with_value(|value| {
                                value
                                    .as_ref()
                                    .map(|value| value.schedule_status == "planned")
                                    .unwrap_or(false)
                            });
                            let can_done = schedule.with_value(|value| {
                                value
                                    .as_ref()
                                    .map(|value| value.schedule_status == "in_service")
                                    .unwrap_or(false)
                            });
                            let open_assignment = open_assignment.clone();
                            let cancel_assignment = cancel_assignment.clone();
                            let update_status = update_status.clone();
                            let open_detail = open_detail.clone();
                            let is_worker = is_worker.clone();
                            view! {
                                <Show when=move || { has_schedule && !is_worker.get() }>
                                    <Show when=move || { can_assign }>
                                        <button
                                            class="btn btn-soft btn-primary btn-xs"
                                            on:click=move |_| {
                                                if let Some(value) = schedule.with_value(|value| value.clone()) {
                                                    open_assignment.run(value);
                                                }
                                            }
                                        >
                                            "排班"
                                        </button>
                                    </Show>
                                    <Show when=move || { can_cancel }>
                                        <button
                                            class="btn btn-soft btn-error btn-xs"
                                            on:click=move |_| {
                                                if let Some(value) = schedule.with_value(|value| value.clone()) {
                                                    cancel_assignment.run(value.order_uuid.clone());
                                                }
                                            }
                                        >
                                            "取消"
                                        </button>
                                    </Show>
                                </Show>
                                <Show when=move || { has_schedule && is_worker.get() }>
                                    <Show when=move || { can_start }>
                                        <button
                                            class="btn btn-soft btn-warning btn-xs"
                                            on:click=move |_| {
                                                if let Some(value) = schedule.with_value(|value| value.clone()) {
                                                    update_status.run((
                                                        value.order_uuid.clone(),
                                                        "in_service".to_string(),
                                                    ));
                                                }
                                            }
                                        >
                                            "开始服务"
                                        </button>
                                    </Show>
                                    <Show when=move || { can_done }>
                                        <button
                                            class="btn btn-soft btn-success btn-xs"
                                            on:click=move |_| {
                                                if let Some(value) = schedule.with_value(|value| value.clone()) {
                                                    update_status.run((
                                                        value.order_uuid.clone(),
                                                        "done".to_string(),
                                                    ));
                                                }
                                            }
                                        >
                                            "完成服务"
                                        </button>
                                    </Show>
                                </Show>
                                <button
                                    class="btn btn-ghost btn-xs"
                                    on:click=move |_| {
                                        if let Some(value) = schedule.with_value(|value| value.clone()) {
                                            open_detail.run(value);
                                        }
                                    }
                                >
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
                    data.as_ref()
                        .map(|data| view! { <Pagination total_items=data.1 /> })
                })
            }}
        </Transition>
    }
}

#[component]
pub fn NewScheduleModal(
    show: RwSignal<bool>,
    new_order_uuid: RwSignal<String>,
    available_orders: Resource<(Vec<Order>, u64)>,
    contact_labels: RwSignal<HashMap<String, String>>,
    pending_contacts: RwSignal<HashSet<String>>,
    service_catalogs: Resource<Vec<ServiceCatalog>>,
    new_service_type: RwSignal<String>,
    duration_overridden: RwSignal<bool>,
    new_duration_minutes: RwSignal<i64>,
    new_expected_start_at: RwSignal<String>,
    new_expected_end_at: RwSignal<String>,
    users: Resource<Vec<User>>,
    #[prop(into)] new_schedule_time_window: Signal<Option<(NaiveDateTime, NaiveDateTime)>>,
    new_schedule_conflict_snapshot: RwSignal<Option<Vec<Schedule>>>,
    new_assigned_user_uuid: RwSignal<String>,
    #[prop(into)] conflict_state: Signal<ConflictState>,
    #[prop(into)] can_submit_new: Signal<bool>,
    creating_schedule: RwSignal<bool>,
    submit_new_schedule: Callback<()>,
) -> impl IntoView {
    view! {
        <Modal show=show>
            <Show when=move || show.get() fallback=|| ()>
                <h3 class="text-lg font-semibold mb-4">"安排订单"</h3>
                <div class="space-y-3">
                    <Transition fallback=move || view! {
                        <label class="form-control w-full">
                            <div class="label"><span class="label-text">"订单"</span></div>
                            <input class="input input-bordered w-full" value="加载中..." readonly=true />
                        </label>
                    }>
                        {move || {
                            let order_uuid = new_order_uuid.get();
                            let items = available_orders
                                .get()
                                .map(|(items, _)| items)
                                .unwrap_or_default();
                            let contact_labels_snapshot = contact_labels.get();
                            let pending_snapshot = pending_contacts.get();
                            let label = items
                                .iter()
                                .find(|order| order.uuid == order_uuid)
                                .map(|order| {
                                    order_option_label(order, &contact_labels_snapshot, &pending_snapshot)
                                })
                                .unwrap_or_else(|| {
                                    if order_uuid.trim().is_empty() {
                                        "未选择订单".to_string()
                                    } else {
                                        format!("订单 {}", shorten_uuid(&order_uuid))
                                    }
                                });

                            view! {
                                <label class="form-control w-full">
                                    <div class="label"><span class="label-text">"当前订单"</span></div>
                                    <input
                                        class="input input-bordered w-full"
                                        prop:value=label
                                        readonly=true
                                    />
                                </label>
                            }
                            .into_any()
                        }}
                    </Transition>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <label class="form-control w-full">
                            <div class="label"><span class="label-text">"服务类型"</span></div>
                            <Transition fallback=move || view! {
                                <select class="select select-bordered w-full" disabled=true>
                                    <option value="">"加载中..."</option>
                                </select>
                            }>
                                {move || {
                                    let items = service_catalogs.get().unwrap_or_default();
                                    let options = items
                                        .into_iter()
                                        .map(|item: ServiceCatalog| {
                                            let name = item.name.clone();
                                            let label = if item.base_price_cents > 0 {
                                                format!("{} ({} 分)", name, item.base_price_cents)
                                            } else {
                                                name.clone()
                                            };
                                            view! { <option value={name}>{label}</option> }
                                        })
                                        .collect::<Vec<_>>();
                                    view! {
                                        <select
                                            class="select select-bordered w-full"
                                            prop:value=move || new_service_type.get()
                                            on:change=move |ev| {
                                                duration_overridden.set(false);
                                                new_service_type.set(event_target_value(&ev))
                                            }
                                        >
                                            <option value="">"请选择服务类型"</option>
                                            {options}
                                        </select>
                                    }
                                    .into_any()
                                }}
                            </Transition>
                        </label>
                        <label class="form-control w-full">
                            <div class="label"><span class="label-text">"服务时长"</span></div>
                            {move || {
                                let mut options: Vec<(i64, String)> = DURATION_OPTIONS
                                    .iter()
                                    .map(|(value, label)| (*value, (*label).to_string()))
                                    .collect();
                                let current = new_duration_minutes.get();
                                if current > 0 && !options.iter().any(|(value, _)| *value == current) {
                                    options.insert(0, (current, duration_label(current)));
                                }
                                let options = options
                                    .into_iter()
                                    .map(|(value, label)| {
                                        view! { <option value={value.to_string()}>{label}</option> }
                                    })
                                    .collect::<Vec<_>>();
                                view! {
                                    <select
                                        class="select select-bordered w-full"
                                        prop:value=move || {
                                            let value = new_duration_minutes.get();
                                            if value <= 0 {
                                                String::new()
                                            } else {
                                                value.to_string()
                                            }
                                        }
                                        on:change=move |ev| {
                                            let value = event_target_value(&ev);
                                            let minutes = value.parse::<i64>().unwrap_or(0);
                                            new_duration_minutes.set(minutes);
                                            duration_overridden.set(true);
                                        }
                                    >
                                        <option value="">"请选择服务时长"</option>
                                        {options}
                                    </select>
                                }
                            }}
                        </label>
                    </div>
                    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <label class="form-control w-full">
                            <div class="label"><span class="label-text">"期望时间"</span></div>
                            <FlyonDateTimePicker
                                value=new_expected_start_at
                                class="input input-bordered".to_string()
                            />
                        </label>
                        <label class="form-control w-full">
                            <div class="label"><span class="label-text">"服务结束"</span></div>
                            <input
                                class="input input-bordered w-full"
                                prop:value=move || new_expected_end_at.get()
                                placeholder="自动计算"
                                readonly
                            />
                        </label>
                    </div>
                    <label class="form-control w-full">
                        <div class="label"><span class="label-text">"指派员工"</span></div>
                        <Transition fallback=move || view! {
                            <select class="select select-bordered w-full" disabled=true>
                                <option value="">"加载中..."</option>
                            </select>
                        }>
                            {move || {
                                let items = users.get().unwrap_or_default();
                                let Some((start, end)) = new_schedule_time_window.get() else {
                                    return view! {
                                        <select class="select select-bordered w-full" disabled=true>
                                            <option value="">"请先选择期望时间"</option>
                                        </select>
                                    }
                                    .into_any();
                                };
                                let Some(conflict_items) = new_schedule_conflict_snapshot.get() else {
                                    return view! {
                                        <select class="select select-bordered w-full" disabled=true>
                                            <option value="">"可排班员工加载中..."</option>
                                        </select>
                                    }
                                    .into_any();
                                };
                                let options = items
                                    .into_iter()
                                    .filter(|user| {
                                        find_conflicting_schedule_for_user(
                                            &user.uuid,
                                            start,
                                            end,
                                            &conflict_items,
                                        )
                                        .is_none()
                                    })
                                    .map(|user| {
                                        let label = user_display_label(&user);
                                        view! { <option value={user.uuid}>{label}</option> }
                                    })
                                    .collect::<Vec<_>>();
                                let is_empty = options.is_empty();
                                view! {
                                    <select
                                        class="select select-bordered w-full"
                                        disabled=is_empty
                                        prop:value=move || new_assigned_user_uuid.get()
                                        on:change=move |ev| {
                                            new_assigned_user_uuid.set(event_target_value(&ev))
                                        }
                                    >
                                        <option value="">
                                            {if is_empty { "该时段暂无空闲员工" } else { "请选择员工" }}
                                        </option>
                                        {options}
                                    </select>
                                }
                                .into_any()
                            }}
                        </Transition>
                    </label>
                    <div class="text-sm">
                        {move || {
                            let (class, text) = match conflict_state.get() {
                                ConflictState::Unknown => (
                                    "text-base-content/60",
                                    "选择员工与时间后自动校验冲突".to_string(),
                                ),
                                ConflictState::Available => {
                                    ("text-success", "可排班".to_string())
                                }
                                ConflictState::Conflict(label) => (
                                    "text-error",
                                    format!("该时段已排客户 {}，是否更换员工 / 时间？", label),
                                ),
                            };
                            view! { <span class=class>{text}</span> }
                        }}
                    </div>
                    <div class="flex justify-end gap-2">
                        <button class="btn" on:click=move |_| show.set(false)>
                            "取消"
                        </button>
                        <button
                            class=move || {
                                if can_submit_new.get() {
                                    "btn btn-primary"
                                } else {
                                    "btn btn-primary btn-disabled"
                                }
                            }
                            disabled=move || !can_submit_new.get()
                            on:click=move |_| submit_new_schedule.run(())
                        >
                            {move || if creating_schedule.get() { "创建中..." } else { "确认排班" }}
                        </button>
                    </div>
                </div>
            </Show>
        </Modal>
    }
}

#[component]
pub fn AssignmentModal(
    show: RwSignal<bool>,
    users: Resource<Vec<User>>,
    assigned_user_uuid: RwSignal<String>,
    scheduled_start_at: RwSignal<String>,
    scheduled_end_at: RwSignal<String>,
    dispatch_note: RwSignal<String>,
    submit_assignment: Callback<()>,
) -> impl IntoView {
    view! {
        <Modal show=show>
            <h3 class="text-lg font-semibold mb-4">"排班信息"</h3>
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
                    <div class="label"><span class="label-text">"备注"</span></div>
                    <textarea
                        class="textarea textarea-bordered w-full"
                        rows="2"
                        prop:value=move || dispatch_note.get()
                        on:input=move |ev| dispatch_note.set(event_target_value(&ev))
                    ></textarea>
                </label>
                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show.set(false)>
                        "取消"
                    </button>
                    <button class="btn btn-primary" on:click=move |_| submit_assignment.run(())>
                        "保存"
                    </button>
                </div>
            </div>
        </Modal>
    }
}

#[component]
pub fn ScheduleDetailModal(
    show: RwSignal<bool>,
    detail_schedule: RwSignal<Option<Schedule>>,
    schedule_feedbacks: Resource<Vec<OrderFeedback>>,
    #[prop(into)] current_user_uuid: Signal<String>,
    #[prop(into)] is_worker: Signal<bool>,
    feedback_rating: RwSignal<String>,
    feedback_content: RwSignal<String>,
    creating_feedback: RwSignal<bool>,
    submit_feedback: Callback<()>,
) -> impl IntoView {
    view! {
        <Modal show=show box_class=DETAIL_MODAL_BOX_CLASS>
            <div class="space-y-4">
                <h3 class="text-lg font-semibold">"排班详情"</h3>
                {move || {
                    if let Some(schedule) = detail_schedule.get() {
                        view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                {detail_item("订单ID", schedule.order_uuid.clone())}
                                {detail_item("客户UUID", display_optional(schedule.customer_uuid.clone()))}
                                {detail_item("员工UUID", display_optional(schedule.assigned_user_uuid.clone()))}
                                {detail_item("排班状态", schedule_status_label(&schedule).to_string())}
                                {detail_item("订单状态", schedule.order_status.clone())}
                                {detail_item("服务开始", display_optional(schedule.scheduled_start_at.clone()))}
                                {detail_item("服务结束", display_optional(schedule.scheduled_end_at.clone()))}
                                {detail_item("派工备注", display_optional(schedule.dispatch_note.clone()))}
                                {detail_item("备注", display_optional(schedule.notes.clone()))}
                                {detail_item("创建时间", schedule.inserted_at.clone())}
                                {detail_item("更新时间", schedule.updated_at.clone())}
                            </div>
                        }
                        .into_any()
                    } else {
                        view! { <div class="text-sm text-base-content/60">"暂无详情"</div> }
                            .into_any()
                    }
                }}

                <div class="rounded-box border border-base-200 p-4 space-y-4">
                    <div class="font-medium">"服务反馈"</div>
                    {move || {
                        let current_user_uuid = current_user_uuid.get();
                        let feedback_items = schedule_feedbacks.get().unwrap_or_default();
                        let already_submitted = feedback_items
                            .iter()
                            .any(|item| item.user_uuid.as_deref() == Some(current_user_uuid.as_str()));
                        let can_submit = detail_schedule
                            .get()
                            .map(|schedule| {
                                is_worker.get()
                                    && schedule.schedule_status == "done"
                                    && schedule.order_status == "completed"
                                    && schedule.assigned_user_uuid.as_deref()
                                        == Some(current_user_uuid.as_str())
                                    && !already_submitted
                            })
                            .unwrap_or(false);

                        view! {
                            <div class="space-y-4">
                                <Show when=move || already_submitted>
                                    <div class="text-sm text-success">
                                        "你已提交过该订单反馈，当前为只读状态"
                                    </div>
                                </Show>
                                <Show
                                    when=move || can_submit
                                    fallback=move || view! {
                                        <div class="text-sm text-base-content/60">
                                            "仅完成服务的执行人员可提交反馈，其他角色可查看历史反馈"
                                        </div>
                                    }
                                >
                                    <div class="grid gap-3">
                                        <select
                                            class="select select-bordered"
                                            prop:value=move || feedback_rating.get()
                                            on:change=move |ev| feedback_rating.set(event_target_value(&ev))
                                        >
                                            <option value="">"请选择评分（可选）"</option>
                                            <option value="5">"5 分"</option>
                                            <option value="4">"4 分"</option>
                                            <option value="3">"3 分"</option>
                                            <option value="2">"2 分"</option>
                                            <option value="1">"1 分"</option>
                                        </select>
                                        <textarea
                                            class="textarea textarea-bordered min-h-24"
                                            prop:value=move || feedback_content.get()
                                            on:input=move |ev| feedback_content.set(event_target_value(&ev))
                                            placeholder="填写服务内容、异常情况或客户现场反馈"
                                        ></textarea>
                                        <div class="flex justify-end">
                                            <button
                                                class=move || {
                                                    if creating_feedback.get() {
                                                        "btn btn-sm btn-primary btn-disabled"
                                                    } else {
                                                        "btn btn-sm btn-primary"
                                                    }
                                                }
                                                disabled=move || creating_feedback.get()
                                                on:click=move |_| submit_feedback.run(())
                                            >
                                                {move || if creating_feedback.get() { "提交中..." } else { "提交反馈" }}
                                            </button>
                                        </div>
                                    </div>
                                </Show>

                                <Transition fallback=move || view! {
                                    <div class="text-sm text-base-content/60">"加载中..."</div>
                                }>
                                    {move || {
                                        schedule_feedbacks.get().map(|items| {
                                            if items.is_empty() {
                                                view! {
                                                    <div class="text-sm text-base-content/60">"暂无服务反馈"</div>
                                                }
                                                .into_any()
                                            } else {
                                                view! {
                                                    <div class="space-y-3">
                                                        <For
                                                            each=move || items.clone().into_iter()
                                                            key=|item: &OrderFeedback| item.uuid.clone()
                                                            children=move |item| {
                                                                let rating_label = item
                                                                    .rating
                                                                    .map(|value| format!("{} 分", value))
                                                                    .unwrap_or_else(|| "未评分".to_string());
                                                                let user_label = item
                                                                    .user_name
                                                                    .clone()
                                                                    .filter(|value| !value.trim().is_empty())
                                                                    .or_else(|| item.user_uuid.clone())
                                                                    .unwrap_or_else(|| "系统".to_string());
                                                                view! {
                                                                    <div class="rounded-box bg-base-200/50 p-3 space-y-2">
                                                                        <div class="flex items-center justify-between gap-2">
                                                                            <span class="text-sm font-medium">{user_label}</span>
                                                                            <span class="text-xs text-base-content/60">{item.created_at.clone()}</span>
                                                                        </div>
                                                                        <div class="text-xs text-base-content/60">{rating_label}</div>
                                                                        <div class="text-sm whitespace-pre-wrap break-all">{item.content.clone()}</div>
                                                                    </div>
                                                                }
                                                            }
                                                        />
                                                    </div>
                                                }
                                                .into_any()
                                            }
                                        })
                                    }}
                                </Transition>
                            </div>
                        }
                        .into_any()
                    }}
                </div>
                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show.set(false)>
                        "关闭"
                    </button>
                </div>
            </div>
        </Modal>
    }
}
