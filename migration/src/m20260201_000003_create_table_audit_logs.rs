use sea_orm_migration::prelude::*;

const PUBLIC_SCHEMA: &str = "public";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table((Alias::new(PUBLIC_SCHEMA), AuditLog::Table))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(AuditLog::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(ColumnDef::new(AuditLog::ActorId).uuid().null())
                    .col(ColumnDef::new(AuditLog::ActorRole).string().null())
                    .col(ColumnDef::new(AuditLog::Action).string().not_null())
                    .col(ColumnDef::new(AuditLog::Entity).string().not_null())
                    .col(ColumnDef::new(AuditLog::EntityId).string().null())
                    .col(ColumnDef::new(AuditLog::BeforeData).json_binary().null())
                    .col(ColumnDef::new(AuditLog::AfterData).json_binary().null())
                    .col(ColumnDef::new(AuditLog::Ip).string().null())
                    .col(ColumnDef::new(AuditLog::UserAgent).string().null())
                    .col(
                        ColumnDef::new(AuditLog::CreatedAt)
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
                    .name("ix_public_audit_log_actor_id")
                    .table((Alias::new(PUBLIC_SCHEMA), AuditLog::Table))
                    .col(AuditLog::ActorId)
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("ix_public_audit_log_entity")
                    .table((Alias::new(PUBLIC_SCHEMA), AuditLog::Table))
                    .col(AuditLog::Entity)
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new(PUBLIC_SCHEMA), AuditLog::Table))
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum AuditLog {
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
