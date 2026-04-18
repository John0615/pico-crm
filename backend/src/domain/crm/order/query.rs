use shared::order::OrderQuery as OrderQueryParams;
use shared::order::OrderChangeLogDto;

pub trait OrderQuery: Send + Sync {
    type Result: std::fmt::Debug + Send + Sync;

    fn list_orders(
        &self,
        query: OrderQueryParams,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send;

    fn get_order(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send;

    fn list_order_change_logs(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Vec<OrderChangeLogDto>, String>> + Send;
}
