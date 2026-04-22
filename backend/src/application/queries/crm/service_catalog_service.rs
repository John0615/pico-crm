use crate::domain::crm::service_catalog::ServiceCatalogQuery;
use shared::service_catalog::{
    ServiceCatalog as SharedServiceCatalog, ServiceCatalogQuery as ServiceCatalogQueryParams,
};

pub struct ServiceCatalogQueryService<Q: ServiceCatalogQuery<Result = SharedServiceCatalog>> {
    query: Q,
}

impl<Q: ServiceCatalogQuery<Result = SharedServiceCatalog>> ServiceCatalogQueryService<Q> {
    pub fn new(query: Q) -> Self {
        Self { query }
    }

    pub async fn fetch_service_catalogs(
        &self,
        query: ServiceCatalogQueryParams,
    ) -> Result<(Vec<SharedServiceCatalog>, u64), String> {
        self.query.list_service_catalogs(query).await
    }

    pub async fn fetch_service_catalog(
        &self,
        uuid: String,
    ) -> Result<Option<SharedServiceCatalog>, String> {
        self.query.get_service_catalog(uuid).await
    }
}
