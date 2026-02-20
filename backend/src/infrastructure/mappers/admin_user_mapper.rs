use crate::domain::models::user::{Status, User};
use crate::infrastructure::entity::admin_users::{ActiveModel, Model};
use sea_orm::{ActiveValue, IntoActiveModel};
use sea_orm::entity::prelude::Uuid;

pub struct AdminUserMapper;

impl AdminUserMapper {
    pub fn to_domain(model: Model) -> User {
        let status = match model.status.as_str() {
            "active" => Status::Active,
            "inactive" => Status::Inactive,
            _ => Status::Inactive,
        };

        User::from_db_record(
            model.uuid.to_string(),
            model.user_name,
            model.password,
            model.email,
            model.phone_number,
            None,
            model.role,
            Some(true),
            status,
            None,
            model.last_login_at,
            None,
            model.inserted_at,
            model.updated_at,
        )
    }

    pub fn to_update_active_entity(user: User, original: Model) -> ActiveModel {
        let mut active_model = original.into_active_model();
        active_model.user_name = ActiveValue::Set(user.user_name);
        active_model.email = ActiveValue::Set(user.email);
        active_model.phone_number = ActiveValue::Set(user.phone_number);
        active_model.role = ActiveValue::Set(user.role);
        active_model.status = ActiveValue::Set(match user.status {
            Status::Active => "active".to_string(),
            Status::Inactive => "inactive".to_string(),
        });
        active_model.last_login_at = ActiveValue::Set(user.last_login_at);
        active_model.updated_at = ActiveValue::Set(user.updated_at);
        active_model
    }

    pub fn to_active_entity(user: User) -> ActiveModel {
        ActiveModel {
            uuid: ActiveValue::Set(Uuid::parse_str(&user.uuid).expect("Invalid UUID")),
            user_name: ActiveValue::Set(user.user_name),
            password: ActiveValue::Set(user.password),
            email: ActiveValue::Set(user.email),
            phone_number: ActiveValue::Set(user.phone_number),
            role: ActiveValue::Set(user.role),
            status: ActiveValue::Set(match user.status {
                Status::Active => "active".to_string(),
                Status::Inactive => "inactive".to_string(),
            }),
            last_login_at: ActiveValue::Set(user.last_login_at),
            inserted_at: ActiveValue::Set(user.inserted_at),
            updated_at: ActiveValue::Set(user.updated_at),
        }
    }
}
