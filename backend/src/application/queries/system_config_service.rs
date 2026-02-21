use crate::application::mappers::system_config_mapper::to_category_dto;
use crate::domain::repositories::system_config::SystemConfigRepository;
use shared::system_config::SystemConfigCategoryDto;

pub struct SystemConfigQueryService<R: SystemConfigRepository> {
    repo: R,
}

impl<R: SystemConfigRepository> SystemConfigQueryService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn list_configs(&self) -> Result<Vec<SystemConfigCategoryDto>, String> {
        let categories = self.repo.list_categories_with_items().await?;
        Ok(categories.into_iter().map(to_category_dto).collect())
    }
}
