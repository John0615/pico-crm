use crate::domain::crm::service_catalog::{ServiceCatalog, UpdateServiceCatalog};
use crate::infrastructure::entity::service_catalogs::{ActiveModel, Model};
use crate::infrastructure::utils::parse_date_time_to_string;
use sea_orm::ActiveValue::Set;
use sea_orm::IntoActiveModel;
use sea_orm::entity::prelude::*;
use shared::service_catalog::ServiceCatalog as SharedServiceCatalog;

pub struct ServiceCatalogMapper;

impl ServiceCatalogMapper {
    pub fn to_domain(model: Model) -> ServiceCatalog {
        ServiceCatalog {
            uuid: model.uuid.to_string(),
            name: model.name,
            description: model.description,
            base_price_cents: model.base_price_cents,
            default_duration_minutes: model.default_duration_minutes,
            is_active: model.is_active,
            sort_order: model.sort_order,
            inserted_at: model.inserted_at,
            updated_at: model.updated_at,
        }
    }

    pub fn to_view(model: Model) -> SharedServiceCatalog {
        SharedServiceCatalog {
            uuid: model.uuid.to_string(),
            name: model.name,
            description: model.description,
            base_price_cents: model.base_price_cents,
            default_duration_minutes: model.default_duration_minutes,
            is_active: model.is_active,
            sort_order: model.sort_order,
            inserted_at: parse_date_time_to_string(model.inserted_at),
            updated_at: parse_date_time_to_string(model.updated_at),
        }
    }

    pub fn to_active_entity(catalog: ServiceCatalog) -> ActiveModel {
        ActiveModel {
            uuid: Set(Uuid::parse_str(&catalog.uuid).expect("invalid service catalog uuid")),
            name: Set(catalog.name),
            description: Set(catalog.description),
            base_price_cents: Set(catalog.base_price_cents),
            default_duration_minutes: Set(catalog.default_duration_minutes),
            is_active: Set(catalog.is_active),
            sort_order: Set(catalog.sort_order),
            inserted_at: Set(catalog.inserted_at),
            updated_at: Set(catalog.updated_at),
        }
    }

    pub fn to_update_active_entity(update: UpdateServiceCatalog, original: Model) -> ActiveModel {
        let mut active = original.into_active_model();
        active.name = Set(update.name.trim().to_string());
        active.description = Set(update.description);
        active.base_price_cents = Set(update.base_price_cents);
        active.default_duration_minutes = Set(update.default_duration_minutes);
        active.is_active = Set(update.is_active);
        active.sort_order = Set(update.sort_order);
        active.updated_at = Set(chrono::Utc::now());
        active
    }
}
