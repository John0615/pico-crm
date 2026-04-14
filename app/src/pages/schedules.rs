use crate::components::features::get_user_info;
use crate::components::ui::date_picker::{FlyonDatePicker, FlyonDateTimePicker};
use crate::components::ui::modal::Modal;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable};
use crate::components::ui::toast::{error, success};
use crate::server::contact_handlers::{fetch_contacts, get_contact};
use crate::server::order_handlers::fetch_orders;
use crate::server::schedule_handlers::{
    cancel_schedule, create_schedule, fetch_schedules, update_schedule, update_schedule_status,
};
use crate::server::service_request_handlers::get_service_request;
use crate::server::user_handlers::{fetch_users, get_user};
use crate::utils::api::call_api;
#[cfg(feature = "ssr")]
use chrono::Utc;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike, Weekday};
#[cfg(not(feature = "ssr"))]
use js_sys::Date as JsDate;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use shared::contact::{Contact, ContactQuery};
use shared::order::{Order, OrderQuery};
use shared::schedule::{
    CreateScheduleAssignment, Schedule, ScheduleQuery, UpdateScheduleAssignment,
    UpdateScheduleStatus,
};
use shared::user::{User, UserListQuery};
use shared::ListResult;
use std::collections::{HashMap, HashSet};

impl Identifiable for Schedule {
    fn id(&self) -> String {
        format!(
            "{}-{}-{}",
            self.order_uuid, self.schedule_status, self.updated_at
        )
    }
}

