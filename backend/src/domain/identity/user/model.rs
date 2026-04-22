use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use chrono::{DateTime, Utc};
use rand::rngs::OsRng;
use sea_orm::entity::prelude::Uuid;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Status {
    Inactive,
    Active,
}

impl Status {
    pub fn is_active(&self) -> bool {
        matches!(self, Status::Active)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum EmploymentStatus {
    Active,
    OnLeave,
    Resigned,
}

impl EmploymentStatus {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "active" => Ok(Self::Active),
            "on_leave" => Ok(Self::OnLeave),
            "resigned" => Ok(Self::Resigned),
            _ => Err(format!("Invalid employment status: {}", value)),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::OnLeave => "on_leave",
            Self::Resigned => "resigned",
        }
    }

    pub fn is_dispatchable(&self) -> bool {
        matches!(self, Self::Active)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum HealthStatus {
    Healthy,
    Attention,
    Expired,
}

impl HealthStatus {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "healthy" => Ok(Self::Healthy),
            "attention" => Ok(Self::Attention),
            "expired" => Ok(Self::Expired),
            _ => Err(format!("Invalid health status: {}", value)),
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Healthy => "healthy",
            Self::Attention => "attention",
            Self::Expired => "expired",
        }
    }
}

#[derive(Debug, Clone)]
pub struct User {
    pub uuid: String,
    pub user_name: String,
    pub password: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
    pub merchant_uuid: Option<String>,
    pub role: String,
    pub is_admin: Option<bool>,
    pub status: Status,
    pub employment_status: EmploymentStatus,
    pub skills: Vec<String>,
    pub service_areas: Vec<String>,
    pub training_records: Vec<String>,
    pub certificates: Vec<String>,
    pub health_status: HealthStatus,
    pub employee_note: Option<String>,
    pub joined_at: Option<DateTime<Utc>>,
    pub completed_service_count: Option<u64>,
    pub feedback_count: Option<u64>,
    pub average_rating: Option<f64>,
    pub after_sales_case_count: Option<u64>,
    pub complaint_case_count: Option<u64>,
    pub refund_case_count: Option<u64>,
    pub rework_count: Option<u64>,
    pub avatar_url: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    pub fn new(
        user_name: String,
        password: String,
        email: Option<String>,
        phone_number: Option<String>,
    ) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4().to_string(),
            user_name,
            password,
            email,
            phone_number,
            merchant_uuid: None,
            role: "operator".to_string(),
            is_admin: Some(false),
            status: Status::Active,
            employment_status: EmploymentStatus::Active,
            skills: Vec::new(),
            service_areas: Vec::new(),
            training_records: Vec::new(),
            certificates: Vec::new(),
            health_status: HealthStatus::Healthy,
            employee_note: None,
            joined_at: None,
            completed_service_count: None,
            feedback_count: None,
            average_rating: None,
            after_sales_case_count: None,
            complaint_case_count: None,
            refund_case_count: None,
            rework_count: None,
            avatar_url: None,
            last_login_at: None,
            email_verified_at: None,
            inserted_at: now,
            updated_at: now,
        }
    }

    pub fn activate(&mut self) {
        self.status = Status::Active;
        self.updated_at = Utc::now();
    }

    pub fn deactivate(&mut self) {
        self.status = Status::Inactive;
        self.updated_at = Utc::now();
    }

    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    pub fn is_admin(&self) -> bool {
        self.is_admin.unwrap_or(false)
    }

    pub fn can_be_dispatched(&self) -> bool {
        self.is_active()
            && self.employment_status.is_dispatchable()
            && self.health_status == HealthStatus::Healthy
            && !self.is_admin()
            && self.role == "user"
    }

    pub fn set_admin(&mut self, is_admin: bool) {
        self.is_admin = Some(is_admin);
        if is_admin {
            self.role = "admin".to_string();
        }
        self.updated_at = Utc::now();
    }

    pub fn set_role(&mut self, role: String) {
        self.role = role;
        self.updated_at = Utc::now();
    }

    pub fn set_merchant_uuid(&mut self, merchant_uuid: String) {
        self.merchant_uuid = Some(merchant_uuid);
        self.updated_at = Utc::now();
    }

    pub fn set_employment_status(&mut self, employment_status: EmploymentStatus) {
        self.employment_status = employment_status;
        self.updated_at = Utc::now();
    }

    pub fn update_employee_profile(
        &mut self,
        employment_status: Option<EmploymentStatus>,
        skills: Vec<String>,
        service_areas: Vec<String>,
        training_records: Vec<String>,
        certificates: Vec<String>,
        health_status: Option<HealthStatus>,
        employee_note: Option<String>,
        joined_at: Option<DateTime<Utc>>,
    ) -> Result<(), String> {
        validate_profile_fields(
            &skills,
            &service_areas,
            &training_records,
            &certificates,
            employee_note.as_deref(),
        )?;

        if let Some(employment_status) = employment_status {
            self.employment_status = employment_status;
        }
        self.skills = normalize_list(skills, 12)?;
        self.service_areas = normalize_list(service_areas, 12)?;
        self.training_records = normalize_list(training_records, 12)?;
        self.certificates = normalize_list(certificates, 12)?;
        if let Some(health_status) = health_status {
            self.health_status = health_status;
        }
        self.employee_note = employee_note.and_then(|value| {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        });
        self.joined_at = joined_at;
        self.updated_at = Utc::now();
        Ok(())
    }

    pub fn record_login(&mut self) {
        self.last_login_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn verify_email(&mut self) {
        self.email_verified_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn is_email_verified(&self) -> bool {
        self.email_verified_at.is_some()
    }

    pub fn update_info(
        &mut self,
        user_name: Option<String>,
        email: Option<String>,
        phone_number: Option<String>,
        avatar_url: Option<String>,
    ) {
        if let Some(name) = user_name {
            self.user_name = name;
        }
        if let Some(email) = email {
            self.email = Some(email);
        }
        if let Some(phone) = phone_number {
            self.phone_number = Some(phone);
        }
        if let Some(avatar) = avatar_url {
            self.avatar_url = Some(avatar);
        }
        self.updated_at = Utc::now();
    }

    pub fn change_password(&mut self, new_password: String) {
        self.password = new_password;
        self.updated_at = Utc::now();
    }

    pub fn generate_password_hash(&self, password: &str) -> Result<String, String> {
        Self::hash_password_inner(password)
    }

    pub fn hash_password(password: &str) -> Result<String, String> {
        Self::hash_password_inner(password)
    }

    fn hash_password_inner(password: &str) -> Result<String, String> {
        if password.is_empty() {
            return Err("错误：密码不能为空".to_string());
        }

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("生成密码哈希失败：{}", e))?;

        Ok(hash.to_string())
    }

    pub fn verify_password(&self, password: &str) -> Result<bool, String> {
        if password.is_empty() {
            return Err("错误：待验证的密码不能为空".to_string());
        }
        if self.password.is_empty() {
            return Err("错误：存储的哈希字符串不能为空".to_string());
        }

        let parsed_hash = PasswordHash::new(&self.password)
            .map_err(|e| format!("解析密码哈希字符串失败：{}", e))?;
        let argon2 = Argon2::default();
        let is_match = argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|e| format!("验证密码失败：{}", e))
            .is_ok();

        Ok(is_match)
    }

    #[allow(clippy::too_many_arguments)]
    pub fn from_db_record(
        uuid: String,
        user_name: String,
        password: String,
        email: Option<String>,
        phone_number: Option<String>,
        merchant_uuid: Option<String>,
        role: String,
        is_admin: Option<bool>,
        status: Status,
        employment_status: EmploymentStatus,
        skills: Vec<String>,
        service_areas: Vec<String>,
        training_records: Vec<String>,
        certificates: Vec<String>,
        health_status: HealthStatus,
        employee_note: Option<String>,
        joined_at: Option<DateTime<Utc>>,
        avatar_url: Option<String>,
        last_login_at: Option<DateTime<Utc>>,
        email_verified_at: Option<DateTime<Utc>>,
        inserted_at: DateTime<Utc>,
        updated_at: DateTime<Utc>,
    ) -> Self {
        Self {
            uuid,
            user_name,
            password,
            email,
            phone_number,
            merchant_uuid,
            role,
            is_admin,
            status,
            employment_status,
            skills,
            service_areas,
            training_records,
            certificates,
            health_status,
            employee_note,
            joined_at,
            completed_service_count: None,
            feedback_count: None,
            average_rating: None,
            after_sales_case_count: None,
            complaint_case_count: None,
            refund_case_count: None,
            rework_count: None,
            avatar_url,
            last_login_at,
            email_verified_at,
            inserted_at,
            updated_at,
        }
    }
}

