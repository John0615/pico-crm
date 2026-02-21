use axum::{
    body::Body,
    extract::State,
    http::{header::COOKIE, HeaderValue, Request, Response, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use backend::domain::auth::provider::{AuthCredential, AuthProvider};
use backend::domain::repositories::admin_user::AdminUserRepository;
use backend::infrastructure::auth::jwt_provider::JwtAuthProvider;
use backend::infrastructure::config::app::AppConfig;
use backend::infrastructure::db::Database;
use backend::infrastructure::tenant::{schema_name_from_merchant, TenantContext};
use backend::infrastructure::repositories::admin_user_repository_impl::SeaOrmAdminUserRepository;
use chrono::Utc;
use cookie::{Cookie, CookieJar};
use sea_orm::{ConnectionTrait, DatabaseBackend, Statement};
use shared::user::User;
use uuid::Uuid;

fn get_cookie_jar_from_req<B>(req: &Request<B>) -> CookieJar {
    let mut jar = CookieJar::new();
    if let Some(cookie_header) = req.headers().get(COOKIE) {
        if let Ok(cookie_str) = cookie_header.to_str() {
            for cookie_str in cookie_str.split(';').map(|s| s.trim().to_string()) {
                if let Ok(c) = Cookie::parse(cookie_str) {
                    jar.add(c);
                }
            }
        }
    }
    jar
}

fn server_auth_check(cookie_jar: &CookieJar) -> Result<String, String> {
    let session = cookie_jar
        .get("user_session")
        .map(|c| c.value().to_string())
        .unwrap_or_default();

    match session.as_str() {
        "" => Err("未登录，请先登录".to_string()),
        token => Ok(token.to_string()),
    }
}

pub async fn global_route_auth_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let white_list = ["/", "/login"];
    let path = req.uri().path().to_string();

    if white_list.contains(&path.as_str()) {
        return Ok(next.run(req).await);
    }

    let cookie_jar = get_cookie_jar_from_req(&req);
    match server_auth_check(&cookie_jar) {
        Ok(_) => Ok(next.run(req).await),
        Err(_) => {
            let mut res = Redirect::temporary("/login").into_response();
            res.headers_mut().insert(
                axum::http::header::LOCATION,
                HeaderValue::from_static("/login"),
            );
            *res.status_mut() = StatusCode::SEE_OTHER;
            Ok(res)
        }
    }
}

pub async fn global_api_auth_middleware(
    State(db): State<Database>,
    mut req: Request<Body>,
    next: Next,
) -> Result<Response<Body>, StatusCode> {
    let white_list = ["/login", "/api/logout", "/api/login", "/api/register_merchant"];
    let path = req.uri().path().to_string();

    if white_list.contains(&path.as_str()) {
        return Ok(next.run(req).await);
    }

    let cookie_jar = get_cookie_jar_from_req(&req);
    match server_auth_check(&cookie_jar) {
        Ok(token) => {
            let auth = JwtAuthProvider::new(db.connection.clone());
            let claims = auth.get_claims(&token).map_err(|err| {
                println!("error: {:?}", err);
                StatusCode::UNAUTHORIZED
            })?;

            let config = AppConfig::from_env().map_err(|err| {
                println!("error: {:?}", err);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            let is_admin = claims.role == "admin";
            let role = claims.role.clone();
            let merchant_id = claims.merchant_id.clone();
            let user_name = claims.user_name.clone();
            let is_admin_path = path.starts_with("/api/admin") || path.starts_with("/admin");
            let is_common_path = matches!(path.as_str(), "/api/logout" | "/api/get_user_info");

            if is_admin && !is_admin_path && !is_common_path {
                return handle_auth_failure(&path).await;
            }
            if !is_admin && is_admin_path {
                return handle_auth_failure(&path).await;
            }

            let schema_name = if is_admin {
                "public".to_string()
            } else {
                schema_name_from_merchant(&config.tenant_schema_prefix, &claims.merchant_id)
                    .map_err(|err| {
                        println!("error: {:?}", err);
                        StatusCode::UNAUTHORIZED
                    })?
            };

            req.extensions_mut().insert(TenantContext {
                merchant_id: merchant_id.clone(),
                role: role.clone(),
                schema_name,
            });

            let user: Option<User> = if is_admin {
                let admin_repo = SeaOrmAdminUserRepository::new(db.connection.clone());
                let admin_user = admin_repo
                    .find_by_username(&user_name)
                    .await
                    .map_err(|err| {
                        println!("error: {:?}", err);
                        StatusCode::UNAUTHORIZED
                    })?;

                match admin_user {
                    Some(user) if user.is_active() => Some(user.into()),
                    _ => {
                        return handle_auth_failure(&path).await;
                    }
                }
            } else {
                let merchant_status = fetch_merchant_status(&db, &merchant_id)
                    .await
                    .map_err(|err| {
                        println!("error: {:?}", err);
                        err
                    })?;
                if !merchant_status.is_active {
                    return handle_auth_failure(&path).await;
                }

                let user = auth
                    .get_current_user(&AuthCredential(token.clone()))
                    .await
                    .map_err(|err| {
                        println!("error: {:?}", err);
                        StatusCode::UNAUTHORIZED
                    })?;

                match user {
                    Some(user) if user.is_active() => Some(user.into()),
                    _ => {
                        return handle_auth_failure(&path).await;
                    }
                }
            };

            if let Some(user) = user {
                req.extensions_mut().insert(user);
            } else {
                return handle_auth_failure(&path).await;
            }

            Ok(next.run(req).await)
        }
        Err(_) => {
            // 未登录，直接在中间件层面拦截
            handle_auth_failure(&path).await
        }
    }
}

async fn handle_auth_failure(path: &str) -> Result<Response<Body>, StatusCode> {
    if path.starts_with("/api") {
        // 对于 API 请求，返回符合 Leptos ServerFnError 格式的响应
        // ServerFnError 使用特殊的字符串格式: "variant|message"
        // 使用 MiddlewareError 变体，这是 ServerFnError 专门用于中间件错误的类型
        let error_message = "MiddlewareError|未登录，请先登录后再操作";

        let mut res = Response::new(Body::from(error_message));
        *res.status_mut() = StatusCode::UNAUTHORIZED;
        res.headers_mut().insert(
            axum::http::header::CONTENT_TYPE,
            HeaderValue::from_static("text/plain"),
        );
        Ok(res)
    } else {
        let mut res = Response::new(Body::empty());
        *res.status_mut() = StatusCode::SEE_OTHER;
        res.headers_mut().insert(
            axum::http::header::LOCATION,
            HeaderValue::from_static("/login"),
        );
        Ok(res)
    }
}

struct MerchantStatus {
    is_active: bool,
}

async fn fetch_merchant_status(
    db: &Database,
    merchant_id: &str,
) -> Result<MerchantStatus, StatusCode> {
    let merchant_uuid = Uuid::parse_str(merchant_id).map_err(|err| {
        println!("error: invalid merchant uuid: {}", err);
        StatusCode::UNAUTHORIZED
    })?;
    let stmt = Statement::from_sql_and_values(
        DatabaseBackend::Postgres,
        "SELECT status, expired_at FROM public.merchant WHERE uuid = $1",
        vec![merchant_uuid.into()],
    );
    let row = db
        .connection
        .query_one(stmt)
        .await
        .map_err(|err| {
            println!("error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let Some(row) = row else {
        return Ok(MerchantStatus { is_active: false });
    };

    let status: String = row
        .try_get("", "status")
        .map_err(|err| {
            println!("error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let expired_at: Option<chrono::DateTime<Utc>> = row
        .try_get("", "expired_at")
        .map_err(|err| {
            println!("error: {:?}", err);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    if status != "active" {
        return Ok(MerchantStatus { is_active: false });
    }

    if let Some(expired_at) = expired_at {
        if expired_at <= Utc::now() {
            return Ok(MerchantStatus { is_active: false });
        }
    }

    Ok(MerchantStatus { is_active: true })
}
