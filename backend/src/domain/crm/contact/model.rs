use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Contact {
    pub uuid: String,
    pub name: String,
    pub phone: String,
    pub address: Option<String>,
    pub community: Option<String>,
    pub building: Option<String>,
    pub house_area_sqm: Option<i32>,
    pub service_need: Option<String>,
    pub tags: Vec<String>,
    pub last_service_at: Option<DateTime<Utc>>,
    pub follow_up_status: FollowUpStatus,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UpdateContact {
    pub uuid: String,
    pub name: String,
    pub phone: String,
    pub address: Option<String>,
    pub community: Option<String>,
    pub building: Option<String>,
    pub house_area_sqm: Option<i32>,
    pub service_need: Option<String>,
    pub tags: Vec<String>,
    pub last_service_at: Option<DateTime<Utc>>,
    pub follow_up_status: FollowUpStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum FollowUpStatus {
    Pending,
    Contacted,
    Quoted,
    Scheduled,
    Completed,
}

impl Contact {
    pub fn new(name: String, phone: String) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4().to_string(),
            name,
            phone,
            address: None,
            community: None,
            building: None,
            house_area_sqm: None,
            service_need: None,
            tags: Vec::new(),
            last_service_at: None,
            follow_up_status: FollowUpStatus::Pending,
            inserted_at: now,
            updated_at: now,
        }
    }

    pub fn verify(&self) -> Result<(), String> {
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.phone.trim().is_empty() {
            return Err("Phone cannot be empty".to_string());
        }

        validate_profile_fields(
            &self.address,
            &self.community,
            &self.building,
            &self.house_area_sqm,
            &self.service_need,
            &self.tags,
        )
    }
}

impl UpdateContact {
    pub fn verify(&self) -> Result<(), String> {
        if self.uuid.trim().is_empty() {
            return Err("Contact uuid cannot be empty".to_string());
        }
        if self.name.trim().is_empty() {
            return Err("Name cannot be empty".to_string());
        }
        if self.phone.trim().is_empty() {
            return Err("Phone cannot be empty".to_string());
        }

        validate_profile_fields(
            &self.address,
            &self.community,
            &self.building,
            &self.house_area_sqm,
            &self.service_need,
            &self.tags,
        )
    }
}

impl FollowUpStatus {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "pending" => Ok(Self::Pending),
            "contacted" => Ok(Self::Contacted),
            "quoted" => Ok(Self::Quoted),
            "scheduled" => Ok(Self::Scheduled),
            "completed" => Ok(Self::Completed),
            _ => Err(format!("Invalid follow up status: {}", value)),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Contacted => "contacted",
            Self::Quoted => "quoted",
            Self::Scheduled => "scheduled",
            Self::Completed => "completed",
        }
    }
}

fn validate_profile_fields(
    address: &Option<String>,
    community: &Option<String>,
    building: &Option<String>,
    house_area_sqm: &Option<i32>,
    service_need: &Option<String>,
    tags: &[String],
) -> Result<(), String> {
    validate_max_len(address.as_deref(), "address", 255)?;
    validate_max_len(community.as_deref(), "community", 64)?;
    validate_max_len(building.as_deref(), "building", 64)?;
    validate_max_len(service_need.as_deref(), "service_need", 500)?;

    if let Some(area) = house_area_sqm {
        if *area < 0 {
            return Err("house_area_sqm must be non-negative".to_string());
        }
    }

    if tags.len() > 8 {
        return Err("tags count cannot exceed 8".to_string());
    }
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            return Err("tags cannot contain empty value".to_string());
        }
        if trimmed.chars().count() > 20 {
            return Err("tag length cannot exceed 20".to_string());
        }
    }

    Ok(())
}

fn validate_max_len(value: Option<&str>, field: &str, max: usize) -> Result<(), String> {
    if let Some(value) = value {
        if value.chars().count() > max {
            return Err(format!("{} length cannot exceed {}", field, max));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_rejects_negative_house_area() {
        let mut contact = Contact::new("张三".to_string(), "13800138000".to_string());
        contact.house_area_sqm = Some(-1);

        let err = contact
            .verify()
            .expect_err("negative area should be rejected");
        assert!(err.contains("house_area_sqm"));
    }

    #[test]
    fn verify_rejects_too_many_tags() {
        let mut contact = Contact::new("李四".to_string(), "13800138001".to_string());
        contact.tags = (0..9).map(|idx| format!("标签{}", idx)).collect();

        let err = contact
            .verify()
            .expect_err("too many tags should be rejected");
        assert!(err.contains("tags count"));
    }

    #[test]
    fn update_contact_verify_rejects_overlong_address() {
        let update = UpdateContact {
            uuid: "11111111-1111-1111-1111-111111111111".to_string(),
            name: "王五".to_string(),
            phone: "13800138002".to_string(),
            address: Some("A".repeat(256)),
            community: None,
            building: None,
            house_area_sqm: None,
            service_need: None,
            tags: vec![],
            last_service_at: None,
            follow_up_status: FollowUpStatus::Pending,
        };

        let err = update
            .verify()
            .expect_err("overlong address should be rejected");
        assert!(err.contains("address length"));
    }

    #[test]
    fn follow_up_status_parse_rejects_unknown_value() {
        let err = FollowUpStatus::parse("unknown").expect_err("unknown status should fail");
        assert!(err.contains("Invalid follow up status"));
    }
}
