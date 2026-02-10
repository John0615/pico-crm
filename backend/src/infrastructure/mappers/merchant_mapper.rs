use crate::domain::models::merchant::Merchant;
use crate::infrastructure::entity::merchant::{ActiveModel, Model};
use sea_orm::{ActiveValue, IntoActiveModel};
use sea_orm::entity::prelude::Uuid;

pub struct MerchantMapper;

impl MerchantMapper {
    pub fn to_active_entity(merchant: Merchant) -> ActiveModel {
        ActiveModel {
            uuid: ActiveValue::Set(
                Uuid::parse_str(&merchant.uuid).expect("Invalid merchant UUID"),
            ),
            name: ActiveValue::Set(merchant.name),
            short_name: ActiveValue::Set(merchant.short_name),
            schema_name: ActiveValue::Set(merchant.schema_name),
            contact_name: ActiveValue::Set(merchant.contact_name),
            contact_phone: ActiveValue::Set(merchant.contact_phone),
            merchant_type: ActiveValue::Set(merchant.merchant_type),
            plan_type: ActiveValue::Set(merchant.plan_type),
            status: ActiveValue::Set(merchant.status),
            trial_end_at: ActiveValue::Set(merchant.trial_end_at),
            expired_at: ActiveValue::Set(merchant.expired_at),
            created_at: ActiveValue::Set(merchant.created_at),
            updated_at: ActiveValue::Set(merchant.updated_at),
        }
    }

    pub fn to_domain(model: Model) -> Merchant {
        Merchant {
            uuid: model.uuid.to_string(),
            name: model.name,
            short_name: model.short_name,
            schema_name: model.schema_name,
            contact_name: model.contact_name,
            contact_phone: model.contact_phone,
            merchant_type: model.merchant_type,
            plan_type: model.plan_type,
            status: model.status,
            trial_end_at: model.trial_end_at,
            expired_at: model.expired_at,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    pub fn to_update_active_entity(model: Model, status: String) -> ActiveModel {
        let mut active_model = model.into_active_model();
        active_model.status = ActiveValue::Set(status);
        active_model
    }
}
