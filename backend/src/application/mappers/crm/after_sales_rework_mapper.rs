use crate::application::utils::parse_utc_time_to_string;
use crate::domain::crm::after_sales_rework::AfterSalesRework as DomainAfterSalesRework;
use shared::after_sales::AfterSalesRework as SharedAfterSalesRework;

impl From<DomainAfterSalesRework> for SharedAfterSalesRework {
    fn from(value: DomainAfterSalesRework) -> Self {
        Self {
            uuid: value.uuid,
            case_uuid: value.case_uuid,
            assigned_user_uuid: value.assigned_user_uuid,
            assigned_user_name: value.assigned_user_name,
            scheduled_start_at: parse_utc_time_to_string(value.scheduled_start_at),
            scheduled_end_at: parse_utc_time_to_string(value.scheduled_end_at),
            note: value.note,
            status: value.status,
            created_at: parse_utc_time_to_string(value.created_at),
        }
    }
}
