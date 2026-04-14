use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum ValidationError {
    InvalidFormat { field: String, reason: String },
    BusinessRule { rule: String, details: String },
    MissingField(String),
}

impl fmt::Display for ValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::InvalidFormat { field, reason } => {
                write!(f, "字段'{}'格式错误: {}", field, reason)
            }
            Self::BusinessRule { rule, details } => {
                write!(f, "违反业务规则[{}]: {}", rule, details)
            }
            Self::MissingField(field) => write!(f, "缺少必填字段: {}", field),
        }
    }
}

impl std::error::Error for ValidationError {}

impl ValidationError {
    pub fn invalid_format(field: &str, reason: &str) -> Self {
        Self::InvalidFormat {
            field: field.to_string(),
            reason: reason.to_string(),
        }
    }

    pub fn business_rule(rule: &str, details: &str) -> Self {
        Self::BusinessRule {
            rule: rule.to_string(),
            details: details.to_string(),
        }
    }
}
