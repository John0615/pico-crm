use crate::domain::repositories::admin_user::AdminUserRepository;
use crate::infrastructure::auth::jwt_provider::JwtAuthProvider;

pub struct AdminAuthAppService<R: AdminUserRepository> {
    repo: R,
    jwt: JwtAuthProvider,
}

impl<R: AdminUserRepository> AdminAuthAppService<R> {
    pub fn new(repo: R, jwt: JwtAuthProvider) -> Self {
        Self { repo, jwt }
    }

    pub async fn authenticate(&self, user_name: &str, password: &str) -> Result<String, String> {
        let mut admin = self
            .repo
            .find_by_username(user_name)
            .await?
            .ok_or_else(|| "用户名或密码错误".to_string())?;

        if !admin.verify_password(password)? {
            return Err("用户名或密码错误".to_string());
        }

        if !admin.is_active() {
            return Err("管理员账户已被禁用".to_string());
        }

        admin.set_admin(true);

        let token = self
            .jwt
            .issue_token(&admin, "public".to_string(), "admin".to_string())
            .map_err(|e| format!("生成 Token 失败：{}", e))?;

        admin.record_login();
        let _ = self.repo.update_user(admin).await;

        Ok(token)
    }
}
