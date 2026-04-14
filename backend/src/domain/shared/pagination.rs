use crate::domain::shared::errors::pagination::PaginationError;

#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: u64,
    pub size: u64,
}

impl Pagination {
    pub fn new(page: u64, size: u64) -> Result<Self, PaginationError> {
        const MAX_PAGE_SIZE: u64 = 100;

        if page == 0 {
            return Err(PaginationError::InvalidPage);
        }
        if size > MAX_PAGE_SIZE {
            return Err(PaginationError::SizeExceeded {
                max_size: MAX_PAGE_SIZE,
                actual_size: size,
            });
        }
        Ok(Self { page, size })
    }

    pub fn page(&self) -> u64 {
        self.page
    }

    pub fn size(&self) -> u64 {
        self.size
    }

    pub fn offset(&self) -> u64 {
        (self.page - 1) * self.size
    }
}

#[derive(Debug)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub has_more: bool,
}
