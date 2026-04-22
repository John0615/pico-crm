use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;

use crate::domain::crm::after_sales::{AfterSalesCaseRecord, CreateAfterSalesCaseRecord};
use crate::infrastructure::entity::after_sales_case_records::{ActiveModel, Model};
use crate::infrastructure::utils::parse_date_time_to_string;
use shared::after_sales::AfterSalesCaseRecord as SharedAfterSalesCaseRecord;

pub struct AfterSalesCaseRecordMapper;

impl AfterSalesCaseRecordMapper {
    pub fn to_domain(model: Model, operator_name: Option<String>) -> AfterSalesCaseRecord {
        AfterSalesCaseRecord {
            uuid: model.uuid.to_string(),
            case_uuid: model.case_uuid.to_string(),
            operator_uuid: model.operator_uuid.map(|value| value.to_string()),
            operator_name,
            content: model.content,
            status: model.status,
            created_at: model.inserted_at,
        }
    }

    pub fn to_view(model: Model, operator_name: Option<String>) -> SharedAfterSalesCaseRecord {
        SharedAfterSalesCaseRecord {
            uuid: model.uuid.to_string(),
            case_uuid: model.case_uuid.to_string(),
            operator_uuid: model.operator_uuid.map(|value| value.to_string()),
            operator_name,
            content: model.content,
            status: model.status,
            created_at: parse_date_time_to_string(model.inserted_at),
        }
    }

    pub fn to_active_entity(record: CreateAfterSalesCaseRecord) -> Result<ActiveModel, String> {
        let case_uuid =
            Uuid::parse_str(&record.case_uuid).map_err(|e| format!("invalid case_uuid: {}", e))?;
        let operator_uuid = record
            .operator_uuid
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|e| format!("invalid operator_uuid: {}", e))?;

        Ok(ActiveModel {
            uuid: Set(Uuid::new_v4()),
            case_uuid: Set(case_uuid),
            operator_uuid: Set(operator_uuid),
            content: Set(record.content.trim().to_string()),
            status: Set(record
                .status
                .unwrap_or_else(|| "processing".to_string())
                .trim()
                .to_string()),
            inserted_at: Set(chrono::Utc::now()),
        })
    }
}
