use crate::domain::models::order::OrderStatus;
use crate::domain::models::schedule::ScheduleStatus;
use crate::infrastructure::entity::orders::Model;
use crate::infrastructure::utils::parse_date_time_to_string;
use shared::schedule::Schedule;

pub struct ScheduleMapper;

impl ScheduleMapper {
    pub fn to_view(entity: Model) -> Schedule {
        let order_status = entity.status.clone();
        let schedule_status = OrderStatus::parse(&order_status)
            .map(|status| ScheduleStatus::from_order_status(&status).as_str().to_string())
            .unwrap_or_else(|_| "planned".to_string());

        Schedule {
            order_uuid: entity.uuid.to_string(),
            order_status,
            schedule_status,
            contact_uuid: entity.contact_uuid.map(|value| value.to_string()),
            assigned_user_uuid: entity.assigned_user_uuid.map(|value| value.to_string()),
            scheduled_start_at: entity.scheduled_start_at.map(parse_date_time_to_string),
            scheduled_end_at: entity.scheduled_end_at.map(parse_date_time_to_string),
            dispatch_note: entity.dispatch_note,
            notes: entity.notes,
            inserted_at: parse_date_time_to_string(entity.inserted_at),
            updated_at: parse_date_time_to_string(entity.updated_at),
        }
    }
}
