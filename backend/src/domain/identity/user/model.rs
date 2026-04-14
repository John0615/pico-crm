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
            avatar_url,
            last_login_at,
            email_verified_at,
            inserted_at,
            updated_at,
        }
    }
}
