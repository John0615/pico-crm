use crate::domain::models::{contact::Contact, pagination::Pagination};
use crate::domain::specifications::contact_spec::ContactSpecification;

pub trait ContactRepository: Send + Sync {
    fn create_contact(
        &self,
        contact: Contact,
    ) -> impl std::future::Future<Output = Result<Contact, String>> + Send;

    fn contacts(
        &self,
        spec: ContactSpecification,
        pagination: Pagination,
    ) -> impl std::future::Future<Output = Result<(Vec<Contact>, u64), String>> + Send;

    // fn get_contact(
    //     &self,
    //     uuid: String,
    // ) -> impl std::future::Future<Output = Result<Contact, String>> + Send + Sync;
    // fn update_contact(
    //     &self,
    //     uuid: String,
    //     contact: Contact,
    // ) -> impl std::future::Future<Output = Result<Contact, String>> + Send + Sync;
    // fn delete_contact(&self, uuid: String)
    // -> impl std::future::Future<Output = Result<(), String>>;
}
