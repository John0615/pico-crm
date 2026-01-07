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
    pub is_admin: Option<bool>,
    pub status: Status,
    pub avatar_url: Option<String>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub email_verified_at: Option<DateTime<Utc>>,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl User {
    /// 创建新用户
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
            is_admin: Some(false),
            status: Status::Inactive,
            avatar_url: None,
            last_login_at: None,
            email_verified_at: None,
            inserted_at: now,
            updated_at: now,
        }
    }

    /// 激活用户
    pub fn activate(&mut self) {
        self.status = Status::Active;
        self.updated_at = Utc::now();
    }

    /// 禁用用户
    pub fn deactivate(&mut self) {
        self.status = Status::Inactive;
        self.updated_at = Utc::now();
    }

    /// 检查用户是否活跃
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    /// 检查用户是否为管理员
    pub fn is_admin(&self) -> bool {
        self.is_admin.unwrap_or(false)
    }

    /// 设置管理员权限
    pub fn set_admin(&mut self, is_admin: bool) {
        self.is_admin = Some(is_admin);
        self.updated_at = Utc::now();
    }

    /// 记录登录时间
    pub fn record_login(&mut self) {
        self.last_login_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// 验证邮箱
    pub fn verify_email(&mut self) {
        self.email_verified_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    /// 检查邮箱是否已验证
    pub fn is_email_verified(&self) -> bool {
        self.email_verified_at.is_some()
    }

    /// 更新用户信息
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

    /// 修改密码
    pub fn change_password(&mut self, new_password: String) {
        self.password = new_password;
        self.updated_at = Utc::now();
    }

    pub fn generate_password_hash(&self, password: &str) -> Result<String, String> {
        // 校验密码是否为空
        if password.is_empty() {
            return Err("错误：密码不能为空".to_string());
        }

        // 生成加密安全的随机盐值
        let salt = SaltString::generate(&mut OsRng);

        // 使用默认的 Argon2id 配置（安全且通用）
        let argon2 = Argon2::default();

        // 计算密码哈希并处理错误
        let hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| format!("生成密码哈希失败：{}", e))?;

        // 转换为字符串返回（包含盐值、参数等所有信息）
        Ok(hash.to_string())
    }

    pub fn verify_password(&self, password: &str) -> Result<bool, String> {
        // 校验输入是否为空
        if password.is_empty() {
            return Err("错误：待验证的密码不能为空".to_string());
        }
        if self.password.is_empty() {
            return Err("错误：存储的哈希字符串不能为空".to_string());
        }

        // 解析存储的哈希字符串（自动提取盐值、算法参数等）
        let parsed_hash = PasswordHash::new(&self.password)
            .map_err(|e| format!("解析密码哈希字符串失败：{}", e))?;

        // 使用默认配置验证密码
        let argon2 = Argon2::default();

        // 验证密码是否匹配并处理错误
        let is_match = argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .map_err(|e| format!("验证密码失败：{}", e))
            .is_ok();

        Ok(is_match)
    }

    /// 从数据库记录创建用户
    pub fn from_db_record(
        uuid: String,
        user_name: String,
        password: String,
        email: Option<String>,
        phone_number: Option<String>,
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

/// 用户创建请求
#[derive(Debug, Clone)]
pub struct CreateUserRequest {
    pub user_name: String,
    pub password: String,
    pub email: Option<String>,
    pub phone_number: Option<String>,
}

impl CreateUserRequest {
    /// 转换为User实体
    pub fn to_user(self) -> User {
        User::new(self.user_name, self.password, self.email, self.phone_number)
    }
}
