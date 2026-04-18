use chrono::{DateTime, Utc};
use disintegrate::Decision;

use super::events::{seed_created_event, OrderEventEnvelope};
use super::state::OrderState;
use crate::domain::crm::order::{
    Order, OrderAssignmentUpdate, OrderDetailsUpdate, OrderStatus, SettlementStatus,
};
use crate::domain::crm::schedule::{validate_time_window, ScheduleStatus};

pub struct CreateOrderDecision {
    tenant_schema: String,
    order: Order,
    operator_uuid: Option<String>,
}

impl CreateOrderDecision {
    pub fn new(
        tenant_schema: impl Into<String>,
        order: Order,
        operator_uuid: Option<String>,
    ) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            order,
            operator_uuid,
        }
    }
}

impl Decision for CreateOrderDecision {
    type Event = OrderEventEnvelope;
    type StateQuery = OrderState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        OrderState::new(&self.tenant_schema, &self.order.uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if state.exists {
            return Err(format!("order {} already exists", self.order.uuid));
        }

        self.order.verify()?;

        Ok(vec![seed_created_event(
            &self.tenant_schema,
            &self.order,
            self.operator_uuid.clone(),
        )])
    }
}

pub struct UpdateOrderDetailsDecision {
    tenant_schema: String,
    order_uuid: String,
    update: OrderDetailsUpdate,
    updated_at: DateTime<Utc>,
    operator_uuid: Option<String>,
}

impl UpdateOrderDetailsDecision {
    pub fn new(
        tenant_schema: impl Into<String>,
        order_uuid: impl Into<String>,
        update: OrderDetailsUpdate,
        updated_at: DateTime<Utc>,
        operator_uuid: Option<String>,
    ) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            order_uuid: order_uuid.into(),
            update,
            updated_at,
            operator_uuid,
        }
    }
}

impl Decision for UpdateOrderDetailsDecision {
    type Event = OrderEventEnvelope;
    type StateQuery = OrderState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        OrderState::new(&self.tenant_schema, &self.order_uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        let mut order = state.to_domain()?;
        order.update_details(self.update.clone())?;

        Ok(vec![OrderEventEnvelope::OrderDetailsUpdated {
            tenant_schema: self.tenant_schema.clone(),
            order_uuid: self.order_uuid.clone(),
            operator_uuid: self.operator_uuid.clone(),
            customer_uuid: order.customer_uuid.clone(),
            amount_cents: order.amount_cents,
            notes: order.notes.clone(),
            updated_at: self.updated_at,
        }])
    }
}

pub struct UpdateOrderStatusDecision {
    tenant_schema: String,
    order_uuid: String,
    next_status: OrderStatus,
    updated_at: DateTime<Utc>,
    operator_uuid: Option<String>,
}

impl UpdateOrderStatusDecision {
    pub fn new(
        tenant_schema: impl Into<String>,
        order_uuid: impl Into<String>,
        next_status: OrderStatus,
        updated_at: DateTime<Utc>,
        operator_uuid: Option<String>,
    ) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            order_uuid: order_uuid.into(),
            next_status,
            updated_at,
            operator_uuid,
        }
    }
}

impl Decision for UpdateOrderStatusDecision {
    type Event = OrderEventEnvelope;
    type StateQuery = OrderState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        OrderState::new(&self.tenant_schema, &self.order_uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state.exists {
            return Err(format!("order {} not found", self.order_uuid));
        }

        if self.next_status == OrderStatus::Cancelled {
            return Err("use cancel order flow when changing to cancelled".to_string());
        }

        let current_status = OrderStatus::parse(
            state
                .status
                .as_deref()
                .unwrap_or(OrderStatus::Pending.as_str()),
        )?;
        OrderStatus::validate_transition(current_status, self.next_status)?;

        let completed_at = if self.next_status == OrderStatus::Completed {
            Some(self.updated_at)
        } else {
            None
        };

        Ok(vec![OrderEventEnvelope::OrderStatusChanged {
            tenant_schema: self.tenant_schema.clone(),
            order_uuid: self.order_uuid.clone(),
            operator_uuid: self.operator_uuid.clone(),
            status: self.next_status.as_str().to_string(),
            completed_at,
            updated_at: self.updated_at,
        }])
    }
}

