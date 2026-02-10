use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub app_env: String,
    pub database_url: String,
    pub tenant_schema_prefix: String,
    pub upload_bucket: Option<String>,
    pub upload_region: Option<String>,
    pub sms_api_key: Option<String>,
    pub admin_trial_days_default: Option<i64>,
    pub admin_sms_template_id: Option<String>,
}

impl AppConfig {
    pub fn from_env() -> Result<Self, String> {
        let app_env = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
        let database_url = env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL is not set".to_string())?;
        let tenant_schema_prefix = env::var("TENANT_SCHEMA_PREFIX")
            .unwrap_or_else(|_| "merchant_".to_string());

        let upload_bucket = env::var("UPLOAD_BUCKET").ok();
        let upload_region = env::var("UPLOAD_REGION").ok();
        let sms_api_key = env::var("SMS_API_KEY").ok();

        let admin_trial_days_default = env::var("ADMIN_TRIAL_DAYS_DEFAULT")
            .ok()
            .and_then(|value| value.parse::<i64>().ok());
        let admin_sms_template_id = env::var("ADMIN_SMS_TEMPLATE_ID").ok();

        Ok(Self {
            app_env,
            database_url,
            tenant_schema_prefix,
            upload_bucket,
            upload_region,
            sms_api_key,
            admin_trial_days_default,
            admin_sms_template_id,
        })
    }
}
