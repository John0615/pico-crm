use chrono::{DateTime, Utc};
use uuid::Uuid;

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
    // pub communications: Vec<Communication>,
    // pub deals: Vec<Deal>,
}

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Communication {
//     id: CommunicationId,
//     content: String,
//     created_at: DateTime<Utc>,
//     #[serde(skip)]
//     sentiment_score: Option<f32>,
// }

// #[derive(Debug, Clone, Serialize, Deserialize)]
// pub struct Deal {
//     id: DealId,
//     amount: f64,
//     stage: DealStage,
//     closed_at: Option<DateTime<Utc>>,
// }

#[derive(Debug, Clone)]
pub struct UpdateContact {
    pub uuid: String,
    pub name: String,
    pub company: String,
    pub position: String,
    pub phone: String,
    pub email: String,
    pub value: CustomerValue,
    pub status: CustomerStatus,
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
    pub fn new(
        name: String,
        company: String,
        position: String,
        email: String,
        phone: String,
        value: CustomerValue,
        status: CustomerStatus,
    ) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4().to_string(),
            name,
            company,
            position,
            email,
            phone,
            last_contact: now,
            value,
            status,
            inserted_at: now,
            updated_at: now,
        }
    }

    pub fn from_shared_data(
        name: String,
        company: String,
        position: String,
        email: String,
        phone: String,
        value_level: i32,
        status: i32,
    ) -> Result<Self, String> {
        let value = Self::parse_customer_value(value_level)?;
        let status = Self::parse_customer_status(status)?;
        
        Ok(Self::new(name, company, position, email, phone, value, status))
    }

    pub fn parse_customer_value(value_level: i32) -> Result<CustomerValue, String> {
        match value_level {
            1 => Ok(CustomerValue::Active),
            2 => Ok(CustomerValue::Potential),
            3 => Ok(CustomerValue::Inactive),
            _ => Err(format!("Invalid customer value: {}", value_level)),
        }
    }

    pub fn parse_customer_status(status: i32) -> Result<CustomerStatus, String> {
        match status {
            1 => Ok(CustomerStatus::Signed),
            2 => Ok(CustomerStatus::Pending),
            3 => Ok(CustomerStatus::Churned),
            _ => Err(format!("Invalid customer status: {}", status)),
        }
    }

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
