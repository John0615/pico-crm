use crate::domain::models::merchant::Merchant;
use crate::domain::models::user::User as DomainUser;
use crate::domain::repositories::merchant::MerchantRepository;
use crate::infrastructure::config::app::AppConfig;
use crate::infrastructure::entity::users::{Column as UserColumn, Entity as UserEntity};
use crate::infrastructure::mappers::user_mapper::UserMapper;
use crate::infrastructure::tenant::with_tenant_txn;
use migration::run_tenant_migrations;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, ConnectionTrait, DatabaseBackend, DatabaseConnection,
    EntityTrait, QueryFilter, Statement,
};
use shared::merchant::ProvisionMerchantRequest;
use uuid::Uuid;

pub struct MerchantProvisioningService<R: MerchantRepository> {
    repo: R,
    db: DatabaseConnection,
    tenant_schema_prefix: String,
}

impl<R: MerchantRepository> MerchantProvisioningService<R> {
    pub fn new(repo: R, db: DatabaseConnection) -> Result<Self, String> {
        let config = AppConfig::from_env()?;
        Ok(Self {
            repo,
            db,
            tenant_schema_prefix: config.tenant_schema_prefix,
        })
    }

    pub async fn provision(&self, request: ProvisionMerchantRequest) -> Result<Merchant, String> {
        let owner_user_name = request
            .owner_user_name
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let owner_password = request.owner_password.clone().filter(|value| !value.is_empty());
        if owner_user_name.is_some() ^ owner_password.is_some() {
            return Err("登录用户名和初始密码需要同时提供".to_string());
        }

        if let Some(existing) = self
            .repo
            .find_by_contact_phone(&request.contact_phone)
            .await?
        {
            self.ensure_schema(&existing.schema_name).await?;
            run_tenant_migrations(&self.db, &existing.schema_name)
                .await
                .map_err(|e| format!("tenant migrations failed: {}", e))?;
            if let (Some(user_name), Some(password)) = (&owner_user_name, &owner_password) {
                self.ensure_owner_user(&existing.schema_name, &existing.uuid, user_name, password)
                    .await?;
            }
            return Ok(existing);
        }

        let ProvisionMerchantRequest {
            name,
            short_name,
            contact_name,
            contact_phone,
            merchant_type,
            plan_type,
            ..
        } = request;

        let merchant_uuid = Uuid::new_v4().simple().to_string();
        let schema_name = format!("{}{}", self.tenant_schema_prefix, merchant_uuid);
        let merchant = Merchant::new(
            merchant_uuid.clone(),
            name,
            short_name,
            schema_name.clone(),
            contact_name,
            contact_phone,
            merchant_type,
            plan_type,
            "active".to_string(),
            None,
            None,
        );

        let created = self.repo.create_merchant(merchant).await?;
        let provision_result = self.ensure_schema(&schema_name).await;
        let provision_result = match provision_result {
            Ok(()) => run_tenant_migrations(&self.db, &schema_name)
                .await
                .map_err(|e| format!("tenant migrations failed: {}", e)),
            Err(err) => Err(err),
        };
        if let Err(err) = provision_result {
            let _ = self
                .repo
                .update_status(&created.uuid, "inactive".to_string())
                .await;
            return Err(err);
        }

        if let (Some(user_name), Some(password)) = (owner_user_name, owner_password) {
            if let Err(err) = self
                .ensure_owner_user(&schema_name, &created.uuid, &user_name, &password)
                .await
            {
                let _ = self
                    .repo
                    .update_status(&created.uuid, "inactive".to_string())
                    .await;
                return Err(err);
            }
        }

        Ok(created)
    }

    async fn ensure_schema(&self, schema_name: &str) -> Result<(), String> {
        if !is_safe_schema_name(schema_name) {
            return Err("invalid schema name".to_string());
        }
        let stmt = Statement::from_string(
            DatabaseBackend::Postgres,
            format!("CREATE SCHEMA IF NOT EXISTS \"{}\"", schema_name),
        );
        self.db
            .execute(stmt)
            .await
            .map_err(|e| format!("create schema error: {}", e))?;
        Ok(())
    }

    async fn ensure_owner_user(
        &self,
        schema_name: &str,
        merchant_uuid: &str,
        user_name: &str,
        password: &str,
    ) -> Result<(), String> {
        let schema_name = schema_name.to_string();
        let user_name = user_name.to_string();
        let password = password.to_string();
        let merchant_uuid = merchant_uuid.to_string();
        with_tenant_txn(&self.db, &schema_name, |txn| {
            let user_name = user_name.clone();
            let password = password.clone();
            let merchant_uuid = merchant_uuid.clone();
            Box::pin(async move {
                let existing = UserEntity::find()
                    .filter(UserColumn::UserName.eq(user_name.clone()))
                    .one(txn)
                    .await
                    .map_err(|e| format!("查询用户失败: {}", e))?;
                if existing.is_some() {
                    return Err("登录用户名已存在".to_string());
                }

                let password_hash = DomainUser::hash_password(&password)
                    .map_err(|e| format!("密码加密失败: {}", e))?;
                let mut user = DomainUser::new(user_name, password_hash, None, None);
                user.set_role("operator".to_string());
                user.set_merchant_uuid(merchant_uuid);
                let active = UserMapper::to_active_entity(user);
                active
                    .insert(txn)
                    .await
                    .map_err(|e| format!("创建用户失败: {}", e))?;
                Ok(())
            })
        })
        .await
    }
}

fn is_safe_schema_name(schema_name: &str) -> bool {
    !schema_name.is_empty()
        && schema_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}
