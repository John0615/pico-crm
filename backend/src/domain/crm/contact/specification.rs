use crate::domain::crm::contact::CustomerStatus;
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
                    "必须是有效的国际电话号码格式",
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

    fn is_valid_phone(phone: &str) -> bool {
        phone.starts_with('+') && phone.len() >= 8
    }
}

#[derive(Default, Debug)]
pub struct ContactFilters {
    pub name: Option<String>,
    pub status: Option<CustomerStatus>,
    pub phone: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SortOption {
    ByName(SortDirection),
    ByLastContact(SortDirection),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SortDirection {
    Asc,
    Desc,
}
