use crate::application::utils::parse_utc_time_to_string;
use crate::domain::crm::contact::{
    Contact as DomainContact, ContactFilters, FollowUpStatus, SortDirection,
    SortOption as DomainSortOption, UpdateContact as DomainUpdateContact,
};
use chrono::{DateTime, NaiveDate, NaiveDateTime, TimeZone, Utc};
use shared::contact::{
    Contact, ContactFilters as SharedContactFilters, SortField as SharedSortField,
    SortOption as SharedSortOption, SortOrder as SharedSortOrder,
    UpdateContact as SharedUpdateContact,
};

impl TryFrom<Contact> for DomainContact {
    type Error = String;

    fn try_from(contact: Contact) -> Result<Self, Self::Error> {
        let now = Utc::now();

        Ok(Self {
            uuid: if contact.contact_uuid.trim().is_empty() {
                uuid::Uuid::new_v4().to_string()
            } else {
                contact.contact_uuid
            },
            name: contact.user_name.trim().to_string(),
            phone: contact.phone_number.trim().to_string(),
            address: trim_to_option(contact.address),
            community: trim_to_option(contact.community),
            building: trim_to_option(contact.building),
            house_area_sqm: contact.house_area_sqm,
            service_need: trim_to_option(contact.service_need),
            tags: normalize_tags(contact.tags),
            last_service_at: parse_datetime(contact.last_service_at.as_deref()),
            follow_up_status: parse_follow_up_status(contact.follow_up_status.as_deref())?,
            inserted_at: now,
            updated_at: now,
        })
    }
}

impl TryFrom<SharedUpdateContact> for DomainUpdateContact {
    type Error = String;

    fn try_from(contact: SharedUpdateContact) -> Result<Self, Self::Error> {
        Ok(Self {
            uuid: contact.contact_uuid,
            name: contact.user_name.trim().to_string(),
            phone: contact.phone_number.trim().to_string(),
            address: trim_to_option(contact.address),
            community: trim_to_option(contact.community),
            building: trim_to_option(contact.building),
            house_area_sqm: contact.house_area_sqm,
            service_need: trim_to_option(contact.service_need),
            tags: normalize_tags(contact.tags),
            last_service_at: parse_datetime(contact.last_service_at.as_deref()),
            follow_up_status: parse_follow_up_status(contact.follow_up_status.as_deref())?,
        })
    }
}

impl From<DomainContact> for Contact {
    fn from(contact: DomainContact) -> Self {
        Self {
            contact_uuid: contact.uuid,
            user_name: contact.name,
            phone_number: contact.phone,
            address: contact.address,
            community: contact.community,
            building: contact.building,
            house_area_sqm: contact.house_area_sqm,
            service_need: contact.service_need,
            tags: contact.tags,
            last_service_at: contact.last_service_at.map(parse_utc_time_to_string),
            follow_up_status: Some(contact.follow_up_status.as_str().to_string()),
            after_sales_case_count: None,
            complaint_case_count: None,
            refund_case_count: None,
            rework_count: None,
            inserted_at: parse_utc_time_to_string(contact.inserted_at),
            updated_at: parse_utc_time_to_string(contact.updated_at),
        }
    }
}

impl From<SharedSortOption> for DomainSortOption {
    fn from(opt: SharedSortOption) -> Self {
        let direction = match opt.order {
            SharedSortOrder::Asc => SortDirection::Asc,
            SharedSortOrder::Desc => SortDirection::Desc,
        };

        match opt.field {
            SharedSortField::Name => Self::ByName(direction),
        }
    }
}

impl From<SharedContactFilters> for ContactFilters {
    fn from(filters: SharedContactFilters) -> Self {
        Self {
            name: trim_to_option(filters.user_name),
            phone: trim_to_option(filters.phone_number),
            address_keyword: trim_to_option(filters.address_keyword),
            tag: trim_to_option(filters.tag),
            follow_up_status: trim_to_option(filters.follow_up_status),
        }
    }
}

fn parse_follow_up_status(value: Option<&str>) -> Result<FollowUpStatus, String> {
    let raw = value.unwrap_or("pending").trim();
    if raw.is_empty() {
        return Ok(FollowUpStatus::Pending);
    }
    FollowUpStatus::parse(raw)
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for tag in tags {
        let trimmed = tag.trim();
        if trimmed.is_empty() {
            continue;
        }
        if normalized.iter().any(|item| item == trimmed) {
            continue;
        }
        normalized.push(trimmed.to_string());
    }
    normalized
}

fn trim_to_option(value: Option<String>) -> Option<String> {
    value.and_then(|value| {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    })
}

fn parse_datetime(value: Option<&str>) -> Option<DateTime<Utc>> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    if let Ok(dt) = DateTime::parse_from_rfc3339(value) {
        return Some(dt.with_timezone(&Utc));
    }

    let normalized = value.replace('T', " ");
    if let Ok(dt) = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M:%S") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(dt) = NaiveDateTime::parse_from_str(&normalized, "%Y-%m-%d %H:%M") {
        return Some(Utc.from_utc_datetime(&dt));
    }
    if let Ok(date) = NaiveDate::parse_from_str(&normalized, "%Y-%m-%d") {
        if let Some(dt) = date.and_hms_opt(0, 0, 0) {
            return Some(Utc.from_utc_datetime(&dt));
        }
    }
    None
}
