#[cfg(feature = "ssr")]
use chrono::Utc;
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Timelike, Weekday};
#[cfg(not(feature = "ssr"))]
use js_sys::Date as JsDate;
use leptos::prelude::*;
use shared::contact::Contact;
use shared::order::Order;
use shared::schedule::Schedule;
use shared::user::User;
use std::collections::{HashMap, HashSet};

#[derive(Clone, PartialEq)]
pub enum ConflictState {
    Unknown,
    Available,
    Conflict(String),
}

#[derive(Clone)]
pub struct CalendarItem {
    pub schedule: Schedule,
    pub start: NaiveDateTime,
    pub end: NaiveDateTime,
}

#[derive(Clone)]
pub struct CalendarColumn {
    pub id: String,
    pub label: String,
}

pub const DURATION_OPTIONS: [(i64, &str); 5] = [
    (60, "1小时"),
    (90, "1.5小时"),
    (120, "2小时"),
    (180, "3小时"),
    (240, "4小时"),
];

pub fn schedule_status_label(schedule: &Schedule) -> &'static str {
    match schedule.schedule_status.as_str() {
        "planned" => {
            if schedule_has_assignment(schedule) {
                "已排班"
            } else {
                "待排班"
            }
        }
        "in_service" => "服务中",
        "done" => "已完成",
        "cancelled" => "已取消",
        _ => "未知",
    }
}

pub fn schedule_status_badge_class(status: &str) -> &'static str {
    match status {
        "planned" => "badge-info",
        "in_service" => "badge-warning",
        "done" => "badge-success",
        "cancelled" => "badge-error",
        _ => "badge-info",
    }
}

pub fn schedule_has_assignment(schedule: &Schedule) -> bool {
    schedule
        .assigned_user_uuid
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
        || schedule
            .scheduled_start_at
            .as_deref()
            .map(|value| !value.trim().is_empty())
            .unwrap_or(false)
}

pub fn order_option_label(
    order: &Order,
    contact_labels: &HashMap<String, String>,
    pending_contacts: &HashSet<String>,
) -> String {
    let contact_label = order_contact_label(order, contact_labels, pending_contacts);
    let short_id = shorten_uuid(&order.uuid);
    format!("{} · 订单 {}", contact_label, short_id)
}

pub fn order_contact_label(
    order: &Order,
    contact_labels: &HashMap<String, String>,
    pending_contacts: &HashSet<String>,
) -> String {
    if let Some(name) = order.customer_name.as_ref().map(|value| value.trim()) {
        if !name.is_empty() {
            return name.to_string();
        }
    }

    order
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
        .unwrap_or_else(|| "未知客户".to_string())
}

pub fn order_status_label(status: &str) -> &'static str {
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

pub fn order_status_badge_class(status: &str) -> &'static str {
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

pub fn shorten_uuid(value: &str) -> String {
    if value.len() > 8 {
        value[..8].to_string()
    } else {
        value.to_string()
    }
}

pub fn duration_label(minutes: i64) -> String {
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

pub fn build_dispatch_note(service_type: &str, duration_label: &str) -> String {
    format!("服务类型: {} | 服务时长: {}", service_type, duration_label)
}

pub fn extract_service_type(dispatch_note: Option<&String>) -> Option<String> {
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

pub fn service_type_card_classes(service_type: &str) -> Option<&'static str> {
    match service_type {
        "保洁" => Some("border-sky-200 bg-sky-50 text-sky-900"),
        "维修" => Some("border-emerald-200 bg-emerald-50 text-emerald-900"),
        "家电清洗" => Some("border-amber-200 bg-amber-50 text-amber-900"),
        _ => None,
    }
}

pub fn schedule_card_classes(schedule: &Schedule) -> &'static str {
    let service_type = extract_service_type(schedule.dispatch_note.as_ref());
    if let Some(service_type) = service_type {
        if let Some(classes) = service_type_card_classes(&service_type) {
            return classes;
        }
    }
    schedule_status_classes(&schedule.schedule_status)
}

