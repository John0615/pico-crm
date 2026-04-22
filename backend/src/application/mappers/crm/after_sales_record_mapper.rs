use crate::application::utils::parse_utc_time_to_string;
use crate::domain::crm::after_sales::AfterSalesCaseRecord as DomainAfterSalesCaseRecord;
use shared::after_sales::AfterSalesCaseRecord as SharedAfterSalesCaseRecord;

impl From<DomainAfterSalesCaseRecord> for SharedAfterSalesCaseRecord {
    fn from(value: DomainAfterSalesCaseRecord) -> Self {
        Self {
            uuid: value.uuid,
            case_uuid: value.case_uuid,
            operator_uuid: value.operator_uuid,
            operator_name: value.operator_name,
            content: value.content,
            status: value.status,
            created_at: parse_utc_time_to_string(value.created_at),
        }
    }
}
