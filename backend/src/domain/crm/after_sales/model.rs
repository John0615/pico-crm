use chrono::{DateTime, Utc};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AfterSalesCase {
    pub uuid: String,
    pub order_uuid: String,
    pub operator_uuid: Option<String>,
    pub operator_name: Option<String>,
    pub case_type: String,
    pub description: String,
    pub status: String,
    pub refund_amount_cents: Option<i64>,
    pub refund_reason: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateAfterSalesCase {
    pub order_uuid: String,
    pub operator_uuid: Option<String>,
    pub case_type: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct UpdateAfterSalesRefund {
    pub case_uuid: String,
    pub refund_amount_cents: Option<i64>,
    pub refund_reason: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AfterSalesCaseRecord {
    pub uuid: String,
    pub case_uuid: String,
    pub operator_uuid: Option<String>,
    pub operator_name: Option<String>,
    pub content: String,
    pub status: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateAfterSalesCaseRecord {
    pub case_uuid: String,
    pub operator_uuid: Option<String>,
    pub content: String,
    pub status: Option<String>,
}

pub trait AfterSalesCaseRepository: Send + Sync {
    fn create_case(
        &self,
        case: CreateAfterSalesCase,
    ) -> impl std::future::Future<Output = Result<AfterSalesCase, String>> + Send;

    fn update_refund(
        &self,
        refund: UpdateAfterSalesRefund,
    ) -> impl std::future::Future<Output = Result<AfterSalesCase, String>> + Send;

    fn create_case_record(
        &self,
        record: CreateAfterSalesCaseRecord,
    ) -> impl std::future::Future<Output = Result<AfterSalesCaseRecord, String>> + Send;
}

pub trait AfterSalesCaseQuery: Send + Sync {
    type Result: Debug + Send + Sync;

    fn list_cases(
        &self,
        order_uuid: String,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Result>, String>> + Send;

    type RecordResult: Debug + Send + Sync;

    fn list_case_records(
        &self,
        case_uuid: String,
    ) -> impl std::future::Future<Output = Result<Vec<Self::RecordResult>, String>> + Send;
}

impl CreateAfterSalesCase {
    pub fn verify(&self) -> Result<(), String> {
        if self.order_uuid.trim().is_empty() {
            return Err("order_uuid is required".to_string());
        }
        Uuid::parse_str(self.order_uuid.trim())
            .map_err(|e| format!("invalid order_uuid: {}", e))?;

        if let Some(operator_uuid) = self
            .operator_uuid
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            Uuid::parse_str(operator_uuid.trim())
                .map_err(|e| format!("invalid operator_uuid: {}", e))?;
        }

        let case_type = self.case_type.trim();
        if case_type.is_empty() {
            return Err("case_type is required".to_string());
        }
        if case_type.chars().count() > 30 {
            return Err("case_type length cannot exceed 30".to_string());
        }

        let description = self.description.trim();
        if description.is_empty() {
            return Err("description is required".to_string());
        }
        if description.chars().count() > 2000 {
            return Err("description length cannot exceed 2000".to_string());
        }

        Ok(())
    }
}

impl UpdateAfterSalesRefund {
    pub fn verify(&self) -> Result<(), String> {
        if self.case_uuid.trim().is_empty() {
            return Err("case_uuid is required".to_string());
        }
        Uuid::parse_str(self.case_uuid.trim()).map_err(|e| format!("invalid case_uuid: {}", e))?;
        if let Some(amount) = self.refund_amount_cents {
            if amount < 0 {
                return Err("refund_amount_cents must be non-negative".to_string());
            }
        }
        if let Some(reason) = self.refund_reason.as_ref() {
            if reason.chars().count() > 500 {
                return Err("refund_reason length cannot exceed 500".to_string());
            }
        }
        Ok(())
    }
}

impl CreateAfterSalesCaseRecord {
    pub fn verify(&self) -> Result<(), String> {
        if self.case_uuid.trim().is_empty() {
            return Err("case_uuid is required".to_string());
        }
        Uuid::parse_str(self.case_uuid.trim()).map_err(|e| format!("invalid case_uuid: {}", e))?;

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

        validate_after_sales_status(self.status.as_deref().unwrap_or("processing"))?;
        Ok(())
    }
}

pub fn validate_after_sales_status(value: &str) -> Result<(), String> {
    match value {
        "open" | "processing" | "resolved" | "closed" => Ok(()),
        _ => Err(format!("invalid after sales status: {}", value)),
    }
}
