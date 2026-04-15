use chrono::{DateTime, Utc};
use disintegrate::Decision;

use super::events::{OrderEventEnvelope, seed_created_event};
use super::state::OrderState;
use crate::domain::crm::order::{Order, OrderAssignmentUpdate, OrderStatus, SettlementStatus};
use crate::domain::crm::schedule::{ScheduleStatus, validate_time_window};

pub struct CreateOrderDecision {
    tenant_schema: String,
    order: Order,
}

impl CreateOrderDecision {
    pub fn new(tenant_schema: impl Into<String>, order: Order) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            order,
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

        Ok(vec![seed_created_event(&self.tenant_schema, &self.order)])
    }
}

pub struct UpdateOrderStatusDecision {
    tenant_schema: String,
    order_uuid: String,
    next_status: OrderStatus,
    updated_at: DateTime<Utc>,
}

impl UpdateOrderStatusDecision {
    pub fn new(
        tenant_schema: impl Into<String>,
        order_uuid: impl Into<String>,
        next_status: OrderStatus,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            order_uuid: order_uuid.into(),
            next_status,
            updated_at,
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

        let current_status = OrderStatus::parse(
            state
                .status
                .as_deref()
                .unwrap_or(OrderStatus::Pending.as_str()),
        )?;
        OrderStatus::validate_transition(current_status, self.next_status)?;

        Ok(vec![OrderEventEnvelope::OrderStatusChanged {
            tenant_schema: self.tenant_schema.clone(),
            order_uuid: self.order_uuid.clone(),
            status: self.next_status.as_str().to_string(),
            updated_at: self.updated_at,
        }])
    }
}

pub struct UpdateOrderAssignmentDecision {
    tenant_schema: String,
    order_uuid: String,
    update: OrderAssignmentUpdate,
    updated_at: DateTime<Utc>,
}

impl UpdateOrderAssignmentDecision {
    pub fn new(
        tenant_schema: impl Into<String>,
        order_uuid: impl Into<String>,
        update: OrderAssignmentUpdate,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            order_uuid: order_uuid.into(),
            update,
            updated_at,
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
}

impl UpdateOrderSettlementDecision {
    pub fn new(
        tenant_schema: impl Into<String>,
        order_uuid: impl Into<String>,
        settlement_status: SettlementStatus,
        settlement_note: Option<String>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            order_uuid: order_uuid.into(),
            settlement_status,
            settlement_note,
            updated_at,
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
            .when(CreateOrderDecision::new("tenant_a", order.clone()))
            .then([seed_created_event("tenant_a", &order)]);
    }

    #[test]
    fn it_updates_order_status() {
        TestHarness::given([seed_created_event("tenant_a", &sample_order())])
            .when(UpdateOrderStatusDecision::new(
                "tenant_a",
                "order-1",
                OrderStatus::Confirmed,
                ts(2),
            ))
            .then([OrderEventEnvelope::OrderStatusChanged {
                tenant_schema: "tenant_a".to_string(),
                order_uuid: "order-1".to_string(),
                status: "confirmed".to_string(),
                updated_at: ts(2),
            }]);
    }

    #[test]
    fn it_rejects_assignment_updates_for_completed_orders() {
        let completed = OrderEventEnvelope::OrderStatusChanged {
            tenant_schema: "tenant_a".to_string(),
            order_uuid: "order-1".to_string(),
            status: OrderStatus::Completed.as_str().to_string(),
            updated_at: ts(2),
        };

        TestHarness::given([seed_created_event("tenant_a", &sample_order()), completed])
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
            ))
            .then_err(
                "schedule assignment can only be updated in planned status (current: done)"
                    .to_string(),
            );
    }

    #[test]
    fn it_rejects_invalid_status_transition() {
        TestHarness::given([seed_created_event("tenant_a", &sample_order())])
            .when(UpdateOrderStatusDecision::new(
                "tenant_a",
                "order-1",
                OrderStatus::Completed,
                ts(2),
            ))
            .then_err("invalid order status transition: pending -> completed".to_string());
    }
}
