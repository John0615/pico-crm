use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(AuditLogs::Table)
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditLogs::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(AuditLogs::ActorId).uuid().null())
                    .col(ColumnDef::new(AuditLogs::ActorRole).string().null())
                    .col(ColumnDef::new(AuditLogs::Action).string().not_null())
                    .col(ColumnDef::new(AuditLogs::Entity).string().not_null())
                    .col(ColumnDef::new(AuditLogs::EntityId).string().null())
                    .col(ColumnDef::new(AuditLogs::BeforeData).json_binary().null())
                    .col(ColumnDef::new(AuditLogs::AfterData).json_binary().null())
                    .col(ColumnDef::new(AuditLogs::Ip).string().null())
                    .col(ColumnDef::new(AuditLogs::UserAgent).string().null())
                    .col(
                        ColumnDef::new(AuditLogs::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_entity_entity_id")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::Entity)
                    .col(AuditLogs::EntityId)
                    .to_owned(),
            )
            .await?;
        manager
            .create_index(
                Index::create()
                    .name("idx_audit_logs_actor_id")
                    .table(AuditLogs::Table)
                    .col(AuditLogs::ActorId)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(AuditLogs::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLogs {
    Table,
    Uuid,
    ActorId,
    ActorRole,
    Action,
    Entity,
    EntityId,
    BeforeData,
    AfterData,
    Ip,
    UserAgent,
    CreatedAt,
}
