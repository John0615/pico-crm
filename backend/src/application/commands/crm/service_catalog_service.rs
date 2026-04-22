use crate::domain::crm::service_catalog::{
    CreateServiceCatalog, ServiceCatalogRepository, UpdateServiceCatalog,
};
use shared::service_catalog::{
    CreateServiceCatalogRequest, ServiceCatalog as SharedServiceCatalog,
    UpdateServiceCatalogRequest,
};

pub struct ServiceCatalogAppService<R: ServiceCatalogRepository> {
    repo: R,
}

impl<R: ServiceCatalogRepository> ServiceCatalogAppService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_service_catalog(
        &self,
        payload: CreateServiceCatalogRequest,
    ) -> Result<SharedServiceCatalog, String> {
        let create = CreateServiceCatalog {
            name: payload.name,
            description: payload.description,
            base_price_cents: payload.base_price_cents,
            default_duration_minutes: payload.default_duration_minutes,
            is_active: payload.is_active,
            sort_order: payload.sort_order.unwrap_or(0),
        };
        create.verify()?;

        let created = self
            .repo
            .create_service_catalog(create.into_domain())
            .await?;
        Ok(created.into())
    }

    pub async fn update_service_catalog(
        &self,
        uuid: String,
        payload: UpdateServiceCatalogRequest,
    ) -> Result<SharedServiceCatalog, String> {
        let update = UpdateServiceCatalog {
            uuid,
            name: payload.name,
            description: payload.description,
            base_price_cents: payload.base_price_cents,
            default_duration_minutes: payload.default_duration_minutes,
            is_active: payload.is_active,
            sort_order: payload.sort_order.unwrap_or(0),
        };
        update.verify()?;

        let updated = self.repo.update_service_catalog(update).await?;
        Ok(updated.into())
    }
}
