use crate::domain::models::order::{
    Order, OrderAssignmentUpdate, OrderStatus, SettlementStatus,
};
use crate::infrastructure::entity::orders::{ActiveModel, Model};
use crate::infrastructure::utils::parse_date_time_to_string;
use chrono::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;
use sea_orm::IntoActiveModel;
use shared::order::Order as SharedOrder;

pub struct OrderMapper;

impl OrderMapper {
    pub fn to_view(entity: Model) -> SharedOrder {
        SharedOrder {
            uuid: entity.uuid.to_string(),
            request_id: entity.request_id.map(|value| value.to_string()),
            customer_uuid: entity.customer_uuid.map(|value| value.to_string()),
            customer_name: None,
            scheduled_start_at: entity.scheduled_start_at.map(parse_date_time_to_string),
            scheduled_end_at: entity.scheduled_end_at.map(parse_date_time_to_string),
            status: entity.status,
            settlement_status: entity.settlement_status,
            amount_cents: entity.amount_cents,
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
            settlement_status,
            amount_cents: entity.amount_cents,
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
            customer_uuid: Set(order.customer_uuid.as_ref().map(|value| {
                Uuid::parse_str(value).expect("Invalid customer UUID")
            })),
            status: Set(order.status.as_str().to_string()),
            amount_cents: Set(order.amount_cents),
            notes: Set(order.notes),
            request_id: Set(order.request_id.map(|value| Uuid::parse_str(&value).expect("Invalid request UUID"))),
            scheduled_start_at: Set(order.scheduled_start_at),
            scheduled_end_at: Set(order.scheduled_end_at),
            dispatch_note: Set(order.dispatch_note),
            settlement_status: Set(order.settlement_status.as_str().to_string()),
            settlement_note: Set(order.settlement_note),
            inserted_at: Set(order.inserted_at),
            updated_at: Set(order.updated_at),
        }
    }

    pub fn to_status_active_entity(original: Model, status: OrderStatus) -> ActiveModel {
        let mut active = original.into_active_model();
        active.status = Set(status.as_str().to_string());
        active.updated_at = Set(Utc::now());
        active
    }

    pub fn to_assignment_active_entity(original: Model, update: OrderAssignmentUpdate) -> ActiveModel {
        let mut active = original.into_active_model();
        active.scheduled_start_at = Set(update.scheduled_start_at);
        active.scheduled_end_at = Set(update.scheduled_end_at);
        active.dispatch_note = Set(update.dispatch_note);
        active.updated_at = Set(Utc::now());
        active
    }

    pub fn to_settlement_active_entity(
        original: Model,
        settlement_status: SettlementStatus,
        settlement_note: Option<String>,
    ) -> ActiveModel {
        let mut active = original.into_active_model();
        active.settlement_status = Set(settlement_status.as_str().to_string());
        active.settlement_note = Set(settlement_note);
        active.updated_at = Set(Utc::now());
        active
    }
}
