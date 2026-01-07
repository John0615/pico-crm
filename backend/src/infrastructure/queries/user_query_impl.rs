use crate::{
    domain::{
        models::user::{Status, User},
        queries::user::UserQuery,
    },
    infrastructure::entity::users::{Column, Entity, Model},
    infrastructure::utils::naive_to_utc,
};
use sea_orm::entity::prelude::*;

use async_trait::async_trait;
use sea_orm::DatabaseConnection;

pub struct SeaOrmUserQuery {
    db: DatabaseConnection,
}

impl SeaOrmUserQuery {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    fn map_model(u: Model) -> User {
        let status = match u.status.as_str() {
            "inactive" => Status::Inactive,
            "active" => Status::Active,
            _ => Status::Inactive,
        };
        User {
            uuid: u.uuid.to_string(),
            user_name: u.user_name,
            password: u.password,
            email: u.email,
            phone_number: u.phone_number,
            is_admin: u.is_admin,
            status,
            avatar_url: u.avatar_url,
            last_login_at: u.last_login_at.map(naive_to_utc),
            email_verified_at: u.email_verified_at.map(naive_to_utc),
            inserted_at: naive_to_utc(u.inserted_at),
            updated_at: naive_to_utc(u.updated_at),
        }
    }
}

#[async_trait]
impl UserQuery for SeaOrmUserQuery {
    type Result = User;

    fn get_user(
        &self,
        user_name: &str,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send {
        async move {
            let user = Entity::find()
                .filter(Column::UserName.eq(user_name))
                .one(&self.db)
                .await
                .map_err(|e| format!("查询失败: {}", e))?
                .map(Self::map_model);
            Ok(user)
        }
    }
}
