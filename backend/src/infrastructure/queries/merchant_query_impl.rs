use crate::domain::models::merchant::Merchant;
use crate::domain::queries::merchant::MerchantQuery;
use crate::infrastructure::entity::merchant::{Column, Entity};
use crate::infrastructure::mappers::merchant_mapper::MerchantMapper;
use shared::merchant::{MerchantListQuery, MerchantPagedResult};
use sea_orm::{
    ColumnTrait, Condition, ConnectionTrait, DatabaseBackend, DatabaseConnection, DatabaseTransaction, EntityTrait,
    PaginatorTrait, QueryFilter, QueryOrder, QuerySelect, Statement, TransactionTrait,
};
use sea_orm::entity::prelude::Uuid;

pub struct SeaOrmMerchantQuery {
    db: DatabaseConnection,
}

impl SeaOrmMerchantQuery {
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

impl MerchantQuery for SeaOrmMerchantQuery {
    type Result = Merchant;

    async fn list_merchants(
        &self,
        query: MerchantListQuery,
    ) -> Result<MerchantPagedResult<Self::Result>, String> {
        self.with_public_txn(|txn| {
            Box::pin(async move {
                let mut select = Entity::find();
                let mut condition = Condition::all();

                if let Some(name) = &query.name {
                    if !name.is_empty() {
                        condition = condition.add(Column::Name.contains(name));
                    }
                }
                if let Some(status) = &query.status {
                    if !status.is_empty() {
                        condition = condition.add(Column::Status.eq(status));
                    }
                }
                if let Some(plan_type) = &query.plan_type {
                    if !plan_type.is_empty() {
                        condition = condition.add(Column::PlanType.eq(plan_type));
                    }
                }
                if let Some(contact_phone) = &query.contact_phone {
                    if !contact_phone.is_empty() {
                        condition = condition.add(Column::ContactPhone.contains(contact_phone));
                    }
                }

                select = select.filter(condition);

                let total = select
                    .clone()
                    .count(txn)
                    .await
                    .map_err(|e| format!("count merchants error: {}", e))?;

                let models = select
                    .order_by_desc(Column::CreatedAt)
                    .offset(Some((query.page - 1) * query.page_size))
                    .limit(Some(query.page_size))
                    .all(txn)
                    .await
                    .map_err(|e| format!("list merchants error: {}", e))?;

                let items = models
                    .into_iter()
                    .map(MerchantMapper::to_domain)
                    .collect();

                Ok(MerchantPagedResult { items, total })
            })
        })
        .await
    }

    async fn find_by_uuid(&self, uuid: &str) -> Result<Option<Self::Result>, String> {
        let uuid = Uuid::parse_str(uuid).map_err(|e| format!("invalid uuid: {}", e))?;
        self.with_public_txn(|txn| {
            Box::pin(async move {
                let merchant = Entity::find_by_id(uuid)
                    .one(txn)
                    .await
                    .map_err(|e| format!("query merchant error: {}", e))?;
                Ok(merchant.map(MerchantMapper::to_domain))
            })
        })
        .await
    }
}
