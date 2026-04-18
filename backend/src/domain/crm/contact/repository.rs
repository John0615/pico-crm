use super::model::{Contact, UpdateContact};

pub trait ContactRepository: Send + Sync {
    fn create_contact(
        &self,
        contact: Contact,
    ) -> impl std::future::Future<Output = Result<Contact, String>> + Send;

    fn update_contact(
        &self,
        contact: UpdateContact,
    ) -> impl std::future::Future<Output = Result<Contact, String>> + Send;

    fn delete_contact(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    fn find_contact_by_phone_number(
        &self,
        phone_number: &str,
    ) -> impl std::future::Future<Output = Result<Option<Contact>, String>> + Send;
}