pub fn contact_display_label(contact: &Contact) -> String {
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

pub fn user_display_label(user: &User) -> String {
    let skill_hint = if user.skills.is_empty() {
        None
    } else {
        Some(
            user.skills
                .iter()
                .take(2)
                .cloned()
                .collect::<Vec<_>>()
                .join("/"),
        )
    };

    if let Some(phone) = user.phone_number.clone().filter(|value| !value.is_empty()) {
        if let Some(skill_hint) = skill_hint {
            format!("{} ({}, {})", user.user_name, phone, skill_hint)
        } else {
            format!("{} ({})", user.user_name, phone)
        }
    } else if let Some(email) = user.email.clone().filter(|value| !value.is_empty()) {
        if let Some(skill_hint) = skill_hint {
            format!("{} ({}, {})", user.user_name, email, skill_hint)
        } else {
            format!("{} ({})", user.user_name, email)
        }
    } else if let Some(skill_hint) = skill_hint {
        format!("{} ({})", user.user_name, skill_hint)
    } else {
        user.user_name.clone()
    }
}

pub fn normalize_optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

pub fn normalize_datetime_local(value: &str) -> Option<String> {
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

pub fn to_datetime_local(value: Option<String>) -> String {
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

pub fn is_end_before_start(start: &str, end: &str) -> bool {
    match (
        normalize_datetime_local(start),
        normalize_datetime_local(end),
    ) {
        (Some(start), Some(end)) => end <= start,
        _ => false,
    }
}

pub fn format_time_window(start: Option<String>, end: Option<String>) -> String {
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

pub fn display_optional(value: Option<String>) -> String {
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

pub fn detail_item(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-1">
            <span class="text-xs text-base-content/60">{label}</span>
            <span class="text-sm break-all">{value}</span>
        </div>
    }
}

pub fn today_date() -> NaiveDate {
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

pub fn upcoming_date_range() -> (String, String) {
    let start = today_date();
    let end = start + Duration::days(1);
    (format_date(start), format_date(end))
}

pub fn parse_calendar_date(value: &str) -> Option<NaiveDate> {
    let trimmed = value.trim();
    if trimmed.len() < 10 {
        return None;
    }
    NaiveDate::parse_from_str(&trimmed[..10], "%Y-%m-%d").ok()
}

pub fn parse_calendar_datetime(value: &str) -> Option<NaiveDateTime> {
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

pub fn duration_minutes_between(start: Option<&str>, end: Option<&str>) -> Option<i64> {
    let start = start.and_then(parse_calendar_datetime)?;
    let end = end.and_then(parse_calendar_datetime)?;
    let minutes = end.signed_duration_since(start).num_minutes();
    if minutes > 0 {
        Some(minutes)
    } else {
        None
    }
}

pub fn add_minutes_to_local(start: &str, minutes: i64) -> Option<String> {
    let start_time = parse_calendar_datetime(start)?;
    let end_time = start_time + Duration::minutes(minutes);
    Some(end_time.format("%Y-%m-%d %H:%M").to_string())
}

pub fn format_date(date: NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

pub fn calendar_week_range(start: &str, end: &str) -> (NaiveDate, NaiveDate) {
    let anchor = parse_calendar_date(start)
        .or_else(|| parse_calendar_date(end))
        .unwrap_or_else(today_date);
    let offset = anchor.weekday().num_days_from_monday() as i64;
    let week_start = anchor - Duration::days(offset);
    let week_end = week_start + Duration::days(6);
    (week_start, week_end)
}

pub fn active_schedule_date_range(
    view_mode: &str,
    list_start: &str,
    list_end: &str,
    calendar_start: &str,
    calendar_end: &str,
) -> (String, String) {
    if view_mode == "calendar" {
        calendar_week_range_strings(calendar_start, calendar_end)
    } else {
        (list_start.to_string(), list_end.to_string())
    }
}

pub fn calendar_week_range_strings(start: &str, end: &str) -> (String, String) {
    let (week_start, week_end) = calendar_week_range(start, end);
    (format_date(week_start), format_date(week_end))
}

pub fn shift_week_range(
    offset_weeks: i64,
    date_start: RwSignal<String>,
    date_end: RwSignal<String>,
) {
    let (week_start, _) = calendar_week_range(&date_start.get(), &date_end.get());
    let new_start = week_start + Duration::days(offset_weeks * 7);
    let new_end = new_start + Duration::days(6);
    date_start.set(format_date(new_start));
    date_end.set(format_date(new_end));
}

pub fn weekday_label(date: NaiveDate) -> &'static str {
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

pub fn build_calendar_items(
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

pub fn calendar_time_bounds(items: &[CalendarItem]) -> (u32, u32) {
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

pub fn calendar_columns(
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

pub fn group_calendar_items(
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

pub fn schedule_status_classes(status: &str) -> &'static str {
    match status {
        "planned" => "border-info bg-info/10 text-info",
        "in_service" => "border-warning bg-warning/10 text-warning",
        "done" => "border-success bg-success/10 text-success",
        "cancelled" => "border-error bg-error/10 text-error",
        _ => "border-base-200 bg-base-100 text-base-content",
    }
}

pub fn schedule_contact_label(
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

pub fn format_time_range(start: NaiveDateTime, end: NaiveDateTime) -> String {
    format!(
        "{:02}:{:02} - {:02}:{:02}",
        start.hour(),
        start.minute(),
        end.hour(),
        end.minute()
    )
}

pub fn is_overlapping_window_naive(
    start_a: NaiveDateTime,
    end_a: NaiveDateTime,
    start_b: NaiveDateTime,
    end_b: NaiveDateTime,
) -> bool {
    start_a < end_b && end_a > start_b
}

pub fn schedule_conflict_query_range(start: NaiveDateTime, end: NaiveDateTime) -> (String, String) {
    (
        format!("{} 00:00:00", start.date().format("%Y-%m-%d")),
        format!("{} 23:59:59", end.date().format("%Y-%m-%d")),
    )
}

pub fn find_conflicting_schedule_for_user<'a>(
    user_id: &str,
    start: NaiveDateTime,
    end: NaiveDateTime,
    items: &'a [Schedule],
) -> Option<&'a Schedule> {
    for schedule in items {
        let Some(assigned) = schedule.assigned_user_uuid.as_deref() else {
            continue;
        };
        if assigned != user_id {
            continue;
        }
        let Some(existing_start_raw) = schedule.scheduled_start_at.as_deref() else {
            continue;
        };
        let Some(existing_end_raw) = schedule.scheduled_end_at.as_deref() else {
            continue;
        };
        let Some(existing_start) = parse_calendar_datetime(existing_start_raw) else {
            continue;
        };
        let Some(existing_end) = parse_calendar_datetime(existing_end_raw) else {
            continue;
        };
        if is_overlapping_window_naive(start, end, existing_start, existing_end) {
            return Some(schedule);
        }
    }
    None
}

pub fn calendar_event_position(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn list_view_keeps_its_own_date_filters() {
        let dates = active_schedule_date_range(
            "list",
            "2026-04-01",
            "2026-04-30",
            "2026-04-14",
            "2026-04-20",
        );

        assert_eq!(dates, ("2026-04-01".to_string(), "2026-04-30".to_string()));
    }

    #[test]
    fn calendar_view_uses_calendar_week_range_only() {
        let dates = active_schedule_date_range(
            "calendar",
            "2026-04-01",
            "2026-04-30",
            "2026-04-16",
            "2026-04-18",
        );

        assert_eq!(dates, ("2026-04-13".to_string(), "2026-04-19".to_string()));
    }

    #[test]
    fn conflict_detection_matches_overlap_rules() {
        let start_a = NaiveDate::from_ymd_opt(2026, 4, 20)
            .unwrap()
            .and_hms_opt(9, 0, 0)
            .unwrap();
        let end_a = NaiveDate::from_ymd_opt(2026, 4, 20)
            .unwrap()
            .and_hms_opt(10, 0, 0)
            .unwrap();
        let start_b = NaiveDate::from_ymd_opt(2026, 4, 20)
            .unwrap()
            .and_hms_opt(9, 30, 0)
            .unwrap();
        let end_b = NaiveDate::from_ymd_opt(2026, 4, 20)
            .unwrap()
            .and_hms_opt(10, 30, 0)
            .unwrap();

        assert!(is_overlapping_window_naive(start_a, end_a, start_b, end_b));
    }
}
