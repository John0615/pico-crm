use crate::domain::models::user::User;

pub trait AdminUserRepository: Send + Sync {
    fn find_by_username(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send;

    fn update_user(
        &self,
        user: User,
    ) -> impl std::future::Future<Output = Result<User, String>> + Send;
}
