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
        operator_uuid: Option<String>,
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
            .update_order_assignment(order_uuid, update, operator_uuid.clone())
            .await?;
        let next_status = OrderStatus::next_after_schedule_assignment(order.status);
        let updated = if next_status != updated.status {
            self.order_repo
                .update_order_status(
                    updated.uuid.clone(),
                    next_status.as_str().to_string(),
                    operator_uuid,
                )
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
        operator_uuid: Option<String>,
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
            .update_order_assignment(order_uuid, update, operator_uuid.clone())
            .await?;
        let next_status = OrderStatus::next_after_schedule_assignment(order.status);
        let updated = if next_status != updated.status {
            self.order_repo
                .update_order_status(
                    updated.uuid.clone(),
                    next_status.as_str().to_string(),
                    operator_uuid,
                )
                .await?
        } else {
            updated
        };
        Ok(build_schedule_view(updated, Some(assignment)))
    }

    pub async fn cancel_schedule(
        &self,
        order_uuid: String,
        operator_uuid: Option<String>,
    ) -> Result<Schedule, String> {
        self.update_schedule_status(
            order_uuid,
            UpdateScheduleStatus {
                status: "cancelled".to_string(),
            },
            operator_uuid,
        )
        .await
    }

    pub async fn update_schedule_status(
        &self,
        order_uuid: String,
        payload: UpdateScheduleStatus,
        operator_uuid: Option<String>,
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
            .update_order_status(
                order_uuid.clone(),
                target_order_status.as_str().to_string(),
                operator_uuid,
            )
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::crm::order::{
        Order as DomainOrder, OrderAssignmentUpdate, OrderDetailsUpdate, OrderRepository,
        OrderStatus, SettlementStatus,
    };
    use crate::domain::crm::schedule::{ScheduleAssignment, ScheduleRepository, ScheduleStatus};
    use chrono::{Duration, TimeZone};
    use std::collections::HashMap;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct MockOrderRepository {
        orders: Arc<Mutex<HashMap<String, DomainOrder>>>,
    }

    impl MockOrderRepository {
        fn with_order(order: DomainOrder) -> Self {
            let mut orders = HashMap::new();
            orders.insert(order.uuid.clone(), order);
            Self {
                orders: Arc::new(Mutex::new(orders)),
            }
        }
    }

    impl OrderRepository for MockOrderRepository {
        fn create_order(
            &self,
            order: DomainOrder,
            _operator_uuid: Option<String>,
        ) -> impl std::future::Future<Output = Result<DomainOrder, String>> + Send {
            let orders = self.orders.clone();
            async move {
                orders
                    .lock()
                    .expect("orders lock")
                    .insert(order.uuid.clone(), order.clone());
                Ok(order)
            }
        }

        fn find_order(
            &self,
            uuid: String,
        ) -> impl std::future::Future<Output = Result<Option<DomainOrder>, String>> + Send {
            let orders = self.orders.clone();
            async move { Ok(orders.lock().expect("orders lock").get(&uuid).cloned()) }
        }

        fn update_order_status(
            &self,
            uuid: String,
            status: String,
            _operator_uuid: Option<String>,
        ) -> impl std::future::Future<Output = Result<DomainOrder, String>> + Send {
            let orders = self.orders.clone();
            async move {
                let mut orders = orders.lock().expect("orders lock");
                let order = orders
                    .get_mut(&uuid)
                    .ok_or_else(|| format!("order {} not found", uuid))?;
                order.update_status(OrderStatus::parse(&status)?);
                Ok(order.clone())
            }
        }

        fn update_order_assignment(
            &self,
            uuid: String,
            update: OrderAssignmentUpdate,
            _operator_uuid: Option<String>,
        ) -> impl std::future::Future<Output = Result<DomainOrder, String>> + Send {
            let orders = self.orders.clone();
            async move {
                let mut orders = orders.lock().expect("orders lock");
                let order = orders
                    .get_mut(&uuid)
                    .ok_or_else(|| format!("order {} not found", uuid))?;
                order.update_assignment(update)?;
                Ok(order.clone())
            }
        }

        fn update_order_details(
            &self,
            uuid: String,
            update: OrderDetailsUpdate,
            _operator_uuid: Option<String>,
        ) -> impl std::future::Future<Output = Result<DomainOrder, String>> + Send {
            let orders = self.orders.clone();
            async move {
                let mut orders = orders.lock().expect("orders lock");
                let order = orders
                    .get_mut(&uuid)
                    .ok_or_else(|| format!("order {} not found", uuid))?;
                order.update_details(update)?;
                Ok(order.clone())
            }
        }

        fn update_order_settlement(
            &self,
            uuid: String,
            settlement_status: String,
            settlement_note: Option<String>,
            paid_amount_cents: Option<i64>,
            payment_method: Option<String>,
            paid_at: Option<DateTime<Utc>>,
            _operator_uuid: Option<String>,
        ) -> impl std::future::Future<Output = Result<DomainOrder, String>> + Send {
            let orders = self.orders.clone();
            async move {
                let mut orders = orders.lock().expect("orders lock");
                let order = orders
                    .get_mut(&uuid)
                    .ok_or_else(|| format!("order {} not found", uuid))?;
                order.update_settlement(
                    SettlementStatus::parse(&settlement_status)?,
                    settlement_note,
                    paid_amount_cents,
                    payment_method,
                    paid_at,
                )?;
                Ok(order.clone())
            }
        }

        fn cancel_order(
            &self,
            uuid: String,
            reason: String,
            _operator_uuid: Option<String>,
        ) -> impl std::future::Future<Output = Result<DomainOrder, String>> + Send {
            let orders = self.orders.clone();
            async move {
                let mut orders = orders.lock().expect("orders lock");
                let order = orders
                    .get_mut(&uuid)
                    .ok_or_else(|| format!("order {} not found", uuid))?;
                order.cancel(reason)?;
                Ok(order.clone())
            }
        }
    }

    #[derive(Clone, Default)]
    struct MockScheduleRepository {
        schedules: Arc<Mutex<HashMap<String, ScheduleAssignment>>>,
    }

    impl MockScheduleRepository {
        fn with_assignments(assignments: Vec<ScheduleAssignment>) -> Self {
            let schedules = assignments
                .into_iter()
                .map(|assignment| (assignment.order_uuid.clone(), assignment))
                .collect::<HashMap<_, _>>();
            Self {
                schedules: Arc::new(Mutex::new(schedules)),
            }
        }
    }

    impl ScheduleRepository for MockScheduleRepository {
        fn find_by_order(
            &self,
            order_uuid: String,
        ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send
        {
            let schedules = self.schedules.clone();
            async move {
                Ok(schedules
                    .lock()
                    .expect("schedules lock")
                    .get(&order_uuid)
                    .cloned())
            }
        }

        fn create_assignment(
            &self,
            assignment: ScheduleAssignment,
        ) -> impl std::future::Future<Output = Result<ScheduleAssignment, String>> + Send {
            let schedules = self.schedules.clone();
            async move {
                schedules
                    .lock()
                    .expect("schedules lock")
                    .insert(assignment.order_uuid.clone(), assignment.clone());
                Ok(assignment)
            }
        }

        fn update_assignment(
            &self,
            order_uuid: String,
            assigned_user_uuid: String,
            start_at: DateTime<Utc>,
            end_at: DateTime<Utc>,
            notes: Option<String>,
        ) -> impl std::future::Future<Output = Result<ScheduleAssignment, String>> + Send {
            let schedules = self.schedules.clone();
            async move {
                let mut schedules = schedules.lock().expect("schedules lock");
                let schedule = schedules
                    .get_mut(&order_uuid)
                    .ok_or_else(|| format!("schedule for order {} not found", order_uuid))?;
                schedule.assigned_user_uuid = assigned_user_uuid;
                schedule.start_at = start_at;
                schedule.end_at = end_at;
                schedule.notes = notes;
                schedule.updated_at = Utc::now();
                Ok(schedule.clone())
            }
        }

        fn delete_by_order(
            &self,
            order_uuid: String,
        ) -> impl std::future::Future<Output = Result<(), String>> + Send {
            let schedules = self.schedules.clone();
            async move {
                schedules
                    .lock()
                    .expect("schedules lock")
                    .remove(&order_uuid);
                Ok(())
            }
        }

        fn update_status(
            &self,
            order_uuid: String,
            status: ScheduleStatus,
        ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send
        {
            let schedules = self.schedules.clone();
            async move {
                let mut schedules = schedules.lock().expect("schedules lock");
                let Some(schedule) = schedules.get_mut(&order_uuid) else {
                    return Ok(None);
                };
                schedule.status = status;
                schedule.updated_at = Utc::now();
                Ok(Some(schedule.clone()))
            }
        }

        fn find_conflict(
            &self,
            assigned_user_uuid: String,
            start_at: DateTime<Utc>,
            end_at: DateTime<Utc>,
            exclude_order_uuid: Option<String>,
        ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send
        {
            let schedules = self.schedules.clone();
            async move {
                let conflict = schedules
                    .lock()
                    .expect("schedules lock")
                    .values()
                    .find(|assignment| {
                        assignment.assigned_user_uuid == assigned_user_uuid
                            && exclude_order_uuid.as_deref() != Some(assignment.order_uuid.as_str())
                            && crate::domain::crm::schedule::is_overlapping_window(
                                assignment.start_at,
                                assignment.end_at,
                                start_at,
                                end_at,
                            )
                    })
                    .cloned();
                Ok(conflict)
            }
        }
    }

    fn ts(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 4, 21, hour, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn sample_order(status: OrderStatus) -> DomainOrder {
        let mut order = DomainOrder::new_from_request(
            "request-1".to_string(),
            "customer-1".to_string(),
            Some(ts(9)),
            Some(ts(10)),
            Some("note".to_string()),
        );
        order.uuid = "order-1".to_string();
        order.status = status;
        if status == OrderStatus::Completed {
            order.completed_at = Some(ts(10));
        }
        order
    }

    fn sample_assignment(
        order_uuid: &str,
        user_uuid: &str,
        start: DateTime<Utc>,
    ) -> ScheduleAssignment {
        ScheduleAssignment {
            uuid: format!("schedule-{}", order_uuid),
            order_uuid: order_uuid.to_string(),
            assigned_user_uuid: user_uuid.to_string(),
            start_at: start,
            end_at: start + Duration::hours(1),
            status: ScheduleStatus::Planned,
            notes: Some("dispatch".to_string()),
            inserted_at: start,
            updated_at: start,
        }
    }

    #[tokio::test]
    async fn create_schedule_rejects_overlapping_assignments_for_same_worker() {
        let order_repo = MockOrderRepository::with_order(sample_order(OrderStatus::Pending));
        let schedule_repo = MockScheduleRepository::with_assignments(vec![sample_assignment(
            "order-2",
            "worker-1",
            ts(9),
        )]);
        let service = ScheduleAppService::new(order_repo, schedule_repo);

        let err = service
            .create_schedule(
                "order-1".to_string(),
                CreateScheduleAssignment {
                    assigned_user_uuid: "worker-1".to_string(),
                    scheduled_start_at: "2026-04-21T09:30:00Z".to_string(),
                    scheduled_end_at: "2026-04-21T10:30:00Z".to_string(),
                    dispatch_note: Some("conflict".to_string()),
                },
                Some("operator-1".to_string()),
            )
            .await
            .expect_err("conflicting assignment should be rejected");

        assert!(err.contains("overlaps"));
    }

    #[tokio::test]
    async fn update_schedule_status_rejects_invalid_transition_from_planned_to_done() {
        let order_repo = MockOrderRepository::with_order(sample_order(OrderStatus::Dispatching));
        let schedule_repo = MockScheduleRepository::with_assignments(vec![sample_assignment(
            "order-1",
            "worker-1",
            ts(9),
        )]);
        let service = ScheduleAppService::new(order_repo, schedule_repo);

        let err = service
            .update_schedule_status(
                "order-1".to_string(),
                UpdateScheduleStatus {
                    status: "done".to_string(),
                },
                Some("operator-1".to_string()),
            )
            .await
            .expect_err("planned -> done should be rejected");

        assert!(err.contains("Invalid schedule status transition: planned -> done"));
    }

    #[tokio::test]
    async fn update_schedule_status_allows_transition_from_planned_to_in_service() {
        let order_repo = MockOrderRepository::with_order(sample_order(OrderStatus::Dispatching));
        let schedule_repo = MockScheduleRepository::with_assignments(vec![sample_assignment(
            "order-1",
            "worker-1",
            ts(9),
        )]);
        let service = ScheduleAppService::new(order_repo.clone(), schedule_repo.clone());

        let updated = service
            .update_schedule_status(
                "order-1".to_string(),
                UpdateScheduleStatus {
                    status: "in_service".to_string(),
                },
                Some("operator-1".to_string()),
            )
            .await
            .expect("planned -> in_service should succeed");

        assert_eq!(updated.schedule_status, "in_service");
        assert_eq!(updated.order_status, "in_service");

        let order = order_repo
            .find_order("order-1".to_string())
            .await
            .expect("order query should succeed")
            .expect("order should exist");
        assert_eq!(order.status, OrderStatus::InService);

        let schedule = schedule_repo
            .find_by_order("order-1".to_string())
            .await
            .expect("schedule query should succeed")
            .expect("schedule should exist");
        assert_eq!(schedule.status, ScheduleStatus::InService);
    }

    #[tokio::test]
    async fn create_schedule_promotes_pending_order_to_dispatching() {
        let order_repo = MockOrderRepository::with_order(sample_order(OrderStatus::Pending));
        let schedule_repo = MockScheduleRepository::default();
        let service = ScheduleAppService::new(order_repo.clone(), schedule_repo.clone());

        let created = service
            .create_schedule(
                "order-1".to_string(),
                CreateScheduleAssignment {
                    assigned_user_uuid: "worker-1".to_string(),
                    scheduled_start_at: "2026-04-21T09:00:00Z".to_string(),
                    scheduled_end_at: "2026-04-21T10:00:00Z".to_string(),
                    dispatch_note: Some("dispatch".to_string()),
                },
                Some("operator-1".to_string()),
            )
            .await
            .expect("schedule should be created");

        assert_eq!(created.schedule_status, "planned");
        assert_eq!(created.order_status, "dispatching");

        let order = order_repo
            .find_order("order-1".to_string())
            .await
            .expect("order query should succeed")
            .expect("order should exist");
        assert_eq!(order.status, OrderStatus::Dispatching);

        let schedule = schedule_repo
            .find_by_order("order-1".to_string())
            .await
            .expect("schedule query should succeed")
            .expect("schedule should exist");
        assert_eq!(schedule.assigned_user_uuid, "worker-1");
    }
}
