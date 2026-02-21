use crate::domain::queries::service_request::ServiceRequestQuery as DomainServiceRequestQuery;
use shared::service_request::{ServiceRequest, ServiceRequestQuery};
use shared::ListResult;

pub struct ServiceRequestQueryService<R: DomainServiceRequestQuery> {
    query: R,
}

impl<R: DomainServiceRequestQuery<Result = ServiceRequest>> ServiceRequestQueryService<R> {
    pub fn new(query: R) -> Self {
        Self { query }
    }

    pub async fn fetch_requests(
        &self,
        params: ServiceRequestQuery,
    ) -> Result<ListResult<ServiceRequest>, String> {
        let (items, total) = self.query.list_requests(params).await?;
        Ok(ListResult { items, total })
    }

    pub async fn fetch_request(
        &self,
        uuid: String,
    ) -> Result<Option<ServiceRequest>, String> {
        self.query.get_request(uuid).await
    }
}
