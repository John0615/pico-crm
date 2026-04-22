use crate::components::features::get_user_info;
use crate::components::features::schedules::*;
use crate::components::ui::table::Identifiable;
use crate::components::ui::toast::{error, success};
use crate::server::contact_handlers::{fetch_contacts, get_contact};
use crate::server::order_handlers::fetch_orders;
use crate::server::schedule_handlers::{
    cancel_schedule, create_schedule, create_schedule_feedback, fetch_schedule_feedbacks,
    fetch_schedules, update_schedule, update_schedule_status,
};
use crate::server::service_catalog_handlers::fetch_service_catalogs;
use crate::server::service_request_handlers::get_service_request;
use crate::server::user_handlers::{fetch_users, get_user};
use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use shared::contact::ContactQuery;
use shared::order::{CreateOrderFeedbackRequest, OrderQuery};
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
    let view_mode = RwSignal::new("backlog".to_string());
    let status_filter = RwSignal::new(String::new());
    let user_filter = RwSignal::new(String::new());
    let list_date_start = RwSignal::new(String::new());
    let list_date_end = RwSignal::new(String::new());
    let calendar_date_start = RwSignal::new(String::new());
    let calendar_date_end = RwSignal::new(String::new());

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
    let feedback_content = RwSignal::new(String::new());
    let feedback_rating = RwSignal::new(String::new());
    let creating_feedback = RwSignal::new(false);
    let new_schedule_conflict_snapshot: RwSignal<Option<Vec<Schedule>>> = RwSignal::new(None);
    let contact_labels: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let user_labels: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let pending_contacts: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
    let pending_users: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());

    Effect::new(move |_| {
        if view_mode.get() != "calendar" {
            return;
        }
        if !calendar_date_start.get().is_empty() || !calendar_date_end.get().is_empty() {
            return;
        }
        let (start, end) = calendar_week_range_strings("", "");
        calendar_date_start.set(start);
        calendar_date_end.set(end);
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

    let is_worker = Memo::new(move |_| {
        current_user.with(|value| {
            value
                .as_ref()
                .map(|user| {
                    !user.is_admin.unwrap_or(false)
                        && user.role != "operator"
                        && user.role != "merchant"
                        && user.role != "admin"
                })
                .unwrap_or(false)
        })
    });

    Effect::new(move |_| {
        if is_worker.get() && view_mode.get() == "backlog" {
            view_mode.set("list".to_string());
        }
    });

    let data = Resource::new(
        move || {
            (
                view_mode.get(),
                status_filter.get(),
                user_filter.get(),
                list_date_start.get(),
                list_date_end.get(),
                calendar_date_start.get(),
                calendar_date_end.get(),
                *refresh_count.read(),
                query.with(|value| value.clone()),
            )
        },
        |(
            view_mode,
            status,
            user,
            list_start,
            list_end,
            calendar_start,
            calendar_end,
            _,
            query,
        )| async move {
            let is_calendar = view_mode == "calendar";
            let (start_date, end_date) = active_schedule_date_range(
                &view_mode,
                &list_start,
                &list_end,
                &calendar_start,
                &calendar_end,
            );
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

    let new_schedule_time_window = Memo::new(move |_| {
        let start_raw = new_expected_start_at.get();
        let duration = new_duration_minutes.get();
        if start_raw.trim().is_empty() || duration <= 0 {
            return None;
        }
        let start = parse_calendar_datetime(&start_raw)?;
        let end_raw = add_minutes_to_local(&start_raw, duration)?;
        let end = parse_calendar_datetime(&end_raw)?;
        Some((start, end))
    });

    Effect::new(move || {
        let map = query.with(|value| value.clone());
        if let Some(status) = map.get("status") {
            if status_filter.get() != status {
                status_filter.set(status);
            }
            view_mode.set("list".to_string());
        }
        if let Some(start) = map.get("start_date") {
            if list_date_start.get() != start {
                list_date_start.set(start);
            }
            view_mode.set("list".to_string());
        }
        if let Some(end) = map.get("end_date") {
            if list_date_end.get() != end {
                list_date_end.set(end);
            }
            view_mode.set("list".to_string());
        }
        if map.get("upcoming").is_some() {
            let (start, end) = upcoming_date_range();
            list_date_start.set(start);
            list_date_end.set(end);
            view_mode.set("list".to_string());
        }
    });

    let new_schedule_conflicts = Resource::new(
        move || {
            (
                show_new_modal.get(),
                new_schedule_time_window.get(),
                refresh_count.get(),
            )
        },
        |(open, window, _)| async move {
            if !open {
                return Vec::new();
            }
            let Some((start, end)) = window else {
                return Vec::new();
            };
            let (query_start, query_end) = schedule_conflict_query_range(start, end);
            let params = ScheduleQuery {
                page: 1,
                page_size: 500,
                status: None,
                assigned_user_uuid: None,
                start_date: Some(query_start),
                end_date: Some(query_end),
            };
            match call_api(fetch_schedules(params)).await {
                Ok(result) => result.items,
                Err(err) => {
                    logging::error!("Error loading schedule conflicts: {err}");
                    Vec::new()
                }
            }
        },
    );

    Effect::new(move || {
        new_schedule_conflict_snapshot.set(new_schedule_conflicts.get());
    });

    let schedule_feedbacks = Resource::new(
        move || {
            (
                detail_schedule
                    .get()
                    .map(|schedule| schedule.order_uuid.clone()),
                refresh_count.get(),
            )
        },
        |(order_uuid, _)| async move {
            match order_uuid {
                Some(order_uuid) if !order_uuid.trim().is_empty() => {
                    call_api(fetch_schedule_feedbacks(order_uuid))
                        .await
                        .unwrap_or_default()
                }
                _ => Vec::new(),
            }
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

    let available_orders = Resource::new(
        move || {
            (
                show_new_modal.get(),
                view_mode.get(),
                *refresh_count.read(),
                query.with(|value| value.clone()),
            )
        },
        |(open, mode, _, query)| async move {
            if !open && mode != "backlog" {
                return (Vec::new(), 0);
            }
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
                status: None,
                statuses: Some(vec!["pending".to_string(), "confirmed".to_string()]),
                settlement_status: None,
                customer_uuid: None,
                start_date: None,
                end_date: None,
            };
            match call_api(fetch_orders(params)).await {
                Ok(result) => (result.items, result.total),
                Err(err) => {
                    logging::error!("Error loading orders: {err}");
                    (Vec::new(), 0)
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
                role: None,
                status: None,
                employment_status: None,
                skill: None,
                dispatchable_only: Some(true),
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

    let service_catalogs = Resource::new(
        move || (),
        |_| async move {
            match call_api(fetch_service_catalogs(1, 1000, Some(true))).await {
                Ok(result) => result.items,
                Err(err) => {
                    logging::error!("Error loading service catalogs: {err}");
                    Vec::new()
                }
            }
        },
    );

    Effect::new(move |_| {
        if duration_overridden.get() {
            return;
        }
        let selected = new_service_type.get();
        if selected.trim().is_empty() {
            return;
        }
        let catalogs = service_catalogs.get().unwrap_or_default();
        if let Some(item) = catalogs.iter().find(|item| item.name == selected) {
            if let Some(minutes) = item.default_duration_minutes {
                new_duration_minutes.set(minutes as i64);
            }
        }
    });

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
        let Some((items, _)) = available_orders.get() else {
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
            if !duration_overridden.get_untracked() {
                if let Some(minutes) = duration_minutes_between(
                    request.appointment_start_at.as_deref(),
                    request.appointment_end_at.as_deref(),
                ) {
                    if new_order_uuid.get_untracked() == order_uuid_guard {
                        new_duration_minutes.set(minutes);
                    }
                }
            }
            if !expected_start_overridden.get_untracked()
                && new_expected_start_at.get_untracked().trim().is_empty()
                && new_order_uuid.get_untracked() == order_uuid_guard
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

        let existing = contact_labels.get();
        let mut pending = pending_contacts.get();
        let mut missing_ids: HashSet<String> = HashSet::new();

        if let Some((items, _)) = data.get() {
            for schedule in &items {
                let Some(contact_id) = schedule.customer_uuid.clone() else {
                    continue;
                };
                if contact_id.is_empty()
                    || existing.contains_key(&contact_id)
                    || pending.contains(&contact_id)
                {
                    continue;
                }
                pending.insert(contact_id.clone());
                missing_ids.insert(contact_id);
            }
        }

        if let Some((items, _)) = available_orders.get() {
            for order in &items {
                let Some(contact_id) = order.customer_uuid.clone() else {
                    continue;
                };
                if contact_id.is_empty()
                    || existing.contains_key(&contact_id)
                    || pending.contains(&contact_id)
                {
                    continue;
                }
                pending.insert(contact_id.clone());
                missing_ids.insert(contact_id);
            }
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
        if user_id.trim().is_empty() {
            return ConflictState::Unknown;
        }
        let Some((start, end)) = new_schedule_time_window.get() else {
            return ConflictState::Unknown;
        };
        let Some(items) = new_schedule_conflict_snapshot.get() else {
            return ConflictState::Unknown;
        };
        let labels = contact_labels.get();
        let pending = pending_contacts.get();
        if let Some(schedule) = find_conflicting_schedule_for_user(&user_id, start, end, &items) {
            let label = schedule_contact_label(schedule, &labels, &pending);
            return ConflictState::Conflict(label);
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
        let selected_user = new_assigned_user_uuid.get();
        if selected_user.trim().is_empty() {
            return;
        }
        let Some((start, end)) = new_schedule_time_window.get() else {
            new_assigned_user_uuid.set(String::new());
            return;
        };
        let Some(items) = new_schedule_conflict_snapshot.get() else {
            return;
        };
        if find_conflicting_schedule_for_user(&selected_user, start, end, &items).is_some() {
            new_assigned_user_uuid.set(String::new());
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

    let open_assignment = Callback::new(move |schedule: Schedule| {
        assignment_order_uuid.set(schedule.order_uuid.clone());
        assigned_user_uuid.set(schedule.assigned_user_uuid.clone().unwrap_or_default());
        scheduled_start_at.set(to_datetime_local(schedule.scheduled_start_at.clone()));
        scheduled_end_at.set(to_datetime_local(schedule.scheduled_end_at.clone()));
        dispatch_note.set(schedule.dispatch_note.clone().unwrap_or_default());
        assignment_is_new.set(schedule.assigned_user_uuid.is_none());
        show_assignment_modal.set(true);
    });

    let open_new_schedule_for_order = Callback::new(move |order_uuid: String| {
        new_order_uuid.set(order_uuid);
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
    });

    let open_detail = Callback::new(move |schedule: Schedule| {
        feedback_content.set(String::new());
        feedback_rating.set(String::new());
        detail_schedule.set(Some(schedule));
        show_detail_modal.set(true);
    });

    let submit_feedback = Callback::new(move |_| {
        let Some(schedule) = detail_schedule.get() else {
            return;
        };
        if schedule.schedule_status != "done" || schedule.order_status != "completed" {
            error("仅已完成服务可提交反馈".to_string());
            return;
        }
        if creating_feedback.get() {
            return;
        }
        let content = feedback_content.get();
        if content.trim().is_empty() {
            error("请填写服务反馈内容".to_string());
            return;
        }
        let rating = if feedback_rating.get().trim().is_empty() {
            None
        } else {
            match feedback_rating.get().trim().parse::<i32>() {
                Ok(value) if (1..=5).contains(&value) => Some(value),
                _ => {
                    error("评分必须在 1 到 5 之间".to_string());
                    return;
                }
            }
        };

        let payload = CreateOrderFeedbackRequest { rating, content };
        creating_feedback.set(true);
        spawn_local(async move {
            match call_api(create_schedule_feedback(
                schedule.order_uuid.clone(),
                payload,
            ))
            .await
            {
                Ok(_) => {
                    success("服务反馈已提交".to_string());
                    feedback_content.set(String::new());
                    feedback_rating.set(String::new());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("提交反馈失败: {}", err)),
            }
            creating_feedback.set(false);
        });
    });

    let submit_new_schedule = Callback::new(move |_| {
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
    });

    let submit_assignment = Callback::new(move |_| {
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
    });

    let cancel_assignment = Callback::new(move |uuid: String| {
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
    });

    let update_status = Callback::new(move |(uuid, status): (String, String)| {
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
    });

    let is_worker_signal = Signal::derive(move || is_worker.get());
    let current_user_uuid = Signal::derive(move || {
        current_user.with(|value| {
            value
                .as_ref()
                .map(|user| user.uuid.clone())
                .unwrap_or_default()
        })
    });
    let new_schedule_time_window_signal = Signal::derive(move || new_schedule_time_window.get());
    let conflict_state_signal = Signal::derive(move || conflict_state.get());
    let can_submit_new_signal = Signal::derive(move || can_submit_new.get());

    view! {
        <Title text="排班管理 - PicoCRM"/>
        <div class="space-y-4">
            <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <h1 class="text-2xl font-semibold">"排班管理"</h1>
                <div class="flex flex-wrap items-center gap-2">
                    <div class="tabs tabs-boxed">
                        <Show when=move || !is_worker.get() fallback=|| ()>
                            <button
                                class=move || {
                                    if view_mode.get() == "backlog" {
                                        "tab tab-active"
                                    } else {
                                        "tab"
                                    }
                                }
                                on:click=move |_| view_mode.set("backlog".to_string())
                            >
                                "未排班订单"
                            </button>
                        </Show>
                        <button
                            class=move || {
                                if view_mode.get() == "list" {
                                    "tab tab-active"
                                } else {
                                    "tab"
                                }
                            }
                            on:click=move |_| view_mode.set("list".to_string())
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
                            on:click=move |_| view_mode.set("calendar".to_string())
                        >
                            "日历"
                        </button>
                    </div>
                </div>
            </div>

            <Show when=move || view_mode.get() != "backlog">
                <SchedulesFiltersCard
                    view_mode=view_mode
                    status_filter=status_filter
                    user_filter=user_filter
                    list_date_start=list_date_start
                    list_date_end=list_date_end
                    calendar_date_start=calendar_date_start
                    calendar_date_end=calendar_date_end
                    users=users.clone()
                    is_worker=is_worker_signal.clone()
                />
            </Show>

            <Show when=move || view_mode.get() == "backlog">
                <SchedulesBacklogPanel
                    available_orders=available_orders.clone()
                    contact_labels=contact_labels
                    pending_contacts=pending_contacts
                    open_new_schedule_for_order=open_new_schedule_for_order.clone()
                />
            </Show>

            <Show when=move || view_mode.get() == "calendar">
                <SchedulesCalendarPanel
                    data=data.clone()
                    calendar_date_start=calendar_date_start
                    calendar_date_end=calendar_date_end
                    user_labels=user_labels
                    contact_labels=contact_labels
                    pending_contacts=pending_contacts
                    user_filter=user_filter
                    is_worker=is_worker_signal.clone()
                    open_assignment=open_assignment.clone()
                    open_detail=open_detail.clone()
                />
            </Show>

            <Show when=move || view_mode.get() == "list">
                <SchedulesListPanel
                    data=data.clone()
                    contact_labels=contact_labels
                    pending_contacts=pending_contacts
                    user_labels=user_labels
                    pending_users=pending_users
                    is_worker=is_worker_signal.clone()
                    open_assignment=open_assignment.clone()
                    cancel_assignment=cancel_assignment.clone()
                    update_status=update_status.clone()
                    open_detail=open_detail.clone()
                />
            </Show>
        </div>

        <NewScheduleModal
            show=show_new_modal
            new_order_uuid=new_order_uuid
            available_orders=available_orders.clone()
            contact_labels=contact_labels
            pending_contacts=pending_contacts
            service_catalogs=service_catalogs.clone()
            new_service_type=new_service_type
            duration_overridden=duration_overridden
            new_duration_minutes=new_duration_minutes
            new_expected_start_at=new_expected_start_at
            new_expected_end_at=new_expected_end_at
            users=users.clone()
            new_schedule_time_window=new_schedule_time_window_signal
            new_schedule_conflict_snapshot=new_schedule_conflict_snapshot
            new_assigned_user_uuid=new_assigned_user_uuid
            conflict_state=conflict_state_signal
            can_submit_new=can_submit_new_signal
            creating_schedule=creating_schedule
            submit_new_schedule=submit_new_schedule.clone()
        />

        <AssignmentModal
            show=show_assignment_modal
            users=users.clone()
            assigned_user_uuid=assigned_user_uuid
            scheduled_start_at=scheduled_start_at
            scheduled_end_at=scheduled_end_at
            dispatch_note=dispatch_note
            submit_assignment=submit_assignment.clone()
        />

        <ScheduleDetailModal
            show=show_detail_modal
            detail_schedule=detail_schedule
            schedule_feedbacks=schedule_feedbacks.clone()
            current_user_uuid=current_user_uuid
            is_worker=is_worker_signal
            feedback_rating=feedback_rating
            feedback_content=feedback_content
            creating_feedback=creating_feedback
            submit_feedback=submit_feedback.clone()
        />
    }
}
