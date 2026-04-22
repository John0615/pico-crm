use std::fmt::Debug;

use shared::service_catalog::ServiceCatalogQuery as ServiceCatalogQueryParams;

pub trait ServiceCatalogQuery: Send + Sync {
    type Result: Debug + Send + Sync;

    fn list_service_catalogs(
        &self,
        query: ServiceCatalogQueryParams,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send;

    fn get_service_catalog(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send;
}
