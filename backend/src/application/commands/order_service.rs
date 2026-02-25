use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use crate::domain::models::order::{
    Order as DomainOrder, OrderAssignmentUpdate, OrderStatus, SettlementStatus,
};
use crate::domain::models::schedule::{ScheduleAssignment, ScheduleStatus, validate_time_window};
use crate::domain::repositories::order::OrderRepository;
use crate::domain::repositories::schedule::ScheduleRepository;
use crate::domain::repositories::service_request::ServiceRequestRepository;
use crate::domain::queries::service_request::ServiceRequestQuery as ServiceRequestQueryTrait;
use shared::order::{
    CreateOrderFromRequest, Order as SharedOrder, UpdateOrderAssignment, UpdateOrderSettlement,
    UpdateOrderStatus,
};
use shared::service_request::ServiceRequest;

pub struct OrderAppService<R, Q, SR, S>
where
    R: OrderRepository,
    Q: ServiceRequestQueryTrait<Result = ServiceRequest>,
    SR: ServiceRequestRepository,
    S: ScheduleRepository,
{
    order_repo: R,
    request_query: Q,
    request_repo: SR,
    schedule_repo: S,
}

impl<R, Q, SR, S> OrderAppService<R, Q, SR, S>
where
    R: OrderRepository,
    Q: ServiceRequestQueryTrait<Result = ServiceRequest>,
    SR: ServiceRequestRepository,
    S: ScheduleRepository,
{
    pub fn new(order_repo: R, request_query: Q, request_repo: SR, schedule_repo: S) -> Self {
        Self {
            order_repo,
            request_query,
            request_repo,
            schedule_repo,
        }
    }

    pub async fn create_from_request(
        &self,
        payload: CreateOrderFromRequest,
    ) -> Result<SharedOrder, String> {
        let request = self
            .request_query
            .get_request(payload.request_id.clone())
            .await?
            .ok_or_else(|| "service request not found".to_string())?;

        if request.status != "confirmed" {
            return Err("service request must be confirmed before creating order".to_string());
        }

        let order = DomainOrder::new_from_request(
            payload.request_id.clone(),
            request.customer_uuid.clone(),
            payload.notes,
        );
        order.verify()?;

        let created = self.order_repo.create_order(order).await?;

        let _ = self
            .request_repo
            .update_service_request_status(payload.request_id, "converted".to_string())
            .await?;

        Ok(created.into())
    }

    pub async fn update_status(
        &self,
        uuid: String,
        payload: UpdateOrderStatus,
    ) -> Result<SharedOrder, String> {
        OrderStatus::parse(&payload.status)?;
        let updated = self.order_repo.update_order_status(uuid, payload.status).await?;
        let schedule_status = ScheduleStatus::from_order_status(&updated.status);
        let _ = self
            .schedule_repo
            .update_status(updated.uuid.clone(), schedule_status)
            .await?;
        Ok(updated.into())
    }

    pub async fn update_assignment(
        &self,
        uuid: String,
        payload: UpdateOrderAssignment,
    ) -> Result<SharedOrder, String> {
        let assigned_user_uuid = payload
            .assigned_user_uuid
            .clone()
            .filter(|value| !value.trim().is_empty())
            .ok_or_else(|| "assigned_user_uuid is required".to_string())?;
        let start = parse_datetime(payload.scheduled_start_at.as_deref())
            .ok_or_else(|| "scheduled_start_at is required".to_string())?;
        let end = parse_datetime(payload.scheduled_end_at.as_deref())
            .ok_or_else(|| "scheduled_end_at is required".to_string())?;
        validate_time_window(start, end)?;

        let order = self
            .order_repo
            .find_order(uuid.clone())
            .await?
            .ok_or_else(|| format!("order {} not found", uuid))?;
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
                Some(uuid.clone()),
            )
            .await?
        {
            return Err(format!(
                "schedule time overlaps with existing assignment {}",
                conflict.order_id
            ));
        }

        match self.schedule_repo.find_by_order(uuid.clone()).await? {
            Some(_) => {
                self.schedule_repo
                    .update_assignment(
                        uuid.clone(),
                        assigned_user_uuid.clone(),
                        start.clone(),
                        end.clone(),
                        payload.dispatch_note.clone(),
                    )
                    .await?;
            }
            None => {
                let schedule_status = ScheduleStatus::from_order_status(&order.status);
                let assignment = ScheduleAssignment {
                    uuid: String::new(),
                    order_id: uuid.clone(),
                    assigned_user_uuid: assigned_user_uuid.clone(),
                    start_at: start.clone(),
                    end_at: end.clone(),
                    status: schedule_status,
                    notes: payload.dispatch_note.clone(),
                    inserted_at: Utc::now(),
                    updated_at: Utc::now(),
                };
                self.schedule_repo.create_assignment(assignment).await?;
            }
        }

        let update = OrderAssignmentUpdate {
            assigned_user_uuid: None,
            scheduled_start_at: Some(start),
            scheduled_end_at: Some(end),
            dispatch_note: payload.dispatch_note,
        };
        let updated = self.order_repo.update_order_assignment(uuid, update).await?;
        Ok(updated.into())
    }

    pub async fn update_settlement(
        &self,
        uuid: String,
        payload: UpdateOrderSettlement,
    ) -> Result<SharedOrder, String> {
        SettlementStatus::parse(&payload.settlement_status)?;
        let updated = self
            .order_repo
            .update_order_settlement(uuid, payload.settlement_status, payload.settlement_note)
            .await?;
        Ok(updated.into())
    }
}

fn parse_datetime(value: Option<&str>) -> Option<DateTime<Utc>> {
    let Some(value) = value else { return None };
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
