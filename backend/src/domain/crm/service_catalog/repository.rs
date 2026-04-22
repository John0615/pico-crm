use super::model::{ServiceCatalog, UpdateServiceCatalog};

pub trait ServiceCatalogRepository: Send + Sync {
    fn create_service_catalog(
        &self,
        catalog: ServiceCatalog,
    ) -> impl std::future::Future<Output = Result<ServiceCatalog, String>> + Send;

    fn update_service_catalog(
        &self,
        catalog: UpdateServiceCatalog,
    ) -> impl std::future::Future<Output = Result<ServiceCatalog, String>> + Send;
}
