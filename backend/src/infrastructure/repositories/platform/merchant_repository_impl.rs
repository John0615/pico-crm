use async_trait::async_trait;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, DatabaseBackend,
    DatabaseConnection, DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter, Statement,
    TransactionTrait,
};

use crate::domain::platform::merchant::{Merchant, MerchantRepository, MerchantUpdate};
use crate::infrastructure::entity::merchant::{Column, Entity};
use crate::infrastructure::mappers::platform::merchant_mapper::MerchantMapper;

pub struct SeaOrmMerchantRepository {
    db: DatabaseConnection,
}

impl SeaOrmMerchantRepository {
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

    fn update_merchant(
        &self,
        uuid: &str,
        update: MerchantUpdate,
    ) -> impl std::future::Future<Output = Result<Merchant, String>> + Send {
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

                    let mut active_model = merchant.into_active_model();
                    if let Some(name) = update.name {
                        active_model.name = ActiveValue::Set(name);
                    }
                    if let Some(short_name) = update.short_name {
                        active_model.short_name = ActiveValue::Set(Some(short_name));
                    }
                    if let Some(contact_name) = update.contact_name {
                        active_model.contact_name = ActiveValue::Set(contact_name);
                    }
                    if let Some(contact_phone) = update.contact_phone {
                        active_model.contact_phone = ActiveValue::Set(contact_phone);
                    }
                    if let Some(merchant_type) = update.merchant_type {
                        active_model.merchant_type = ActiveValue::Set(Some(merchant_type));
                    }
                    if let Some(status) = update.status {
                        active_model.status = ActiveValue::Set(status);
                    }
                    if let Some(plan_type) = update.plan_type {
                        active_model.plan_type = ActiveValue::Set(Some(plan_type));
                    }
                    if let Some(trial_end_at) = update.trial_end_at {
                        active_model.trial_end_at = ActiveValue::Set(Some(trial_end_at));
                    }
                    if let Some(expired_at) = update.expired_at {
                        active_model.expired_at = ActiveValue::Set(Some(expired_at));
                    }
                    active_model.updated_at = ActiveValue::Set(Utc::now());

                    let updated = active_model
                        .update(txn)
                        .await
                        .map_err(|e| format!("update merchant error: {}", e))?;
                    Ok(MerchantMapper::to_domain(updated))
                })
            })
            .await
        }
    }
}
