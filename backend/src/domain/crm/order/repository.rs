use super::model::{Order, OrderAssignmentUpdate};

pub trait OrderRepository: Send + Sync {
    fn create_order(
        &self,
        order: Order,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send;

    fn find_order(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Order>, String>> + Send;

    fn update_order_status(
        &self,
        uuid: String,
        status: String,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send;

    fn update_order_assignment(
        &self,
        uuid: String,
        update: OrderAssignmentUpdate,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send;

    fn update_order_settlement(
        &self,
        uuid: String,
        settlement_status: String,
        settlement_note: Option<String>,
    ) -> impl std::future::Future<Output = Result<Order, String>> + Send;
}
