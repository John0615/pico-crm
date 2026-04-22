use crate::domain::crm::order::{CreateOrderFeedback, OrderFeedbackRepository};
use shared::order::{CreateOrderFeedbackRequest, OrderFeedback as SharedOrderFeedback};

pub struct OrderFeedbackAppService<R: OrderFeedbackRepository> {
    repo: R,
}

impl<R: OrderFeedbackRepository> OrderFeedbackAppService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_feedback(
        &self,
        order_uuid: String,
        payload: CreateOrderFeedbackRequest,
        user_uuid: Option<String>,
    ) -> Result<SharedOrderFeedback, String> {
        let feedback: CreateOrderFeedback = (order_uuid, payload, user_uuid).try_into()?;
        feedback.verify()?;

        if let Some(user_uuid) = feedback.user_uuid.clone() {
            if self
                .repo
                .has_feedback_for_order_user(feedback.order_uuid.clone(), user_uuid)
                .await?
            {
                return Err("当前服务人员已提交过该订单反馈".to_string());
            }
        }

        let created = self.repo.create_feedback(feedback).await?;
        Ok(created.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::crm::order::{OrderFeedback, OrderFeedbackRepository};
    use chrono::Utc;
    use std::sync::{Arc, Mutex};

    #[derive(Clone, Default)]
    struct MockOrderFeedbackRepository {
        exists: Arc<Mutex<bool>>,
    }

    impl OrderFeedbackRepository for MockOrderFeedbackRepository {
        fn has_feedback_for_order_user(
            &self,
            _order_uuid: String,
            _user_uuid: String,
        ) -> impl std::future::Future<Output = Result<bool, String>> + Send {
            let exists = self.exists.clone();
            async move { Ok(*exists.lock().expect("exists lock")) }
        }

        fn create_feedback(
            &self,
            feedback: crate::domain::crm::order::CreateOrderFeedback,
        ) -> impl std::future::Future<Output = Result<OrderFeedback, String>> + Send {
            async move {
                Ok(OrderFeedback {
                    uuid: "feedback-1".to_string(),
                    order_uuid: feedback.order_uuid,
                    user_uuid: feedback.user_uuid,
                    user_name: None,
                    rating: feedback.rating,
                    content: feedback.content,
                    created_at: Utc::now(),
                })
            }
        }
    }

    #[tokio::test]
    async fn create_feedback_rejects_duplicate_submission_for_same_worker() {
        let repo = MockOrderFeedbackRepository {
            exists: Arc::new(Mutex::new(true)),
        };
        let service = OrderFeedbackAppService::new(repo);

        let err = service
            .create_feedback(
                "11111111-1111-1111-1111-111111111111".to_string(),
                CreateOrderFeedbackRequest {
                    rating: Some(5),
                    content: "服务完成".to_string(),
                },
                Some("22222222-2222-2222-2222-222222222222".to_string()),
            )
            .await
            .expect_err("duplicate feedback should be rejected");

        assert!(err.contains("已提交过"));
    }
}
