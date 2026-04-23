use sea_orm::ActiveValue::Set;
use sea_orm::entity::prelude::*;

use crate::domain::crm::order::{CreateOrderFeedback, OrderFeedback};
use crate::infrastructure::entity::order_feedback::{ActiveModel, Model};
use crate::infrastructure::utils::parse_date_time_to_string;
use shared::order::OrderFeedback as SharedOrderFeedback;

pub struct OrderFeedbackMapper;

impl OrderFeedbackMapper {
    pub fn to_domain(model: Model, user_name: Option<String>) -> OrderFeedback {
        OrderFeedback {
            uuid: model.uuid.to_string(),
            order_uuid: model.order_id.to_string(),
            user_uuid: model.user_uuid.map(|value| value.to_string()),
            user_name,
            rating: model.rating,
            content: model.content.unwrap_or_default(),
            created_at: model.inserted_at,
        }
    }

    pub fn to_view(model: Model, user_name: Option<String>) -> SharedOrderFeedback {
        SharedOrderFeedback {
            uuid: model.uuid.to_string(),
            order_uuid: model.order_id.to_string(),
            user_uuid: model.user_uuid.map(|value| value.to_string()),
            user_name,
            rating: model.rating,
            content: model.content.unwrap_or_default(),
            created_at: parse_date_time_to_string(model.inserted_at),
        }
    }

    pub fn to_active_entity(feedback: CreateOrderFeedback) -> Result<ActiveModel, String> {
        let order_uuid = Uuid::parse_str(&feedback.order_uuid)
            .map_err(|e| format!("invalid order_uuid: {}", e))?;
        let user_uuid = feedback
            .user_uuid
            .as_deref()
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|e| format!("invalid user_uuid: {}", e))?;

        Ok(ActiveModel {
            uuid: Set(Uuid::new_v4()),
            merchant_id: Set(None),
            order_id: Set(order_uuid),
            worker_id: Set(None),
            user_uuid: Set(user_uuid),
            rating: Set(feedback.rating),
            content: Set(Some(feedback.content)),
            inserted_at: Set(chrono::Utc::now()),
            updated_at: Set(chrono::Utc::now()),
        })
    }
}
