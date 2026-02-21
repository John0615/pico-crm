use crate::domain::models::system_config::{
    SystemConfigCategory, SystemConfigItem, SystemConfigItemUpdate,
};

pub trait SystemConfigRepository: Send + Sync {
    fn list_categories_with_items(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<SystemConfigCategory>, String>> + Send;

    fn find_items_by_keys(
        &self,
        keys: Vec<String>,
    ) -> impl std::future::Future<Output = Result<Vec<SystemConfigItem>, String>> + Send;

    fn update_items(
        &self,
        updates: Vec<SystemConfigItemUpdate>,
    ) -> impl std::future::Future<Output = Result<Vec<SystemConfigItem>, String>> + Send;
}
