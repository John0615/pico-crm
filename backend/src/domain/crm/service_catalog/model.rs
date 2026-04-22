use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ServiceCatalog {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub base_price_cents: i64,
    pub default_duration_minutes: Option<i32>,
    pub is_active: bool,
    pub sort_order: i32,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateServiceCatalog {
    pub name: String,
    pub description: Option<String>,
    pub base_price_cents: i64,
    pub default_duration_minutes: Option<i32>,
    pub is_active: bool,
    pub sort_order: i32,
}

#[derive(Debug, Clone)]
pub struct UpdateServiceCatalog {
    pub uuid: String,
    pub name: String,
    pub description: Option<String>,
    pub base_price_cents: i64,
    pub default_duration_minutes: Option<i32>,
    pub is_active: bool,
    pub sort_order: i32,
}

impl ServiceCatalog {
    pub fn verify(&self) -> Result<(), String> {
        verify_fields(
            &self.name,
            self.base_price_cents,
            self.default_duration_minutes,
            self.sort_order,
        )
    }
}

impl CreateServiceCatalog {
    pub fn verify(&self) -> Result<(), String> {
        verify_fields(
            &self.name,
            self.base_price_cents,
            self.default_duration_minutes,
            self.sort_order,
        )
    }

    pub fn into_domain(self) -> ServiceCatalog {
        let now = Utc::now();
        ServiceCatalog {
            uuid: Uuid::new_v4().to_string(),
            name: self.name.trim().to_string(),
            description: self.description.and_then(normalize_optional),
            base_price_cents: self.base_price_cents,
            default_duration_minutes: self.default_duration_minutes,
            is_active: self.is_active,
            sort_order: self.sort_order,
            inserted_at: now,
            updated_at: now,
        }
    }
}

impl UpdateServiceCatalog {
    pub fn verify(&self) -> Result<(), String> {
        if self.uuid.trim().is_empty() {
            return Err("uuid is required".to_string());
        }
        Uuid::parse_str(self.uuid.trim()).map_err(|e| format!("invalid uuid: {}", e))?;
        verify_fields(
            &self.name,
            self.base_price_cents,
            self.default_duration_minutes,
            self.sort_order,
        )
    }
}

fn verify_fields(
    name: &str,
    base_price_cents: i64,
    default_duration_minutes: Option<i32>,
    _sort_order: i32,
) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("name is required".to_string());
    }
    if trimmed.chars().count() > 50 {
        return Err("name length cannot exceed 50".to_string());
    }
    if base_price_cents < 0 {
        return Err("base_price_cents must be non-negative".to_string());
    }
    if let Some(duration) = default_duration_minutes {
        if duration <= 0 {
            return Err("default_duration_minutes must be positive".to_string());
        }
    }
    Ok(())
}

fn normalize_optional(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