#[component]
pub fn SchedulesPage() -> impl IntoView {
    let query = use_query_map();
    let refresh_count = RwSignal::new(0);
    let (view_mode, set_view_mode) = signal("calendar".to_string());
    let (status_filter, set_status_filter) = signal(String::new());
    let (user_filter, set_user_filter) = signal(String::new());
    let date_start = RwSignal::new(String::new());
    let date_end = RwSignal::new(String::new());

    let show_assignment_modal = RwSignal::new(false);
    let show_new_modal = RwSignal::new(false);
    let show_detail_modal = RwSignal::new(false);
    let assignment_order_uuid = RwSignal::new(String::new());
    let assignment_is_new = RwSignal::new(true);
    let assigned_user_uuid = RwSignal::new(String::new());
    let scheduled_start_at = RwSignal::new(String::new());
    let scheduled_end_at = RwSignal::new(String::new());
    let dispatch_note = RwSignal::new(String::new());
    let new_order_uuid = RwSignal::new(String::new());
    let new_service_type = RwSignal::new(String::new());
    let new_duration_minutes = RwSignal::new(0i64);
    let duration_overridden = RwSignal::new(false);
    let duration_prefill_order = RwSignal::new(String::new());
    let expected_start_overridden = RwSignal::new(false);
    let expected_start_prefill_order = RwSignal::new(String::new());
    let expected_start_prefill_value = RwSignal::new(String::new());
    let new_expected_start_at = RwSignal::new(String::new());
    let new_expected_end_at = RwSignal::new(String::new());
    let new_assigned_user_uuid = RwSignal::new(String::new());
    let creating_schedule = RwSignal::new(false);
    let detail_schedule: RwSignal<Option<Schedule>> = RwSignal::new(None);
    let contact_labels: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let user_labels: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let pending_contacts: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
    let pending_users: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());

    Effect::new(move |_| {
        if view_mode.get() != "calendar" {
            return;
        }
        if !date_start.get().is_empty() || !date_end.get().is_empty() {
            return;
        }
        let (start, end) = calendar_week_range_strings("", "");
        date_start.set(start);
        date_end.set(end);
    });

    Effect::new(move |_| {
        let start = new_expected_start_at.get();
        let minutes = new_duration_minutes.get();
        if start.trim().is_empty() || minutes <= 0 {
            new_expected_end_at.set(String::new());
            return;
        }
        if let Some(end) = add_minutes_to_local(&start, minutes) {
            new_expected_end_at.set(end);
        } else {
            new_expected_end_at.set(String::new());
        }
    });

    Effect::new(move |_| {
        let start = new_expected_start_at.get();
        if start.trim().is_empty() {
            expected_start_overridden.set(false);
            expected_start_prefill_value.set(String::new());
            return;
        }
        if start != expected_start_prefill_value.get() {
            expected_start_overridden.set(true);
        }
    });

    let current_user = Resource::new(
        move || (),
        |_| async move {
            call_api(get_user_info()).await.unwrap_or_else(|e| {
                logging::error!("Error loading user info: {e}");
                User::default()
            })
        },
    );

    let is_worker = RwSignal::new(false);
    Effect::new(move |_| {
        let worker = current_user.with(|value| {
            value
                .as_ref()
                .map(|user| {
                    !user.is_admin.unwrap_or(false)
                        && user.role != "operator"
                        && user.role != "merchant"
                        && user.role != "admin"
                })
                .unwrap_or(false)
        });
        is_worker.set(worker);
    });

    let data = Resource::new(
        move || {
            (
                view_mode.get(),
                status_filter.get(),
                user_filter.get(),
                date_start.get(),
                date_end.get(),
                *refresh_count.read(),
                query.with(|value| value.clone()),
            )
        },
        |(view_mode, status, user, start, end, _, query)| async move {
            let is_calendar = view_mode == "calendar";
            let (start_date, end_date) = if is_calendar {
                calendar_week_range_strings(&start, &end)
            } else {
                (start.clone(), end.clone())
            };
            let page = if is_calendar {
                1
            } else {
                query
                    .get("page")
                    .unwrap_or_default()
                    .parse::<u64>()
                    .unwrap_or(1)
            };
            let page_size = if is_calendar {
                500
            } else {
                query
                    .get("page_size")
                    .unwrap_or_default()
                    .parse::<u64>()
                    .unwrap_or(10)
            };

            let params = ScheduleQuery {
                page,
                page_size,
                status: (!status.is_empty()).then_some(status),
                assigned_user_uuid: (!user.is_empty()).then_some(user),
                start_date: (!start_date.is_empty()).then_some(start_date),
                end_date: (!end_date.is_empty()).then_some(end_date),
            };

            let result = call_api(fetch_schedules(params)).await.unwrap_or_else(|e| {
                logging::error!("Error loading schedules: {e}");
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
            set_view_mode.set("list".to_string());
        }
        if let Some(start) = map.get("start_date") {
            if date_start.get() != start {
                date_start.set(start);
            }
            set_view_mode.set("list".to_string());
        }
        if let Some(end) = map.get("end_date") {
            if date_end.get() != end {
                date_end.set(end);
            }
            set_view_mode.set("list".to_string());
        }
        if map.get("upcoming").is_some() {
            let (start, end) = upcoming_date_range();
            date_start.set(start);
            date_end.set(end);
            set_view_mode.set("list".to_string());
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

    let available_orders = Resource::new(
        move || show_new_modal.get(),
        |open| async move {
            if !open {
                return Vec::new();
            }
            let params = OrderQuery {
                page: 1,
                page_size: 200,
                status: None,
                customer_uuid: None,
                start_date: None,
                end_date: None,
            };
            match call_api(fetch_orders(params)).await {
                Ok(result) => result.items,
                Err(err) => {
                    logging::error!("Error loading orders: {err}");
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
                map.insert(
                    contact.contact_uuid.clone(),
                    contact_display_label(&contact),
                );
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
        let order_uuid = new_order_uuid.get();
        if order_uuid.trim().is_empty() {
            duration_prefill_order.set(String::new());
            expected_start_prefill_order.set(String::new());
            return;
        }
        if duration_prefill_order.get() != order_uuid {
            duration_overridden.set(false);
            duration_prefill_order.set(order_uuid.clone());
        }
        if expected_start_prefill_order.get() != order_uuid {
            expected_start_overridden.set(false);
            expected_start_prefill_order.set(order_uuid.clone());
            expected_start_prefill_value.set(String::new());
        }
        if duration_overridden.get() {
            if expected_start_overridden.get() {
                return;
            }
        }
        let Some(items) = available_orders.get() else {
            return;
        };
        let Some(order) = items.into_iter().find(|order| order.uuid == order_uuid) else {
            return;
        };
        if !duration_overridden.get() {
            if let Some(minutes) = duration_minutes_between(
                order.scheduled_start_at.as_deref(),
                order.scheduled_end_at.as_deref(),
            ) {
                new_duration_minutes.set(minutes);
                return;
            }
        }
        let Some(request_id) = order.request_id.clone() else {
            return;
        };
        let order_uuid_guard = order_uuid.clone();
        spawn_local(async move {
            let request = match call_api(get_service_request(request_id.clone())).await {
                Ok(result) => result,
                Err(err) => {
                    logging::error!("Error loading service request: {err}");
                    return;
                }
            };
            let Some(request) = request else {
                return;
            };
            if !duration_overridden.get() {
                if let Some(minutes) = duration_minutes_between(
                    request.appointment_start_at.as_deref(),
                    request.appointment_end_at.as_deref(),
                ) {
                    if new_order_uuid.get() == order_uuid_guard {
                        new_duration_minutes.set(minutes);
                    }
                }
            }
            if !expected_start_overridden.get()
                && new_expected_start_at.get().trim().is_empty()
                && new_order_uuid.get() == order_uuid_guard
            {
                let start_value = to_datetime_local(request.appointment_start_at.clone());
                if !start_value.trim().is_empty() {
                    expected_start_prefill_value.set(start_value.clone());
                    new_expected_start_at.set(start_value);
                }
            }
        });
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
        for schedule in &items {
            let Some(contact_id) = schedule.customer_uuid.clone() else {
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

    let conflict_state = Memo::new(move |_| {
        let user_id = new_assigned_user_uuid.get();
        let start_raw = new_expected_start_at.get();
        let duration = new_duration_minutes.get();
        if user_id.trim().is_empty() || start_raw.trim().is_empty() || duration <= 0 {
            return ConflictState::Unknown;
        }
        let Some(start) = parse_calendar_datetime(&start_raw) else {
            return ConflictState::Unknown;
        };
        let Some(end_raw) = add_minutes_to_local(&start_raw, duration) else {
            return ConflictState::Unknown;
        };
        let Some(end) = parse_calendar_datetime(&end_raw) else {
            return ConflictState::Unknown;
        };
        let Some((items, _)) = data.get() else {
            return ConflictState::Unknown;
        };
        let labels = contact_labels.get();
        let pending = pending_contacts.get();
        for schedule in items {
            let Some(assigned) = schedule.assigned_user_uuid.clone() else {
                continue;
            };
            if assigned != user_id {
                continue;
            }
            let Some(existing_start_raw) = schedule.scheduled_start_at.clone() else {
                continue;
            };
            let Some(existing_end_raw) = schedule.scheduled_end_at.clone() else {
                continue;
            };
            let Some(existing_start) = parse_calendar_datetime(&existing_start_raw) else {
                continue;
            };
            let Some(existing_end) = parse_calendar_datetime(&existing_end_raw) else {
                continue;
            };
            if is_overlapping_window_naive(start, end, existing_start, existing_end) {
                let label = schedule_contact_label(&schedule, &labels, &pending);
                return ConflictState::Conflict(label);
            }
        }
        ConflictState::Available
    });

    let can_submit_new = Memo::new(move |_| {
        !creating_schedule.get()
            && !new_order_uuid.get().trim().is_empty()
            && !new_service_type.get().trim().is_empty()
            && new_duration_minutes.get() > 0
            && !new_expected_start_at.get().trim().is_empty()
            && !new_assigned_user_uuid.get().trim().is_empty()
            && matches!(conflict_state.get(), ConflictState::Available)
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
        for schedule in &items {
            let Some(user_id) = schedule.assigned_user_uuid.clone() else {
                continue;
            };
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

    let open_assignment = move |schedule: Schedule| {
        assignment_order_uuid.set(schedule.order_uuid.clone());
        assigned_user_uuid.set(schedule.assigned_user_uuid.clone().unwrap_or_default());
        scheduled_start_at.set(to_datetime_local(schedule.scheduled_start_at.clone()));
        scheduled_end_at.set(to_datetime_local(schedule.scheduled_end_at.clone()));
        dispatch_note.set(schedule.dispatch_note.clone().unwrap_or_default());
        assignment_is_new.set(schedule.assigned_user_uuid.is_none());
        show_assignment_modal.set(true);
    };

    let open_new_schedule = move |_| {
        new_order_uuid.set(String::new());
        new_service_type.set(String::new());
        new_duration_minutes.set(0);
        duration_overridden.set(false);
        duration_prefill_order.set(String::new());
        expected_start_overridden.set(false);
        expected_start_prefill_order.set(String::new());
        expected_start_prefill_value.set(String::new());
        new_expected_start_at.set(String::new());
        new_expected_end_at.set(String::new());
        new_assigned_user_uuid.set(String::new());
        creating_schedule.set(false);
        show_new_modal.set(true);
    };

    let open_detail = move |schedule: Schedule| {
        detail_schedule.set(Some(schedule));
        show_detail_modal.set(true);
    };

    let submit_new_schedule = move |_| {
        if creating_schedule.get() {
            return;
        }
        let order_uuid = new_order_uuid.get();
        if order_uuid.trim().is_empty() {
            error("请选择订单".to_string());
            return;
        }
        let service_type = new_service_type.get();
        if service_type.trim().is_empty() {
            error("请选择服务类型".to_string());
            return;
        }
        let duration_minutes = new_duration_minutes.get();
        if duration_minutes <= 0 {
            error("请选择服务时长".to_string());
            return;
        }
        let start_raw = new_expected_start_at.get();
        if start_raw.trim().is_empty() {
            error("请选择期望时间".to_string());
            return;
        }
        let Some(end_raw) = add_minutes_to_local(&start_raw, duration_minutes) else {
            error("期望时间格式错误".to_string());
            return;
        };
        let assigned_user = new_assigned_user_uuid.get();
        if assigned_user.trim().is_empty() {
            error("请选择员工".to_string());
            return;
        }

        let conflict = if let Some((items, _)) = data.get() {
            let start = parse_calendar_datetime(&start_raw);
            let end = parse_calendar_datetime(&end_raw);
            if let (Some(start), Some(end)) = (start, end) {
                let labels = contact_labels.get();
                let pending = pending_contacts.get();
                let mut conflict_label: Option<String> = None;
                for schedule in items {
                    let Some(assigned) = schedule.assigned_user_uuid.clone() else {
                        continue;
                    };
                    if assigned != assigned_user {
                        continue;
                    }
                    let Some(existing_start_raw) = schedule.scheduled_start_at.clone() else {
                        continue;
                    };
                    let Some(existing_end_raw) = schedule.scheduled_end_at.clone() else {
                        continue;
                    };
                    let Some(existing_start) = parse_calendar_datetime(&existing_start_raw) else {
                        continue;
                    };
                    let Some(existing_end) = parse_calendar_datetime(&existing_end_raw) else {
                        continue;
                    };
                    if is_overlapping_window_naive(start, end, existing_start, existing_end) {
                        let label = schedule_contact_label(&schedule, &labels, &pending);
                        conflict_label = Some(label);
                        break;
                    }
                }
                conflict_label
            } else {
                None
            }
        } else {
            None
        };
        if let Some(label) = conflict {
            error(format!("该时段已排客户 {}，请调整员工或时间", label));
            return;
        }

        let start_normalized =
            normalize_datetime_local(&start_raw).unwrap_or_else(|| start_raw.clone());
        let end_normalized = normalize_datetime_local(&end_raw).unwrap_or_else(|| end_raw.clone());
        let duration_label = duration_label(duration_minutes);
        let dispatch_note = build_dispatch_note(&service_type, &duration_label);
        creating_schedule.set(true);

        spawn_local(async move {
            let result: Result<(), String> = async {
                let schedule_payload = CreateScheduleAssignment {
                    assigned_user_uuid: assigned_user.clone(),
                    scheduled_start_at: start_normalized.clone(),
                    scheduled_end_at: end_normalized.clone(),
                    dispatch_note: Some(dispatch_note),
                };
                let _ = call_api(create_schedule(order_uuid.clone(), schedule_payload)).await?;
                Ok(())
            }
            .await;

            creating_schedule.set(false);
            match result {
                Ok(_) => {
                    success("排班已创建".to_string());
                    show_new_modal.set(false);
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => {
                    error(format!("创建失败: {}", err));
                }
            }
        });
    };

    let submit_assignment = move |_| {
        if assigned_user_uuid.get().trim().is_empty() {
            error("请选择员工".to_string());
            return;
        }
        if scheduled_start_at.get().trim().is_empty() || scheduled_end_at.get().trim().is_empty() {
            error("请填写排班时间".to_string());
            return;
        }
        if is_end_before_start(&scheduled_start_at.get(), &scheduled_end_at.get()) {
            error("结束时间必须晚于开始时间".to_string());
            return;
        }

        let uuid = assignment_order_uuid.get();
        let payload = UpdateScheduleAssignment {
            assigned_user_uuid: normalize_optional(&assigned_user_uuid.get()),
            scheduled_start_at: normalize_datetime_local(&scheduled_start_at.get()),
            scheduled_end_at: normalize_datetime_local(&scheduled_end_at.get()),
            dispatch_note: normalize_optional(&dispatch_note.get()),
        };
        let is_new = assignment_is_new.get();
        spawn_local(async move {
            let result = if is_new {
                let create = CreateScheduleAssignment {
                    assigned_user_uuid: payload.assigned_user_uuid.clone().unwrap_or_default(),
                    scheduled_start_at: payload.scheduled_start_at.clone().unwrap_or_default(),
                    scheduled_end_at: payload.scheduled_end_at.clone().unwrap_or_default(),
                    dispatch_note: payload.dispatch_note.clone(),
                };
                call_api(create_schedule(uuid, create)).await
            } else {
                call_api(update_schedule(uuid, payload)).await
            };
            match result {
                Ok(_) => {
                    success("排班已更新".to_string());
                    show_assignment_modal.set(false);
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("更新失败: {}", err)),
            }
        });
    };

    let cancel_assignment = move |uuid: String| {
        spawn_local(async move {
            let result = call_api(cancel_schedule(uuid)).await;
            match result {
                Ok(_) => {
                    success("排班已取消".to_string());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("取消失败: {}", err)),
            }
        });
    };

    let update_status = move |uuid: String, status: String| {
        spawn_local(async move {
            let payload = UpdateScheduleStatus { status };
            let result = call_api(update_schedule_status(uuid, payload)).await;
            match result {
                Ok(_) => {
                    success("状态已更新".to_string());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("更新失败: {}", err)),
            }
        });
    };

    view! {
        <Title text="排班管理 - PicoCRM"/>
        <div class="space-y-4">
            <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <h1 class="text-2xl font-semibold">"排班管理"</h1>
                <div class="flex flex-wrap items-center gap-2">
                    <div class="tabs tabs-boxed">
                        <button
                            class=move || {
                                if view_mode.get() == "list" {
                                    "tab tab-active"
                                } else {
                                    "tab"
                                }
                            }
                            on:click=move |_| set_view_mode.set("list".to_string())
                        >
                            "列表"
                        </button>
                        <button
                            class=move || {
                                if view_mode.get() == "calendar" {
                                    "tab tab-active"
                                } else {
                                    "tab"
                                }
                            }
                            on:click=move |_| set_view_mode.set("calendar".to_string())
                        >
                            "日历"
                        </button>
                    </div>
                    <Show when=move || !is_worker.get() fallback=|| ()>
                        <button class="btn btn-primary btn-sm" on:click=open_new_schedule>
                            "+ 新增排班"
                        </button>
                    </Show>
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
                            <option value="planned">"待排班"</option>
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
                    </Show>
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

            <Show when=move || view_mode.get() == "calendar">
                {move || {
                    let (week_start, week_end) = calendar_week_range(&date_start.get(), &date_end.get());
                    let week_label = format!(
                        "{} ~ {}",
                        format_date(week_start),
                        format_date(week_end)
                    );
                    let row_height: i32 = 48;

                    view! {
                        <div class="space-y-3">
                            <div class="flex flex-col gap-2 md:flex-row md:items-center md:justify-between">
                                <div class="flex items-center gap-2">
                                    <button
                                        class="btn btn-sm"
                                        on:click=move |_| shift_week_range(-1, date_start, date_end)
                                    >
                                        "上一周"
                                    </button>
                                    <button
                                        class="btn btn-sm"
                                        on:click=move |_| shift_week_range(1, date_start, date_end)
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
                                            return view! { <div class="p-6 text-sm text-base-content/60">"加载中..."</div> }.into_any();
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
                                                    <Show when=move || !is_worker.get() fallback=|| ()>
                                                        <button class="btn btn-primary btn-sm" on:click=open_new_schedule>
                                                            "+ 新增排班"
                                                        </button>
                                                    </Show>
                                                </div>
                                            }.into_any();
                                        }
                                        let (start_hour, end_hour) = calendar_time_bounds(&calendar_items);
                                        let columns_store = StoredValue::new(calendar_columns(
                                            &calendar_items,
                                            &user_labels_snapshot,
                                            &user_filter_value,
                                        ));
                                        let grouped_store = StoredValue::new(group_calendar_items(calendar_items));
                                        let hours_store =
                                            StoredValue::new((start_hour..end_hour).collect::<Vec<_>>());
                                        let contact_labels_store = StoredValue::new(contact_labels_snapshot);
                                        let pending_contacts_store = StoredValue::new(pending_contacts_snapshot);
                                        let total_height = ((end_hour - start_hour) as i32) * row_height;
                                        let grid_style = format!(
                                            "grid-template-columns: 84px repeat({}, minmax(180px, 1fr));",
                                            columns_store.with_value(|cols| cols.len())
                                        );
                                        let truncated = total > items.len() as u64;

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
                                                                .with_value(|map| {
                                                                    map.get(&day).cloned().unwrap_or_default()
                                                                }),
                                                        );
                                                        let columns_for_day = columns_store;
                                                        let hours_for_day = hours_store;
                                                        let contact_labels_for_day = contact_labels_store;
                                                        let pending_contacts_for_day = pending_contacts_store;
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
                                                                                                let status_label = schedule_status_label(&schedule.schedule_status).to_string();
                                                                                                let status_class = schedule_card_classes(&schedule);
                                                                                                let contact_label = contact_labels_for_day
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
                                                                                                let time_label = format_time_range(item.start, item.end);
                                                                                                let contact_label_title = contact_label.clone();
                                                                                                let time_label_title = time_label.clone();
                                                                                                let can_edit = !is_worker.get() && schedule.schedule_status == "planned";
                                                                                                let schedule_for_click = schedule.clone();
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
                                                                                                                    open_assignment(schedule_for_click.clone());
                                                                                                                } else {
                                                                                                                    open_detail(schedule_for_click.clone());
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
                                        }.into_any()
                                    }}
                                </Transition>
                            </div>
                        </div>
                    }
                    .into_view()
                }}
            </Show>

            <Show when=move || view_mode.get() != "calendar">
            <div class="overflow-x-auto overflow-y-auto h-[calc(100vh-260px)] bg-base-100 rounded-lg shadow">
                <DaisyTable data=data>
                    <Column
                        slot:columns
                        prop="order_uuid".to_string()
                        label="订单ID".to_string()
                        class="font-semibold"
                    >
                        {
                            let item: Option<Schedule> = use_context::<Schedule>();
                            view! { <span class="text-xs">{item.as_ref().map(|v| v.order_uuid.clone()).unwrap_or_default()}</span> }
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
                                .map(|schedule| format_time_window(schedule.scheduled_start_at.clone(), schedule.scheduled_end_at.clone()))
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
                            let badge_class = schedule_status_badge_class(&status_value);
                            view! {
                                <span class=format!("badge {}", badge_class)>
                                    {schedule_status_label(&status_value)}
                                </span>
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
                                let item: Option<Schedule> = use_context::<Schedule>();
                                let schedule = StoredValue::new(item);
                                let has_schedule = schedule.with_value(|value| value.is_some());
                                let can_assign = schedule
                                    .with_value(|value| {
                                        value
                                            .as_ref()
                                            .map(|value| value.schedule_status == "planned")
                                            .unwrap_or(false)
                                    });
                                let can_cancel = schedule
                                    .with_value(|value| {
                                        value
                                            .as_ref()
                                            .map(|value| {
                                                value.schedule_status == "planned"
                                                    || value.schedule_status == "in_service"
                                            })
                                            .unwrap_or(false)
                                    });
                                let can_start = schedule
                                    .with_value(|value| {
                                        value
                                            .as_ref()
                                            .map(|value| value.schedule_status == "planned")
                                            .unwrap_or(false)
                                    });
                                let can_done = schedule
                                    .with_value(|value| {
                                        value
                                            .as_ref()
                                            .map(|value| value.schedule_status == "in_service")
                                            .unwrap_or(false)
                                    });
                                view! {
                                    <Show
                                        when=move || {
                                            has_schedule && !is_worker.get()
                                        }
                                    >
                                        <Show
                                            when=move || {
                                                can_assign
                                            }
                                        >
                                            <button
                                                class="btn btn-soft btn-primary btn-xs"
                                                on:click=move |_| {
                                                    if let Some(value) = schedule.with_value(|value| value.clone()) {
                                                        open_assignment(value);
                                                    }
                                                }
                                            >
                                                "排班"
                                            </button>
                                        </Show>
                                        <Show
                                            when=move || {
                                                can_cancel
                                            }
                                        >
                                            <button
                                                class="btn btn-soft btn-error btn-xs"
                                                on:click=move |_| {
                                                    if let Some(value) = schedule.with_value(|value| value.clone()) {
                                                        cancel_assignment(value.order_uuid.clone());
                                                    }
                                                }
                                            >
                                                "取消"
                                            </button>
                                        </Show>
                                    </Show>
                                    <Show
                                        when=move || {
                                            has_schedule && is_worker.get()
                                        }
                                    >
                                        <Show
                                            when=move || {
                                                can_start
                                            }
                                        >
                                            <button
                                                class="btn btn-soft btn-warning btn-xs"
                                                on:click=move |_| {
                                                    if let Some(value) = schedule.with_value(|value| value.clone()) {
                                                        update_status(value.order_uuid.clone(), "in_service".to_string());
                                                    }
                                                }
                                            >
                                                "开始服务"
                                            </button>
                                        </Show>
                                        <Show
                                            when=move || {
                                                can_done
                                            }
                                        >
                                            <button
                                                class="btn btn-soft btn-success btn-xs"
                                                on:click=move |_| {
                                                    if let Some(value) = schedule.with_value(|value| value.clone()) {
                                                        update_status(value.order_uuid.clone(), "done".to_string());
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
                                                open_detail(value);
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
                        data.as_ref().map(|data| view! { <Pagination total_items=data.1 /> })
                    })
                }}
            </Transition>
            </Show>
        </div>

        <Modal show=show_new_modal>
            <h3 class="text-lg font-semibold mb-4">"新增排班"</h3>
            <div class="space-y-3">
                <label class="form-control w-full">
                    <div class="label"><span class="label-text">"客户（未排班订单）"</span></div>
                    <Transition fallback=move || view! {
                        <select class="select select-bordered w-full" disabled=true>
                            <option value="">"加载中..."</option>
                        </select>
                    }>
                        {move || {
                            let items = available_orders.get().unwrap_or_default();
                            let contact_labels_snapshot = contact_labels.get();
                            let pending_snapshot = pending_contacts.get();
                            let mut options = Vec::new();
                            for order in items.into_iter().filter(order_is_schedulable) {
                                let label = order_option_label(&order, &contact_labels_snapshot, &pending_snapshot);
                                options.push(view! { <option value={order.uuid.clone()}>{label}</option> });
                            }
                            let is_empty = options.is_empty();
                            view! {
                                <select
                                    class="select select-bordered w-full"
                                    prop:value=move || new_order_uuid.get()
                                    disabled=is_empty
                                    on:change=move |ev| new_order_uuid.set(event_target_value(&ev))
                                >
                                    <option value="">
                                        {if is_empty { "暂无可排班订单" } else { "请选择客户" }}
                                    </option>
                                    {options}
                                </select>
                            }
                        }}
                    </Transition>
                </label>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <label class="form-control w-full">
                        <div class="label"><span class="label-text">"服务类型"</span></div>
                        {move || {
                            let options = SERVICE_TYPE_OPTIONS
                                .iter()
                                .map(|value| view! { <option value={*value}>{*value}</option> })
                                .collect::<Vec<_>>();
                            view! {
                                <select
                                    class="select select-bordered w-full"
                                    prop:value=move || new_service_type.get()
                                    on:change=move |ev| new_service_type.set(event_target_value(&ev))
                                >
                                    <option value="">"请选择服务类型"</option>
                                    {options}
                                </select>
                            }
                        }}
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
                                    prop:value=move || new_assigned_user_uuid.get()
                                    on:change=move |ev| new_assigned_user_uuid.set(event_target_value(&ev))
                                >
                                    <option value="">"请选择员工"</option>
                                    {options}
                                </select>
                            }
                        }}
                    </Transition>
                </label>
                <div class="text-sm">
                    {move || {
                        let (class, text) = match conflict_state.get() {
                            ConflictState::Unknown => ("text-base-content/60", "选择员工与时间后自动校验冲突".to_string()),
                            ConflictState::Available => ("text-success", "可排班".to_string()),
                            ConflictState::Conflict(label) => ("text-error", format!("该时段已排客户 {}，是否更换员工 / 时间？", label)),
                        };
                        view! { <span class=class>{text}</span> }
                    }}
                </div>
                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_new_modal.set(false)>
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
                        on:click=submit_new_schedule
                    >
                        {move || if creating_schedule.get() { "创建中..." } else { "确认排班" }}
                    </button>
                </div>
            </div>
        </Modal>

        <Modal show=show_assignment_modal>
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
                <h3 class="text-lg font-semibold">"排班详情"</h3>
                {move || {
                    if let Some(schedule) = detail_schedule.get() {
                        view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                {detail_item("订单ID", schedule.order_uuid.clone())}
                                {detail_item("客户UUID", display_optional(schedule.customer_uuid.clone()))}
                                {detail_item("员工UUID", display_optional(schedule.assigned_user_uuid.clone()))}
                                {detail_item("排班状态", schedule.schedule_status.clone())}
                                {detail_item("订单状态", schedule.order_status.clone())}
                                {detail_item("服务开始", display_optional(schedule.scheduled_start_at.clone()))}
                                {detail_item("服务结束", display_optional(schedule.scheduled_end_at.clone()))}
                                {detail_item("派工备注", display_optional(schedule.dispatch_note.clone()))}
                                {detail_item("备注", display_optional(schedule.notes.clone()))}
                                {detail_item("创建时间", schedule.inserted_at.clone())}
                                {detail_item("更新时间", schedule.updated_at.clone())}
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

#[derive(Clone, PartialEq)]
enum ConflictState {
    Unknown,
    Available,
    Conflict(String),
}

#[derive(Clone)]
struct CalendarItem {
    schedule: Schedule,
    start: NaiveDateTime,
    end: NaiveDateTime,
}

#[derive(Clone)]
struct CalendarColumn {
    id: String,
    label: String,
}

const SERVICE_TYPE_OPTIONS: [&str; 3] = ["保洁", "维修", "家电清洗"];
const DURATION_OPTIONS: [(i64, &str); 5] = [
    (60, "1小时"),
    (90, "1.5小时"),
    (120, "2小时"),
    (180, "3小时"),
    (240, "4小时"),
];

fn schedule_status_label(status: &str) -> &'static str {
    match status {
        "planned" => "待排班",
        "in_service" => "服务中",
        "done" => "已完成",
        "cancelled" => "已取消",
        _ => "未知",
    }
}

fn schedule_status_badge_class(status: &str) -> &'static str {
    match status {
        "planned" => "badge-info",
        "in_service" => "badge-warning",
        "done" => "badge-success",
        "cancelled" => "badge-error",
        _ => "badge-info",
    }
}

fn order_is_schedulable(order: &Order) -> bool {
    let status = order.status.as_str();
    let status_ok = matches!(status, "pending" | "confirmed" | "dispatching");
    let start_missing = order
        .scheduled_start_at
        .as_ref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true);
    let end_missing = order
        .scheduled_end_at
        .as_ref()
        .map(|value| value.trim().is_empty())
        .unwrap_or(true);
    status_ok && start_missing && end_missing
}

fn order_option_label(
    order: &Order,
    contact_labels: &HashMap<String, String>,
    pending_contacts: &HashSet<String>,
) -> String {
    let contact_label = order
        .customer_uuid
        .as_ref()
        .map(|id| {
            contact_labels.get(id).cloned().unwrap_or_else(|| {
                if pending_contacts.contains(id) {
                    "加载中...".to_string()
                } else {
                    "未知客户".to_string()
                }
            })
        })
        .unwrap_or_else(|| "未知客户".to_string());
    let short_id = shorten_uuid(&order.uuid);
    format!("{} · 订单 {}", contact_label, short_id)
}

fn shorten_uuid(value: &str) -> String {
    if value.len() > 8 {
        value[..8].to_string()
    } else {
        value.to_string()
    }
}

fn duration_label(minutes: i64) -> String {
    for (value, label) in DURATION_OPTIONS {
        if value == minutes {
            return label.to_string();
        }
    }
    if minutes % 60 == 0 {
        format!("{}小时", minutes / 60)
    } else {
        format!("{}分钟", minutes)
    }
}

fn build_dispatch_note(service_type: &str, duration_label: &str) -> String {
    format!("服务类型: {} | 服务时长: {}", service_type, duration_label)
}

fn extract_service_type(dispatch_note: Option<&String>) -> Option<String> {
    let note = dispatch_note?;
    if note.trim().is_empty() {
        return None;
    }
    let note = note.replace('：', ":");
    let marker = "服务类型:";
    let start = note.find(marker)?;
    let rest = note[start + marker.len()..].trim();
    let end = rest
        .find(|c: char| c == '|' || c == ';' || c == '；')
        .unwrap_or(rest.len());
    let value = rest[..end].trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_string())
    }
}

fn service_type_card_classes(service_type: &str) -> Option<&'static str> {
    match service_type {
        "保洁" => Some("border-sky-200 bg-sky-50 text-sky-900"),
        "维修" => Some("border-emerald-200 bg-emerald-50 text-emerald-900"),
        "家电清洗" => Some("border-amber-200 bg-amber-50 text-amber-900"),
        _ => None,
    }
}

fn schedule_card_classes(schedule: &Schedule) -> &'static str {
    let service_type = extract_service_type(schedule.dispatch_note.as_ref());
    if let Some(service_type) = service_type {
        if let Some(classes) = service_type_card_classes(&service_type) {
            return classes;
        }
    }
    schedule_status_classes(&schedule.schedule_status)
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
    let Some(value) = value else {
        return String::new();
    };
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
    match (
        normalize_datetime_local(start),
        normalize_datetime_local(end),
    ) {
        (Some(start), Some(end)) => end <= start,
        _ => false,
    }
}

fn format_time_window(start: Option<String>, end: Option<String>) -> String {
    let start = start.unwrap_or_default();
    let end = end.unwrap_or_default();
    if start.trim().is_empty() && end.trim().is_empty() {
        "-".to_string()
    } else if end.trim().is_empty() {
        format!("{} ~", start)
    } else if start.trim().is_empty() {
        format!("~ {}", end)
    } else {
        format!("{} ~ {}", start, end)
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

fn today_date() -> NaiveDate {
    #[cfg(feature = "ssr")]
    {
        Utc::now().date_naive()
    }
    #[cfg(not(feature = "ssr"))]
    {
        let date = JsDate::new_0();
        NaiveDate::from_ymd_opt(
            date.get_full_year() as i32,
            (date.get_month() + 1) as u32,
            date.get_date() as u32,
        )
        .unwrap_or_else(|| NaiveDate::from_ymd_opt(1970, 1, 1).unwrap())
    }
}

fn upcoming_date_range() -> (String, String) {
    let start = today_date();
    let end = start + Duration::days(1);
    (format_date(start), format_date(end))
}

fn parse_calendar_date(value: &str) -> Option<NaiveDate> {
    let trimmed = value.trim();
    if trimmed.len() < 10 {
        return None;
    }
    NaiveDate::parse_from_str(&trimmed[..10], "%Y-%m-%d").ok()
}

fn parse_calendar_datetime(value: &str) -> Option<NaiveDateTime> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = trimmed.replace('T', " ");
    if normalized.len() >= 19 {
        NaiveDateTime::parse_from_str(&normalized[..19], "%Y-%m-%d %H:%M:%S").ok()
    } else if normalized.len() >= 16 {
        NaiveDateTime::parse_from_str(&normalized[..16], "%Y-%m-%d %H:%M").ok()
    } else if normalized.len() >= 10 {
        NaiveDate::parse_from_str(&normalized[..10], "%Y-%m-%d")
            .ok()
            .and_then(|date| date.and_hms_opt(0, 0, 0))
    } else {
        None
    }
}

fn duration_minutes_between(start: Option<&str>, end: Option<&str>) -> Option<i64> {
    let start = start.and_then(parse_calendar_datetime)?;
    let end = end.and_then(parse_calendar_datetime)?;
    let minutes = end.signed_duration_since(start).num_minutes();
    if minutes > 0 {
        Some(minutes)
    } else {
        None
    }
}

fn add_minutes_to_local(start: &str, minutes: i64) -> Option<String> {
    let start_time = parse_calendar_datetime(start)?;
    let end_time = start_time + Duration::minutes(minutes);
    Some(end_time.format("%Y-%m-%d %H:%M").to_string())
}

fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

fn calendar_week_range(start: &str, end: &str) -> (NaiveDate, NaiveDate) {
    let anchor = parse_calendar_date(start)
        .or_else(|| parse_calendar_date(end))
        .unwrap_or_else(today_date);
    let offset = anchor.weekday().num_days_from_monday() as i64;
    let week_start = anchor - Duration::days(offset);
    let week_end = week_start + Duration::days(6);
    (week_start, week_end)
}

fn calendar_week_range_strings(start: &str, end: &str) -> (String, String) {
    let (week_start, week_end) = calendar_week_range(start, end);
    (format_date(week_start), format_date(week_end))
}

fn shift_week_range(offset_weeks: i64, date_start: RwSignal<String>, date_end: RwSignal<String>) {
    let (week_start, _) = calendar_week_range(&date_start.get(), &date_end.get());
    let new_start = week_start + Duration::days(offset_weeks * 7);
    let new_end = new_start + Duration::days(6);
    date_start.set(format_date(new_start));
    date_end.set(format_date(new_end));
}

fn weekday_label(date: NaiveDate) -> &'static str {
    match date.weekday() {
        Weekday::Mon => "周一",
        Weekday::Tue => "周二",
        Weekday::Wed => "周三",
        Weekday::Thu => "周四",
        Weekday::Fri => "周五",
        Weekday::Sat => "周六",
        Weekday::Sun => "周日",
    }
}

fn build_calendar_items(
    items: &[Schedule],
    week_start: NaiveDate,
    week_end: NaiveDate,
) -> Vec<CalendarItem> {
    let mut result = Vec::new();
    for schedule in items {
        let Some(start_raw) = schedule.scheduled_start_at.as_ref() else {
            continue;
        };
        let Some(end_raw) = schedule.scheduled_end_at.as_ref() else {
            continue;
        };
        let Some(start) = parse_calendar_datetime(start_raw) else {
            continue;
        };
        let Some(mut end) = parse_calendar_datetime(end_raw) else {
            continue;
        };
        if end <= start {
            continue;
        }
        if end.date() != start.date() {
            if let Some(clamped) = start.date().and_hms_opt(23, 59, 59) {
                end = clamped;
            }
        }
        let day = start.date();
        if day < week_start || day > week_end {
            continue;
        }
        result.push(CalendarItem {
            schedule: schedule.clone(),
            start,
            end,
        });
    }
    result
}

fn calendar_time_bounds(items: &[CalendarItem]) -> (u32, u32) {
    if items.is_empty() {
        return (8, 20);
    }
    let min_hour = items
        .iter()
        .map(|item| item.start.hour())
        .min()
        .unwrap_or(8);
    let max_hour = items.iter().map(|item| item.end.hour()).max().unwrap_or(20);
    let start_hour = std::cmp::min(8, min_hour.saturating_sub(1));
    let mut end_hour = std::cmp::max(20, (max_hour + 1).min(24));
    if end_hour <= start_hour {
        end_hour = (start_hour + 1).min(24);
    }
    (start_hour, end_hour)
}

fn calendar_columns(
    items: &[CalendarItem],
    user_labels: &HashMap<String, String>,
    user_filter: &str,
) -> Vec<CalendarColumn> {
    let mut user_ids: HashSet<String> = HashSet::new();
    let mut has_unassigned = false;
    for item in items {
        if let Some(user_id) = item
            .schedule
            .assigned_user_uuid
            .clone()
            .filter(|id| !id.is_empty())
        {
            user_ids.insert(user_id);
        } else {
            has_unassigned = true;
        }
    }
    if !user_filter.trim().is_empty() {
        user_ids.clear();
        user_ids.insert(user_filter.to_string());
        has_unassigned = false;
    }

    let mut columns = user_ids
        .into_iter()
        .map(|id| {
            let label = user_labels.get(&id).cloned().unwrap_or_else(|| id.clone());
            CalendarColumn { id, label }
        })
        .collect::<Vec<_>>();
    columns.sort_by(|a, b| a.label.cmp(&b.label));
    if has_unassigned {
        columns.insert(
            0,
            CalendarColumn {
                id: String::new(),
                label: "未分配".to_string(),
            },
        );
    }
    columns
}

fn group_calendar_items(
    items: Vec<CalendarItem>,
) -> HashMap<NaiveDate, HashMap<String, Vec<CalendarItem>>> {
    let mut grouped: HashMap<NaiveDate, HashMap<String, Vec<CalendarItem>>> = HashMap::new();
    for item in items {
        let day = item.start.date();
        let user_key = item.schedule.assigned_user_uuid.clone().unwrap_or_default();
        grouped
            .entry(day)
            .or_default()
            .entry(user_key)
            .or_default()
            .push(item);
    }
    for day_items in grouped.values_mut() {
        for items in day_items.values_mut() {
            items.sort_by_key(|item| item.start);
        }
    }
    grouped
}

fn schedule_status_classes(status: &str) -> &'static str {
    match status {
        "planned" => "border-info bg-info/10 text-info",
        "in_service" => "border-warning bg-warning/10 text-warning",
        "done" => "border-success bg-success/10 text-success",
        "cancelled" => "border-error bg-error/10 text-error",
        _ => "border-base-200 bg-base-100 text-base-content",
    }
}

fn schedule_contact_label(
    schedule: &Schedule,
    contact_labels: &HashMap<String, String>,
    pending_contacts: &HashSet<String>,
) -> String {
    let Some(contact_id) = schedule.customer_uuid.clone() else {
        return "未关联客户".to_string();
    };
    if contact_id.is_empty() {
        return "未关联客户".to_string();
    }
    contact_labels.get(&contact_id).cloned().unwrap_or_else(|| {
        if pending_contacts.contains(&contact_id) {
            "加载中...".to_string()
        } else {
            "未知客户".to_string()
        }
    })
}

fn format_time_range(start: NaiveDateTime, end: NaiveDateTime) -> String {
    format!(
        "{:02}:{:02} - {:02}:{:02}",
        start.hour(),
        start.minute(),
        end.hour(),
        end.minute()
    )
}

fn is_overlapping_window_naive(
    start_a: NaiveDateTime,
    end_a: NaiveDateTime,
    start_b: NaiveDateTime,
    end_b: NaiveDateTime,
) -> bool {
    start_a < end_b && end_a > start_b
}

fn calendar_event_position(
    start: NaiveDateTime,
    end: NaiveDateTime,
    start_hour: u32,
    end_hour: u32,
    row_height: i32,
) -> Option<(i32, i32)> {
    let total_minutes = ((end_hour as i32 - start_hour as i32) * 60).max(0);
    if total_minutes == 0 {
        return None;
    }
    let mut start_minutes = (start.hour() as i32 - start_hour as i32) * 60 + start.minute() as i32;
    let mut end_minutes = (end.hour() as i32 - start_hour as i32) * 60 + end.minute() as i32;
    if start_minutes < 0 {
        start_minutes = 0;
    }
    if end_minutes > total_minutes {
        end_minutes = total_minutes;
    }
    if end_minutes <= start_minutes {
        return None;
    }
    let top = start_minutes * row_height / 60;
    let mut height = (end_minutes - start_minutes) * row_height / 60;
    if height < 18 {
        height = 18;
    }
    let max_height = total_minutes * row_height / 60;
    if top + height > max_height {
        height = max_height - top;
    }
    if height <= 0 {
        return None;
    }
    Some((top, height))
}
