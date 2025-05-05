use crate::domain::models::contact::Contact as DomainContact;
use shared::contact::Contact;

impl From<DomainContact> for Contact {
    fn from(_contact: DomainContact) -> Self {
        Self {
            contact_uuid: String::new(),
            user_name: String::new(),
            company: String::new(),
            position: String::new(),
            phone_number: String::new(),
            email: String::new(),
            last_contact: String::new(),
            value_level: 1,
            status: 1,
            inserted_at: String::new(),
            updated_at: String::new(),
        }
    }
}
