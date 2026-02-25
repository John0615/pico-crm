use crate::domain::models::order::OrderStatus;
use crate::domain::models::schedule::ScheduleStatus;
use crate::infrastructure::entity::orders::Model as OrderModel;
use crate::infrastructure::entity::schedules::Model as ScheduleModel;
use crate::infrastructure::utils::parse_date_time_to_string;
use shared::schedule::Schedule;

pub struct ScheduleMapper;

impl ScheduleMapper {
    pub fn to_view(order: OrderModel, schedule: Option<ScheduleModel>) -> Schedule {
        let order_status = order.status.clone();
        let schedule_status = OrderStatus::parse(&order_status)
            .map(|status| ScheduleStatus::from_order_status(&status).as_str().to_string())
            .unwrap_or_else(|_| "planned".to_string());

        let assigned_user_uuid = schedule
            .as_ref()
            .map(|value| value.assigned_user_uuid.to_string());
        let scheduled_start_at = schedule
            .as_ref()
            .map(|value| parse_date_time_to_string(value.start_at));
        let scheduled_end_at = schedule
            .as_ref()
            .map(|value| parse_date_time_to_string(value.end_at));
        let dispatch_note = schedule
            .as_ref()
            .and_then(|value| value.notes.clone());
        let inserted_at = schedule
            .as_ref()
            .map(|value| parse_date_time_to_string(value.inserted_at))
            .unwrap_or_else(|| parse_date_time_to_string(order.inserted_at));
        let updated_at = schedule
            .as_ref()
            .map(|value| parse_date_time_to_string(value.updated_at))
            .unwrap_or_else(|| parse_date_time_to_string(order.updated_at));

        Schedule {
            order_uuid: order.uuid.to_string(),
            order_status,
            schedule_status,
            customer_uuid: order.customer_uuid.map(|value| value.to_string()),
            assigned_user_uuid,
            scheduled_start_at,
            scheduled_end_at,
            dispatch_note,
            notes: order.notes,
            inserted_at,
            updated_at,
        }
    }
}