pub struct CancelOrderDecision {
    tenant_schema: String,
    order_uuid: String,
    reason: String,
    updated_at: DateTime<Utc>,
    operator_uuid: Option<String>,
}

impl CancelOrderDecision {
    pub fn new(
        tenant_schema: impl Into<String>,
        order_uuid: impl Into<String>,
        reason: impl Into<String>,
        updated_at: DateTime<Utc>,
        operator_uuid: Option<String>,
    ) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            order_uuid: order_uuid.into(),
            reason: reason.into(),
            updated_at,
            operator_uuid,
        }
    }
}

impl Decision for CancelOrderDecision {
    type Event = OrderEventEnvelope;
    type StateQuery = OrderState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        OrderState::new(&self.tenant_schema, &self.order_uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        let mut order = state.to_domain()?;
        order.cancel(self.reason.clone())?;

        Ok(vec![OrderEventEnvelope::OrderCancelled {
            tenant_schema: self.tenant_schema.clone(),
            order_uuid: self.order_uuid.clone(),
            operator_uuid: self.operator_uuid.clone(),
            cancellation_reason: order.cancellation_reason.unwrap_or_default(),
            updated_at: self.updated_at,
        }])
    }
}

pub struct UpdateOrderAssignmentDecision {
    tenant_schema: String,
    order_uuid: String,
    update: OrderAssignmentUpdate,
    updated_at: DateTime<Utc>,
    operator_uuid: Option<String>,
}

impl UpdateOrderAssignmentDecision {
    pub fn new(
        tenant_schema: impl Into<String>,
        order_uuid: impl Into<String>,
        update: OrderAssignmentUpdate,
        updated_at: DateTime<Utc>,
        operator_uuid: Option<String>,
    ) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            order_uuid: order_uuid.into(),
            update,
            updated_at,
            operator_uuid,
        }
    }
}

impl Decision for UpdateOrderAssignmentDecision {
    type Event = OrderEventEnvelope;
    type StateQuery = OrderState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        OrderState::new(&self.tenant_schema, &self.order_uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state.exists {
            return Err(format!("order {} not found", self.order_uuid));
        }

        let current_status = OrderStatus::parse(
            state
                .status
                .as_deref()
                .unwrap_or(OrderStatus::Pending.as_str()),
        )?;
        let schedule_status = ScheduleStatus::from_order_status(&current_status);
        if !schedule_status.allows_assignment_update() {
            return Err(format!(
                "schedule assignment can only be updated in planned status (current: {})",
                schedule_status.as_str()
            ));
        }

        if let (Some(start), Some(end)) =
            (self.update.scheduled_start_at, self.update.scheduled_end_at)
        {
            validate_time_window(start, end)?;
        }

        Ok(vec![OrderEventEnvelope::OrderAssignmentUpdated {
            tenant_schema: self.tenant_schema.clone(),
            order_uuid: self.order_uuid.clone(),
            operator_uuid: self.operator_uuid.clone(),
            scheduled_start_at: self.update.scheduled_start_at,
            scheduled_end_at: self.update.scheduled_end_at,
            dispatch_note: self.update.dispatch_note.clone(),
            updated_at: self.updated_at,
        }])
    }
}

pub struct UpdateOrderSettlementDecision {
    tenant_schema: String,
    order_uuid: String,
    settlement_status: SettlementStatus,
    settlement_note: Option<String>,
    updated_at: DateTime<Utc>,
    operator_uuid: Option<String>,
}

impl UpdateOrderSettlementDecision {
    pub fn new(
        tenant_schema: impl Into<String>,
        order_uuid: impl Into<String>,
        settlement_status: SettlementStatus,
        settlement_note: Option<String>,
        updated_at: DateTime<Utc>,
        operator_uuid: Option<String>,
    ) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            order_uuid: order_uuid.into(),
            settlement_status,
            settlement_note,
            updated_at,
            operator_uuid,
        }
    }
}

impl Decision for UpdateOrderSettlementDecision {
    type Event = OrderEventEnvelope;
    type StateQuery = OrderState;
    type Error = String;

    fn state_query(&self) -> Self::StateQuery {
        OrderState::new(&self.tenant_schema, &self.order_uuid)
    }

