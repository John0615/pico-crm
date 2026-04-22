use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;

use crate::domain::crm::after_sales::{AfterSalesCase, CreateAfterSalesCase};
use crate::infrastructure::entity::after_sales_cases::{ActiveModel, Model};
use crate::infrastructure::utils::parse_date_time_to_string;
use shared::after_sales::AfterSalesCase as SharedAfterSalesCase;

pub struct AfterSalesCaseMapper;

impl AfterSalesCaseMapper {
    pub fn to_domain(model: Model, operator_name: Option<String>) -> AfterSalesCase {
        AfterSalesCase {
            uuid: model.uuid.to_string(),
            order_uuid: model.order_uuid.to_string(),
            operator_uuid: model.operator_uuid.map(|value| value.to_string()),
            operator_name,
            case_type: model.case_type,
            description: model.description,
            status: model.status,
            refund_amount_cents: model.refund_amount_cents,
            refund_reason: model.refund_reason,
            created_at: model.inserted_at,
        }
    }

    pub fn to_view(model: Model, operator_name: Option<String>) -> SharedAfterSalesCase {
        SharedAfterSalesCase {
            uuid: model.uuid.to_string(),
            order_uuid: model.order_uuid.to_string(),
            operator_uuid: model.operator_uuid.map(|value| value.to_string()),
            operator_name,
            case_type: model.case_type,
            description: model.description,
            status: model.status,
            refund_amount_cents: model.refund_amount_cents,
            refund_reason: model.refund_reason,
            created_at: parse_date_time_to_string(model.inserted_at),
        }
    }

    pub fn to_active_entity(case: CreateAfterSalesCase) -> Result<ActiveModel, String> {
        let order_uuid =
            Uuid::parse_str(&case.order_uuid).map_err(|e| format!("invalid order_uuid: {}", e))?;
        let operator_uuid = case
            .operator_uuid
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|e| format!("invalid operator_uuid: {}", e))?;

        Ok(ActiveModel {
            uuid: Set(Uuid::new_v4()),
            order_uuid: Set(order_uuid),
            operator_uuid: Set(operator_uuid),
            case_type: Set(case.case_type.trim().to_string()),
            description: Set(case.description.trim().to_string()),
            status: Set("open".to_string()),
            refund_amount_cents: Set(None),
            refund_reason: Set(None),
            inserted_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        })
    }
}
