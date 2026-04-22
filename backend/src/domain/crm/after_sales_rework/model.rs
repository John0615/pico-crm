use chrono::{DateTime, Utc};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AfterSalesRework {
    pub uuid: String,
    pub case_uuid: String,
    pub assigned_user_uuid: String,
    pub assigned_user_name: Option<String>,
    pub scheduled_start_at: DateTime<Utc>,
    pub scheduled_end_at: DateTime<Utc>,
    pub note: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateAfterSalesRework {
    pub case_uuid: String,
    pub assigned_user_uuid: String,
    pub scheduled_start_at: DateTime<Utc>,
    pub scheduled_end_at: DateTime<Utc>,
    pub note: Option<String>,
}

pub trait AfterSalesReworkRepository: Send + Sync {
    fn create_rework(
        &self,
        rework: CreateAfterSalesRework,
    ) -> impl std::future::Future<Output = Result<AfterSalesRework, String>> + Send;
}

pub trait AfterSalesReworkQuery: Send + Sync {
    type Result: Debug + Send + Sync;

    fn list_reworks(
        &self,
        case_uuid: String,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Result>, String>> + Send;
}

impl CreateAfterSalesRework {
    pub fn verify(&self) -> Result<(), String> {
        if self.case_uuid.trim().is_empty() {
            return Err("case_uuid is required".to_string());
        }
        Uuid::parse_str(self.case_uuid.trim()).map_err(|e| format!("invalid case_uuid: {}", e))?;
        if self.assigned_user_uuid.trim().is_empty() {
            return Err("assigned_user_uuid is required".to_string());
        }
        Uuid::parse_str(self.assigned_user_uuid.trim())
            .map_err(|e| format!("invalid assigned_user_uuid: {}", e))?;
        if self.scheduled_end_at <= self.scheduled_start_at {
            return Err("scheduled_end_at must be after scheduled_start_at".to_string());
        }
        if let Some(note) = self.note.as_ref() {
            if note.chars().count() > 500 {
                return Err("note length cannot exceed 500".to_string());
            }
        }
        Ok(())
    }
}
