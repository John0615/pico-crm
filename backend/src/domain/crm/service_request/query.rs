use shared::service_request::ServiceRequestQuery as ServiceRequestQueryParams;

pub trait ServiceRequestQuery: Send + Sync {
    type Result: std::fmt::Debug + Send + Sync;

    fn list_requests(
        &self,
        query: ServiceRequestQueryParams,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send;

    fn get_request(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send;
}
