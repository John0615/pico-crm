use crate::domain::auth::provider::{AuthCredential, AuthProvider};
use crate::domain::models::user::User as DomainUser;
use shared::user::User;

pub struct AuthAppService<A: AuthProvider> {
    auth_provider: A,
}

impl<A: AuthProvider> AuthAppService<A> {
    pub fn new(auth_provider: A) -> Self {
        Self { auth_provider }
    }

    pub async fn authenticate(&self, user_name: &str, password: &str) -> Result<String, String> {
        let AuthCredential(token) = self.auth_provider.authenticate(user_name, password).await?;
        Ok(token)
    }

    pub async fn get_user_by_token(&self, token: String) -> Result<Option<User>, String> {
        let auth_credential = AuthCredential(token);
        let user = self.auth_provider.get_current_user(&auth_credential).await?.map(|user: DomainUser| user.into());
        Ok(user)
    }
}
