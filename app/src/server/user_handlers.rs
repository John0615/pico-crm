use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::user::{CreateUserRequest, PagedResult, User, UserListQuery};

// 重新导出shared中的类型，供前端使用
pub use shared::user::{
    CreateUserRequest as ExportedCreateUserRequest, PagedResult as ExportedPagedResult,
    User as ExportedUser, UserListQuery as ExportedUserListQuery,
};

// 获取用户列表
#[server(
    name = FetchUsers,
    prefix = "/api",
    endpoint = "/fetch_users",
)]
pub async fn fetch_users(params: UserListQuery) -> Result<PagedResult<User>, ServerFnError> {
    use backend::application::queries::user_service::UserAppService;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::queries::user_query_impl::SeaOrmUserQuery;
    use leptos::prelude::*;

    // 从上下文获取数据库连接池
    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();

    // 创建查询repository和service
    let query_repository = SeaOrmUserQuery::new(db);
    let query_service = UserAppService::new(query_repository);

    // 转换查询参数中的状态值
    let mut query_params = params;
    if let Some(status) = query_params.status.as_mut() {
        *status = match status.as_str() {
            "活跃" => "active".to_string(),
            "禁用" => "inactive".to_string(),
            "待激活" => "pending".to_string(),
            _ => status.clone(),
        };
    }

    // 调用查询服务
    let result = query_service
        .list_users(query_params)
        .await
        .map_err(|e| ServerFnError::new(format!("查询用户失败: {}", e)))?;

    // 直接返回shared::User格式的结果
    Ok(result)
}

// 创建用户
#[server(
    name = CreateUser,
    prefix = "/api",
    endpoint = "/create_user",
)]
pub async fn create_user(request: CreateUserRequest) -> Result<User, ServerFnError> {
    use backend::application::commands::user_service::UserCommandService;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::repositories::user_repository_impl::SeaOrmUserRepository;
    use leptos::prelude::*;

    // 从上下文获取数据库连接池
    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();

    // 创建repository和service
    let repository = SeaOrmUserRepository::new(db);
    let service = UserCommandService::new(repository);

    // 调用service创建用户
    let created_user = service
        .create_user(request)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    // 直接返回shared::User格式
    Ok(created_user)
}

// 更新用户
#[server(
    name = UpdateUser,
    prefix = "/api",
    endpoint = "/update_user",
)]
pub async fn update_user(uuid: String, request: CreateUserRequest) -> Result<User, ServerFnError> {
    use backend::application::commands::user_service::UserCommandService;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::repositories::user_repository_impl::SeaOrmUserRepository;
    use leptos::prelude::*;

    // 从上下文获取数据库连接池
    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();

    // 创建repository和service
    let repository = SeaOrmUserRepository::new(db);
    let service = UserCommandService::new(repository);

    // 调用service更新用户
    let updated_user = service
        .update_user(&uuid, request)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    // 直接返回shared::User格式
    Ok(updated_user)
}

// 删除用户
#[server(
    name = DeleteUser,
    prefix = "/api",
    endpoint = "/delete_user",
)]
pub async fn delete_user(uuid: String) -> Result<(), ServerFnError> {
    use backend::application::commands::user_service::UserCommandService;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::repositories::user_repository_impl::SeaOrmUserRepository;
    use leptos::prelude::*;

    // 从上下文获取数据库连接池
    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();

    // 创建repository和service
    let repository = SeaOrmUserRepository::new(db);
    let service = UserCommandService::new(repository);

    // 调用service删除用户
    service
        .delete_user(&uuid)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    Ok(())
}
