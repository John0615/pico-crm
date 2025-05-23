use crate::domain::models::contact::Contact;

pub trait ContactRepository: Send + Sync {
    fn create_contact(
        &self,
        contact: Contact,
    ) -> impl std::future::Future<Output = Result<Contact, String>> + Send;

    fn contacts(
        &self,
        page: u64,
        page_size: u64,
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
