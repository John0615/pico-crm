use crate::domain::crm::contact::{ContactFollowRecord, CreateContactFollowRecord};
use crate::infrastructure::entity::contact_follow_records::{
    ActiveModel as ActiveContactFollowRecordEntity, Model as ContactFollowRecordEntity,
};
use crate::infrastructure::utils::parse_date_time_to_string;
use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::Uuid;
use shared::contact::ContactFollowRecord as SharedContactFollowRecord;

pub struct ContactFollowRecordMapper;

impl ContactFollowRecordMapper {
    pub fn to_domain(
        entity: ContactFollowRecordEntity,
        operator_name: Option<String>,
    ) -> ContactFollowRecord {
        ContactFollowRecord {
            uuid: entity.uuid.to_string(),
            contact_uuid: entity.contact_uuid.to_string(),
            operator_uuid: entity.operator_uuid.map(|value| value.to_string()),
            operator_name,
            content: entity.content,
            next_follow_up_at: entity.next_follow_up_at,
            created_at: entity.created_at,
        }
    }

    pub fn to_view(
        entity: ContactFollowRecordEntity,
        operator_name: Option<String>,
    ) -> SharedContactFollowRecord {
        SharedContactFollowRecord {
            uuid: entity.uuid.to_string(),
            contact_uuid: entity.contact_uuid.to_string(),
            operator_uuid: entity.operator_uuid.map(|value| value.to_string()),
            operator_name,
            content: entity.content,
            next_follow_up_at: entity.next_follow_up_at.map(parse_date_time_to_string),
            created_at: parse_date_time_to_string(entity.created_at),
        }
    }

    pub fn to_active_entity(
        record: CreateContactFollowRecord,
    ) -> Result<ActiveContactFollowRecordEntity, String> {
        let contact_uuid = Uuid::parse_str(&record.contact_uuid)
            .map_err(|e| format!("invalid contact_uuid: {}", e))?;
        let operator_uuid = record
            .operator_uuid
            .as_deref()
            .filter(|value| !value.trim().is_empty())
            .map(|value| Uuid::parse_str(value).map_err(|e| format!("invalid operator_uuid: {}", e)))
            .transpose()?;

        Ok(ActiveContactFollowRecordEntity {
            uuid: Set(Uuid::new_v4()),
            contact_uuid: Set(contact_uuid),
            operator_uuid: Set(operator_uuid),
            content: Set(record.content.trim().to_string()),
            next_follow_up_at: Set(record.next_follow_up_at),
            created_at: Set(chrono::Utc::now()),
        })
    }
}
