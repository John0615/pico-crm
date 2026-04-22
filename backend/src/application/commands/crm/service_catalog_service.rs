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

    pub async fn delete_service_catalog(&self, uuid: String) -> Result<(), String> {
        // First check if the catalog is in use
        let in_use = self.repo.is_service_catalog_in_use(uuid.clone()).await?;
        if in_use {
            return Err("该服务项目已被服务需求使用，无法删除".to_string());
        }

        self.repo.delete_service_catalog(uuid).await?;
        Ok(())
    }
}
