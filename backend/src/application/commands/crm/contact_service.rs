use crate::domain::crm::contact::ContactRepository;
use shared::contact::{Contact, UpdateContact};

pub struct ContactAppService<R: ContactRepository> {
    contact_repo: R,
}

impl<R: ContactRepository> ContactAppService<R> {
    pub fn new(contact_repo: R) -> Self {
        Self { contact_repo }
    }

    pub async fn create_contact(&self, contact: Contact) -> Result<(), String> {
        let domain_contact: crate::domain::crm::contact::Contact = contact.try_into()?;

        domain_contact.verify()?;
        self.ensure_phone_number_available(&domain_contact.phone, None)
            .await?;

        let _new_contact = self.contact_repo.create_contact(domain_contact).await?;
        Ok(())
    }

    pub async fn update_contact(&self, contact: UpdateContact) -> Result<(), String> {
        let contact: crate::domain::crm::contact::UpdateContact = contact.try_into()?;
        contact.verify()?;
        self.ensure_phone_number_available(&contact.phone, Some(contact.uuid.as_str()))
            .await?;

        let _new_contact = self.contact_repo.update_contact(contact).await?;
        Ok(())
    }

    pub async fn delete_contact(&self, uuid: String) -> Result<(), String> {
        let _deleted_contact = self.contact_repo.delete_contact(uuid).await?;
        Ok(())
    }

    async fn ensure_phone_number_available(
        &self,
        phone_number: &str,
        current_uuid: Option<&str>,
    ) -> Result<(), String> {
        if let Some(existing) = self
            .contact_repo
            .find_contact_by_phone_number(phone_number)
            .await?
        {
            if current_uuid != Some(existing.uuid.as_str()) {
                return Err("联系电话已存在".to_string());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::crm::contact::{
        Contact as DomainContact, ContactRepository, UpdateContact as DomainUpdateContact,
    };
    use chrono::Utc;
    use std::sync::{Arc, Mutex};

    #[derive(Default)]
    struct FakeContactRepository {
        existing: Arc<Mutex<Option<DomainContact>>>,
        created: Arc<Mutex<Vec<DomainContact>>>,
        updated: Arc<Mutex<Vec<DomainUpdateContact>>>,
    }

    impl ContactRepository for FakeContactRepository {
        fn create_contact(
            &self,
            contact: DomainContact,
        ) -> impl std::future::Future<Output = Result<DomainContact, String>> + Send {
            let created = self.created.clone();
            async move {
                created.lock().expect("lock created").push(contact.clone());
                Ok(contact)
            }
        }

        fn update_contact(
            &self,
            contact: DomainUpdateContact,
        ) -> impl std::future::Future<Output = Result<DomainContact, String>> + Send {
            let updated = self.updated.clone();
            async move {
                updated.lock().expect("lock updated").push(contact.clone());
                Ok(DomainContact {
                    uuid: contact.uuid,
                    name: contact.name,
                    phone: contact.phone,
                    address: contact.address,
                    community: contact.community,
                    building: contact.building,
                    house_area_sqm: contact.house_area_sqm,
                    service_need: contact.service_need,
                    tags: contact.tags,
                    last_service_at: contact.last_service_at,
                    follow_up_status: contact.follow_up_status,
                    inserted_at: Utc::now(),
                    updated_at: Utc::now(),
                })
            }
        }

        fn delete_contact(
            &self,
            _uuid: String,
        ) -> impl std::future::Future<Output = Result<(), String>> + Send {
            async { Ok(()) }
        }

        fn find_contact_by_phone_number(
            &self,
            _phone_number: &str,
        ) -> impl std::future::Future<Output = Result<Option<DomainContact>, String>> + Send
        {
            let existing = self.existing.clone();
            async move { Ok(existing.lock().expect("lock existing").clone()) }
        }
    }

    #[tokio::test]
    async fn create_contact_rejects_duplicate_phone_number() {
        let repo = FakeContactRepository {
            existing: Arc::new(Mutex::new(Some(DomainContact {
                uuid: "existing-contact".to_string(),
                name: "已存在客户".to_string(),
                phone: "13800138000".to_string(),
                address: None,
                community: None,
                building: None,
                house_area_sqm: None,
                service_need: None,
                tags: vec![],
                last_service_at: None,
                follow_up_status: crate::domain::crm::contact::FollowUpStatus::Pending,
                inserted_at: Utc::now(),
                updated_at: Utc::now(),
            }))),
            ..Default::default()
        };
        let service = ContactAppService::new(repo);

        let err = service
            .create_contact(Contact {
                user_name: "新客户".to_string(),
                phone_number: "13800138000".to_string(),
                ..Default::default()
            })
            .await
            .expect_err("duplicate phone should be rejected");

        assert_eq!(err, "联系电话已存在");
    }

    #[tokio::test]
    async fn update_contact_allows_same_phone_for_current_contact() {
        let repo = FakeContactRepository {
            existing: Arc::new(Mutex::new(Some(DomainContact {
                uuid: "contact-1".to_string(),
                name: "当前客户".to_string(),
                phone: "13800138000".to_string(),
                address: None,
                community: None,
                building: None,
                house_area_sqm: None,
                service_need: None,
                tags: vec![],
                last_service_at: None,
                follow_up_status: crate::domain::crm::contact::FollowUpStatus::Pending,
                inserted_at: Utc::now(),
                updated_at: Utc::now(),
            }))),
            ..Default::default()
        };
        let updated = repo.updated.clone();
        let service = ContactAppService::new(repo);

        service
            .update_contact(UpdateContact {
                contact_uuid: "contact-1".to_string(),
                user_name: "当前客户".to_string(),
                phone_number: "13800138000".to_string(),
                ..Default::default()
            })
            .await
            .expect("same contact should be allowed to keep phone");

        assert_eq!(updated.lock().expect("lock updated").len(), 1);
    }
}
