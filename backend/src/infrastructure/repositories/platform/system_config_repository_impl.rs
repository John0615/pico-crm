use async_trait::async_trait;
use chrono::Utc;
use sea_orm::entity::prelude::*;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    DatabaseTransaction, EntityTrait, IntoActiveModel, QueryFilter, QueryOrder, Statement,
    TransactionTrait,
};
use std::collections::HashMap;

use crate::domain::platform::system_config::{
    SystemConfigCategory, SystemConfigItem, SystemConfigItemUpdate, SystemConfigRepository,
};
use crate::infrastructure::entity::system_config_categories as categories;
use crate::infrastructure::entity::system_config_items as items;
use crate::infrastructure::mappers::platform::system_config_mapper::SystemConfigMapper;

pub struct SeaOrmSystemConfigRepository {
    db: DatabaseConnection,
}

impl SeaOrmSystemConfigRepository {
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
impl SystemConfigRepository for SeaOrmSystemConfigRepository {
    fn list_categories_with_items(
        &self,
    ) -> impl std::future::Future<Output = Result<Vec<SystemConfigCategory>, String>> + Send {
        async move {
            self.with_public_txn(|txn| {
                Box::pin(async move {
                    let category_models = categories::Entity::find()
                        .filter(categories::Column::IsActive.eq(true))
                        .order_by_asc(categories::Column::SortOrder)
                        .all(txn)
                        .await
                        .map_err(|e| format!("query system config categories error: {}", e))?;

                    let item_models = items::Entity::find()
                        .order_by_asc(items::Column::SortOrder)
                        .all(txn)
                        .await
                        .map_err(|e| format!("query system config items error: {}", e))?;

                    let mut grouped: HashMap<String, Vec<SystemConfigItem>> = HashMap::new();
                    for model in item_models {
                        let item = SystemConfigMapper::item_to_domain(model);
                        grouped
                            .entry(item.category_code.clone())
                            .or_default()
                            .push(item);
                    }

                    let mut categories_domain = Vec::new();
                    for model in category_models {
                        let mut items = grouped.remove(&model.code).unwrap_or_default();
                        items.sort_by_key(|item| item.sort_order);
                        categories_domain
                            .push(SystemConfigMapper::category_to_domain(model, items));
                    }

                    Ok(categories_domain)
                })
            })
            .await
        }
    }

    fn find_items_by_keys(
        &self,
        keys: Vec<String>,
    ) -> impl std::future::Future<Output = Result<Vec<SystemConfigItem>, String>> + Send {
        async move {
            self.with_public_txn(|txn| {
                Box::pin(async move {
                    let models = items::Entity::find()
                        .filter(items::Column::Key.is_in(keys))
                        .all(txn)
                        .await
                        .map_err(|e| format!("query system config items error: {}", e))?;

                    Ok(models
                        .into_iter()
                        .map(SystemConfigMapper::item_to_domain)
                        .collect())
                })
            })
            .await
        }
    }

    fn update_items(
        &self,
        updates: Vec<SystemConfigItemUpdate>,
    ) -> impl std::future::Future<Output = Result<Vec<SystemConfigItem>, String>> + Send {
        async move {
            self.with_public_txn(|txn| {
                Box::pin(async move {
                    let now = Utc::now();
                    let keys: Vec<String> = updates.iter().map(|u| u.key.clone()).collect();
                    let mut update_map: HashMap<String, serde_json::Value> = HashMap::new();
                    for update in updates {
                        update_map.insert(update.key, update.value);
                    }

                    let models = items::Entity::find()
                        .filter(items::Column::Key.is_in(keys))
                        .all(txn)
                        .await
                        .map_err(|e| format!("query system config items error: {}", e))?;

                    let mut updated_items = Vec::new();
                    for model in models {
                        let Some(new_value) = update_map.get(&model.key) else {
                            continue;
                        };
                        let mut active = model.into_active_model();
                        active.value = ActiveValue::Set(Some(new_value.clone()));
                        active.updated_at = ActiveValue::Set(now);
                        let saved = active
                            .update(txn)
                            .await
                            .map_err(|e| format!("update system config item error: {}", e))?;
                        updated_items.push(SystemConfigMapper::item_to_domain(saved));
                    }

                    Ok(updated_items)
                })
            })
            .await
        }
    }
}
