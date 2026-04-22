use crate::application::utils::parse_utc_time_to_string;
use crate::domain::crm::after_sales::AfterSalesCase as DomainAfterSalesCase;
use shared::after_sales::AfterSalesCase as SharedAfterSalesCase;

impl From<DomainAfterSalesCase> for SharedAfterSalesCase {
    fn from(value: DomainAfterSalesCase) -> Self {
        Self {
            uuid: value.uuid,
            order_uuid: value.order_uuid,
            operator_uuid: value.operator_uuid,
            operator_name: value.operator_name,
            case_type: value.case_type,
            description: value.description,
            status: value.status,
            refund_amount_cents: value.refund_amount_cents,
            refund_reason: value.refund_reason,
            created_at: parse_utc_time_to_string(value.created_at),
        }
    }
}
