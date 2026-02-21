use crate::application::utils::parse_utc_time_to_string;
use crate::domain::models::order::Order as DomainOrder;
use shared::order::Order as SharedOrder;

impl From<DomainOrder> for SharedOrder {
    fn from(order: DomainOrder) -> Self {
        Self {
            uuid: order.uuid,
            request_id: order.request_id,
            contact_uuid: order.contact_uuid,
            assigned_user_uuid: order.assigned_user_uuid,
            scheduled_start_at: order
                .scheduled_start_at
                .map(parse_utc_time_to_string),
            scheduled_end_at: order.scheduled_end_at.map(parse_utc_time_to_string),
            status: order.status.as_str().to_string(),
            settlement_status: order.settlement_status.as_str().to_string(),
            amount_cents: order.amount_cents,
            notes: order.notes,
            dispatch_note: order.dispatch_note,
            settlement_note: order.settlement_note,
            inserted_at: parse_utc_time_to_string(order.inserted_at),
            updated_at: parse_utc_time_to_string(order.updated_at),
        }
    }
}
