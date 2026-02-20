use crate::application::utils::parse_utc_time_to_string;
use crate::domain::models::merchant::Merchant;
use shared::merchant::MerchantSummary;

impl From<Merchant> for MerchantSummary {
    fn from(merchant: Merchant) -> Self {
        Self {
            uuid: merchant.uuid,
            name: merchant.name,
            short_name: merchant.short_name,
            schema_name: merchant.schema_name,
            contact_name: merchant.contact_name,
            contact_phone: merchant.contact_phone,
            merchant_type: merchant.merchant_type,
            plan_type: merchant.plan_type,
            status: merchant.status,
            trial_end_at: merchant.trial_end_at.map(parse_utc_time_to_string),
            expired_at: merchant.expired_at.map(parse_utc_time_to_string),
            created_at: parse_utc_time_to_string(merchant.created_at),
            updated_at: parse_utc_time_to_string(merchant.updated_at),
        }
    }
}
