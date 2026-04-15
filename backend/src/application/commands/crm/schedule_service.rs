use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};

use crate::application::utils::parse_utc_time_to_string;
use crate::domain::crm::order::{
    Order as DomainOrder, OrderAssignmentUpdate, OrderRepository, OrderStatus,
};
use crate::domain::crm::schedule::{
    ScheduleAssignment, ScheduleRepository, ScheduleStatus, validate_time_window,
};
use shared::schedule::{
    CreateScheduleAssignment, Schedule, UpdateScheduleAssignment, UpdateScheduleStatus,
};

pub struct ScheduleAppService<R: OrderRepository, S: ScheduleRepository> {
    order_repo: R,
    schedule_repo: S,
}

impl<R: OrderRepository, S: ScheduleRepository> ScheduleAppService<R, S> {
    pub fn new(order_repo: R, schedule_repo: S) -> Self {
        Self {
            order_repo,
            schedule_repo,
        }
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

        let order = self
            .order_repo
            .find_order(order_uuid.clone())
            .await?
            .ok_or_else(|| format!("order {} not found", order_uuid))?;

        let current_schedule = ScheduleStatus::from_order_status(&order.status);
        if !current_schedule.allows_assignment_update() {
            return Err(format!(
                "schedule assignment can only be updated in planned status (current: {})",
                current_schedule.as_str()
            ));
        }

        if let Some(conflict) = self
            .schedule_repo
            .find_conflict(
                assigned_user_uuid.clone(),
                start.clone(),
                end.clone(),
                Some(order_uuid.clone()),
            )
            .await?
        {
            return Err(format!(
                "schedule time overlaps with existing assignment {}",
                conflict.order_uuid
            ));
        }

        let schedule_status = ScheduleStatus::from_order_status(&order.status);
        let assignment = ScheduleAssignment {
            uuid: String::new(),
            order_uuid: order_uuid.clone(),
            assigned_user_uuid: assigned_user_uuid.clone(),
            start_at: start.clone(),
            end_at: end.clone(),
            status: schedule_status,
            notes: payload.dispatch_note.clone(),
            inserted_at: Utc::now(),
            updated_at: Utc::now(),
        };

        let existing = self.schedule_repo.find_by_order(order_uuid.clone()).await?;
        let assignment = if existing.is_some() {
            self.schedule_repo
                .update_assignment(
                    order_uuid.clone(),
                    assigned_user_uuid.clone(),
                    start.clone(),
                    end.clone(),
                    payload.dispatch_note.clone(),
                )
                .await?
        } else {
            self.schedule_repo.create_assignment(assignment).await?
        };

        let update = OrderAssignmentUpdate {
            assigned_user_uuid: None,
            scheduled_start_at: Some(start),
            scheduled_end_at: Some(end),
            dispatch_note: payload.dispatch_note,
        };
        let updated = self
            .order_repo
            .update_order_assignment(order_uuid, update)
            .await?;
        let next_status = OrderStatus::next_after_schedule_assignment(order.status);
        let updated = if next_status != updated.status {
            self.order_repo
                .update_order_status(updated.uuid.clone(), next_status.as_str().to_string())
                .await?
        } else {
            updated
        };
        Ok(build_schedule_view(updated, Some(assignment)))
    }

