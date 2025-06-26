use crate::domain::repositories::contact::ContactRepository;
use crate::domain::services::contact_service::ContactService;
use shared::{
    ListResult,
    contact::{Contact, ContactQuery},
};

pub struct ContactAppService<R: ContactRepository> {
    contact_service: ContactService<R>,
}

impl<R: ContactRepository> ContactAppService<R> {
    pub fn new(contact_service: ContactService<R>) -> Self {
        Self { contact_service }
    }

    pub async fn fetch_contacts(
        &self,
        params: ContactQuery,
    ) -> Result<ListResult<Contact>, String> {
        let (contacts, total) = self
            .contact_service
            .fetch_contacts(params.page, params.page_size)
            .await?;
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
