use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};

use crate::domain::models::order::{Order as DomainOrder, OrderAssignmentUpdate};
use crate::domain::models::schedule::{ScheduleStatus, validate_time_window};
use crate::domain::repositories::order::OrderRepository;
use crate::infrastructure::utils::parse_date_time_to_string;
use shared::schedule::{
    CreateScheduleAssignment, Schedule, UpdateScheduleAssignment, UpdateScheduleStatus,
};

pub struct ScheduleAppService<R: OrderRepository> {
    order_repo: R,
}

impl<R: OrderRepository> ScheduleAppService<R> {
    pub fn new(order_repo: R) -> Self {
        Self { order_repo }
    }

    pub async fn create_schedule(
        &self,
        order_uuid: String,
        payload: CreateScheduleAssignment,
    ) -> Result<Schedule, String> {
        let assigned_user_uuid = normalize_required(payload.assigned_user_uuid, "assigned user")?;
        let start = parse_required_datetime(&payload.scheduled_start_at, "scheduled_start_at")?;
        let end = parse_required_datetime(&payload.scheduled_end_at, "scheduled_end_at")?;
        validate_time_window(start, end)?;

        let update = OrderAssignmentUpdate {
            assigned_user_uuid: Some(assigned_user_uuid),
            scheduled_start_at: Some(start),
            scheduled_end_at: Some(end),
            dispatch_note: payload.dispatch_note,
        };
        let updated = self.order_repo.update_order_assignment(order_uuid, update).await?;
        Ok(to_schedule_view(updated))
    }

    pub async fn update_schedule(
        &self,
        order_uuid: String,
        payload: UpdateScheduleAssignment,
    ) -> Result<Schedule, String> {
        let assigned_user_uuid = normalize_optional(payload.assigned_user_uuid);
        let start = parse_optional_datetime(payload.scheduled_start_at.as_deref(), "scheduled_start_at")?;
        let end = parse_optional_datetime(payload.scheduled_end_at.as_deref(), "scheduled_end_at")?;

        if (start.is_some() && end.is_none()) || (start.is_none() && end.is_some()) {
            return Err("scheduled_start_at and scheduled_end_at must be provided together".to_string());
        }
        if let (Some(start), Some(end)) = (start, end) {
            validate_time_window(start, end)?;
        }

        let update = OrderAssignmentUpdate {
            assigned_user_uuid,
            scheduled_start_at: start,
            scheduled_end_at: end,
            dispatch_note: payload.dispatch_note,
        };
        let updated = self.order_repo.update_order_assignment(order_uuid, update).await?;
        Ok(to_schedule_view(updated))
    }

    pub async fn cancel_schedule(&self, order_uuid: String) -> Result<Schedule, String> {
        self.update_schedule_status(
            order_uuid,
            UpdateScheduleStatus {
                status: "cancelled".to_string(),
            },
        )
        .await
    }

    pub async fn update_schedule_status(
        &self,
        order_uuid: String,
        payload: UpdateScheduleStatus,
    ) -> Result<Schedule, String> {
        let target_status = ScheduleStatus::parse(&payload.status)?;
        let target_order_status = target_status
            .target_order_status()
            .ok_or_else(|| "planned status does not require an order status update".to_string())?;

        let current = self
            .order_repo
            .find_order(order_uuid.clone())
            .await?
            .ok_or_else(|| format!("order {} not found", order_uuid))?;
        let current_schedule = ScheduleStatus::from_order_status(&current.status);
        ScheduleStatus::validate_transition(current_schedule, target_status)?;

        let updated = self
            .order_repo
            .update_order_status(order_uuid, target_order_status.as_str().to_string())
            .await?;
        Ok(to_schedule_view(updated))
    }
}

fn to_schedule_view(order: DomainOrder) -> Schedule {
    Schedule {
        order_uuid: order.uuid,
        order_status: order.status.as_str().to_string(),
        schedule_status: ScheduleStatus::from_order_status(&order.status)
            .as_str()
            .to_string(),
        contact_uuid: order.contact_uuid,
        assigned_user_uuid: order.assigned_user_uuid,
        scheduled_start_at: order.scheduled_start_at.map(parse_date_time_to_string),
        scheduled_end_at: order.scheduled_end_at.map(parse_date_time_to_string),
        dispatch_note: order.dispatch_note,
        notes: order.notes,
        inserted_at: parse_date_time_to_string(order.inserted_at),
        updated_at: parse_date_time_to_string(order.updated_at),
    }
}

fn normalize_required(value: String, field: &str) -> Result<String, String> {
    let trimmed = value.trim().to_string();
    if trimmed.is_empty() {
        return Err(format!("{} is required", field));
    }
    Ok(trimmed)
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|v| {
        let trimmed = v.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn parse_required_datetime(value: &str, field: &str) -> Result<DateTime<Utc>, String> {
    parse_datetime(value).ok_or_else(|| format!("invalid {}", field))
}

fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    if value.trim().is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
            return Some(Utc.from_utc_datetime(&dt));
        }
    }
    None
}

fn parse_optional_datetime(
    value: Option<&str>,
    field: &str,
) -> Result<Option<DateTime<Utc>>, String> {
    let Some(value) = value else { return Ok(None) };
    if value.trim().is_empty() {
        return Ok(None);
    }
    parse_datetime(value).ok_or_else(|| format!("invalid {}", field)).map(Some)
}
