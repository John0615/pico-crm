use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Contact {
    pub uuid: String,
    pub name: String,
    pub phone: String,
    pub last_contact: DateTime<Utc>,
    pub value: CustomerValue,
    pub status: CustomerStatus,
    pub inserted_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct UpdateContact {
    pub uuid: String,
    pub name: String,
    pub phone: String,
    pub value: CustomerValue,
    pub status: CustomerStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CustomerStatus {
    Signed,
    Pending,
    Churned,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CustomerValue {
    Active,
    Potential,
    Inactive,
}

impl Contact {
    pub fn new(name: String, phone: String, value: CustomerValue, status: CustomerStatus) -> Self {
        let now = Utc::now();
        Self {
            uuid: Uuid::new_v4().to_string(),
            name,
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
        phone: String,
        value_level: i32,
        status: i32,
    ) -> Result<Self, String> {
        let value = Self::parse_customer_value(value_level)?;
        let status = Self::parse_customer_status(status)?;

        Ok(Self::new(name, phone, value, status))
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
        ""
    }

    pub fn phone(&self) -> &str {
        &self.phone
    }
}
