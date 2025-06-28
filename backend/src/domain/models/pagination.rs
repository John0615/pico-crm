use crate::domain::errors::pagination::PaginationError;

/// 分页协议（业务概念）
#[derive(Debug, Clone)]
pub struct Pagination {
    pub page: u64,
    pub size: u64,
}

impl Pagination {
    /// 创建分页（应用层调用）
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

    /// 获取当前页（从1开始）
    pub fn page(&self) -> u64 {
        self.page
    }

    /// 获取每页数量（业务规则保证 ≤100）
    pub fn size(&self) -> u64 {
        self.size
    }

    /// 计算偏移量（仓储层使用）
    pub fn offset(&self) -> u64 {
        (self.page - 1) * self.size
    }
}

/// 分页结果
#[derive(Debug)]
pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total: u64,
    pub has_more: bool,
}
