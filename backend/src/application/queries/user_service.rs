use crate::domain::models::user::User as DomainUser;
use crate::domain::queries::user::UserQuery as UQuery;
use shared::user::{User, UserListQuery, PagedResult};

pub struct UserAppService<R: UQuery> {
    user_query: R,
}

impl<R: UQuery<Result = DomainUser>> UserAppService<R> {
    pub fn new(user_query: R) -> Self {
        Self { user_query }
    }

    pub async fn fetch_user(
        &self,
        user_name: String,
        password: String,
    ) -> Result<Option<User>, String> {
        let result = self
            .user_query
            .get_user(&user_name)
            .await
            .map_err(|e| e.to_string())?;
        let new_result = if let Some(user) = result {
            if user.verify_password(&password)? {
                Some(user.into())
            } else {
                None
            }
        } else {
            None
        };
        Ok(new_result)
    }

    pub async fn list_users(&self, query: UserListQuery) -> Result<PagedResult<User>, String> {
        let result = self
            .user_query
            .list_users(query)
            .await
            .map_err(|e| e.to_string())?;

        // 转换为shared::User
        let shared_users: Vec<User> = result
            .items
            .into_iter()
            .map(|domain_user| domain_user.into())
            .collect();

        Ok(PagedResult {
            items: shared_users,
            total: result.total,
        })
    }
}
