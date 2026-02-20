use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::user::{CreateUserRequest, PagedResult, User, UserListQuery};

// 获取单个用户
#[server(
    name = GetUser,
    prefix = "/api",
    endpoint = "/get_user",
)]
pub async fn get_user(uuid: String) -> Result<User, ServerFnError> {
    use backend::infrastructure::queries::user_query_impl::SeaOrmUserQuery;
    use backend::application::queries::user_service::UserAppService;
    use backend::infrastructure::db::Database;
    use leptos::prelude::*;
    
    // 从上下文获取数据库连接池
    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();
    
    // 创建查询repository和service
    let query_repository = SeaOrmUserQuery::new(db);
    let query_service = UserAppService::new(query_repository);
    
    // 调用查询服务获取用户
    let user = query_service.get_user_by_uuid(&uuid).await
        .map_err(|e| ServerFnError::new(format!("查询用户失败: {}", e)))?
        .ok_or_else(|| ServerFnError::new("用户不存在".to_string()))?;
    
    Ok(user)
}
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

    // 添加调试日志
    leptos::logging::log!("查询参数: {:?}", params);

    // 调用查询服务
    let result = query_service
        .list_users(params)
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
    use axum::extract::Extension;
    use backend::application::commands::user_service::UserCommandService;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::repositories::user_repository_impl::SeaOrmUserRepository;
    use leptos::prelude::*;
    use leptos_axum::extract;

    let Extension(current_user): Extension<User> = extract().await?;
    if current_user.is_admin.unwrap_or(false) || current_user.role != "operator" {
        return Err(ServerFnError::new("无权限创建用户".to_string()));
    }

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

// 切换用户状态
#[server(
    name = ToggleUserStatusFn,
    prefix = "/api",
    endpoint = "/toggle_user_status",
)]
pub async fn toggle_user_status(uuid: String) -> Result<User, ServerFnError> {
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

    // 获取当前用户状态并切换
    let current_user = service.get_user_by_uuid(&uuid).await
        .map_err(|e| ServerFnError::new(e))?
        .ok_or_else(|| ServerFnError::new("用户不存在".to_string()))?;

    let updated_user = if current_user.status == "active" {
        // 如果当前是活跃状态，则禁用
        service.deactivate_user(&uuid).await
            .map_err(|e| ServerFnError::new(e))?
    } else {
        // 如果当前是非活跃状态，则激活
        service.activate_user(&uuid).await
            .map_err(|e| ServerFnError::new(e))?
    };

    Ok(updated_user)
}
