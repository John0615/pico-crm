use crate::domain::{models::contact::Contact, repositories::contact::ContactRepository};

pub struct ContactsService<R: ContactRepository> {
    repository: R,
}

impl<R: ContactRepository> ContactsService<R> {
    pub fn new(repository: R) -> Self {
        ContactsService { repository }
    }

    pub async fn create_contact(&self, contact: Contact) -> Result<Contact, String> {
        self.repository.create_contact(contact).await
    }
}
