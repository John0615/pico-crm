use serde_json::Value;

use crate::domain::models::system_config::{SystemConfigCategory, SystemConfigItem};
use shared::system_config::{SystemConfigCategoryDto, SystemConfigItemDto};

const MASKED_VALUE: &str = "******";

pub fn to_item_dto(item: SystemConfigItem) -> SystemConfigItemDto {
    let default_value = item.default_value;
    let effective_value = item.value.unwrap_or_else(|| default_value.clone());

    let masked = Value::String(MASKED_VALUE.to_string());
    let value = if item.is_sensitive {
        masked.clone()
    } else {
        effective_value
    };
    let default_value = if item.is_sensitive {
        masked
    } else {
        default_value
    };

    SystemConfigItemDto {
        key: item.key,
        category_code: item.category_code,
        label: item.label,
        description: item.description,
        value_type: item.value_type,
        default_value,
        value,
        validation: item.validation,
        ui_schema: item.ui_schema,
        is_required: item.is_required,
        is_editable: item.is_editable,
        is_sensitive: item.is_sensitive,
        sort_order: item.sort_order,
    }
}

pub fn to_category_dto(category: SystemConfigCategory) -> SystemConfigCategoryDto {
    let items = category.items.into_iter().map(to_item_dto).collect();
    SystemConfigCategoryDto {
        code: category.code,
        name: category.name,
        description: category.description,
        sort_order: category.sort_order,
        is_active: category.is_active,
        items,
    }
}
