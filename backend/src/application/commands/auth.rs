use crate::domain::auth::provider::{AuthCredential, AuthProvider};

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
}
