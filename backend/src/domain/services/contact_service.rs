use crate::domain::{models::contact::Contact, repositories::contact::ContactRepository};

pub struct ContactService<R: ContactRepository> {
    repository: R,
}

impl<R: ContactRepository> ContactService<R> {
    pub fn new(repository: R) -> Self {
        ContactService { repository }
    }

    pub async fn create_contact(&self, contact: Contact) -> Result<Contact, String> {
        self.repository.create_contact(contact).await
    }

    pub async fn fetch_contacts(&self, page: u64, page_size: u64) -> Result<Vec<Contact>, String> {
        self.repository.contacts(page, page_size).await
    }
}
