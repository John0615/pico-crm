use crate::application::utils::parse_utc_time_to_string;
use crate::domain::models::user::{Status, User as DomainUser};
use shared::user::User;

impl From<DomainUser> for User {
    fn from(user: DomainUser) -> Self {
        let status = match user.status {
            Status::Inactive => "inactive".to_string(),
            Status::Active => "active".to_string(),
        };
        Self {
            uuid: user.uuid,
            user_name: user.user_name,
            email: user.email,
            phone_number: user.phone_number,
            is_admin: user.is_admin,
            status: status,
            avatar_url: user.avatar_url,
            last_login_at: user.last_login_at.map(parse_utc_time_to_string),
            email_verified_at: user.email_verified_at.map(parse_utc_time_to_string),
            inserted_at: parse_utc_time_to_string(user.inserted_at),
            updated_at: parse_utc_time_to_string(user.updated_at),
        }
    }
}