    pub async fn update_schedule(
        &self,
        order_uuid: String,
        payload: UpdateScheduleAssignment,
    ) -> Result<Schedule, String> {
        let assigned_user_uuid = normalize_optional(payload.assigned_user_uuid);
        let start =
            parse_optional_datetime(payload.scheduled_start_at.as_deref(), "scheduled_start_at")?;
        let end = parse_optional_datetime(payload.scheduled_end_at.as_deref(), "scheduled_end_at")?;

        if (start.is_some() && end.is_none()) || (start.is_none() && end.is_some()) {
            return Err(
                "scheduled_start_at and scheduled_end_at must be provided together".to_string(),
            );
        }
        if let (Some(start), Some(end)) = (start, end) {
            validate_time_window(start, end)?;
        }

        let order = self
            .order_repo
            .find_order(order_uuid.clone())
            .await?
            .ok_or_else(|| format!("order {} not found", order_uuid))?;
        let current_schedule = ScheduleStatus::from_order_status(&order.status);
        if !current_schedule.allows_assignment_update() {
            return Err(format!(
                "schedule assignment can only be updated in planned status (current: {})",
                current_schedule.as_str()
            ));
        }

        let assigned_user_uuid = assigned_user_uuid
            .clone()
            .ok_or_else(|| "assigned_user_uuid is required".to_string())?;
        let start = start.ok_or_else(|| "scheduled_start_at is required".to_string())?;
        let end = end.ok_or_else(|| "scheduled_end_at is required".to_string())?;
        if let Some(conflict) = self
            .schedule_repo
            .find_conflict(
                assigned_user_uuid.clone(),
                start.clone(),
                end.clone(),
                Some(order_uuid.clone()),
            )
            .await?
        {
            return Err(format!(
                "schedule time overlaps with existing assignment {}",
                conflict.order_uuid
            ));
        }

        let assignment = match self.schedule_repo.find_by_order(order_uuid.clone()).await? {
            Some(_) => {
                self.schedule_repo
                    .update_assignment(
                        order_uuid.clone(),
                        assigned_user_uuid.clone(),
                        start.clone(),
                        end.clone(),
                        payload.dispatch_note.clone(),
                    )
                    .await?
            }
            None => {
                let schedule_status = ScheduleStatus::from_order_status(&order.status);
                let new_assignment = ScheduleAssignment {
                    uuid: String::new(),
                    order_uuid: order_uuid.clone(),
                    assigned_user_uuid: assigned_user_uuid.clone(),
                    start_at: start.clone(),
                    end_at: end.clone(),
                    status: schedule_status,
                    notes: payload.dispatch_note.clone(),
                    inserted_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                self.schedule_repo.create_assignment(new_assignment).await?
            }
        };

        let update = OrderAssignmentUpdate {
            assigned_user_uuid: None,
            scheduled_start_at: Some(start),
            scheduled_end_at: Some(end),
            dispatch_note: payload.dispatch_note,
        };
        let updated = self
            .order_repo
            .update_order_assignment(order_uuid, update)
            .await?;
        let next_status = OrderStatus::next_after_schedule_assignment(order.status);
        let updated = if next_status != updated.status {
            self.order_repo
                .update_order_status(updated.uuid.clone(), next_status.as_str().to_string())
                .await?
        } else {
            updated
        };
        Ok(build_schedule_view(updated, Some(assignment)))
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
            .update_order_status(order_uuid.clone(), target_order_status.as_str().to_string())
            .await?;
        let assignment = self
            .schedule_repo
            .update_status(order_uuid.clone(), target_status)
            .await?;
        Ok(build_schedule_view(updated, assignment))
    }
}

fn build_schedule_view(order: DomainOrder, assignment: Option<ScheduleAssignment>) -> Schedule {
    let assigned_user_uuid = assignment
        .as_ref()
        .map(|value| value.assigned_user_uuid.clone());
    let scheduled_start_at = assignment
        .as_ref()
        .map(|value| parse_utc_time_to_string(value.start_at));
    let scheduled_end_at = assignment
        .as_ref()
        .map(|value| parse_utc_time_to_string(value.end_at));
    let dispatch_note = assignment.as_ref().and_then(|value| value.notes.clone());
    let inserted_at = assignment
        .as_ref()
        .map(|value| parse_utc_time_to_string(value.inserted_at))
        .unwrap_or_else(|| parse_utc_time_to_string(order.inserted_at));
    let updated_at = assignment
        .as_ref()
        .map(|value| parse_utc_time_to_string(value.updated_at))
        .unwrap_or_else(|| parse_utc_time_to_string(order.updated_at));
    Schedule {
        order_uuid: order.uuid,
        order_status: order.status.as_str().to_string(),
        schedule_status: ScheduleStatus::from_order_status(&order.status)
            .as_str()
            .to_string(),
        customer_uuid: order.customer_uuid,
        assigned_user_uuid,
        scheduled_start_at,
        scheduled_end_at,
        dispatch_note,
        notes: order.notes,
        inserted_at,
        updated_at,
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
    parse_datetime(value)
        .ok_or_else(|| format!("invalid {}", field))
        .map(Some)
}
