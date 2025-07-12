use crate::{
    domain::models::contact::{Contact, CustomerStatus, CustomerValue, UpdateContact},
    infrastructure::entity::contacts::ActiveModel as ActiveContactEntity,
    infrastructure::entity::contacts::Model as ContactEntity,
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

    pub fn to_update_active_entity(
        update_data: UpdateContact,
        original_entity: &ContactEntity,
    ) -> ActiveContactEntity {
        let uuid = Uuid::parse_str(&update_data.uuid).expect("Invalid UUID");
        // 转换枚举字段（仅当更新数据中存在时才覆盖）
        let value_level = match update_data.value {
            CustomerValue::Active => 1,
            CustomerValue::Potential => 2,
            CustomerValue::Inactive => 3,
        };

        let status = match update_data.status {
            CustomerStatus::Signed => 1,
            CustomerStatus::Pending => 2,
            CustomerStatus::Churned => 3,
        };

        // 构建更新后的实体（仅修改提供的字段）
        ActiveContactEntity {
            contact_uuid: Set(uuid),
            user_name: Set(update_data.name),
            email: Set(update_data.email),
            phone_number: Set(update_data.phone),
            inserted_at: Set(original_entity.inserted_at.clone()),
            updated_at: Set(Utc::now().naive_utc()),
            company: Set(update_data.company),
            position: Set(update_data.position),
            last_contact: Set(original_entity.last_contact.clone()),
            value_level: Set(value_level),
            creator_uuid: Set(original_entity.creator_uuid.clone()),
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
