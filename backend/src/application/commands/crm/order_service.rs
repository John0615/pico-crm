use crate::domain::crm::order::{
    Order as DomainOrder, OrderAssignmentUpdate, OrderRepository, OrderStatus, SettlementStatus,
};
use crate::domain::crm::schedule::{
    ScheduleAssignment, ScheduleRepository, ScheduleStatus, validate_time_window,
};
use crate::domain::crm::service_request::{
    ServiceRequestQuery as ServiceRequestQueryTrait, ServiceRequestRepository,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
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
            parse_datetime(request.appointment_start_at.as_deref()),
            parse_datetime(request.appointment_end_at.as_deref()),
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
        let updated = self
            .order_repo
            .update_order_status(uuid, payload.status)
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
                conflict.order_uuid
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
                    order_uuid: uuid.clone(),
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
        let updated = self
            .order_repo
            .update_order_assignment(uuid, update)
            .await?;
        let next_status = OrderStatus::next_after_schedule_assignment(order.status);
        let updated = if next_status != updated.status {
            self.order_repo
                .update_order_status(updated.uuid.clone(), next_status.as_str().to_string())
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::utils::parse_utc_time_to_string;
    use crate::domain::crm::order::{Order, OrderAssignmentUpdate, OrderRepository};
    use crate::domain::crm::schedule::{ScheduleAssignment, ScheduleRepository, ScheduleStatus};
    use crate::domain::crm::service_request::{
        ServiceRequestQuery as ServiceRequestQueryTrait, ServiceRequestRepository,
        UpdateServiceRequest,
    };
    use chrono::TimeZone;
    use shared::service_request::ServiceRequest as SharedServiceRequest;
    use std::sync::{Arc, Mutex};

    struct FakeOrderRepository {
        created: Arc<Mutex<Option<Order>>>,
    }

    impl OrderRepository for FakeOrderRepository {
        fn create_order(
            &self,
            order: Order,
        ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
            let created = self.created.clone();
            async move {
                *created.lock().expect("lock created order") = Some(order.clone());
                Ok(order)
            }
        }

        fn find_order(
            &self,
            _uuid: String,
        ) -> impl std::future::Future<Output = Result<Option<Order>, String>> + Send {
            async { unreachable!("find_order is not used in create_from_request test") }
        }

        fn update_order_status(
            &self,
            _uuid: String,
            _status: String,
        ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
            async { unreachable!("update_order_status is not used in create_from_request test") }
        }

        fn update_order_assignment(
            &self,
            _uuid: String,
            _update: OrderAssignmentUpdate,
        ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
            async {
                unreachable!("update_order_assignment is not used in create_from_request test")
            }
        }

        fn update_order_settlement(
            &self,
            _uuid: String,
            _settlement_status: String,
            _settlement_note: Option<String>,
        ) -> impl std::future::Future<Output = Result<Order, String>> + Send {
            async {
                unreachable!("update_order_settlement is not used in create_from_request test")
            }
        }
    }

    struct FakeRequestQuery {
        request: SharedServiceRequest,
    }

    impl ServiceRequestQueryTrait for FakeRequestQuery {
        type Result = SharedServiceRequest;

        fn list_requests(
            &self,
            _query: shared::service_request::ServiceRequestQuery,
        ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send
        {
            async { unreachable!("list_requests is not used in create_from_request test") }
        }

        fn get_request(
            &self,
            _uuid: String,
        ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send
        {
            let request = self.request.clone();
            async move { Ok(Some(request)) }
        }
    }

    struct FakeRequestRepository {
        status_updates: Arc<Mutex<Vec<(String, String)>>>,
    }

    impl ServiceRequestRepository for FakeRequestRepository {
        fn create_service_request(
            &self,
            _request: crate::domain::crm::service_request::ServiceRequest,
        ) -> impl std::future::Future<Output = Result<String, String>> + Send {
            async { unreachable!("create_service_request is not used in create_from_request test") }
        }

        fn update_service_request(
            &self,
            _request: UpdateServiceRequest,
        ) -> impl std::future::Future<Output = Result<(), String>> + Send {
            async { unreachable!("update_service_request is not used in create_from_request test") }
        }

        fn update_service_request_status(
            &self,
            uuid: String,
            status: String,
        ) -> impl std::future::Future<Output = Result<(), String>> + Send {
            let status_updates = self.status_updates.clone();
            async move {
                status_updates
                    .lock()
                    .expect("lock status updates")
                    .push((uuid, status));
                Ok(())
            }
        }
    }

    struct FakeScheduleRepository;

    impl ScheduleRepository for FakeScheduleRepository {
        fn find_by_order(
            &self,
            _order_uuid: String,
        ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send
        {
            async { Ok(None) }
        }

        fn create_assignment(
            &self,
            assignment: ScheduleAssignment,
        ) -> impl std::future::Future<Output = Result<ScheduleAssignment, String>> + Send {
            async move { Ok(assignment) }
        }

        fn update_assignment(
            &self,
            _order_uuid: String,
            _assigned_user_uuid: String,
            _start_at: DateTime<Utc>,
            _end_at: DateTime<Utc>,
            _notes: Option<String>,
        ) -> impl std::future::Future<Output = Result<ScheduleAssignment, String>> + Send {
            async { unreachable!("update_assignment is not used in create_from_request test") }
        }

        fn delete_by_order(
            &self,
            _order_uuid: String,
        ) -> impl std::future::Future<Output = Result<(), String>> + Send {
            async { Ok(()) }
        }

        fn update_status(
            &self,
            _order_uuid: String,
            _status: ScheduleStatus,
        ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send
        {
            async { Ok(None) }
        }

        fn find_conflict(
            &self,
            _assigned_user_uuid: String,
            _start_at: DateTime<Utc>,
            _end_at: DateTime<Utc>,
            _exclude_order_uuid: Option<String>,
        ) -> impl std::future::Future<Output = Result<Option<ScheduleAssignment>, String>> + Send
        {
            async { Ok(None) }
        }
    }

    #[tokio::test]
    async fn create_from_request_copies_appointment_window_to_order() {
        let created = Arc::new(Mutex::new(None));
        let status_updates = Arc::new(Mutex::new(Vec::new()));
        let start = Utc.with_ymd_and_hms(2026, 4, 15, 9, 0, 0).unwrap();
        let end = Utc.with_ymd_and_hms(2026, 4, 15, 11, 0, 0).unwrap();
        let expected_start = start.to_rfc3339();
        let expected_end = end.to_rfc3339();
        let expected_start_display = parse_utc_time_to_string(start);
        let expected_end_display = parse_utc_time_to_string(end);

        let service = OrderAppService::new(
            FakeOrderRepository {
                created: created.clone(),
            },
            FakeRequestQuery {
                request: SharedServiceRequest {
                    uuid: "11111111-1111-1111-1111-111111111111".to_string(),
                    customer_uuid: "22222222-2222-2222-2222-222222222222".to_string(),
                    creator_uuid: "33333333-3333-3333-3333-333333333333".to_string(),
                    contact_name: None,
                    creator_name: None,
                    service_content: "深度保洁".to_string(),
                    appointment_start_at: Some(expected_start.clone()),
                    appointment_end_at: Some(expected_end.clone()),
                    status: "confirmed".to_string(),
                    source: "sales_manual".to_string(),
                    notes: Some("客户希望上午上门".to_string()),
                    inserted_at: expected_start.clone(),
                    updated_at: expected_start.clone(),
                },
            },
            FakeRequestRepository {
                status_updates: status_updates.clone(),
            },
            FakeScheduleRepository,
        );

        let created_order = service
            .create_from_request(CreateOrderFromRequest {
                request_id: "11111111-1111-1111-1111-111111111111".to_string(),
                notes: Some("转订单".to_string()),
            })
            .await
            .expect("create order from request");

        let stored = created
            .lock()
            .expect("lock created order")
            .clone()
            .expect("created order should be recorded");
        assert_eq!(
            stored.request_id.as_deref(),
            Some("11111111-1111-1111-1111-111111111111")
        );
        assert_eq!(
            stored.customer_uuid.as_deref(),
            Some("22222222-2222-2222-2222-222222222222")
        );
        assert_eq!(stored.scheduled_start_at, Some(start));
        assert_eq!(stored.scheduled_end_at, Some(end));
        assert_eq!(stored.status, OrderStatus::Pending);
        assert_eq!(created_order.status, "pending");
        assert_eq!(
            created_order.scheduled_start_at.as_deref(),
            Some(expected_start_display.as_str())
        );
        assert_eq!(
            created_order.scheduled_end_at.as_deref(),
            Some(expected_end_display.as_str())
        );

        assert_eq!(
            status_updates
                .lock()
                .expect("lock status updates")
                .as_slice(),
            &[(
                "11111111-1111-1111-1111-111111111111".to_string(),
                "converted".to_string(),
            )]
        );
    }
}
