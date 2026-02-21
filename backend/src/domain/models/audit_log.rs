use serde_json::Value;

#[derive(Debug, Clone)]
pub struct AuditLogCreate {
    pub actor_id: Option<String>,
    pub actor_role: Option<String>,
    pub action: String,
    pub entity: String,
    pub entity_id: Option<String>,
    pub before_data: Option<Value>,
    pub after_data: Option<Value>,
    pub ip: Option<String>,
    pub user_agent: Option<String>,
}
