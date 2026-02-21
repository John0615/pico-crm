use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{ConnectionTrait, DatabaseBackend, Statement};

const PUBLIC_SCHEMA: &str = "public";
const SYSTEM_CONFIG_ITEMS: &str = "system_config_items";
const SYSTEM_CONFIG_CATEGORIES: &str = "system_config_categories";

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        ensure_table_names(manager).await?;

        manager
            .create_table(
                Table::create()
                    .table((Alias::new(PUBLIC_SCHEMA), Alias::new(SYSTEM_CONFIG_ITEMS)))
                    .if_not_exists()
                    .col(
                        ColumnDef::new(SystemConfigItem::Uuid)
                            .uuid()
                            .not_null()
                            .primary_key()
                            .default(Expr::cust("gen_random_uuid()")),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItem::CategoryCode)
                            .string()
                            .not_null(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItem::Key)
                            .string()
                            .not_null()
                            .unique_key(),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItem::Label)
                            .string()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SystemConfigItem::Description).string().null())
                    .col(
                        ColumnDef::new(SystemConfigItem::ValueType)
                            .string()
                            .not_null()
                            .default("string"),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItem::DefaultValue)
                            .json_binary()
                            .not_null(),
                    )
                    .col(ColumnDef::new(SystemConfigItem::Value).json_binary().null())
                    .col(ColumnDef::new(SystemConfigItem::Validation).json_binary().null())
                    .col(ColumnDef::new(SystemConfigItem::UiSchema).json_binary().null())
                    .col(
                        ColumnDef::new(SystemConfigItem::IsRequired)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItem::IsEditable)
                            .boolean()
                            .not_null()
                            .default(true),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItem::IsSensitive)
                            .boolean()
                            .not_null()
                            .default(false),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItem::SortOrder)
                            .integer()
                            .not_null()
                            .default(0),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItem::CreatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .col(
                        ColumnDef::new(SystemConfigItem::UpdatedAt)
                            .timestamp_with_time_zone()
                            .not_null()
                            .default(Expr::current_timestamp()),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk_public_system_config_items_category")
                            .from(
                                (Alias::new(PUBLIC_SCHEMA), Alias::new(SYSTEM_CONFIG_ITEMS)),
                                SystemConfigItem::CategoryCode,
                            )
                            .to(
                                (Alias::new(PUBLIC_SCHEMA), Alias::new(SYSTEM_CONFIG_CATEGORIES)),
                                SystemConfigCategory::Code,
                            ),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_index(
                Index::create()
                    .name("ix_public_system_config_items_category")
                    .table((Alias::new(PUBLIC_SCHEMA), Alias::new(SYSTEM_CONFIG_ITEMS)))
                    .col(SystemConfigItem::CategoryCode)
                    .to_owned(),
            )
            .await?;

        seed_system_config(manager).await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(
                Table::drop()
                    .table((Alias::new(PUBLIC_SCHEMA), Alias::new(SYSTEM_CONFIG_ITEMS)))
                    .to_owned(),
            )
            .await
    }
}

async fn ensure_table_names(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let sql = r#"
DO $$
BEGIN
  IF EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_schema = 'public' AND table_name = 'system_config_category'
  ) AND NOT EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_schema = 'public' AND table_name = 'system_config_categories'
  ) THEN
    ALTER TABLE public.system_config_category RENAME TO system_config_categories;
  END IF;

  IF EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_schema = 'public' AND table_name = 'system_config_item'
  ) AND NOT EXISTS (
    SELECT 1 FROM information_schema.tables
    WHERE table_schema = 'public' AND table_name = 'system_config_items'
  ) THEN
    ALTER TABLE public.system_config_item RENAME TO system_config_items;
  END IF;
END $$;
"#;

    manager
        .get_connection()
        .execute(Statement::from_string(DatabaseBackend::Postgres, sql))
        .await?;

    let create_categories = r#"
CREATE TABLE IF NOT EXISTS public.system_config_categories (
  code varchar NOT NULL PRIMARY KEY,
  name varchar NOT NULL,
  description varchar NULL,
  sort_order integer NOT NULL DEFAULT 0,
  is_active boolean NOT NULL DEFAULT true,
  created_at timestamptz NOT NULL DEFAULT current_timestamp,
  updated_at timestamptz NOT NULL DEFAULT current_timestamp
);
"#;

    manager
        .get_connection()
        .execute(Statement::from_string(
            DatabaseBackend::Postgres,
            create_categories,
        ))
        .await?;
    Ok(())
}

