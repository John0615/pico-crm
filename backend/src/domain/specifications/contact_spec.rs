use crate::domain::errors::validation::ValidationError;

/// 联系人查询规约
#[derive(Debug)]
pub struct ContactSpecification {
    pub filters: ContactFilters,
    pub sort: Vec<SortOption>,
}

impl ContactSpecification {
    pub fn new(
        filters: Option<ContactFilters>,
        sort: Option<Vec<SortOption>>,
    ) -> Result<Self, ValidationError> {
        let filters = filters.unwrap_or_default();
        let sort = sort.unwrap_or_default();

        // 校验filters
        Self::validate_filters(&filters)?;

        // 校验sort规则
        Self::validate_sort(&sort)?;

        Ok(Self { filters, sort })
    }

    fn validate_filters(filters: &ContactFilters) -> Result<(), ValidationError> {
        // 1. 状态校验
        if let Some(status) = &filters.status {
            let valid_statuses = Self::valid_statuses();
            if !valid_statuses.contains(&status.as_str()) {
                return Err(ValidationError::business_rule(
                    "contact_status",
                    &format!("允许的状态: {:?}", valid_statuses),
                ));
            }
        }

        // 2. 邮箱格式校验
        if let Some(email) = &filters.email {
            if !Self::is_valid_email(email) {
                return Err(ValidationError::invalid_format(
                    "email",
                    "必须包含@符号且长度≤254",
                ));
            }
        }

        // 3. 电话格式校验
        if let Some(phone) = &filters.phone {
            if !Self::is_valid_phone(phone) {
                return Err(ValidationError::invalid_format(
                    "phone",
                    "必须是有效的国际电话号码格式",
                ));
            }
        }

        Ok(())
    }

    fn validate_sort(sort: &[SortOption]) -> Result<(), ValidationError> {
        // 1. 禁止重复排序字段
        let mut fields = std::collections::HashSet::new();
        for opt in sort {
            let field = match opt {
                SortOption::ByName(_) => "name",
                SortOption::ByLastContact(_) => "last_contact",
            };
            if !fields.insert(field) {
                return Err(ValidationError::business_rule(
                    "sort",
                    &format!("重复的排序字段: {}", field),
                ));
            }
        }

        Ok(())
    }

    fn valid_statuses() -> Vec<&'static str> {
        vec!["active", "inactive", "pending"]
    }

    fn is_valid_email(email: &str) -> bool {
        // 简化的校验逻辑
        email.contains('@') && email.len() <= 254
    }

    fn is_valid_phone(phone: &str) -> bool {
        // 简化的国际电话校验
        phone.starts_with('+') && phone.len() >= 8
    }
}

/// 过滤条件（强制非空校验）
#[derive(Default, Debug)]
pub struct ContactFilters {
    pub name: Option<String>,
    pub status: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
}

/// 排序选项（带业务规则校验）
#[derive(Debug, Clone)]
pub enum SortOption {
    ByName(SortDirection),
    ByLastContact(SortDirection),
}

/// 排序方向
#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}
