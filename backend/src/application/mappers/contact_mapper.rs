use crate::domain::models::contact::{Contact as DomainContact, CustomerStatus, CustomerValue};
use crate::domain::specifications::contact_spec::{SortDirection, SortOption as DomainSortOption};
use chrono::{DateTime, Utc};
use shared::contact::{
    Contact, SortField as SharedSortField, SortOption as SharedSortOption,
    SortOrder as SharedSortOrder,
};

impl From<Contact> for DomainContact {
    fn from(contact: Contact) -> Self {
        let value = match contact.value_level {
            1 => CustomerValue::Active,
            2 => CustomerValue::Potential,
            3 => CustomerValue::Inactive,
            _ => CustomerValue::Inactive,
        };
        let status = match contact.status {
            1 => CustomerStatus::Signed,
            2 => CustomerStatus::Pending,
            3 => CustomerStatus::Churned,
            _ => CustomerStatus::Churned,
        };
        Self {
            uuid: contact.contact_uuid,
            name: contact.user_name,
            company: contact.company,
            position: contact.position,
            phone: contact.phone_number,
            email: contact.email,
            value,
            status,
            last_contact: parse_string_to_utc_time(&contact.last_contact),
            inserted_at: parse_string_to_utc_time(&contact.inserted_at),
            updated_at: parse_string_to_utc_time(&contact.updated_at),
        }
    }
}

impl From<DomainContact> for Contact {
    fn from(contact: DomainContact) -> Self {
        let value_level = match contact.value {
            CustomerValue::Active => 1,
            CustomerValue::Potential => 2,
            CustomerValue::Inactive => 3,
        };
        let status = match contact.status {
            CustomerStatus::Signed => 1,
            CustomerStatus::Pending => 2,
            CustomerStatus::Churned => 3,
        };
        Self {
            contact_uuid: contact.uuid,
            user_name: contact.name,
            company: contact.company,
            position: contact.position,
            phone_number: contact.phone,
            email: contact.email,
            value_level: value_level as i32,
            status: status as i32,
            last_contact: parse_utc_time_to_string(contact.last_contact),
            inserted_at: parse_utc_time_to_string(contact.inserted_at),
            updated_at: parse_utc_time_to_string(contact.updated_at),
        }
    }
}

impl From<SharedSortOption> for DomainSortOption {
    fn from(opt: SharedSortOption) -> Self {
        // 解析排序方向
        let direction = match opt.order {
            SharedSortOrder::Asc => SortDirection::Asc,
            SharedSortOrder::Desc => SortDirection::Desc,
        };

        // 解析排序字段
        match opt.field {
            SharedSortField::Name => Self::ByName(direction),
            SharedSortField::LastContact => Self::ByLastContact(direction),
        }
    }
}

fn parse_utc_time_to_string(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}

fn parse_string_to_utc_time(time_str: &str) -> DateTime<Utc> {
    match DateTime::parse_from_str(time_str, "%Y-%m-%d %H:%M:%S") {
        Ok(dt) => dt.with_timezone(&Utc),
        Err(_) => Utc::now(),
    }
}
