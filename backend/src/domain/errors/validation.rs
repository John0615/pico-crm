use std::fmt;

/// 专注业务规则验证的错误类型
#[derive(Debug, PartialEq, Clone)] // 添加 Clone 以便错误传递
pub enum ValidationError {
    // 字段格式错误（邮箱、电话等）
    InvalidFormat { field: String, reason: String },
    // 违反业务规则（状态流转等）
    BusinessRule { rule: String, details: String },
    // 必填字段缺失
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

// 快捷构造方法
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
