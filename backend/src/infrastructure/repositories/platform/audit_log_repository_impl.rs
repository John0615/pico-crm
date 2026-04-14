use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    DatabaseTransaction, Statement, TransactionTrait,
};

use crate::domain::platform::audit_log::{AuditLogCreate, AuditLogRepository};
use crate::infrastructure::entity::audit_logs::ActiveModel as AuditLogActiveModel;

pub struct SeaOrmAuditLogRepository {
    db: DatabaseConnection,
}

impl SeaOrmAuditLogRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn with_public_txn<T, F>(&self, f: F) -> Result<T, String>
    where
        F: for<'a> FnOnce(
            &'a DatabaseTransaction,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<T, String>> + Send + 'a>,
        >,
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
impl AuditLogRepository for SeaOrmAuditLogRepository {
    fn create_log(
        &self,
        log: AuditLogCreate,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        async move {
            self.with_public_txn(|txn| {
                Box::pin(async move {
                    let actor_id = match log.actor_id {
                        Some(raw) => Some(
                            Uuid::parse_str(&raw)
                                .map_err(|e| format!("invalid actor_id uuid: {}", e))?,
                        ),
                        None => None,
                    };

                    let active = AuditLogActiveModel {
                        uuid: ActiveValue::NotSet,
                        actor_id: ActiveValue::Set(actor_id),
                        actor_role: ActiveValue::Set(log.actor_role),
                        action: ActiveValue::Set(log.action),
                        entity: ActiveValue::Set(log.entity),
                        entity_id: ActiveValue::Set(log.entity_id),
                        before_data: ActiveValue::Set(log.before_data),
                        after_data: ActiveValue::Set(log.after_data),
                        ip: ActiveValue::Set(log.ip),
                        user_agent: ActiveValue::Set(log.user_agent),
                        created_at: ActiveValue::NotSet,
                    };

                    active
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create audit log error: {}", e))?;
                    Ok(())
                })
            })
            .await
        }
    }
}
