pub mod follow_record;
pub mod model;
pub mod query;
pub mod repository;
pub mod specification;

pub use follow_record::{
    ContactFollowRecord, ContactFollowRecordQuery, ContactFollowRecordRepository,
    CreateContactFollowRecord,
};
pub use model::{Contact, FollowUpStatus, UpdateContact};
pub use query::ContactQuery;
pub use repository::ContactRepository;
pub use specification::{ContactFilters, ContactSpecification, SortDirection, SortOption};
