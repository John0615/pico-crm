use async_trait::async_trait;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    DatabaseTransaction, EntityTrait, QueryFilter, Statement, TransactionTrait,
};

use crate::domain::models::merchant::Merchant;
use crate::domain::repositories::merchant::MerchantRepository;
use crate::infrastructure::entity::merchant::{Column, Entity};
use crate::infrastructure::mappers::merchant_mapper::MerchantMapper;

pub struct SeaOrmMerchantRepository {
    db: DatabaseConnection,
}

impl SeaOrmMerchantRepository {
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    async fn with_public_txn<T, F>(&self, f: F) -> Result<T, String>
    where
        F: for<'a> FnOnce(&'a DatabaseTransaction) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>> + Send + 'a>>,
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
impl MerchantRepository for SeaOrmMerchantRepository {
    fn create_merchant(
        &self,
        merchant: Merchant,
    ) -> impl std::future::Future<Output = Result<Merchant, String>> + Send {
        async move {
            self.with_public_txn(|txn| {
                Box::pin(async move {
                    let entity = MerchantMapper::to_active_entity(merchant);
                    let new_entity = entity
                        .insert(txn)
                        .await
                        .map_err(|e| format!("create merchant database error: {}", e))?;
                    Ok(MerchantMapper::to_domain(new_entity))
                })
            })
            .await
        }
    }

    fn find_by_contact_phone(
        &self,
        contact_phone: &str,
    ) -> impl std::future::Future<Output = Result<Option<Merchant>, String>> + Send {
        let contact_phone = contact_phone.to_string();
        async move {
            self.with_public_txn(|txn| {
                Box::pin(async move {
                    let merchant = Entity::find()
                        .filter(Column::ContactPhone.eq(contact_phone))
                        .one(txn)
                        .await
                        .map_err(|e| format!("query merchant error: {}", e))?;
                    Ok(merchant.map(MerchantMapper::to_domain))
                })
            })
            .await
        }
    }

    fn update_status(
        &self,
        uuid: &str,
        status: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send {
        let uuid = uuid.to_string();
        async move {
            self.with_public_txn(|txn| {
                Box::pin(async move {
                    let merchant_uuid = Uuid::parse_str(&uuid)
                        .map_err(|e| format!("invalid merchant uuid: {}", e))?;
                    let merchant = Entity::find_by_id(merchant_uuid)
                        .one(txn)
                        .await
                        .map_err(|e| format!("query merchant error: {}", e))?
                        .ok_or_else(|| format!("merchant {} not found", uuid))?;

                    let active_model = MerchantMapper::to_update_active_entity(merchant, status);
                    active_model
                        .update(txn)
                        .await
                        .map_err(|e| format!("update merchant error: {}", e))?;
                    Ok(())
                })
            })
            .await
        }
    }
}
