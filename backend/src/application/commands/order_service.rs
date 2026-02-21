use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use crate::domain::models::order::{
    Order as DomainOrder, OrderAssignmentUpdate, OrderStatus, SettlementStatus,
};
use crate::domain::repositories::order::OrderRepository;
use crate::domain::repositories::service_request::ServiceRequestRepository;
use crate::domain::queries::service_request::ServiceRequestQuery as ServiceRequestQueryTrait;
use shared::order::{
    CreateOrderFromRequest, Order as SharedOrder, UpdateOrderAssignment, UpdateOrderSettlement,
    UpdateOrderStatus,
};
use shared::service_request::ServiceRequest;

pub struct OrderAppService<R, Q, SR>
where
    R: OrderRepository,
    Q: ServiceRequestQueryTrait<Result = ServiceRequest>,
    SR: ServiceRequestRepository,
{
    order_repo: R,
    request_query: Q,
    request_repo: SR,
}

impl<R, Q, SR> OrderAppService<R, Q, SR>
where
    R: OrderRepository,
    Q: ServiceRequestQueryTrait<Result = ServiceRequest>,
    SR: ServiceRequestRepository,
{
    pub fn new(order_repo: R, request_query: Q, request_repo: SR) -> Self {
        Self {
            order_repo,
            request_query,
            request_repo,
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
            request.contact_uuid.clone(),
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
        Ok(updated.into())
    }

    pub async fn update_assignment(
        &self,
        uuid: String,
        payload: UpdateOrderAssignment,
    ) -> Result<SharedOrder, String> {
        let update = OrderAssignmentUpdate {
            assigned_user_uuid: payload.assigned_user_uuid,
            scheduled_start_at: parse_datetime(payload.scheduled_start_at.as_deref()),
            scheduled_end_at: parse_datetime(payload.scheduled_end_at.as_deref()),
            dispatch_note: payload.dispatch_note,
        };
        if let (Some(start), Some(end)) = (update.scheduled_start_at, update.scheduled_end_at) {
            if end < start {
                return Err("scheduled end must be after start".to_string());
            }
        }
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
