use crate::domain::crm::after_sales_rework::{AfterSalesReworkRepository, CreateAfterSalesRework};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use shared::after_sales::{
    AfterSalesRework as SharedAfterSalesRework, CreateAfterSalesReworkRequest,
};

pub struct AfterSalesReworkAppService<R: AfterSalesReworkRepository> {
    repo: R,
}

impl<R: AfterSalesReworkRepository> AfterSalesReworkAppService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_rework(
        &self,
        case_uuid: String,
        payload: CreateAfterSalesReworkRequest,
    ) -> Result<SharedAfterSalesRework, String> {
        let rework = CreateAfterSalesRework {
            case_uuid,
            assigned_user_uuid: payload.assigned_user_uuid.trim().to_string(),
            scheduled_start_at: parse_required_datetime(
                &payload.scheduled_start_at,
                "scheduled_start_at",
            )?,
            scheduled_end_at: parse_required_datetime(
                &payload.scheduled_end_at,
                "scheduled_end_at",
            )?,
            note: payload.note.and_then(|value| {
                let trimmed = value.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            }),
        };
        rework.verify()?;

        let created = self.repo.create_rework(rework).await?;
        Ok(created.into())
    }
}

fn parse_required_datetime(value: &str, field: &str) -> Result<DateTime<Utc>, String> {
    parse_datetime(value).ok_or_else(|| format!("invalid {}", field))
}

fn parse_datetime(value: &str) -> Option<DateTime<Utc>> {
    if value.trim().is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(date) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
            return Some(Utc.from_utc_datetime(&dt));
        }
    }
    None
}
