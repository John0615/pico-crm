use axum::{
    body::Body,
    extract::State,
    http::{header::COOKIE, HeaderValue, Request, Response, StatusCode},
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use backend::application::commands::auth::AuthAppService;
use backend::infrastructure::auth::jwt_provider::JwtAuthProvider;
use backend::infrastructure::db::Database;
use cookie::{Cookie, CookieJar};

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
    let white_list = ["/", "/login", "/register"];
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
    let white_list = ["/contacts", "/login", "/api/logout", "/api/login"];
    let path = req.uri().path().to_string();

    if white_list.contains(&path.as_str()) {
        return Ok(next.run(req).await);
    }

    let cookie_jar = get_cookie_jar_from_req(&req);
    match server_auth_check(&cookie_jar) {
        Ok(token) => {
            let auth = JwtAuthProvider::new(db.connection.clone());
            let auth_app_service = AuthAppService::new(auth);
            let user_result = auth_app_service
                .get_user_by_token(token)
                .await
                .map_err(|err| {
                    println!("error: {:?}", err);
                    StatusCode::INTERNAL_SERVER_ERROR
                })?;
            println!("user: {:?}", user_result);
            if let Some(user) = user_result {
                req.extensions_mut().insert(user.clone());
                Ok(next.run(req).await)
            } else {
                // token 无效，返回认证失败
                handle_auth_failure(&path).await
            }
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
