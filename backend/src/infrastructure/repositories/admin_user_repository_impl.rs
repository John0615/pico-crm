use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction,
    EntityTrait, QueryFilter, Statement, TransactionTrait,
};

use crate::domain::models::user::User;
use crate::domain::repositories::admin_user::AdminUserRepository;
use crate::infrastructure::entity::admin_users::{Column, Entity};
use crate::infrastructure::mappers::admin_user_mapper::AdminUserMapper;

pub struct SeaOrmAdminUserRepository {
    db: DatabaseConnection,
}

impl SeaOrmAdminUserRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn with_public_txn<T, F>(&self, f: F) -> Result<T, String>
    where
        F: for<'a> FnOnce(
            &'a DatabaseTransaction,
        ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send + 'a>>,
    {
        let txn = self
            .db
            .begin()
            .await
            .map_err(|e| format!("begin transaction error: {}", e))?;
        let stmt = Statement::from_sql_and_values(
            DatabaseBackend::Postgres,
            "SELECT set_config('search_path', $1, true)",
            vec!["public".to_string().into()],
        );
        txn.execute(stmt)
            .await
            .map_err(|e| format!("set search_path error: {}", e))?;

        let result = f(&txn).await?;
        txn.commit()
            .await
            .map_err(|e| format!("commit transaction error: {}", e))?;
        Ok(result)
    }
}

#[async_trait]
impl AdminUserRepository for SeaOrmAdminUserRepository {
    fn find_by_username(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send {
        let username = username.to_string();
        async move {
            self.with_public_txn(|txn| {
                Box::pin(async move {
                    let user = Entity::find()
                        .filter(Column::UserName.eq(username))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query admin user error: {}", e))?;

                    Ok(user.map(AdminUserMapper::to_domain))
                })
            })
            .await
        }
    }

    fn update_user(
        &self,
        user: User,
    ) -> impl std::future::Future<Output = Result<User, String>> + Send {
        async move {
            self.with_public_txn(|txn| {
                Box::pin(async move {
                    let uuid = Uuid::parse_str(&user.uuid)
                        .map_err(|e| format!("invalid admin uuid: {}", e))?;
                    let original = Entity::find_by_id(uuid)
                        .one(txn)
                        .await
                        .map_err(|e| format!("query admin user error: {}", e))?
                        .ok_or_else(|| format!("admin user {} not found", user.uuid))?;

                    let active_model = AdminUserMapper::to_update_active_entity(user, original);
                    let updated = active_model
                        .update(txn)
                        .await
                        .map_err(|e| format!("update admin user error: {}", e))?;
                    Ok(AdminUserMapper::to_domain(updated))
                })
            })
            .await
        }
    }
}
