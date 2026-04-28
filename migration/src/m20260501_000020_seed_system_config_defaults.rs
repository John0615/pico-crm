use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(INSERT_CATEGORIES_SQL)
            .await?;

        manager
            .get_connection()
            .execute_unprepared(INSERT_ITEMS_SQL)
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .get_connection()
            .execute_unprepared(DELETE_ITEMS_SQL)
            .await?;

        manager
            .get_connection()
            .execute_unprepared(DELETE_CATEGORIES_SQL)
            .await?;

        Ok(())
    }
}

const INSERT_CATEGORIES_SQL: &str = r#"
INSERT INTO system_config_categories (
    code,
    name,
    description,
    sort_order,
    is_active,
    created_at,
    updated_at
) VALUES
    ('platform_basic', '平台基础', '平台级展示与商户开通相关的基础设置。', 10, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('notification_sms', '通知与短信', '平台对外通知和短信通道配置。', 20, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP),
    ('file_storage', '文件存储', '上传文件的默认存储方式与对象存储参数。', 30, true, CURRENT_TIMESTAMP, CURRENT_TIMESTAMP)
ON CONFLICT (code) DO NOTHING
"#;

const INSERT_ITEMS_SQL: &str = r#"
INSERT INTO system_config_items (
    uuid,
    category_code,
    key,
    label,
    description,
    value_type,
    default_value,
    value,
    validation,
    ui_schema,
    is_required,
    is_editable,
    is_sensitive,
    sort_order,
    created_at,
    updated_at
) VALUES
    (
        '00000000-0000-0000-0000-000000000020'::uuid,
        'platform_basic',
        'platform.name',
        '平台名称',
        '用于登录页、浏览器标题和平台管理界面的展示名称。',
        'string',
        '"PicoCRM"'::jsonb,
        NULL,
        '{"required":true,"min":2,"max":32}'::jsonb,
        '{"placeholder":"请输入平台名称"}'::jsonb,
        true,
        true,
        false,
        10,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    (
        '00000000-0000-0000-0000-000000000021'::uuid,
        'platform_basic',
        'platform.default_trial_days',
        '商户默认试用天数',
        '平台创建商户时默认附带的试用时长。',
        'number',
        '14'::jsonb,
        NULL,
        '{"required":true,"min":1,"max":365}'::jsonb,
        '{"placeholder":"14"}'::jsonb,
        true,
        true,
        false,
        20,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    (
        '00000000-0000-0000-0000-000000000022'::uuid,
        'platform_basic',
        'platform.maintenance_mode',
        '维护模式',
        '开启后可用于提示当前平台处于维护状态。',
        'bool',
        'false'::jsonb,
        NULL,
        NULL,
        '{"placeholder":"启用维护模式"}'::jsonb,
        false,
        true,
        false,
        30,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    (
        '00000000-0000-0000-0000-000000000023'::uuid,
        'notification_sms',
        'notification.sms_provider',
        '短信服务商',
        '控制平台默认使用的短信通道。',
        'enum',
        '"mock"'::jsonb,
        NULL,
        '{"required":true,"options":["mock","aliyun","tencent"]}'::jsonb,
        '{"options":["mock","aliyun","tencent"]}'::jsonb,
        true,
        true,
        false,
        10,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    (
        '00000000-0000-0000-0000-000000000024'::uuid,
        'notification_sms',
        'notification.sms_api_key',
        '短信 API Key',
        '接入真实短信服务时填写；敏感字段会以掩码展示。',
        'string',
        '""'::jsonb,
        NULL,
        '{"max":128}'::jsonb,
        '{"widget":"password","placeholder":"未配置时留空"}'::jsonb,
        false,
        true,
        true,
        20,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    (
        '00000000-0000-0000-0000-000000000025'::uuid,
        'notification_sms',
        'notification.admin_sms_template_id',
        '商户开通短信模板 ID',
        '平台给新商户发送开通通知时使用的短信模板。',
        'string',
        '""'::jsonb,
        NULL,
        '{"max":64}'::jsonb,
        '{"placeholder":"例如：SMS_123456789"}'::jsonb,
        false,
        true,
        false,
        30,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    (
        '00000000-0000-0000-0000-000000000026'::uuid,
        'file_storage',
        'storage.driver',
        '文件存储驱动',
        '控制上传文件默认使用本地存储还是对象存储。',
        'enum',
        '"local"'::jsonb,
        NULL,
        '{"required":true,"options":["local","s3"]}'::jsonb,
        '{"options":["local","s3"]}'::jsonb,
        true,
        true,
        false,
        10,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    (
        '00000000-0000-0000-0000-000000000027'::uuid,
        'file_storage',
        'storage.bucket',
        '对象存储 Bucket',
        '启用 S3 / 兼容 S3 时填写 Bucket 名称。',
        'string',
        '""'::jsonb,
        NULL,
        '{"max":128}'::jsonb,
        '{"placeholder":"例如：pico-crm-prod"}'::jsonb,
        false,
        true,
        false,
        20,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    ),
    (
        '00000000-0000-0000-0000-000000000028'::uuid,
        'file_storage',
        'storage.region',
        '对象存储 Region',
        '启用 S3 / 兼容 S3 时填写存储地域。',
        'string',
        '""'::jsonb,
        NULL,
        '{"max":64}'::jsonb,
        '{"placeholder":"例如：ap-east-1"}'::jsonb,
        false,
        true,
        false,
        30,
        CURRENT_TIMESTAMP,
        CURRENT_TIMESTAMP
    )
ON CONFLICT (key) DO NOTHING
"#;

const DELETE_ITEMS_SQL: &str = r#"
DELETE FROM system_config_items
WHERE uuid IN (
    '00000000-0000-0000-0000-000000000020'::uuid,
    '00000000-0000-0000-0000-000000000021'::uuid,
    '00000000-0000-0000-0000-000000000022'::uuid,
    '00000000-0000-0000-0000-000000000023'::uuid,
    '00000000-0000-0000-0000-000000000024'::uuid,
    '00000000-0000-0000-0000-000000000025'::uuid,
    '00000000-0000-0000-0000-000000000026'::uuid,
    '00000000-0000-0000-0000-000000000027'::uuid,
    '00000000-0000-0000-0000-000000000028'::uuid
)
"#;

const DELETE_CATEGORIES_SQL: &str = r#"
DELETE FROM system_config_categories
WHERE code IN ('platform_basic', 'notification_sms', 'file_storage')
  AND NOT EXISTS (
    SELECT 1
    FROM system_config_items
    WHERE system_config_items.category_code = system_config_categories.code
  )
"#;
