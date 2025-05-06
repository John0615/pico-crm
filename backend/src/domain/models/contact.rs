use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Contact {
    pub uuid: String,
    pub name: String,
    pub company: String,
    pub position: String,
    pub email: String,
    pub phone: String,
    pub last_contact: DateTime<Utc>,
    pub value: CustomerValue,
    pub status: CustomerStatus,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 领域层的状态类型（强类型，避免魔法字符串）
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CustomerStatus {
    Signed,  // 已签约
    Pending, // 待跟进
    Churned, // 已流失
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomerValue {
    Active,    // 活跃客户
    Potential, // 潜在客户
    Inactive,  // 不活跃客户
}

impl Contact {
    pub fn verify(&self) -> Result<(), String> {
        if self.name.is_empty() {
            Err("Name cannot be empty".to_string())
        } else if self.email.is_empty() {
            Err("Email cannot be empty".to_string())
        } else if self.phone.is_empty() {
            Err("Phone cannot be empty".to_string())
        } else {
            Ok(())
        }
    }

    pub fn uuid(&self) -> String {
        self.uuid.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn email(&self) -> &str {
        &self.email
    }

    pub fn phone(&self) -> &str {
        &self.phone
    }
}
