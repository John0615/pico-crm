use crate::application::utils::parse_utc_time_to_string;
use crate::domain::crm::service_catalog::ServiceCatalog as DomainServiceCatalog;
use shared::service_catalog::ServiceCatalog as SharedServiceCatalog;

impl From<DomainServiceCatalog> for SharedServiceCatalog {
    fn from(value: DomainServiceCatalog) -> Self {
        Self {
            uuid: value.uuid,
            name: value.name,
            description: value.description,
            base_price_cents: value.base_price_cents,
            default_duration_minutes: value.default_duration_minutes,
            is_active: value.is_active,
            sort_order: value.sort_order,
            inserted_at: parse_utc_time_to_string(value.inserted_at),
            updated_at: parse_utc_time_to_string(value.updated_at),
        }
    }
}
