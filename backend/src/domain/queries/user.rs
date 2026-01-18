use std::fmt::Debug;
use shared::user::{UserListQuery, PagedResult};

pub trait UserQuery: Send + Sync {
    type Result: Debug + Send + Sync;

    fn get_user(
        &self,
        user_name: &str,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send;

    fn list_users(
        &self,
        query: UserListQuery,
    ) -> impl std::future::Future<Output = Result<PagedResult<Self::Result>, String>> + Send;
}
