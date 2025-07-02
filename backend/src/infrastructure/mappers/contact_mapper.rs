use crate::{
    domain::models::contact::{Contact, CustomerStatus, CustomerValue},
    entity::contacts::ActiveModel as ActiveContactEntity,
    entity::contacts::Model as ContactEntity,
};
use chrono::prelude::{DateTime, NaiveDateTime, TimeZone, Utc};
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::Uuid;

pub struct ContactMapper;

impl ContactMapper {
    pub fn to_domain(entity: ContactEntity) -> Contact {
        let value = match entity.value_level {
            1 => CustomerValue::Active,
            2 => CustomerValue::Potential,
            3 => CustomerValue::Inactive,
            _ => CustomerValue::Active,
        };
        let status = match entity.status {
            1 => CustomerStatus::Signed,
            2 => CustomerStatus::Pending,
            3 => CustomerStatus::Churned,
            _ => CustomerStatus::Signed,
        };
        Contact {
            uuid: entity.contact_uuid.to_string(),
            name: entity.user_name,
            company: entity.company,
            position: entity.position,
            email: entity.email,
            phone: entity.phone_number,
            last_contact: naive_to_utc(entity.last_contact),
            value,
            status,
            inserted_at: naive_to_utc(entity.inserted_at),
            updated_at: naive_to_utc(entity.updated_at),
        }
    }

    pub fn to_entity(contact: Contact) -> ContactEntity {
        let uuid = Uuid::new_v4();
        let value_level = match contact.value {
            CustomerValue::Active => 1,
            CustomerValue::Potential => 2,
            CustomerValue::Inactive => 3,
        };
        let status = match contact.status {
            CustomerStatus::Signed => 1,
            CustomerStatus::Pending => 2,
            CustomerStatus::Churned => 3,
        };
        ContactEntity {
            contact_uuid: uuid,
            user_name: contact.name,
            email: contact.email,
            phone_number: contact.phone,
            inserted_at: utc_to_naive(contact.inserted_at),
            updated_at: utc_to_naive(contact.updated_at),
            company: contact.company,
            position: contact.position,
            last_contact: utc_to_naive(contact.last_contact),
            value_level,
            status,
            creator_uuid: uuid,
        }
    }

    pub fn to_active_entity(contact: Contact) -> ActiveContactEntity {
        let uuid = if contact.uuid.is_empty() {
            Uuid::new_v4()
        } else {
            Uuid::parse_str(&contact.uuid).expect("解析uuid失败！")
        };
        let value_level = match contact.value {
            CustomerValue::Active => 1,
            CustomerValue::Potential => 2,
            CustomerValue::Inactive => 3,
        };
        let status = match contact.status {
            CustomerStatus::Signed => 1,
            CustomerStatus::Pending => 2,
            CustomerStatus::Churned => 3,
        };
        ActiveContactEntity {
            contact_uuid: Set(uuid),
            user_name: Set(contact.name),
            email: Set(contact.email),
            phone_number: Set(contact.phone),
            inserted_at: Set(utc_to_naive(contact.inserted_at)),
            updated_at: Set(utc_to_naive(contact.updated_at)),
            company: Set(contact.company),
            position: Set(contact.position),
            last_contact: Set(utc_to_naive(contact.last_contact)),
            value_level: Set(value_level),
            creator_uuid: Set(uuid),
            status: Set(status),
        }
    }
}

fn naive_to_utc(naive: NaiveDateTime) -> DateTime<Utc> {
    Utc.from_utc_datetime(&naive)
}

fn utc_to_naive(utc: DateTime<Utc>) -> NaiveDateTime {
    utc.naive_utc()
}
