use crate::domain::identity::auth::{AuthCredential, AuthProvider};
use crate::domain::identity::user::{User as DomainUser, UserRepository};
use shared::user::User;

pub struct AuthAppService<A: AuthProvider, R: UserRepository> {
    auth_provider: A,
    user_repository: R,
}

impl<A: AuthProvider, R: UserRepository> AuthAppService<A, R> {
    pub fn new(auth_provider: A, user_repository: R) -> Self {
        Self {
            auth_provider,
            user_repository,
        }
    }

    pub async fn authenticate(&self, user_name: &str, password: &str) -> Result<String, String> {
        let AuthCredential(token) = self.auth_provider.authenticate(user_name, password).await?;

        // 登录成功后更新最后登录时间
        if let Ok(Some(mut user)) = self.user_repository.find_user_by_username(user_name).await {
            user.record_login();
            let _ = self.user_repository.update_user(user).await; // 忽略更新错误，不影响登录流程
        }

        Ok(token)
    }

    pub async fn get_user_by_token(&self, token: String) -> Result<Option<User>, String> {
        let auth_credential = AuthCredential(token);
        let user = self
            .auth_provider
            .get_current_user(&auth_credential)
            .await?
            .map(|domain_user: DomainUser| domain_user.into());
        Ok(user)
    }
}
