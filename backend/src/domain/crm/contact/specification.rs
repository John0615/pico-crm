use crate::domain::shared::errors::validation::ValidationError;

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

        Self::validate_filters(&filters)?;
        Self::validate_sort(&sort)?;

        Ok(Self { filters, sort })
    }

    fn validate_filters(filters: &ContactFilters) -> Result<(), ValidationError> {
        if let Some(phone) = &filters.phone {
            if !Self::is_valid_phone(phone) {
                return Err(ValidationError::invalid_format(
                    "phone",
                    "必须是有效的手机号码格式",
                ));
            }
        }

        if let Some(address_keyword) = &filters.address_keyword {
            if address_keyword.chars().count() > 255 {
                return Err(ValidationError::business_rule(
                    "address_keyword",
                    "地址关键字长度不能超过255个字符",
                ));
            }
        }

        if let Some(tag) = &filters.tag {
            if tag.trim().is_empty() || tag.chars().count() > 20 {
                return Err(ValidationError::business_rule(
                    "tag",
                    "标签筛选值不能为空且长度不能超过20个字符",
                ));
            }
        }

        if let Some(follow_up_status) = &filters.follow_up_status {
            if !matches!(
                follow_up_status.as_str(),
                "pending" | "contacted" | "quoted" | "scheduled" | "completed"
            ) {
                return Err(ValidationError::business_rule(
                    "follow_up_status",
                    "跟进状态必须是预定义值",
                ));
            }
        }

        Ok(())
    }

    fn validate_sort(sort: &[SortOption]) -> Result<(), ValidationError> {
        let mut fields = std::collections::HashSet::new();
        for opt in sort {
            let field = match opt {
                SortOption::ByName(_) => "name",
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

    fn is_valid_phone(phone: &str) -> bool {
        let trimmed = phone.trim();
        let is_cn_mobile = trimmed.len() == 11
            && trimmed.starts_with('1')
            && trimmed.chars().all(|c| c.is_ascii_digit());
        let is_international = trimmed.starts_with('+')
            && trimmed.len() >= 8
            && trimmed[1..].chars().all(|c| c.is_ascii_digit());
        is_cn_mobile || is_international
    }
}

#[derive(Default, Debug)]
pub struct ContactFilters {
    pub name: Option<String>,
    pub phone: Option<String>,
    pub address_keyword: Option<String>,
    pub tag: Option<String>,
    pub follow_up_status: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SortOption {
    ByName(SortDirection),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_cn_mobile_and_extended_filters() {
        let filters = ContactFilters {
            name: Some("张".to_string()),
            phone: Some("13800138000".to_string()),
            address_keyword: Some("望京".to_string()),
            tag: Some("VIP".to_string()),
            follow_up_status: Some("scheduled".to_string()),
        };

        assert!(ContactSpecification::new(Some(filters), None).is_ok());
    }

    #[test]
    fn rejects_invalid_follow_up_status_filter() {
        let filters = ContactFilters {
            follow_up_status: Some("invalid".to_string()),
            ..Default::default()
        };

        let err = ContactSpecification::new(Some(filters), None)
            .expect_err("invalid follow up status should fail");
        assert_eq!(
            err,
            ValidationError::BusinessRule {
                rule: "follow_up_status".to_string(),
                details: "跟进状态必须是预定义值".to_string(),
            }
        );
    }
}
