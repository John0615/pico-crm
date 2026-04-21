pub use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::TransactionTrait;

mod m20250415_031125_create_table_contacts;
mod m20250707_013515_create_table_users;
mod m20260201_000001_create_table_merchants;
mod m20260201_000002_create_table_admin_users;
mod m20260201_000003_create_table_audit_logs;
mod m20260201_000004_create_tenant_tables;
mod m20260201_000005_add_user_tenant_columns;
mod m20260221_000001_create_table_system_config_categories;
mod m20260221_000002_create_table_system_config_items;
mod m20260221_000003_add_service_requests_and_order_fields;
mod m20260222_000001_add_order_schedule_indexes;
mod m20260223_000004_add_service_request_creator;
mod m20260224_000001_use_schedules_for_assignment;
mod m20260225_000001_fill_orders_customer_from_contact;
mod m20260225_000002_rename_orders_customer_uuid;
mod m20260225_000003_rename_service_requests_customer_uuid;
mod m20260225_000004_backfill_orders_customer_uuid_from_requests;
mod m20260225_000005_drop_customer_extra_fields;
mod m20260414_000001_add_service_request_event_id;
mod m20260415_000002_rename_schedules_order_id;
mod m20260418_000001_extend_customer_fields;
mod m20260418_000002_drop_customer_lifecycle_fields;
mod m20260418_000003_extend_users_employee_fields;
mod m20260418_000004_extend_orders_for_mvp;
mod m20260418_000005_create_order_change_logs;
mod m20260418_000006_drop_order_service_type;
mod m20260420_000001_create_contact_follow_records;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250415_031125_create_table_contacts::Migration),
            Box::new(m20250707_013515_create_table_users::Migration),
            Box::new(m20260201_000004_create_tenant_tables::Migration),
            Box::new(m20260201_000005_add_user_tenant_columns::Migration),
            Box::new(m20260221_000003_add_service_requests_and_order_fields::Migration),
            Box::new(m20260223_000004_add_service_request_creator::Migration),
            Box::new(m20260222_000001_add_order_schedule_indexes::Migration),
            Box::new(m20260224_000001_use_schedules_for_assignment::Migration),
            Box::new(m20260225_000001_fill_orders_customer_from_contact::Migration),
            Box::new(m20260225_000002_rename_orders_customer_uuid::Migration),
            Box::new(m20260225_000003_rename_service_requests_customer_uuid::Migration),
            Box::new(m20260225_000004_backfill_orders_customer_uuid_from_requests::Migration),
            Box::new(m20260225_000005_drop_customer_extra_fields::Migration),
            Box::new(m20260414_000001_add_service_request_event_id::Migration),
            Box::new(m20260415_000002_rename_schedules_order_id::Migration),
            Box::new(m20260418_000001_extend_customer_fields::Migration),
            Box::new(m20260418_000002_drop_customer_lifecycle_fields::Migration),
            Box::new(m20260418_000003_extend_users_employee_fields::Migration),
            Box::new(m20260418_000004_extend_orders_for_mvp::Migration),
            Box::new(m20260418_000005_create_order_change_logs::Migration),
            Box::new(m20260418_000006_drop_order_service_type::Migration),
            Box::new(m20260420_000001_create_contact_follow_records::Migration),
        ]
    }
}

pub struct PublicMigrator;

#[async_trait::async_trait]
impl MigratorTrait for PublicMigrator {
    fn migration_table_name() -> DynIden {
        "seaql_migrations_public".into_iden()
    }

    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260201_000001_create_table_merchants::Migration),
            Box::new(m20260201_000002_create_table_admin_users::Migration),
            Box::new(m20260201_000003_create_table_audit_logs::Migration),
            Box::new(m20260221_000001_create_table_system_config_categories::Migration),
            Box::new(m20260221_000002_create_table_system_config_items::Migration),
        ]
    }
}

pub struct TenantMigrator;

#[async_trait::async_trait]
impl MigratorTrait for TenantMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250415_031125_create_table_contacts::Migration),
            Box::new(m20250707_013515_create_table_users::Migration),
            Box::new(m20260201_000004_create_tenant_tables::Migration),
            Box::new(m20260201_000005_add_user_tenant_columns::Migration),
            Box::new(m20260221_000003_add_service_requests_and_order_fields::Migration),
            Box::new(m20260223_000004_add_service_request_creator::Migration),
            Box::new(m20260222_000001_add_order_schedule_indexes::Migration),
            Box::new(m20260224_000001_use_schedules_for_assignment::Migration),
            Box::new(m20260225_000001_fill_orders_customer_from_contact::Migration),
            Box::new(m20260225_000002_rename_orders_customer_uuid::Migration),
            Box::new(m20260225_000003_rename_service_requests_customer_uuid::Migration),
            Box::new(m20260225_000004_backfill_orders_customer_uuid_from_requests::Migration),
            Box::new(m20260225_000005_drop_customer_extra_fields::Migration),
            Box::new(m20260414_000001_add_service_request_event_id::Migration),
            Box::new(m20260415_000002_rename_schedules_order_id::Migration),
            Box::new(m20260418_000001_extend_customer_fields::Migration),
            Box::new(m20260418_000002_drop_customer_lifecycle_fields::Migration),
            Box::new(m20260418_000003_extend_users_employee_fields::Migration),
            Box::new(m20260418_000004_extend_orders_for_mvp::Migration),
            Box::new(m20260418_000005_create_order_change_logs::Migration),
            Box::new(m20260418_000006_drop_order_service_type::Migration),
            Box::new(m20260420_000001_create_contact_follow_records::Migration),
        ]
    }
}

pub async fn run_tenant_migrations(
    connection: &sea_orm_migration::sea_orm::DatabaseConnection,
    schema: &str,
) -> Result<(), sea_orm_migration::sea_orm::DbErr> {
    if !schema
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(sea_orm_migration::sea_orm::DbErr::Custom(
            "Invalid schema name".to_string(),
        ));
    }

    let txn = connection.begin().await?;
    let stmt = sea_orm_migration::sea_orm::Statement::from_sql_and_values(
        sea_orm_migration::sea_orm::DatabaseBackend::Postgres,
        "SELECT set_config('search_path', $1, true)",
        vec![schema.to_string().into()],
    );
    sea_orm_migration::sea_orm::ConnectionTrait::execute(&txn, stmt).await?;
    TenantMigrator::up(&txn, None).await?;
    txn.commit().await
}