async fn seed_system_config(manager: &SchemaManager<'_>) -> Result<(), DbErr> {
    let categories = r#"
INSERT INTO public.system_config_categories
    (code, name, description, sort_order, is_active, created_at, updated_at)
VALUES
    ('general', '常规设置', '系统基础信息与显示设置', 10, true, now(), now()),
    ('security', '安全设置', '登录与账号安全策略', 20, true, now(), now()),
    ('notification', '通知设置', '邮件与短信通知配置', 30, true, now(), now()),
    ('backup', '备份设置', '备份与保留策略', 40, true, now(), now()),
    ('integration', '集成设置', '第三方服务与外部集成', 50, true, now(), now()),
    ('branding', '品牌设置', '系统外观与品牌标识', 60, true, now(), now()),
    ('features', '功能开关', '功能启用与灰度开关', 70, true, now(), now()),
    ('localization', '本地化', '时区与格式化设置', 80, true, now(), now())
ON CONFLICT (code) DO NOTHING;
"#;
    manager
        .get_connection()
        .execute(Statement::from_string(DatabaseBackend::Postgres, categories))
        .await?;

    let items = r##"
INSERT INTO public.system_config_items
    (uuid, category_code, key, label, description, value_type, default_value, value, validation, ui_schema,
     is_required, is_editable, is_sensitive, sort_order, created_at, updated_at)
VALUES
    (gen_random_uuid(), 'general', 'system.name', '系统名称', '系统显示名称', 'string',
        '"PicoCRM"'::jsonb, '"PicoCRM"'::jsonb, '{"min":1,"max":50}'::jsonb,
        '{"widget":"text","placeholder":"输入系统名称"}'::jsonb, true, true, false, 10, now(), now()),
    (gen_random_uuid(), 'general', 'system.description', '系统描述', '系统用途说明', 'string',
        '""'::jsonb, '""'::jsonb, '{"max":200}'::jsonb,
        '{"widget":"textarea","placeholder":"输入系统描述","rows":4}'::jsonb, false, true, false, 20, now(), now()),
    (gen_random_uuid(), 'general', 'system.language', '系统语言', '系统界面显示语言', 'enum',
        '"zh-CN"'::jsonb, '"zh-CN"'::jsonb, '{"options":["zh-CN","en-US","ja-JP"]}'::jsonb,
        '{"widget":"select"}'::jsonb, true, true, false, 30, now(), now()),
    (gen_random_uuid(), 'general', 'system.maintenance_mode', '维护模式', '启用后系统进入维护状态', 'bool',
        'false'::jsonb, 'false'::jsonb, NULL,
        '{"widget":"toggle"}'::jsonb, false, true, false, 40, now(), now()),

    (gen_random_uuid(), 'security', 'security.password_min_length', '密码最小长度', '用户密码最小长度', 'number',
        '8'::jsonb, '8'::jsonb, '{"min":6,"max":20}'::jsonb,
        '{"widget":"number","min":6,"max":20}'::jsonb, true, true, false, 10, now(), now()),
    (gen_random_uuid(), 'security', 'security.session_timeout_minutes', '会话超时时间', '用户无操作后自动退出时间（分钟）', 'number',
        '30'::jsonb, '30'::jsonb, '{"min":5,"max":480}'::jsonb,
        '{"widget":"number","min":5,"max":480}'::jsonb, true, true, false, 20, now(), now()),
    (gen_random_uuid(), 'security', 'security.enable_2fa', '双因素认证', '启用双因素认证', 'bool',
        'false'::jsonb, 'false'::jsonb, NULL,
        '{"widget":"toggle"}'::jsonb, false, true, false, 30, now(), now()),
    (gen_random_uuid(), 'security', 'security.login_attempt_limit', '登录尝试限制', '限制登录尝试次数', 'number',
        '5'::jsonb, '5'::jsonb, '{"min":3,"max":10}'::jsonb,
        '{"widget":"number","min":3,"max":10}'::jsonb, false, true, false, 40, now(), now()),

    (gen_random_uuid(), 'notification', 'notification.email_enabled', '邮件通知', '启用邮件通知', 'bool',
        'true'::jsonb, 'true'::jsonb, NULL,
        '{"widget":"toggle"}'::jsonb, false, true, false, 10, now(), now()),
    (gen_random_uuid(), 'notification', 'notification.sms_enabled', '短信通知', '启用短信通知', 'bool',
        'false'::jsonb, 'false'::jsonb, NULL,
        '{"widget":"toggle"}'::jsonb, false, true, false, 20, now(), now()),
    (gen_random_uuid(), 'notification', 'notification.email_from', '发件邮箱', '系统通知发件人邮箱', 'string',
        '"no-reply@picocrm.local"'::jsonb, '"no-reply@picocrm.local"'::jsonb, '{"max":100}'::jsonb,
        '{"widget":"text","placeholder":"no-reply@picocrm.local"}'::jsonb, false, true, false, 30, now(), now()),
    (gen_random_uuid(), 'notification', 'notification.sms_signature', '短信签名', '短信发送签名', 'string',
        '""'::jsonb, '""'::jsonb, '{"max":50}'::jsonb,
        '{"widget":"text","placeholder":"输入短信签名"}'::jsonb, false, true, false, 40, now(), now()),

    (gen_random_uuid(), 'backup', 'backup.enabled', '备份开关', '启用系统备份', 'bool',
        'true'::jsonb, 'true'::jsonb, NULL,
        '{"widget":"toggle"}'::jsonb, false, true, false, 10, now(), now()),
    (gen_random_uuid(), 'backup', 'backup.schedule', '备份周期', '自动备份周期', 'enum',
        '"daily"'::jsonb, '"daily"'::jsonb, '{"options":["daily","weekly","monthly"]}'::jsonb,
        '{"widget":"select"}'::jsonb, false, true, false, 20, now(), now()),
    (gen_random_uuid(), 'backup', 'backup.retention_days', '保留天数', '备份保留天数', 'number',
        '30'::jsonb, '30'::jsonb, '{"min":7,"max":365}'::jsonb,
        '{"widget":"number","min":7,"max":365}'::jsonb, false, true, false, 30, now(), now()),

    (gen_random_uuid(), 'integration', 'integration.smtp_host', 'SMTP 主机', '邮件服务主机', 'string',
        '""'::jsonb, '""'::jsonb, '{"max":200}'::jsonb,
        '{"widget":"text","placeholder":"smtp.example.com"}'::jsonb, false, true, false, 10, now(), now()),
    (gen_random_uuid(), 'integration', 'integration.smtp_port', 'SMTP 端口', '邮件服务端口', 'number',
        '587'::jsonb, '587'::jsonb, '{"min":1,"max":65535}'::jsonb,
        '{"widget":"number","min":1,"max":65535}'::jsonb, false, true, false, 20, now(), now()),
    (gen_random_uuid(), 'integration', 'integration.smtp_user', 'SMTP 用户名', '邮件服务账号', 'string',
        '""'::jsonb, '""'::jsonb, '{"max":100}'::jsonb,
        '{"widget":"text","placeholder":"username"}'::jsonb, false, true, false, 30, now(), now()),
    (gen_random_uuid(), 'integration', 'integration.smtp_password', 'SMTP 密码', '邮件服务密码', 'string',
        '""'::jsonb, '""'::jsonb, NULL,
        '{"widget":"password","placeholder":"输入密码"}'::jsonb, false, true, true, 40, now(), now()),

    (gen_random_uuid(), 'branding', 'branding.logo_url', 'Logo 地址', '系统 Logo 图片地址', 'string',
        '""'::jsonb, '""'::jsonb, '{"max":200}'::jsonb,
        '{"widget":"text","placeholder":"https://"}'::jsonb, false, true, false, 10, now(), now()),
    (gen_random_uuid(), 'branding', 'branding.primary_color', '主色调', '系统主色调', 'string',
        '"#2563eb"'::jsonb, '"#2563eb"'::jsonb, '{"max":20}'::jsonb,
        '{"widget":"text","placeholder":"#2563eb"}'::jsonb, false, true, false, 20, now(), now()),
    (gen_random_uuid(), 'branding', 'branding.favicon_url', 'Favicon 地址', '浏览器图标地址', 'string',
        '""'::jsonb, '""'::jsonb, '{"max":200}'::jsonb,
        '{"widget":"text","placeholder":"https://"}'::jsonb, false, true, false, 30, now(), now()),

    (gen_random_uuid(), 'features', 'features.enable_beta', 'Beta 功能', '启用 Beta 功能', 'bool',
        'false'::jsonb, 'false'::jsonb, NULL,
        '{"widget":"toggle"}'::jsonb, false, true, false, 10, now(), now()),
    (gen_random_uuid(), 'features', 'features.enable_file_upload', '文件上传', '启用文件上传功能', 'bool',
        'true'::jsonb, 'true'::jsonb, NULL,
        '{"widget":"toggle"}'::jsonb, false, true, false, 20, now(), now()),

    (gen_random_uuid(), 'localization', 'localization.timezone', '默认时区', '系统默认时区', 'string',
        '"Asia/Shanghai"'::jsonb, '"Asia/Shanghai"'::jsonb, '{"max":100}'::jsonb,
        '{"widget":"text","placeholder":"Asia/Shanghai"}'::jsonb, false, true, false, 10, now(), now()),
    (gen_random_uuid(), 'localization', 'localization.date_format', '日期格式', '日期显示格式', 'string',
        '"YYYY-MM-DD"'::jsonb, '"YYYY-MM-DD"'::jsonb, '{"max":50}'::jsonb,
        '{"widget":"text","placeholder":"YYYY-MM-DD"}'::jsonb, false, true, false, 20, now(), now())
ON CONFLICT (key) DO NOTHING;
"##;
    manager
        .get_connection()
        .execute(Statement::from_string(DatabaseBackend::Postgres, items))
        .await?;
    Ok(())
}

#[derive(DeriveIden)]
enum SystemConfigItem {
    Uuid,
    CategoryCode,
    Key,
    Label,
    Description,
    ValueType,
    DefaultValue,
    Value,
    Validation,
    UiSchema,
    IsRequired,
    IsEditable,
    IsSensitive,
    SortOrder,
    CreatedAt,
    UpdatedAt,
}

#[derive(DeriveIden)]
enum SystemConfigCategory {
    Code,
}
