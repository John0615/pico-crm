use crate::domain::platform::system_config::{SystemConfigCategory, SystemConfigItem};
use crate::infrastructure::entity::system_config_categories as categories;
use crate::infrastructure::entity::system_config_items as items;

pub struct SystemConfigMapper;

impl SystemConfigMapper {
    pub fn item_to_domain(model: items::Model) -> SystemConfigItem {
        SystemConfigItem {
            uuid: model.uuid.to_string(),
            category_code: model.category_code,
            key: model.key,
            label: model.label,
            description: model.description,
            value_type: model.value_type,
            default_value: model.default_value,
            value: model.value,
            validation: model.validation,
            ui_schema: model.ui_schema,
            is_required: model.is_required,
            is_editable: model.is_editable,
            is_sensitive: model.is_sensitive,
            sort_order: model.sort_order,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }

    pub fn category_to_domain(
        model: categories::Model,
        items: Vec<SystemConfigItem>,
    ) -> SystemConfigCategory {
        SystemConfigCategory {
            code: model.code,
            name: model.name,
            description: model.description,
            sort_order: model.sort_order,
            is_active: model.is_active,
            items,
            created_at: model.created_at,
            updated_at: model.updated_at,
        }
    }
}
