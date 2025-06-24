use crate::domain::repositories::contact::ContactRepository;
use crate::domain::services::contact_service::ContactService;
use shared::{ListResult, contact::Contact};

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
    ) -> Result<ListResult<Contact>, String> {
        let (contacts, total) = self.contact_service.fetch_contacts(page, page_size).await?;
        let contacts: Vec<Contact> = contacts.into_iter().map(|contact| contact.into()).collect();
        Ok(ListResult {
            items: contacts,
            total,
        })
    }

    pub async fn create_contact(&self, contact: Contact) -> Result<(), String> {
        let contact = contact.into();
        let _new_contact = self.contact_service.create_contact(contact).await?;
        Ok(())
    }
}
