use chrono::{DateTime, Utc};
use disintegrate::{StateMutate, StateQuery};
use serde::{Deserialize, Serialize};

use super::events::OrderEvent;
use crate::domain::crm::order::{Order, OrderStatus, SettlementStatus};

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, StateQuery)]
#[state_query(OrderEvent)]
pub struct OrderState {
    #[id]
    pub tenant_schema: String,
    #[id]
    pub order_uuid: String,
    pub exists: bool,
    pub request_id: Option<String>,
    pub customer_uuid: Option<String>,
    pub scheduled_start_at: Option<DateTime<Utc>>,
    pub scheduled_end_at: Option<DateTime<Utc>>,
    pub status: Option<String>,
    pub cancellation_reason: Option<String>,
    pub completed_at: Option<DateTime<Utc>>,
    pub settlement_status: Option<String>,
    pub amount_cents: i64,
    pub paid_amount_cents: Option<i64>,
    pub payment_method: Option<String>,
    pub paid_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub dispatch_note: Option<String>,
    pub settlement_note: Option<String>,
    pub inserted_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

impl OrderState {
    pub fn new(tenant_schema: impl Into<String>, order_uuid: impl Into<String>) -> Self {
        Self {
            tenant_schema: tenant_schema.into(),
            order_uuid: order_uuid.into(),
            ..Default::default()
        }
    }

    pub fn to_domain(&self) -> Result<Order, String> {
        if !self.exists {
            return Err(format!("order {} not found", self.order_uuid));
        }

        let status = OrderStatus::parse(
            self.status
                .as_deref()
                .unwrap_or(OrderStatus::Pending.as_str()),
        )?;
        let settlement_status = SettlementStatus::parse(
            self.settlement_status
                .as_deref()
                .unwrap_or(SettlementStatus::Unsettled.as_str()),
        )?;
        let inserted_at = self
            .inserted_at
            .ok_or_else(|| "order inserted_at is missing".to_string())?;
        let updated_at = self
            .updated_at
            .ok_or_else(|| "order updated_at is missing".to_string())?;

        Ok(Order {
            uuid: self.order_uuid.clone(),
            request_id: self.request_id.clone(),
            customer_uuid: self.customer_uuid.clone(),
            scheduled_start_at: self.scheduled_start_at,
            scheduled_end_at: self.scheduled_end_at,
            status,
            cancellation_reason: self.cancellation_reason.clone(),
            completed_at: self.completed_at,
            settlement_status,
            amount_cents: self.amount_cents,
            paid_amount_cents: self.paid_amount_cents,
            payment_method: self.payment_method.clone(),
            paid_at: self.paid_at,
            notes: self.notes.clone(),
            dispatch_note: self.dispatch_note.clone(),
            settlement_note: self.settlement_note.clone(),
            inserted_at,
            updated_at,
        })
    }
}

impl StateMutate for OrderState {
    fn mutate(&mut self, event: Self::Event) {
        match event {
            OrderEvent::OrderCreated {
                tenant_schema,
                order_uuid,
                request_id,
                customer_uuid,
                scheduled_start_at,
                scheduled_end_at,
                status,
                cancellation_reason,
                completed_at,
                settlement_status,
                amount_cents,
                paid_amount_cents,
                payment_method,
                paid_at,
                notes,
                dispatch_note,
                settlement_note,
                inserted_at,
                updated_at,
                ..
            } => {
                self.exists = true;
                self.tenant_schema = tenant_schema;
                self.order_uuid = order_uuid;
                self.request_id = request_id;
                self.customer_uuid = customer_uuid;
                self.scheduled_start_at = scheduled_start_at;
                self.scheduled_end_at = scheduled_end_at;
                self.status = Some(status);
                self.cancellation_reason = cancellation_reason;
                self.completed_at = completed_at;
                self.settlement_status = Some(settlement_status);
                self.amount_cents = amount_cents;
                self.paid_amount_cents = paid_amount_cents;
                self.payment_method = payment_method;
                self.paid_at = paid_at;
                self.notes = notes;
                self.dispatch_note = dispatch_note;
                self.settlement_note = settlement_note;
                self.inserted_at = Some(inserted_at);
                self.updated_at = Some(updated_at);
            }
            OrderEvent::OrderStatusChanged {
                status,
                completed_at,
                updated_at,
                ..
            } => {
                self.status = Some(status);
                self.completed_at = completed_at;
                self.updated_at = Some(updated_at);
            }
            OrderEvent::OrderDetailsUpdated {
                customer_uuid,
                amount_cents,
                notes,
                updated_at,
                ..
            } => {
                self.customer_uuid = customer_uuid;
                self.amount_cents = amount_cents;
                self.notes = notes;
                self.updated_at = Some(updated_at);
            }
            OrderEvent::OrderCancelled {
                cancellation_reason,
                updated_at,
                ..
            } => {
                self.status = Some(OrderStatus::Cancelled.as_str().to_string());
                self.cancellation_reason = Some(cancellation_reason);
                self.completed_at = None;
                self.updated_at = Some(updated_at);
            }
            OrderEvent::OrderAssignmentUpdated {
                scheduled_start_at,
                scheduled_end_at,
                dispatch_note,
                updated_at,
                ..
            } => {
                self.scheduled_start_at = scheduled_start_at;
                self.scheduled_end_at = scheduled_end_at;
                self.dispatch_note = dispatch_note;
                self.updated_at = Some(updated_at);
            }
            OrderEvent::OrderSettlementUpdated {
                settlement_status,
                settlement_note,
                paid_amount_cents,
                payment_method,
                paid_at,
                updated_at,
                ..
            } => {
                self.settlement_status = Some(settlement_status);
                self.settlement_note = settlement_note;
                self.paid_amount_cents = paid_amount_cents;
                self.payment_method = payment_method;
                self.paid_at = paid_at;
                self.updated_at = Some(updated_at);
            }
        }
    }
}
