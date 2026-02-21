use std::collections::HashMap;

use serde_json::{json, Value};

use crate::domain::models::audit_log::AuditLogCreate;
use crate::domain::models::system_config::{
    SystemConfigItem, SystemConfigItemUpdate, SystemConfigItemUpdateRequest,
};
use crate::domain::repositories::audit_log::AuditLogRepository;
use crate::domain::repositories::system_config::SystemConfigRepository;

const MASKED_VALUE: &str = "******";

pub struct SystemConfigCommandService<R: SystemConfigRepository, A: AuditLogRepository> {
    repo: R,
    audit_repo: A,
}

impl<R: SystemConfigRepository, A: AuditLogRepository> SystemConfigCommandService<R, A> {
    pub fn new(repo: R, audit_repo: A) -> Self {
        Self { repo, audit_repo }
    }

    pub async fn update_items(
        &self,
        updates: Vec<SystemConfigItemUpdateRequest>,
        actor_id: Option<String>,
        actor_role: Option<String>,
    ) -> Result<Vec<SystemConfigItem>, String> {
        if updates.is_empty() {
            return Ok(Vec::new());
        }

        let keys: Vec<String> = updates.iter().map(|u| u.key.clone()).collect();
        let existing_items = self.repo.find_items_by_keys(keys).await?;
        let mut existing_map: HashMap<String, SystemConfigItem> = HashMap::new();
        for item in existing_items {
            existing_map.insert(item.key.clone(), item);
        }

        let mut apply_updates = Vec::new();
        let mut before_records = Vec::new();
        let mut after_records = Vec::new();

        for req in updates {
            let Some(item) = existing_map.get(&req.key) else {
                return Err(format!("配置项不存在: {}", req.key));
            };
            if !item.is_editable {
                return Err(format!("配置项不可编辑: {}", req.key));
            }

            let new_value = if req.reset_to_default {
                item.default_value.clone()
            } else {
                req.value
                    .clone()
                    .ok_or_else(|| format!("缺少配置值: {}", req.key))?
            };

            if item.is_sensitive && !req.reset_to_default {
                if let Some(Value::String(raw)) = req.value.as_ref() {
                    if raw.trim().is_empty() || raw == MASKED_VALUE {
                        continue;
                    }
                }
            }

            validate_value(item, &new_value)?;

            let before_value = item
                .value
                .clone()
                .unwrap_or_else(|| item.default_value.clone());
            before_records.push(json!({"key": item.key, "value": before_value}));
            after_records.push(json!({"key": item.key, "value": new_value.clone()}));
            apply_updates.push(SystemConfigItemUpdate {
                key: item.key.clone(),
                value: new_value.clone(),
            });
        }

        if apply_updates.is_empty() {
            return Ok(Vec::new());
        }

        let updated_items = self.repo.update_items(apply_updates).await?;

        let audit = AuditLogCreate {
            actor_id,
            actor_role,
            action: "system_config_update".to_string(),
            entity: "system_config".to_string(),
            entity_id: None,
            before_data: Some(Value::Array(before_records)),
            after_data: Some(Value::Array(after_records)),
            ip: None,
            user_agent: None,
        };
        self.audit_repo.create_log(audit).await?;

        Ok(updated_items)
    }
}

fn validate_value(item: &SystemConfigItem, value: &Value) -> Result<(), String> {
    let validation = item.validation.as_ref();
    let required = item.is_required
        || validation
            .and_then(|v| v.get("required"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

    if required {
        match value {
            Value::Null => return Err(format!("{} 不能为空", item.label)),
            Value::String(s) if s.trim().is_empty() => {
                return Err(format!("{} 不能为空", item.label))
            }
            _ => {}
        }
    }

    match item.value_type.as_str() {
        "string" => {
            let Some(raw) = value.as_str() else {
                return Err(format!("{} 必须是字符串", item.label));
            };
            if let Some(min) = validation.and_then(|v| v.get("min")).and_then(|v| v.as_u64()) {
                if raw.len() < min as usize {
                    return Err(format!("{} 长度不能小于 {}", item.label, min));
                }
            }
            if let Some(max) = validation.and_then(|v| v.get("max")).and_then(|v| v.as_u64()) {
                if raw.len() > max as usize {
                    return Err(format!("{} 长度不能大于 {}", item.label, max));
                }
            }
        }
        "number" => {
            let number = value.as_f64().ok_or_else(|| format!("{} 必须是数字", item.label))?;
            if let Some(min) = validation.and_then(|v| v.get("min")).and_then(|v| v.as_f64()) {
                if number < min {
                    return Err(format!("{} 不能小于 {}", item.label, min));
                }
            }
            if let Some(max) = validation.and_then(|v| v.get("max")).and_then(|v| v.as_f64()) {
                if number > max {
                    return Err(format!("{} 不能大于 {}", item.label, max));
                }
            }
        }
        "bool" => {
            if value.as_bool().is_none() {
                return Err(format!("{} 必须是布尔值", item.label));
            }
        }
        "enum" => {
            let Some(raw) = value.as_str() else {
                return Err(format!("{} 必须是字符串选项", item.label));
            };
            if let Some(options) = validation
                .and_then(|v| v.get("options"))
                .and_then(|v| v.as_array())
            {
                let matched = options.iter().any(|opt| opt.as_str() == Some(raw));
                if !matched {
                    return Err(format!("{} 选项无效", item.label));
                }
            }
        }
        "json" => {}
        _ => {}
    }

    Ok(())
}
