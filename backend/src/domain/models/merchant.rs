use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Merchant {
    pub uuid: String,
    pub name: String,
    pub short_name: Option<String>,
    pub schema_name: String,
    pub contact_name: String,
    pub contact_phone: String,
    pub merchant_type: Option<String>,
    pub plan_type: Option<String>,
    pub status: String,
    pub trial_end_at: Option<DateTime<Utc>>,
    pub expired_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct MerchantUpdate {
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub contact_name: Option<String>,
    pub contact_phone: Option<String>,
    pub merchant_type: Option<String>,
    pub status: Option<String>,
    pub plan_type: Option<String>,
    pub trial_end_at: Option<DateTime<Utc>>,
    pub expired_at: Option<DateTime<Utc>>,
}

impl Merchant {
    pub fn new(
        uuid: String,
        name: String,
        short_name: Option<String>,
        schema_name: String,
        contact_name: String,
        contact_phone: String,
        merchant_type: Option<String>,
        plan_type: Option<String>,
        status: String,
        trial_end_at: Option<DateTime<Utc>>,
        expired_at: Option<DateTime<Utc>>,
    ) -> Self {
        let now = Utc::now();
        Self {
            uuid,
            name,
            short_name,
            schema_name,
            contact_name,
            contact_phone,
            merchant_type,
            plan_type,
            status,
            trial_end_at,
            expired_at,
            created_at: now,
            updated_at: now,
        }
    }
}
