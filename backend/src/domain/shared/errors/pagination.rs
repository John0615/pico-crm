use std::fmt;

#[derive(Debug, PartialEq, Clone)]
pub enum PaginationError {
    SizeExceeded { max_size: u64, actual_size: u64 },
    InvalidPage,
    ParameterConflict,
}

impl fmt::Display for PaginationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SizeExceeded {
                max_size,
                actual_size,
            } => write!(f, "每页数量不能超过{}，当前值：{}", max_size, actual_size),
            Self::InvalidPage => write!(f, "页码必须大于等于1"),
            Self::ParameterConflict => {
                write!(f, "分页参数冲突，请统一使用page/size或offset/limit模式")
            }
        }
    }
}

impl std::error::Error for PaginationError {}
