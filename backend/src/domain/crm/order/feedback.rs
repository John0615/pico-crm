use chrono::{DateTime, Utc};
use std::fmt::Debug;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct OrderFeedback {
    pub uuid: String,
    pub order_uuid: String,
    pub user_uuid: Option<String>,
    pub user_name: Option<String>,
    pub rating: Option<i32>,
    pub content: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct CreateOrderFeedback {
    pub order_uuid: String,
    pub user_uuid: Option<String>,
    pub rating: Option<i32>,
    pub content: String,
}

pub trait OrderFeedbackRepository: Send + Sync {
    fn has_feedback_for_order_user(
        &self,
        order_uuid: String,
        user_uuid: String,
    ) -> impl std::future::Future<Output = Result<bool, String>> + Send;

    fn create_feedback(
        &self,
        feedback: CreateOrderFeedback,
    ) -> impl std::future::Future<Output = Result<OrderFeedback, String>> + Send;
}

pub trait OrderFeedbackQuery: Send + Sync {
    type Result: Debug + Send + Sync;

    fn list_feedbacks(
        &self,
        order_uuid: String,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Result>, String>> + Send;
}

impl CreateOrderFeedback {
    pub fn verify(&self) -> Result<(), String> {
        if self.order_uuid.trim().is_empty() {
            return Err("order_uuid is required".to_string());
        }
        Uuid::parse_str(self.order_uuid.trim())
            .map_err(|e| format!("invalid order_uuid: {}", e))?;

        if let Some(user_uuid) = self
            .user_uuid
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        {
            Uuid::parse_str(user_uuid.trim()).map_err(|e| format!("invalid user_uuid: {}", e))?;
        }

        let content = self.content.trim();
        if content.is_empty() {
            return Err("content is required".to_string());
        }
        if content.chars().count() > 2000 {
            return Err("content length cannot exceed 2000".to_string());
        }

        if let Some(rating) = self.rating {
            if !(1..=5).contains(&rating) {
                return Err("rating must be between 1 and 5".to_string());
            }
        }

        Ok(())
    }
}

impl OrderFeedback {
    pub fn new(
        order_uuid: String,
        user_uuid: Option<String>,
        user_name: Option<String>,
        rating: Option<i32>,
        content: String,
    ) -> Self {
        Self {
            uuid: Uuid::new_v4().to_string(),
            order_uuid,
            user_uuid,
            user_name,
            rating,
            content,
            created_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_feedback_requires_order_uuid_and_content() {
        let err = CreateOrderFeedback {
            order_uuid: String::new(),
            user_uuid: None,
            rating: Some(5),
            content: String::new(),
        }
        .verify()
        .expect_err("empty values should be rejected");

        assert!(err.contains("order_uuid"));
    }

    #[test]
    fn create_feedback_rejects_invalid_rating() {
        let err = CreateOrderFeedback {
            order_uuid: "11111111-1111-1111-1111-111111111111".to_string(),
            user_uuid: None,
            rating: Some(6),
            content: "服务完成".to_string(),
        }
        .verify()
        .expect_err("invalid rating should fail");

        assert!(err.contains("rating"));
    }
}
