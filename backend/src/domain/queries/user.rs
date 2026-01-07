use std::fmt::Debug;

pub trait UserQuery: Send + Sync {
    type Result: Debug + Send + Sync;

    fn get_user(
        &self,
        user_name: &str,
    ) -> impl std::future::Future<Output = Result<Option<Self::Result>, String>> + Send;
}
