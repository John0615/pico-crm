use crate::domain::specifications::contact_spec::ContactSpecification;
use crate::domain::{
    models::{contact::Contact, pagination::Pagination},
    repositories::contact::ContactRepository,
};

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

    pub async fn fetch_contacts(
        &self,
        spec: ContactSpecification,
        pagination: Pagination,
    ) -> Result<(Vec<Contact>, u64), String> {
        self.repository.contacts(spec, pagination).await
    }

    pub async fn fetch_all_contacts(
        &self,
        spec: ContactSpecification,
    ) -> Result<Vec<Contact>, String> {
        self.repository.all_contacts(spec).await
    }
}
