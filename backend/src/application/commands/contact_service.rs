use crate::domain::repositories::contact::ContactRepository;
use shared::contact::{Contact, UpdateContact};

pub struct ContactAppService<R: ContactRepository> {
    contact_repo: R,
}

impl<R: ContactRepository> ContactAppService<R> {
    pub fn new(contact_repo: R) -> Self {
        Self { contact_repo }
    }

    pub async fn create_contact(&self, contact: Contact) -> Result<(), String> {
        let contact = contact.into();
        let _new_contact = self.contact_repo.create_contact(contact).await?;
        Ok(())
    }

    pub async fn update_contact(&self, contact: UpdateContact) -> Result<(), String> {
        let contact = contact.into();
        let _new_contact = self.contact_repo.update_contact(contact).await?;
        Ok(())
    }

    pub async fn delete_contact(&self, uuid: String) -> Result<(), String> {
        let _deleted_contact = self.contact_repo.delete_contact(uuid).await?;
        Ok(())
    }
}
