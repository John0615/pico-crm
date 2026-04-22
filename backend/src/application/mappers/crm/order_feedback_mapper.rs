use crate::application::utils::parse_utc_time_to_string;
use crate::domain::crm::order::CreateOrderFeedback as DomainCreateOrderFeedback;
use shared::order::{CreateOrderFeedbackRequest, OrderFeedback as SharedOrderFeedback};

impl TryFrom<(String, CreateOrderFeedbackRequest, Option<String>)> for DomainCreateOrderFeedback {
    type Error = String;

    fn try_from(
        value: (String, CreateOrderFeedbackRequest, Option<String>),
    ) -> Result<Self, Self::Error> {
        let (order_uuid, payload, user_uuid) = value;
        Ok(Self {
            order_uuid,
            user_uuid,
            rating: payload.rating,
            content: payload.content,
        })
    }
}

impl From<crate::domain::crm::order::OrderFeedback> for SharedOrderFeedback {
    fn from(value: crate::domain::crm::order::OrderFeedback) -> Self {
        Self {
            uuid: value.uuid,
            order_uuid: value.order_uuid,
            user_uuid: value.user_uuid,
            user_name: value.user_name,
            rating: value.rating,
            content: value.content,
            created_at: parse_utc_time_to_string(value.created_at),
        }
    }
}
