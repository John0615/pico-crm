use chrono::{DateTime, Utc};
use disintegrate::Event;
use serde::{Deserialize, Serialize};

use crate::domain::crm::order::Order;

#[derive(Debug, Clone, PartialEq, Eq, Event, Serialize, Deserialize)]
#[stream(
    OrderEvent,
    [
        OrderCreated,
        OrderStatusChanged,
        OrderAssignmentUpdated,
        OrderSettlementUpdated
    ]
)]
pub enum OrderEventEnvelope {
    OrderCreated {
        #[id]
        tenant_schema: String,
        #[id]
        order_uuid: String,
        request_id: Option<String>,
        customer_uuid: Option<String>,
        scheduled_start_at: Option<DateTime<Utc>>,
        scheduled_end_at: Option<DateTime<Utc>>,
        status: String,
        settlement_status: String,
        amount_cents: i64,
        notes: Option<String>,
        dispatch_note: Option<String>,
        settlement_note: Option<String>,
        inserted_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    },
    OrderStatusChanged {
        #[id]
        tenant_schema: String,
        #[id]
        order_uuid: String,
        status: String,
        updated_at: DateTime<Utc>,
    },
    OrderAssignmentUpdated {
        #[id]
        tenant_schema: String,
        #[id]
        order_uuid: String,
        scheduled_start_at: Option<DateTime<Utc>>,
        scheduled_end_at: Option<DateTime<Utc>>,
        dispatch_note: Option<String>,
        updated_at: DateTime<Utc>,
    },
    OrderSettlementUpdated {
        #[id]
        tenant_schema: String,
        #[id]
        order_uuid: String,
        settlement_status: String,
        settlement_note: Option<String>,
        updated_at: DateTime<Utc>,
    },
}

pub fn seed_created_event(tenant_schema: &str, order: &Order) -> OrderEventEnvelope {
    OrderEventEnvelope::OrderCreated {
        tenant_schema: tenant_schema.to_string(),
        order_uuid: order.uuid.clone(),
        request_id: order.request_id.clone(),
        customer_uuid: order.customer_uuid.clone(),
        scheduled_start_at: order.scheduled_start_at,
        scheduled_end_at: order.scheduled_end_at,
        status: order.status.as_str().to_string(),
        settlement_status: order.settlement_status.as_str().to_string(),
        amount_cents: order.amount_cents,
        notes: order.notes.clone(),
        dispatch_note: order.dispatch_note.clone(),
        settlement_note: order.settlement_note.clone(),
        inserted_at: order.inserted_at,
        updated_at: order.updated_at,
    }
}