fn validate_profile_fields(
    skills: &[String],
    service_areas: &[String],
    training_records: &[String],
    certificates: &[String],
    employee_note: Option<&str>,
) -> Result<(), String> {
    let _ = normalize_list(skills.to_vec(), 12)?;
    let _ = normalize_list(service_areas.to_vec(), 12)?;
    let _ = normalize_list(training_records.to_vec(), 12)?;
    let _ = normalize_list(certificates.to_vec(), 12)?;
    if let Some(note) = employee_note {
        if note.chars().count() > 500 {
            return Err("employee_note length cannot exceed 500".to_string());
        }
    }
    Ok(())
}

fn normalize_list(values: Vec<String>, limit: usize) -> Result<Vec<String>, String> {
    let mut normalized = Vec::new();
    for value in values {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.chars().count() > 30 {
            return Err("employee profile item length cannot exceed 30".to_string());
        }
        if normalized.iter().any(|item| item == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_string());
    }

    if normalized.len() > limit {
        return Err(format!("employee profile items cannot exceed {}", limit));
    }

    Ok(normalized)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn employee_dispatch_requires_active_account_and_employment() {
        let mut user = User::new("worker".to_string(), "hash".to_string(), None, None);
        user.set_role("user".to_string());
        assert!(user.can_be_dispatched());

        user.set_employment_status(EmploymentStatus::OnLeave);
        assert!(!user.can_be_dispatched());

        user.set_employment_status(EmploymentStatus::Active);
        user.deactivate();
        assert!(!user.can_be_dispatched());
    }

    #[test]
    fn employee_dispatch_requires_healthy_status() {
        let mut user = User::new("worker".to_string(), "hash".to_string(), None, None);
        user.set_role("user".to_string());
        assert!(user.can_be_dispatched());

        user.health_status = HealthStatus::Attention;
        assert!(!user.can_be_dispatched());
    }

    #[test]
    fn employee_profile_normalizes_skill_lists() {
        let mut user = User::new("worker".to_string(), "hash".to_string(), None, None);
        user.update_employee_profile(
            Some(EmploymentStatus::Active),
            vec![
                "保洁".to_string(),
                "保洁".to_string(),
                " 深度保洁 ".to_string(),
            ],
            vec!["朝阳".to_string(), "海淀".to_string()],
            vec!["岗前培训".to_string()],
            vec!["母婴护理证".to_string()],
            Some(HealthStatus::Healthy),
            Some("  备注  ".to_string()),
            None,
        )
        .expect("employee profile should update");

        assert_eq!(
            user.skills,
            vec!["保洁".to_string(), "深度保洁".to_string()]
        );
        assert_eq!(
            user.service_areas,
            vec!["朝阳".to_string(), "海淀".to_string()]
        );
        assert_eq!(user.training_records, vec!["岗前培训".to_string()]);
        assert_eq!(user.certificates, vec!["母婴护理证".to_string()]);
        assert_eq!(user.health_status, HealthStatus::Healthy);
        assert_eq!(user.employee_note.as_deref(), Some("备注"));
    }
}
