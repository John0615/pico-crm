use crate::domain::crm::order::{
    Order as DomainOrder, OrderAssignmentUpdate, OrderDetailsUpdate, OrderRepository, OrderStatus,
    SettlementStatus,
};
use crate::domain::crm::schedule::{
    ScheduleAssignment, ScheduleRepository, ScheduleStatus, validate_time_window,
};
use crate::domain::crm::service_catalog::ServiceCatalogQuery as ServiceCatalogQueryTrait;
use crate::domain::crm::service_request::{
    ServiceRequestQuery as ServiceRequestQueryTrait, ServiceRequestRepository,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use shared::order::{
    CancelOrderRequest, CreateOrderFromRequest, Order as SharedOrder, UpdateOrderAssignment,
    UpdateOrderRequest, UpdateOrderSettlement, UpdateOrderStatus,
};
use shared::service_request::ServiceRequest;

pub struct OrderAppService<R, Q, SR, S, C>
where
    R: OrderRepository,
    Q: ServiceRequestQueryTrait<Result = ServiceRequest>,
    SR: ServiceRequestRepository,
    S: ScheduleRepository,
    C: ServiceCatalogQueryTrait<Result = shared::service_catalog::ServiceCatalog>,
{
    order_repo: R,
    request_query: Q,
    request_repo: SR,
    schedule_repo: S,
    service_catalog_query: C,
}

impl<R, Q, SR, S, C> OrderAppService<R, Q, SR, S, C>
where
    R: OrderRepository,
    Q: ServiceRequestQueryTrait<Result = ServiceRequest>,
    SR: ServiceRequestRepository,
    S: ScheduleRepository,
    C: ServiceCatalogQueryTrait<Result = shared::service_catalog::ServiceCatalog>,
{
    pub fn new(
        order_repo: R,
        request_query: Q,
        request_repo: SR,
        schedule_repo: S,
        service_catalog_query: C,
    ) -> Self {
        Self {
            order_repo,
            request_query,
            request_repo,
            schedule_repo,
            service_catalog_query,
        }
    }

    pub async fn create_from_request(
        &self,
        payload: CreateOrderFromRequest,
        operator_uuid: Option<String>,
    ) -> Result<SharedOrder, String> {
        let request = self
            .request_query
            .get_request(payload.request_id.clone())
            .await?
            .ok_or_else(|| "service request not found".to_string())?;

        if request.status != "confirmed" {
            return Err("service request must be confirmed before creating order".to_string());
        }

        let mut order = DomainOrder::new_from_request(
            payload.request_id.clone(),
            request.customer_uuid.clone(),
            parse_datetime(request.appointment_start_at.as_deref()),
            parse_datetime(request.appointment_end_at.as_deref()),
            payload.notes,
        );
        if let Some(service_catalog_uuid) = request.service_catalog_uuid.clone() {
            if let Some(service_catalog) = self
                .service_catalog_query
                .get_service_catalog(service_catalog_uuid)
                .await?
            {
                order.amount_cents = service_catalog.base_price_cents;
            }
        }
        order.verify()?;

        let created = self.order_repo.create_order(order, operator_uuid).await?;

        let _ = self
            .request_repo
            .update_service_request_status(payload.request_id, "converted".to_string())
            .await?;

        Ok(created.into())
    }

    pub async fn update_order(
        &self,
        uuid: String,
        payload: UpdateOrderRequest,
        operator_uuid: Option<String>,
    ) -> Result<SharedOrder, String> {
        let update = OrderDetailsUpdate {
            customer_uuid: payload.customer_uuid.trim().to_string(),
            amount_cents: payload.amount_cents,
            notes: payload.notes,
        };
        let updated = self
            .order_repo
            .update_order_details(uuid, update, operator_uuid)
            .await?;
        Ok(updated.into())
    }

    pub async fn cancel_order(
        &self,
        uuid: String,
        payload: CancelOrderRequest,
        operator_uuid: Option<String>,
    ) -> Result<SharedOrder, String> {
        let updated = self
            .order_repo
            .cancel_order(uuid.clone(), payload.reason, operator_uuid.clone())
            .await?;
        let _ = self
            .schedule_repo
            .update_status(uuid, ScheduleStatus::Cancelled)
            .await?;
        Ok(updated.into())
    }

    pub async fn update_status(
        &self,
        uuid: String,
        payload: UpdateOrderStatus,
        operator_uuid: Option<String>,
    ) -> Result<SharedOrder, String> {
        if payload.status == "cancelled" {
            return Err("use cancel order endpoint when cancelling order".to_string());
        }

        OrderStatus::parse(&payload.status)?;
        let updated = self
            .order_repo
            .update_order_status(uuid, payload.status, operator_uuid)
            .await?;
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
        operator_uuid: Option<String>,
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
            .find_conflict(assigned_user_uuid.clone(), start, end, Some(uuid.clone()))
            .await?
        {
            return Err(format!(
                "schedule time overlaps with existing assignment {}",
                conflict.order_uuid
            ));
        }

        match self.schedule_repo.find_by_order(uuid.clone()).await? {
            Some(_) => {
                self.schedule_repo
                    .update_assignment(
                        uuid.clone(),
                        assigned_user_uuid.clone(),
                        start,
                        end,
                        payload.dispatch_note.clone(),
                    )
                    .await?;
            }
            None => {
                let schedule_status = ScheduleStatus::from_order_status(&order.status);
                let assignment = ScheduleAssignment {
                    uuid: String::new(),
                    order_uuid: uuid.clone(),
                    assigned_user_uuid: assigned_user_uuid.clone(),
                    start_at: start,
                    end_at: end,
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
        let updated = self
            .order_repo
            .update_order_assignment(uuid, update, operator_uuid.clone())
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
        Ok(updated.into())
    }

    pub async fn update_settlement(
        &self,
        uuid: String,
        payload: UpdateOrderSettlement,
        operator_uuid: Option<String>,
    ) -> Result<SharedOrder, String> {
        SettlementStatus::parse(&payload.settlement_status)?;
        if let Some(amount) = payload.paid_amount_cents {
            if amount < 0 {
                return Err("paid_amount_cents must be non-negative".to_string());
            }
        }
        let updated = self
            .order_repo
            .update_order_settlement(
                uuid,
                payload.settlement_status,
                payload.settlement_note,
                payload.paid_amount_cents,
                normalize_optional(payload.payment_method),
                parse_optional_datetime(payload.paid_at.as_deref(), "paid_at")?,
                operator_uuid,
            )
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
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
            return Some(Utc.from_utc_datetime(&dt));
        }
    }
    None
}

fn normalize_optional(value: Option<String>) -> Option<String> {
    value.and_then(|raw| {
        let trimmed = raw.trim().to_string();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed)
        }
    })
}

fn parse_optional_datetime(
    value: Option<&str>,
    field: &str,
) -> Result<Option<DateTime<Utc>>, String> {
    let Some(value) = value else { return Ok(None) };
    if value.trim().is_empty() {
        return Ok(None);
    }
    parse_datetime(Some(value))
        .ok_or_else(|| format!("invalid {}", field))
        .map(Some)
}
