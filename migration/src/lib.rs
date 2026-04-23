pub use sea_orm_migration::prelude::*;
mod m20260501_000001_create_table_merchants;
mod m20260501_000002_create_table_admin_users;
mod m20260501_000003_create_table_audit_logs;
mod m20260501_000004_create_table_system_config_categories;
mod m20260501_000005_create_table_system_config_items;
mod m20260501_000006_create_table_users;
mod m20260501_000007_create_table_customers;
mod m20260501_000008_create_table_service_catalogs;
mod m20260501_000009_create_table_service_requests;
mod m20260501_000010_create_table_orders;
mod m20260501_000011_create_table_schedules;
mod m20260501_000012_create_table_contact_follow_records;
mod m20260501_000013_create_table_order_feedback;
mod m20260501_000014_create_table_order_change_logs;
mod m20260501_000015_create_table_after_sales_cases;
mod m20260501_000016_create_table_after_sales_case_records;
mod m20260501_000017_create_table_after_sales_reworks;
mod m20260501_000018_make_user_name_globally_unique;
mod m20260501_000019_make_user_contact_globally_unique;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20260501_000001_create_table_merchants::Migration),
            Box::new(m20260501_000002_create_table_admin_users::Migration),
            Box::new(m20260501_000003_create_table_audit_logs::Migration),
            Box::new(m20260501_000004_create_table_system_config_categories::Migration),
            Box::new(m20260501_000005_create_table_system_config_items::Migration),
            Box::new(m20260501_000006_create_table_users::Migration),
            Box::new(m20260501_000007_create_table_customers::Migration),
            Box::new(m20260501_000008_create_table_service_catalogs::Migration),
            Box::new(m20260501_000009_create_table_service_requests::Migration),
            Box::new(m20260501_000010_create_table_orders::Migration),
            Box::new(m20260501_000011_create_table_schedules::Migration),
            Box::new(m20260501_000012_create_table_contact_follow_records::Migration),
            Box::new(m20260501_000013_create_table_order_feedback::Migration),
            Box::new(m20260501_000014_create_table_order_change_logs::Migration),
            Box::new(m20260501_000015_create_table_after_sales_cases::Migration),
            Box::new(m20260501_000016_create_table_after_sales_case_records::Migration),
            Box::new(m20260501_000017_create_table_after_sales_reworks::Migration),
            Box::new(m20260501_000018_make_user_name_globally_unique::Migration),
            Box::new(m20260501_000019_make_user_contact_globally_unique::Migration),
        ]
    }
}
