use crate::domain::identity::user::{Status, User};
use crate::infrastructure::entity::users::{ActiveModel, Model};
use sea_orm::entity::prelude::*;
use sea_orm::{ActiveValue, IntoActiveModel};

pub struct UserMapper;

impl UserMapper {
    /// 将domain User转换为entity ActiveModel
    pub fn to_active_entity(user: User) -> ActiveModel {
        ActiveModel {
            uuid: ActiveValue::Set(Uuid::parse_str(&user.uuid).expect("Invalid UUID")),
            user_name: ActiveValue::Set(user.user_name),
            password: ActiveValue::Set(user.password),
            email: ActiveValue::Set(user.email),
            phone_number: ActiveValue::Set(user.phone_number),
            merchant_uuid: ActiveValue::Set(
                user.merchant_uuid
                    .as_ref()
                    .map(|value| Uuid::parse_str(value).expect("Invalid merchant UUID")),
            ),
            role: ActiveValue::Set(user.role),
            is_admin: ActiveValue::Set(user.is_admin),
            status: ActiveValue::Set(match user.status {
                Status::Active => "active".to_string(),
                Status::Inactive => "inactive".to_string(),
            }),
            avatar_url: ActiveValue::Set(user.avatar_url),
            last_login_at: ActiveValue::Set(user.last_login_at),
            email_verified_at: ActiveValue::Set(user.email_verified_at),
            inserted_at: ActiveValue::Set(user.inserted_at),
            updated_at: ActiveValue::Set(user.updated_at),
        }
    }

    /// 将entity Model转换为domain User
    pub fn to_domain(model: Model) -> User {
        let status = match model.status.as_str() {
            "active" => Status::Active,
            "inactive" => Status::Inactive,
            _ => Status::Inactive, // 默认为inactive
        };

        let role = if model.is_admin.unwrap_or(false) {
            "admin".to_string()
        } else {
            model.role
        };

        User::from_db_record(
            model.uuid.to_string(),
            model.user_name,
            model.password,
            model.email,
            model.phone_number,
            model.merchant_uuid.map(|value| value.to_string()),
            role,
            model.is_admin,
            status,
            model.avatar_url,
            model.last_login_at,
            model.email_verified_at,
            model.inserted_at,
            model.updated_at,
        )
    }

    /// 将domain User转换为用于更新的ActiveModel
    pub fn to_update_active_entity(user: User, original: Model) -> ActiveModel {
        let mut active_model = original.into_active_model();

        active_model.user_name = ActiveValue::Set(user.user_name);
        active_model.email = ActiveValue::Set(user.email);
        active_model.phone_number = ActiveValue::Set(user.phone_number);
        active_model.merchant_uuid = ActiveValue::Set(
            user.merchant_uuid
                .as_ref()
                .map(|value| Uuid::parse_str(value).expect("Invalid merchant UUID")),
        );
        active_model.role = ActiveValue::Set(user.role);
        active_model.is_admin = ActiveValue::Set(user.is_admin);
        active_model.status = ActiveValue::Set(match user.status {
            Status::Active => "active".to_string(),
            Status::Inactive => "inactive".to_string(),
        });
        active_model.avatar_url = ActiveValue::Set(user.avatar_url);
        active_model.last_login_at = ActiveValue::Set(user.last_login_at);
        active_model.email_verified_at = ActiveValue::Set(user.email_verified_at);
        active_model.updated_at = ActiveValue::Set(user.updated_at);

        active_model
    }
}
