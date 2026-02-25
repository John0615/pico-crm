use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::schedule::{
    CreateScheduleAssignment, Schedule, ScheduleQuery, UpdateScheduleAssignment,
    UpdateScheduleStatus,
};
#[cfg(feature = "ssr")]
use shared::user::User;
use shared::ListResult;

#[cfg(feature = "ssr")]
mod ssr {
    pub use backend::application::commands::schedule_service::ScheduleAppService;
    pub use backend::application::queries::schedule_service::ScheduleQueryService;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::queries::schedule_query_impl::SeaOrmScheduleQuery;
    pub use backend::infrastructure::repositories::order_repository_impl::SeaOrmOrderRepository;
    pub use backend::infrastructure::repositories::schedule_repository_impl::SeaOrmScheduleRepository;
    pub use backend::infrastructure::tenant::TenantContext;
}

#[server(
    name = FetchSchedulesFn,
    prefix = "/api",
    endpoint = "/fetch_schedules",
)]
pub async fn fetch_schedules(params: ScheduleQuery) -> Result<ListResult<Schedule>, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(current_user): Extension<User> = extract().await?;
    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();

    let mut params = params;
    if is_worker(&current_user) {
        params.assigned_user_uuid = Some(current_user.uuid.clone());
    }

    let query = SeaOrmScheduleQuery::new(pool.connection.clone(), tenant.schema_name);
    let service = ScheduleQueryService::new(query);

    let result = service
        .fetch_schedules(params)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = GetScheduleFn,
    prefix = "/api",
    endpoint = "/get_schedule",
)]
pub async fn get_schedule(uuid: String) -> Result<Option<Schedule>, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(current_user): Extension<User> = extract().await?;
    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();

    let query = SeaOrmScheduleQuery::new(pool.connection.clone(), tenant.schema_name);
    let service = ScheduleQueryService::new(query);

    let schedule = service
        .fetch_schedule(uuid)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    if is_worker(&current_user) {
        if let Some(item) = schedule.as_ref() {
            if item.assigned_user_uuid.as_deref() != Some(current_user.uuid.as_str()) {
                return Err(ServerFnError::new("无权限查看该排班".to_string()));
            }
        }
    }

    Ok(schedule)
}

