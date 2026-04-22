use crate::application::utils::parse_utc_time_to_string;
use crate::domain::crm::contact::{
    ContactFollowRecord as DomainContactFollowRecord, CreateContactFollowRecord,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use shared::contact::{
    ContactFollowRecord as SharedContactFollowRecord,
    CreateContactFollowRecordRequest as SharedCreateContactFollowRecordRequest,
};

impl TryFrom<(SharedCreateContactFollowRecordRequest, Option<String>)>
    for CreateContactFollowRecord
{
    type Error = String;

    fn try_from(
        value: (SharedCreateContactFollowRecordRequest, Option<String>),
    ) -> Result<Self, Self::Error> {
        let (payload, operator_uuid) = value;
        Ok(Self {
            contact_uuid: payload.contact_uuid.trim().to_string(),
            operator_uuid: operator_uuid.and_then(|value| {
                let trimmed = value.trim();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed.to_string())
                }
            }),
            content: payload.content.trim().to_string(),
            next_follow_up_at: parse_datetime(payload.next_follow_up_at.as_deref())?,
        })
    }
}

impl From<DomainContactFollowRecord> for SharedContactFollowRecord {
    fn from(value: DomainContactFollowRecord) -> Self {
        Self {
            uuid: value.uuid,
            contact_uuid: value.contact_uuid,
            operator_uuid: value.operator_uuid,
            operator_name: value.operator_name,
            content: value.content,
            next_follow_up_at: value.next_follow_up_at.map(parse_utc_time_to_string),
            created_at: parse_utc_time_to_string(value.created_at),
        }
    }
}

fn parse_datetime(value: Option<&str>) -> Result<Option<DateTime<Utc>>, String> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.trim();
    if value.is_empty() {
        return Ok(None);
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Ok(Some(dt.with_timezone(&Utc)));
    }

    let normalized = value.replace('T', " ");
    if let Ok(dt) = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M:%S") {
        return Ok(Some(Utc.from_utc_datetime(&dt)));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M") {
        return Ok(Some(Utc.from_utc_datetime(&dt)));
    }
    if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%Y-%m-%d") {
        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
            return Ok(Some(Utc.from_utc_datetime(&dt)));
        }
    }

    Err("next_follow_up_at 格式不正确".to_string())
}
