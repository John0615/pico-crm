use super::model::{Order, OrderAssignmentUpdate, OrderDetailsUpdate};

pub trait OrderRepository: Send + Sync {
    fn create_order(
        &self,
        order: Order,
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send;

    fn find_order(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Order>, String>> + Send;

    fn update_order_status(
        &self,
        uuid: String,
        status: String,
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send;

    fn update_order_assignment(
        &self,
        uuid: String,
        update: OrderAssignmentUpdate,
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send;

    fn update_order_details(
        &self,
        uuid: String,
        update: OrderDetailsUpdate,
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send;

    fn update_order_settlement(
        &self,
        uuid: String,
        settlement_status: String,
        settlement_note: Option<String>,
        paid_amount_cents: Option<i64>,
        payment_method: Option<String>,
        paid_at: Option<chrono::DateTime<chrono::Utc>>,
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send;

    fn cancel_order(
        &self,
        uuid: String,
        reason: String,
        operator_uuid: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send;
}