#[server(
    name = CreateScheduleFn,
    prefix = "/api",
    endpoint = "/create_schedule",
)]
pub async fn create_schedule(
    order_uuid: String,
    payload: CreateScheduleAssignment,
) -> Result<Schedule, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(current_user): Extension<User> = extract().await?;
    ensure_operator(&current_user)?;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();

    let schema_name = tenant.schema_name.clone();
    let order_repo = SeaOrmOrderRepository::new(pool.connection.clone(), schema_name.clone());
    let schedule_repo = SeaOrmScheduleRepository::new(pool.connection.clone(), schema_name);
    let service = ScheduleAppService::new(order_repo, schedule_repo);

    let result = service
        .create_schedule(order_uuid, payload)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = UpdateScheduleFn,
    prefix = "/api",
    endpoint = "/update_schedule",
)]
pub async fn update_schedule(
    order_uuid: String,
    payload: UpdateScheduleAssignment,
) -> Result<Schedule, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(current_user): Extension<User> = extract().await?;
    ensure_operator(&current_user)?;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();

    let schema_name = tenant.schema_name.clone();
    let order_repo = SeaOrmOrderRepository::new(pool.connection.clone(), schema_name.clone());
    let schedule_repo = SeaOrmScheduleRepository::new(pool.connection.clone(), schema_name);
    let service = ScheduleAppService::new(order_repo, schedule_repo);

    let result = service
        .update_schedule(order_uuid, payload)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = CancelScheduleFn,
    prefix = "/api",
    endpoint = "/cancel_schedule",
)]
pub async fn cancel_schedule(order_uuid: String) -> Result<Schedule, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(current_user): Extension<User> = extract().await?;
    ensure_operator(&current_user)?;

    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();

    let schema_name = tenant.schema_name.clone();
    let order_repo = SeaOrmOrderRepository::new(pool.connection.clone(), schema_name.clone());
    let schedule_repo = SeaOrmScheduleRepository::new(pool.connection.clone(), schema_name);
    let service = ScheduleAppService::new(order_repo, schedule_repo);

    let result = service
        .cancel_schedule(order_uuid)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[server(
    name = UpdateScheduleStatusFn,
    prefix = "/api",
    endpoint = "/update_schedule_status",
)]
pub async fn update_schedule_status(
    order_uuid: String,
    payload: UpdateScheduleStatus,
) -> Result<Schedule, ServerFnError> {
    use self::ssr::*;
    use axum::Extension;
    use leptos_axum::extract;

    let Extension(current_user): Extension<User> = extract().await?;
    let Extension(tenant): Extension<TenantContext> = extract().await?;
    let pool = expect_context::<Database>();

    if is_worker(&current_user) {
        let query = SeaOrmScheduleQuery::new(pool.connection.clone(), tenant.schema_name.clone());
        let schedule_service = ScheduleQueryService::new(query);
        let schedule = schedule_service
            .fetch_schedule(order_uuid.clone())
            .await
            .map_err(|e| ServerFnError::new(e))?;
        let Some(schedule) = schedule else {
            return Err(ServerFnError::new("排班不存在".to_string()));
        };
        if schedule.assigned_user_uuid.as_deref() != Some(current_user.uuid.as_str()) {
            return Err(ServerFnError::new("无权限更新该排班".to_string()));
        }
        if payload.status != "in_service" && payload.status != "done" {
            return Err(ServerFnError::new("仅允许更新服务开始或完成状态".to_string()));
        }
    } else {
        ensure_operator(&current_user)?;
    }

    let order_repo = SeaOrmOrderRepository::new(pool.connection.clone(), tenant.schema_name.clone());
    let schedule_repo = SeaOrmScheduleRepository::new(pool.connection.clone(), tenant.schema_name);
    let service = ScheduleAppService::new(order_repo, schedule_repo);

    let result = service
        .update_schedule_status(order_uuid, payload)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    Ok(result)
}

#[cfg(feature = "ssr")]
fn ensure_operator(user: &User) -> Result<(), ServerFnError> {
    if user.is_admin.unwrap_or(false) {
        return Err(ServerFnError::new("无权限进行该操作".to_string()));
    }
    if user.role != "operator" && user.role != "merchant" {
        return Err(ServerFnError::new("无权限进行该操作".to_string()));
    }
    Ok(())
}

#[cfg(feature = "ssr")]
fn is_worker(user: &User) -> bool {
    !user.is_admin.unwrap_or(false)
        && user.role != "operator"
        && user.role != "merchant"
        && user.role != "admin"
}

#[cfg(all(test, feature = "ssr"))]
mod tests {
    use super::*;

    fn build_user(role: &str, is_admin: bool) -> User {
        let mut user = User::default();
        user.role = role.to_string();
        user.is_admin = Some(is_admin);
        user.uuid = "user-1".to_string();
        user
    }

    #[test]
    fn operator_permission_checks() {
        let operator = build_user("operator", false);
        assert!(ensure_operator(&operator).is_ok());

        let merchant = build_user("merchant", false);
        assert!(ensure_operator(&merchant).is_ok());

        let worker = build_user("user", false);
        assert!(ensure_operator(&worker).is_err());

        let admin = build_user("admin", true);
        assert!(ensure_operator(&admin).is_err());
    }

    #[test]
    fn worker_detection_matches_roles() {
        let operator = build_user("operator", false);
        assert!(!is_worker(&operator));

        let merchant = build_user("merchant", false);
        assert!(!is_worker(&merchant));

        let admin = build_user("admin", true);
        assert!(!is_worker(&admin));

        let worker = build_user("user", false);
        assert!(is_worker(&worker));
    }
}
