use crate::domain::crm::order::{
    Order, OrderAssignmentUpdate, OrderDetailsUpdate, OrderStatus, SettlementStatus,
};
use crate::infrastructure::entity::orders::{ActiveModel, Model};
use crate::infrastructure::utils::parse_date_time_to_string;
use chrono::{DateTime, Utc};
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use sea_orm::entity::prelude::*;
use shared::order::Order as SharedOrder;

pub struct OrderMapper;

impl OrderMapper {
    pub fn to_view(entity: Model) -> SharedOrder {
        SharedOrder {
            uuid: entity.uuid.to_string(),
            request_id: entity.request_id.map(|value| value.to_string()),
            customer_uuid: entity.customer_uuid.map(|value| value.to_string()),
            service_catalog_uuid: None,
            service_catalog_name: None,
            customer_name: None,
            scheduled_start_at: entity.scheduled_start_at.map(parse_date_time_to_string),
            scheduled_end_at: entity.scheduled_end_at.map(parse_date_time_to_string),
            status: entity.status,
            cancellation_reason: entity.cancellation_reason,
            completed_at: entity.completed_at.map(parse_date_time_to_string),
            settlement_status: entity.settlement_status,
            amount_cents: entity.amount_cents,
            paid_amount_cents: entity.paid_amount_cents,
            payment_method: entity.payment_method,
            paid_at: entity.paid_at.map(parse_date_time_to_string),
            notes: entity.notes,
            dispatch_note: entity.dispatch_note,
            settlement_note: entity.settlement_note,
            inserted_at: parse_date_time_to_string(entity.inserted_at),
            updated_at: parse_date_time_to_string(entity.updated_at),
        }
    }

    pub fn to_domain(entity: Model) -> Order {
        let status = OrderStatus::parse(&entity.status).unwrap_or(OrderStatus::Pending);
        let settlement_status = SettlementStatus::parse(&entity.settlement_status)
            .unwrap_or(SettlementStatus::Unsettled);
        Order {
            uuid: entity.uuid.to_string(),
            request_id: entity.request_id.map(|value| value.to_string()),
            customer_uuid: entity.customer_uuid.map(|value| value.to_string()),
            scheduled_start_at: entity.scheduled_start_at,
            scheduled_end_at: entity.scheduled_end_at,
            status,
            cancellation_reason: entity.cancellation_reason,
            completed_at: entity.completed_at,
            settlement_status,
            amount_cents: entity.amount_cents,
            paid_amount_cents: entity.paid_amount_cents,
            payment_method: entity.payment_method,
            paid_at: entity.paid_at,
            notes: entity.notes,
            dispatch_note: entity.dispatch_note,
            settlement_note: entity.settlement_note,
            inserted_at: entity.inserted_at,
            updated_at: entity.updated_at,
        }
    }

    pub fn to_active_entity(order: Order) -> ActiveModel {
        ActiveModel {
            uuid: Set(Uuid::parse_str(&order.uuid).expect("Invalid UUID")),
            customer_uuid: Set(order
                .customer_uuid
                .as_ref()
                .map(|value| Uuid::parse_str(value).expect("Invalid customer UUID"))),
            status: Set(order.status.as_str().to_string()),
            cancellation_reason: Set(order.cancellation_reason),
            completed_at: Set(order.completed_at),
            amount_cents: Set(order.amount_cents),
            paid_amount_cents: Set(order.paid_amount_cents),
            payment_method: Set(order.payment_method),
            paid_at: Set(order.paid_at),
            notes: Set(order.notes),
            request_id: Set(order
                .request_id
                .map(|value| Uuid::parse_str(&value).expect("Invalid request UUID"))),
            scheduled_start_at: Set(order.scheduled_start_at),
            scheduled_end_at: Set(order.scheduled_end_at),
            dispatch_note: Set(order.dispatch_note),
            settlement_status: Set(order.settlement_status.as_str().to_string()),
            settlement_note: Set(order.settlement_note),
            inserted_at: Set(order.inserted_at),
            updated_at: Set(order.updated_at),
            event_id: Set(0),
        }
    }

    pub fn to_status_active_entity(
        original: Model,
        status: OrderStatus,
        completed_at: Option<DateTime<Utc>>,
    ) -> ActiveModel {
        let mut active = original.into_active_model();
        active.status = Set(status.as_str().to_string());
        active.completed_at = Set(completed_at);
        if status != OrderStatus::Cancelled {
            active.cancellation_reason = Set(None);
        }
        active.updated_at = Set(Utc::now());
        active
    }

    pub fn to_assignment_active_entity(
        original: Model,
        update: OrderAssignmentUpdate,
    ) -> ActiveModel {
        let mut active = original.into_active_model();
        active.scheduled_start_at = Set(update.scheduled_start_at);
        active.scheduled_end_at = Set(update.scheduled_end_at);
        active.dispatch_note = Set(update.dispatch_note);
        active.updated_at = Set(Utc::now());
        active
    }

    pub fn to_details_active_entity(original: Model, update: OrderDetailsUpdate) -> ActiveModel {
        let mut active = original.into_active_model();
        active.customer_uuid = Set(Some(
            Uuid::parse_str(&update.customer_uuid).expect("Invalid customer UUID"),
        ));
        active.amount_cents = Set(update.amount_cents);
        active.notes = Set(update.notes);
        active.updated_at = Set(Utc::now());
        active
    }

    pub fn to_cancelled_active_entity(original: Model, reason: String) -> ActiveModel {
        let mut active = original.into_active_model();
        active.status = Set(OrderStatus::Cancelled.as_str().to_string());
        active.cancellation_reason = Set(Some(reason));
        active.completed_at = Set(None);
        active.updated_at = Set(Utc::now());
        active
    }

    pub fn to_settlement_active_entity(
        original: Model,
        settlement_status: SettlementStatus,
        settlement_note: Option<String>,
        paid_amount_cents: Option<i64>,
        payment_method: Option<String>,
        paid_at: Option<DateTime<Utc>>,
    ) -> ActiveModel {
        let mut active = original.into_active_model();
        active.settlement_status = Set(settlement_status.as_str().to_string());
        active.settlement_note = Set(settlement_note);
        active.paid_amount_cents = Set(paid_amount_cents);
        active.payment_method = Set(payment_method);
        active.paid_at = Set(paid_at);
        active.updated_at = Set(Utc::now());
        active
    }
}
