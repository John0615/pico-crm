use crate::domain::models::contacts::Model;

pub trait ContactRepository: Send + Sync {
    fn create_contact(
        &self,
        contact: Model,
    ) -> impl std::future::Future<Output = Result<Model, String>> + Send + Sync;
    fn contacts(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<Model>, String>> + Send + Sync;
    fn get_contact(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Model, String>> + Send + Sync;
    fn update_contact(
        &self,
        uuid: String,
        contact: Model,
    ) -> impl std::future::Future<Output = Result<Model, String>> + Send + Sync;
    fn delete_contact(&self, uuid: String)
    -> impl std::future::Future<Output = Result<(), String>>;
}
