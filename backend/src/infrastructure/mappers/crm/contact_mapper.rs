use crate::infrastructure::utils::parse_date_time_to_string;
use crate::{
    domain::crm::contact::{Contact, FollowUpStatus, UpdateContact},
    infrastructure::entity::contacts::ActiveModel as ActiveContactEntity,
    infrastructure::entity::contacts::Model as ContactEntity,
};
use chrono::prelude::Utc;
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::{Json, Uuid};
use serde_json::{Value, json};
use shared::contact::Contact as SharedContact;

pub struct ContactMapper;

impl ContactMapper {
    pub fn to_view(entity: ContactEntity) -> SharedContact {
        SharedContact {
            contact_uuid: entity.contact_uuid.to_string(),
            user_name: entity.user_name,
            phone_number: entity.phone_number,
            address: entity.address,
            community: entity.community,
            building: entity.building,
            house_area_sqm: entity.house_area_sqm,
            service_need: entity.service_need,
            tags: json_to_tags(&entity.tags),
            last_service_at: entity.last_service_at.map(parse_date_time_to_string),
            follow_up_status: Some(entity.follow_up_status),
            after_sales_case_count: None,
            complaint_case_count: None,
            refund_case_count: None,
            rework_count: None,
            inserted_at: parse_date_time_to_string(entity.inserted_at),
            updated_at: parse_date_time_to_string(entity.updated_at),
        }
    }

    pub fn to_domain(entity: ContactEntity) -> Contact {
        let follow_up_status =
            FollowUpStatus::parse(&entity.follow_up_status).unwrap_or(FollowUpStatus::Pending);

        Contact {
            uuid: entity.contact_uuid.to_string(),
            name: entity.user_name,
            phone: entity.phone_number,
            address: entity.address,
            community: entity.community,
            building: entity.building,
            house_area_sqm: entity.house_area_sqm,
            service_need: entity.service_need,
            tags: json_to_tags(&entity.tags),
            last_service_at: entity.last_service_at,
            follow_up_status,
            inserted_at: entity.inserted_at,
            updated_at: entity.updated_at,
        }
    }

    pub fn to_active_entity(contact: Contact) -> ActiveContactEntity {
        let uuid = Uuid::parse_str(&contact.uuid).unwrap_or_else(|_| Uuid::new_v4());

        ActiveContactEntity {
            contact_uuid: Set(uuid),
            user_name: Set(contact.name),
            phone_number: Set(contact.phone),
            address: Set(contact.address),
            community: Set(contact.community),
            building: Set(contact.building),
            house_area_sqm: Set(contact.house_area_sqm),
            service_need: Set(contact.service_need),
            tags: Set(Json::from(json!(contact.tags))),
            last_service_at: Set(contact.last_service_at),
            follow_up_status: Set(contact.follow_up_status.as_str().to_string()),
            inserted_at: Set(contact.inserted_at),
            updated_at: Set(contact.updated_at),
            creator_uuid: Set(uuid),
        }
    }

    pub fn to_update_active_entity(
        update_data: UpdateContact,
        original_entity: &ContactEntity,
    ) -> ActiveContactEntity {
        let uuid = Uuid::parse_str(&update_data.uuid).expect("Invalid UUID");

        ActiveContactEntity {
            contact_uuid: Set(uuid),
            user_name: Set(update_data.name),
            phone_number: Set(update_data.phone),
            address: Set(update_data.address),
            community: Set(update_data.community),
            building: Set(update_data.building),
            house_area_sqm: Set(update_data.house_area_sqm),
            service_need: Set(update_data.service_need),
            tags: Set(Json::from(json!(update_data.tags))),
            last_service_at: Set(update_data.last_service_at),
            follow_up_status: Set(update_data.follow_up_status.as_str().to_string()),
            inserted_at: Set(original_entity.inserted_at),
            updated_at: Set(Utc::now()),
            creator_uuid: Set(original_entity.creator_uuid),
        }
    }
}

fn json_to_tags(value: &Json) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| item.as_str())
            .map(ToString::to_string)
            .collect(),
        _ => Vec::new(),
    }
}
