use crate::{
    domain::models::contact::Contact, entity::contacts::ActiveModel as ActiveContactEntity,
    entity::contacts::Model as ContactEntity,
};
use chrono::prelude::{DateTime, Local, NaiveDateTime};
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::Uuid;

pub struct ContactMapper;

impl ContactMapper {
    pub fn to_domain(entity: ContactEntity) -> Contact {
        Contact {
            uuid: entity.contact_uuid.to_string(),
            name: entity.user_name,
            email: entity.email,
            phone: entity.phone_number,
            inserted_at: entity.inserted_at,
            updated_at: entity.updated_at,
        }
    }

    pub fn to_entity(contact: Contact) -> ContactEntity {
        let uuid = Uuid::new_v4();
        let now: DateTime<Local> = Local::now();
        let naive_now: NaiveDateTime = now.naive_local();
        ContactEntity {
            contact_uuid: uuid,
            user_name: contact.name,
            email: contact.email,
            phone_number: contact.phone,
            inserted_at: contact.inserted_at,
            updated_at: contact.updated_at,
            company: String::new(),
            position: String::new(),
            last_contact: naive_now,
            value_level: 1,
            creator_uuid: uuid,
            status: 1,
        }
    }

    pub fn to_active_entity(contact: Contact) -> ActiveContactEntity {
        let uuid = Uuid::new_v4();
        let now: DateTime<Local> = Local::now();
        let naive_now: NaiveDateTime = now.naive_local();
        ActiveContactEntity {
            contact_uuid: Set(uuid),
            user_name: Set(contact.name),
            email: Set(contact.email),
            phone_number: Set(contact.phone),
            inserted_at: Set(contact.inserted_at),
            updated_at: Set(contact.updated_at),
            company: Set(String::new()),
            position: Set(String::new()),
            last_contact: Set(naive_now),
            value_level: Set(1),
            creator_uuid: Set(uuid),
            status: Set(1),
        }
    }
}
