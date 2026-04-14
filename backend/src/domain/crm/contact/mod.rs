pub mod model;
pub mod query;
pub mod repository;
pub mod specification;

pub use model::{Contact, CustomerStatus, CustomerValue, UpdateContact};
pub use query::ContactQuery;
pub use repository::ContactRepository;
pub use specification::{ContactFilters, ContactSpecification, SortDirection, SortOption};
