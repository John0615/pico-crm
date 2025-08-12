use crate::domain::models::pagination::Pagination;
use crate::domain::specifications::contact_spec::ContactSpecification;
use std::fmt::Debug;

pub trait ContactQuery: Send + Sync {
    type Result: Debug + Send + Sync;

    fn contacts(
        &self,
        spec: ContactSpecification,
        pagination: Pagination,
    ) -> impl std::future::Future<Output = Result<(Vec<Self::Result>, u64), String>> + Send;

    fn all_contacts(
        &self,
        spec: ContactSpecification,
    ) -> impl std::future::Future<Output = Result<Vec<Self::Result>, String>> + Send;

    fn get_contact(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send;
}
