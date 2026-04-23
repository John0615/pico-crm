use chrono::{DateTime, Utc};
use disintegrate::Event;
use serde::{Deserialize, Serialize};

use crate::domain::crm::order::Order;

#[derive(Debug, Clone, PartialEq, Eq, Event, Serialize, Deserialize)]
#[stream(
    OrderEvent,
    [
        OrderCreated,
        OrderDetailsUpdated,
        OrderStatusChanged,
        OrderCancelled,
        OrderAssignmentUpdated,
        OrderSettlementUpdated
    ]
)]
pub enum OrderEventEnvelope {
    OrderCreated {
        #[id]
        merchant_id: String,
        #[id]
        order_uuid: String,
        #[serde(default)]
        operator_uuid: Option<String>,
        request_id: Option<String>,
        customer_uuid: Option<String>,
        scheduled_start_at: Option<DateTime<Utc>>,
        scheduled_end_at: Option<DateTime<Utc>>,
        status: String,
        #[serde(default)]
        cancellation_reason: Option<String>,
        #[serde(default)]
        completed_at: Option<DateTime<Utc>>,
        settlement_status: String,
        amount_cents: i64,
        #[serde(default)]
        paid_amount_cents: Option<i64>,
        #[serde(default)]
        payment_method: Option<String>,
        #[serde(default)]
        paid_at: Option<DateTime<Utc>>,
        notes: Option<String>,
        dispatch_note: Option<String>,
        settlement_note: Option<String>,
        inserted_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    },
    OrderDetailsUpdated {
        #[id]
        merchant_id: String,
        #[id]
        order_uuid: String,
        #[serde(default)]
        operator_uuid: Option<String>,
        customer_uuid: Option<String>,
        amount_cents: i64,
        notes: Option<String>,
        updated_at: DateTime<Utc>,
    },
    OrderStatusChanged {
        #[id]
        merchant_id: String,
        #[id]
        order_uuid: String,
        #[serde(default)]
        operator_uuid: Option<String>,
        status: String,
        #[serde(default)]
        completed_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
    },
    OrderCancelled {
        #[id]
        merchant_id: String,
        #[id]
        order_uuid: String,
        #[serde(default)]
        operator_uuid: Option<String>,
        cancellation_reason: String,
        updated_at: DateTime<Utc>,
    },
    OrderAssignmentUpdated {
        #[id]
        merchant_id: String,
        #[id]
        order_uuid: String,
        #[serde(default)]
        operator_uuid: Option<String>,
        scheduled_start_at: Option<DateTime<Utc>>,
        scheduled_end_at: Option<DateTime<Utc>>,
        dispatch_note: Option<String>,
        updated_at: DateTime<Utc>,
    },
    OrderSettlementUpdated {
        #[id]
        merchant_id: String,
        #[id]
        order_uuid: String,
        #[serde(default)]
        operator_uuid: Option<String>,
        settlement_status: String,
        settlement_note: Option<String>,
        #[serde(default)]
        paid_amount_cents: Option<i64>,
        #[serde(default)]
        payment_method: Option<String>,
        #[serde(default)]
        paid_at: Option<DateTime<Utc>>,
        updated_at: DateTime<Utc>,
    },
}

pub fn seed_created_event(
    merchant_id: &str,
    order: &Order,
    operator_uuid: Option<String>,
) -> OrderEventEnvelope {
    OrderEventEnvelope::OrderCreated {
        merchant_id: merchant_id.to_string(),
        order_uuid: order.uuid.clone(),
        operator_uuid,
        request_id: order.request_id.clone(),
        customer_uuid: order.customer_uuid.clone(),
        scheduled_start_at: order.scheduled_start_at,
        scheduled_end_at: order.scheduled_end_at,
        status: order.status.as_str().to_string(),
        cancellation_reason: order.cancellation_reason.clone(),
        completed_at: order.completed_at,
        settlement_status: order.settlement_status.as_str().to_string(),
        amount_cents: order.amount_cents,
        paid_amount_cents: order.paid_amount_cents,
        payment_method: order.payment_method.clone(),
        paid_at: order.paid_at,
        notes: order.notes.clone(),
        dispatch_note: order.dispatch_note.clone(),
        settlement_note: order.settlement_note.clone(),
        inserted_at: order.inserted_at,
        updated_at: order.updated_at,
    }
}
