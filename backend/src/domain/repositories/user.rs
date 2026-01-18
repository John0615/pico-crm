use crate::domain::models::user::User;

pub trait UserRepository: Send + Sync {
    fn create_user(
        &self,
        user: User,
    ) -> impl std::future::Future<Output = Result<User, String>> + Send;

    fn update_user(
        &self,
        user: User,
    ) -> impl std::future::Future<Output = Result<User, String>> + Send;

    fn delete_user(
        &self,
        uuid: String,
    ) -> impl std::future::Future<Output = Result<(), String>> + Send;

    // 命令操作相关的查询方法
    fn find_user_by_uuid(
        &self,
        uuid: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send;

    fn find_user_by_username(
        &self,
        username: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send;

    fn find_user_by_email(
        &self,
        email: &str,
    ) -> impl std::future::Future<Output = Result<Option<User>, String>> + Send;
}