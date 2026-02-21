use shared::order::OrderQuery as OrderQueryParams;

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
}
