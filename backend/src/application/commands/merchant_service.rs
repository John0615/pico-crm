use crate::domain::models::merchant::Merchant;
use crate::domain::repositories::merchant::MerchantRepository;
use crate::infrastructure::config::app::AppConfig;
use migration::run_tenant_migrations;
use sea_orm::{ConnectionTrait, DatabaseBackend, DatabaseConnection, Statement};
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
        if let Some(existing) = self
            .repo
            .find_by_contact_phone(&request.contact_phone)
            .await?
        {
            self.ensure_schema(&existing.schema_name).await?;
            run_tenant_migrations(&self.db, &existing.schema_name)
                .await
                .map_err(|e| format!("tenant migrations failed: {}", e))?;
            return Ok(existing);
        }

        let merchant_uuid = Uuid::new_v4().simple().to_string();
        let schema_name = format!("{}{}", self.tenant_schema_prefix, merchant_uuid);
        let merchant = Merchant::new(
            merchant_uuid.clone(),
            request.name,
            request.short_name,
            schema_name.clone(),
            request.contact_name,
            request.contact_phone,
            request.merchant_type,
            request.plan_type,
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
}

fn is_safe_schema_name(schema_name: &str) -> bool {
    !schema_name.is_empty()
        && schema_name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_')
}
