use chrono::{DateTime, Utc};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ContactFollowRecord {
    pub uuid: String,
    pub contact_uuid: String,
    pub operator_uuid: Option<String>,
    pub operator_name: Option<String>,
    pub content: String,
    pub next_follow_up_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateContactFollowRecord {
    pub contact_uuid: String,
    pub operator_uuid: Option<String>,
    pub content: String,
    pub next_follow_up_at: Option<DateTime<Utc>>,
}

pub trait ContactFollowRecordRepository: Send + Sync {
    fn create_follow_record(
        &self,
        record: CreateContactFollowRecord,
    ) -> impl std::future::Future<Output = Result<ContactFollowRecord, String>> + Send;
}

pub trait ContactFollowRecordQuery: Send + Sync {
    type Result: Debug + Send + Sync;

    fn list_follow_records(
        &self,
        contact_uuid: String,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Result>, String>> + Send;
}

impl CreateContactFollowRecord {
    pub fn verify(&self) -> Result<(), String> {
        if self.contact_uuid.trim().is_empty() {
            return Err("contact_uuid is required".to_string());
        }
        Uuid::parse_str(self.contact_uuid.trim())
            .map_err(|e| format!("invalid contact_uuid: {}", e))?;

        if let Some(operator_uuid) = self
            .operator_uuid
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            Uuid::parse_str(operator_uuid.trim())
                .map_err(|e| format!("invalid operator_uuid: {}", e))?;
        }

        let content = self.content.trim();
        if content.is_empty() {
            return Err("content is required".to_string());
        }
        if content.chars().count() > 2000 {
            return Err("content length cannot exceed 2000".to_string());
        }

        Ok(())
    }
}

impl ContactFollowRecord {
    pub fn new(
        contact_uuid: String,
        operator_uuid: Option<String>,
        operator_name: Option<String>,
        content: String,
        next_follow_up_at: Option<DateTime<Utc>>,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            contact_uuid,
            operator_uuid,
            operator_name,
            content,
            next_follow_up_at,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_follow_record_requires_contact_uuid_and_content() {
        let err = CreateContactFollowRecord {
            contact_uuid: String::new(),
            operator_uuid: None,
            content: String::new(),
            next_follow_up_at: None,
        }
        .verify()
        .expect_err("empty values should be rejected");

        assert!(err.contains("contact_uuid"));
    }

    #[test]
    fn create_follow_record_rejects_overlong_content() {
        let err = CreateContactFollowRecord {
            contact_uuid: "11111111-1111-1111-1111-111111111111".to_string(),
            operator_uuid: None,
            content: "A".repeat(2001),
            next_follow_up_at: None,
        }
        .verify()
        .expect_err("overlong content should fail");

        assert!(err.contains("content length"));
    }
}
