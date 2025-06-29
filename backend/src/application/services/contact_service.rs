use crate::domain::models::pagination::Pagination;
use crate::domain::repositories::contact::ContactRepository;
use crate::domain::services::contact_service::ContactService;
use crate::domain::specifications::contact_spec::{
    ContactFilters, ContactSpecification, SortOption,
};
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
        let sort_options: Vec<SortOption> = params
            .sort
            .unwrap_or_default()
            .into_iter()
            .map(|item| item.into())
            .collect();
        // 构建领域规约
        let pagination =
            Pagination::new(params.page, params.page_size).map_err(|e| e.to_string())?;
        let filters: ContactFilters = params.filters.map(|f| f.into()).unwrap_or_default();
        let spec = ContactSpecification::new(Some(filters), Some(sort_options))
            .map_err(|e| e.to_string())?;
        println!("spec: {:?}", spec);
        let (contacts, total) = self
            .contact_service
            .fetch_contacts(spec, pagination)
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
