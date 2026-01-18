use leptos::prelude::*;
use server_fn::ServerFnError;
use serde::{Deserialize, Serialize};

// 重新导出shared中的结构
pub use shared::user::{User as SharedUser, CreateUserRequest};

// 前端显示用的User结构（与表格显示匹配）
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct User {
    pub id: u32,
    pub uuid: String,
    pub name: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub last_login: String,
    pub created_at: String,
}

// 用户查询参数
#[derive(Clone, Debug, Serialize, Deserialize, Default)]
pub struct UserQuery {
    pub page: u64,
    pub page_size: u64,
    pub name: Option<String>,
    pub role: Option<String>,
    pub status: Option<String>,
}

// 列表结果
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ListResult<T> {
    pub items: Vec<T>,
    pub total: u64,
}

// 将shared::User转换为前端User
impl From<SharedUser> for User {
    fn from(shared_user: SharedUser) -> Self {
        // 从uuid生成简单的数字ID（仅用于前端显示）
        let id = shared_user.uuid.chars()
            .filter(|c| c.is_ascii_digit())
            .collect::<String>()
            .parse::<u32>()
            .unwrap_or(1);

        Self {
            id,
            uuid: shared_user.uuid,
            name: shared_user.user_name,
            email: shared_user.email.unwrap_or_default(),
            role: match shared_user.is_admin {
                Some(true) => "管理员".to_string(),
                Some(false) => "普通用户".to_string(),
                None => "普通用户".to_string(),
            },
            status: match shared_user.status.as_str() {
                "active" => "活跃".to_string(),
                "inactive" => "禁用".to_string(),
                "pending" => "待激活".to_string(),
                _ => "禁用".to_string(),
            },
            last_login: shared_user.last_login_at.unwrap_or("从未登录".to_string()),
            created_at: shared_user.inserted_at,
        }
    }
}

// 获取用户列表
#[server(
    name = FetchUsers,
    prefix = "/api",
    endpoint = "/fetch_users",
)]
pub async fn fetch_users(params: UserQuery) -> Result<ListResult<User>, ServerFnError> {
    use backend::application::queries::user_service::UserAppService;
    use backend::infrastructure::queries::user_query_impl::SeaOrmUserQuery;
    use backend::infrastructure::db::Database;
    use shared::user::UserListQuery;
    use leptos::prelude::*;
    
    // 从上下文获取数据库连接池
    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();
    
    // 创建查询repository和service
    let query_repository = SeaOrmUserQuery::new(db);
    let query_service = UserAppService::new(query_repository);
    
    // 转换查询参数
    let query_params = UserListQuery {
        page: params.page,
        page_size: params.page_size,
        name: params.name,
        role: params.role,
        status: params.status.map(|s| match s.as_str() {
            "活跃" => "active".to_string(),
            "禁用" => "inactive".to_string(),
            "待激活" => "pending".to_string(),
            _ => s,
        }),
    };
    
    // 添加调试日志
    leptos::logging::log!("查询参数: {:?}", query_params);
    
    // 调用查询服务
    let result = query_service.list_users(query_params).await
        .map_err(|e| ServerFnError::new(format!("查询用户失败: {}", e)))?;
    
    // 转换为前端User格式
    let frontend_users: Vec<User> = result.items.into_iter().map(|u| u.into()).collect();
    
    Ok(ListResult {
        items: frontend_users,
        total: result.total,
    })
}

// 创建用户
#[server(
    name = CreateUser,
    prefix = "/api", 
    endpoint = "/create_user",
)]
pub async fn create_user(request: CreateUserRequest) -> Result<User, ServerFnError> {
    use backend::application::commands::user_service::UserCommandService;
    use backend::infrastructure::repositories::user_repository_impl::SeaOrmUserRepository;
    use backend::infrastructure::db::Database;
    use leptos::prelude::*;
    
    // 从上下文获取数据库连接池
    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();
    
    // 创建repository和service
    let repository = SeaOrmUserRepository::new(db);
    let service = UserCommandService::new(repository);
    
    // 调用service创建用户
    let created_user = service.create_user(request).await
        .map_err(|e| ServerFnError::new(e))?;
    
    // 转换为前端User格式
    Ok(created_user.into())
}

// 更新用户
#[server(
    name = UpdateUser,
    prefix = "/api",
    endpoint = "/update_user", 
)]
pub async fn update_user(uuid: String, request: CreateUserRequest) -> Result<User, ServerFnError> {
    use backend::application::commands::user_service::UserCommandService;
    use backend::infrastructure::repositories::user_repository_impl::SeaOrmUserRepository;
    use backend::infrastructure::db::Database;
    use leptos::prelude::*;
    
    // 从上下文获取数据库连接池
    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();
    
    // 创建repository和service
    let repository = SeaOrmUserRepository::new(db);
    let service = UserCommandService::new(repository);
    
    // 调用service更新用户
    let updated_user = service.update_user(&uuid, request).await
        .map_err(|e| ServerFnError::new(e))?;
    
    // 转换为前端User格式
    Ok(updated_user.into())
}

// 删除用户
#[server(
    name = DeleteUser,
    prefix = "/api",
    endpoint = "/delete_user",
)]
pub async fn delete_user(uuid: String) -> Result<(), ServerFnError> {
    use backend::application::commands::user_service::UserCommandService;
    use backend::infrastructure::repositories::user_repository_impl::SeaOrmUserRepository;
    use backend::infrastructure::db::Database;
    use leptos::prelude::*;
    
    // 从上下文获取数据库连接池
    let pool = expect_context::<Database>();
    let db = pool.get_connection().clone();
    
    // 创建repository和service
    let repository = SeaOrmUserRepository::new(db);
    let service = UserCommandService::new(repository);
    
    // 调用service删除用户
    service.delete_user(&uuid).await
        .map_err(|e| ServerFnError::new(e))?;
    
    Ok(())
}