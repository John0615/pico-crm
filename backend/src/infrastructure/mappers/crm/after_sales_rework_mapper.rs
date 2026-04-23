use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;

use crate::domain::crm::after_sales_rework::{AfterSalesRework, CreateAfterSalesRework};
use crate::infrastructure::entity::after_sales_reworks::{ActiveModel, Model};
use crate::infrastructure::utils::parse_date_time_to_string;
use shared::after_sales::AfterSalesRework as SharedAfterSalesRework;

pub struct AfterSalesReworkMapper;

impl AfterSalesReworkMapper {
    pub fn to_domain(model: Model, assigned_user_name: Option<String>) -> AfterSalesRework {
        AfterSalesRework {
            uuid: model.uuid.to_string(),
            case_uuid: model.case_uuid.to_string(),
            assigned_user_uuid: model.assigned_user_uuid.to_string(),
            assigned_user_name,
            scheduled_start_at: model.scheduled_start_at,
            scheduled_end_at: model.scheduled_end_at,
            note: model.note,
            status: model.status,
            created_at: model.inserted_at,
        }
    }

    pub fn to_view(model: Model, assigned_user_name: Option<String>) -> SharedAfterSalesRework {
        SharedAfterSalesRework {
            uuid: model.uuid.to_string(),
            case_uuid: model.case_uuid.to_string(),
            assigned_user_uuid: model.assigned_user_uuid.to_string(),
            assigned_user_name,
            scheduled_start_at: parse_date_time_to_string(model.scheduled_start_at),
            scheduled_end_at: parse_date_time_to_string(model.scheduled_end_at),
            note: model.note,
            status: model.status,
            created_at: parse_date_time_to_string(model.inserted_at),
        }
    }

    pub fn to_active_entity(rework: CreateAfterSalesRework) -> Result<ActiveModel, String> {
        let case_uuid =
            Uuid::parse_str(&rework.case_uuid).map_err(|e| format!("invalid case_uuid: {}", e))?;
        let assigned_user_uuid = Uuid::parse_str(&rework.assigned_user_uuid)
            .map_err(|e| format!("invalid assigned_user_uuid: {}", e))?;

        Ok(ActiveModel {
            uuid: Set(Uuid::new_v4()),
            merchant_id: Set(None),
            case_uuid: Set(case_uuid),
            assigned_user_uuid: Set(assigned_user_uuid),
            scheduled_start_at: Set(rework.scheduled_start_at),
            scheduled_end_at: Set(rework.scheduled_end_at),
            note: Set(rework.note),
            status: Set("planned".to_string()),
            inserted_at: Set(chrono::Utc::now()),
        })
    }
}
