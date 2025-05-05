use crate::domain::models::contact::{Contact as DomainContact, CustomerStatus, CustomerValue};
use chrono::{DateTime, Utc};
use shared::contact::Contact;

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

fn parse_utc_time_to_string(time: DateTime<Utc>) -> String {
    time.format("%Y-%m-%d %H:%M:%S").to_string()
}