    fn process(&self, state: &Self::StateQuery) -> Result<Vec<Self::Event>, Self::Error> {
        if !state.exists {
            return Err(format!("order {} not found", self.order_uuid));
        }

        Ok(vec![OrderEventEnvelope::OrderSettlementUpdated {
            tenant_schema: self.tenant_schema.clone(),
            order_uuid: self.order_uuid.clone(),
            operator_uuid: self.operator_uuid.clone(),
            settlement_status: self.settlement_status.as_str().to_string(),
            settlement_note: self.settlement_note.clone(),
            updated_at: self.updated_at,
        }])
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use disintegrate::TestHarness;

    use super::*;

    fn ts(day: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 3, day, 10, 0, 0)
            .single()
            .expect("valid timestamp")
    }

    fn sample_order() -> Order {
        Order {
            uuid: "order-1".to_string(),
            request_id: Some("request-1".to_string()),
            customer_uuid: Some("customer-1".to_string()),
            scheduled_start_at: None,
            scheduled_end_at: None,
            status: OrderStatus::Pending,
            cancellation_reason: None,
            completed_at: None,
            settlement_status: SettlementStatus::Unsettled,
            amount_cents: 0,
            notes: Some("new".to_string()),
            dispatch_note: None,
            settlement_note: None,
            inserted_at: ts(1),
            updated_at: ts(1),
        }
    }

    #[test]
    fn it_creates_an_order() {
        let order = sample_order();

        TestHarness::given([])
            .when(CreateOrderDecision::new("tenant_a", order.clone(), None))
            .then([seed_created_event("tenant_a", &order, None)]);
    }

    #[test]
    fn it_updates_order_status() {
        TestHarness::given([seed_created_event("tenant_a", &sample_order(), None)])
            .when(UpdateOrderStatusDecision::new(
                "tenant_a",
                "order-1",
                OrderStatus::Confirmed,
                ts(2),
                None,
            ))
            .then([OrderEventEnvelope::OrderStatusChanged {
                tenant_schema: "tenant_a".to_string(),
                order_uuid: "order-1".to_string(),
                operator_uuid: None,
                status: "confirmed".to_string(),
                completed_at: None,
                updated_at: ts(2),
            }]);
    }

    #[test]
    fn it_cancels_order_with_reason() {
        TestHarness::given([seed_created_event("tenant_a", &sample_order(), None)])
            .when(CancelOrderDecision::new(
                "tenant_a",
                "order-1",
                "客户改期".to_string(),
                ts(2),
                None,
            ))
            .then([OrderEventEnvelope::OrderCancelled {
                tenant_schema: "tenant_a".to_string(),
                order_uuid: "order-1".to_string(),
                operator_uuid: None,
                cancellation_reason: "客户改期".to_string(),
                updated_at: ts(2),
            }]);
    }

    #[test]
    fn it_rejects_assignment_updates_for_completed_orders() {
        let completed = OrderEventEnvelope::OrderStatusChanged {
            tenant_schema: "tenant_a".to_string(),
            order_uuid: "order-1".to_string(),
            operator_uuid: None,
            status: OrderStatus::Completed.as_str().to_string(),
            completed_at: Some(ts(2)),
            updated_at: ts(2),
        };

        TestHarness::given([seed_created_event("tenant_a", &sample_order(), None), completed])
            .when(UpdateOrderAssignmentDecision::new(
                "tenant_a",
                "order-1",
                OrderAssignmentUpdate {
                    assigned_user_uuid: Some("user-1".to_string()),
                    scheduled_start_at: Some(ts(3)),
                    scheduled_end_at: Some(ts(3) + chrono::Duration::hours(1)),
                    dispatch_note: Some("dispatch".to_string()),
                },
                ts(3),
                None,
            ))
            .then_err(
                "schedule assignment can only be updated in planned status (current: done)"
                    .to_string(),
            );
    }

    #[test]
    fn it_rejects_invalid_status_transition() {
        TestHarness::given([seed_created_event("tenant_a", &sample_order(), None)])
            .when(UpdateOrderStatusDecision::new(
                "tenant_a",
                "order-1",
                OrderStatus::Completed,
                ts(2),
                None,
            ))
            .then_err("invalid order status transition: pending -> completed".to_string());
    }
}
