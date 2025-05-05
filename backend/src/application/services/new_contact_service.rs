use crate::domain::repositories::contact::ContactRepository;
use crate::domain::services::contact_service::ContactService;
use shared::contact::{Contact, ContactsResult};

pub struct ContactAppService<R: ContactRepository> {
    contact_service: ContactService<R>,
}

impl<R: ContactRepository> ContactAppService<R> {
    pub fn new(contact_service: ContactService<R>) -> Self {
        Self { contact_service }
    }

    pub async fn fetch_contacts(
        &self,
        page: u64,
        page_size: u64,
    ) -> Result<ContactsResult, String> {
        let (contacts, total) = self.contact_service.fetch_contacts(page, page_size).await?;
        let contacts: Vec<Contact> = contacts.into_iter().map(|contact| contact.into()).collect();
        Ok(ContactsResult { contacts, total })
    }
}
