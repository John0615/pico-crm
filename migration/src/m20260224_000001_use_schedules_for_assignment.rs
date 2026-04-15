use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, Statement};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_schedules_employee")
                    .table(Schedules::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Schedules::Table)
                    .rename_column(Schedules::EmployeeId, Schedules::AssignedUserUuid)
                    .to_owned(),
            )
            .await?;

        let backend = manager.get_database_backend();
        let stmt = Statement::from_string(
            backend,
            r#"
            INSERT INTO schedules (
                order_uuid,
                assigned_user_uuid,
                start_at,
                end_at,
                status,
                notes,
                inserted_at,
                updated_at
            )
            SELECT
                o.uuid,
                o.assigned_user_uuid,
                o.scheduled_start_at,
                o.scheduled_end_at,
                CASE
                    WHEN o.status IN ('pending', 'confirmed', 'dispatching') THEN 'planned'
                    WHEN o.status = 'in_service' THEN 'in_service'
                    WHEN o.status = 'completed' THEN 'done'
                    WHEN o.status = 'cancelled' THEN 'cancelled'
                    ELSE 'planned'
                END,
                o.dispatch_note,
                o.inserted_at,
                o.updated_at
            FROM orders o
            WHERE o.assigned_user_uuid IS NOT NULL
              AND o.scheduled_start_at IS NOT NULL
              AND o.scheduled_end_at IS NOT NULL
              AND NOT EXISTS (
                  SELECT 1 FROM schedules s WHERE s.order_uuid = o.uuid
              );
            "#
            .to_string(),
        );
        manager.get_connection().execute(stmt).await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_schedules_assigned_user")
                    .from(Schedules::Table, Schedules::AssignedUserUuid)
                    .to(Users::Table, Users::Uuid)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_index(
                Index::drop()
                    .name("idx_orders_assigned_user")
                    .table(Orders::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_orders_assigned_user")
                    .table(Orders::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .drop_column(Orders::AssignedUserUuid)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(Orders::Table)
                    .add_column_if_not_exists(
                        ColumnDef::new(Orders::AssignedUserUuid).uuid().null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_orders_assigned_user")
                    .from(Orders::Table, Orders::AssignedUserUuid)
                    .to(Users::Table, Users::Uuid)
                    .on_delete(ForeignKeyAction::SetNull)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_orders_assigned_user")
                    .table(Orders::Table)
                    .col(Orders::AssignedUserUuid)
                    .if_not_exists()
                    .to_owned(),
            )
            .await?;

        manager
            .drop_foreign_key(
                ForeignKey::drop()
                    .name("fk_schedules_assigned_user")
                    .table(Schedules::Table)
                    .to_owned(),
            )
            .await?;

        manager
            .alter_table(
                Table::alter()
                    .table(Schedules::Table)
                    .rename_column(Schedules::AssignedUserUuid, Schedules::EmployeeId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_foreign_key(
                ForeignKey::create()
                    .name("fk_schedules_employee")
                    .from(Schedules::Table, Schedules::EmployeeId)
                    .to(Employees::Table, Employees::Uuid)
                    .on_delete(ForeignKeyAction::Restrict)
                    .to_owned(),
            )
            .await?;

        Ok(())
    }
}

#[derive(Iden)]
enum Orders {
    Table,
    AssignedUserUuid,
}

#[derive(Iden)]
enum Schedules {
    Table,
    EmployeeId,
    AssignedUserUuid,
}

#[derive(Iden)]
enum Users {
    Table,
    Uuid,
}

#[derive(Iden)]
enum Employees {
    Table,
    Uuid,
}
