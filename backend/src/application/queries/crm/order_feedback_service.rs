use crate::domain::crm::order::OrderFeedbackQuery;
use shared::order::OrderFeedback as SharedOrderFeedback;

pub struct OrderFeedbackQueryService<Q: OrderFeedbackQuery<Result = SharedOrderFeedback>> {
    query: Q,
}

impl<Q: OrderFeedbackQuery<Result = SharedOrderFeedback>> OrderFeedbackQueryService<Q> {
    pub fn new(query: Q) -> Self {
        Self { query }
    }

    pub async fn fetch_feedbacks(
        &self,
        order_uuid: String,
    ) -> Result<Vec<SharedOrderFeedback>, String> {
        self.query.list_feedbacks(order_uuid).await
    }
}
