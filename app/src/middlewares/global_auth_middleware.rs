use leptos::prelude::*;
use leptos_router::{
    components::{Outlet, A},
    hooks,
};
use serde::{Deserialize, Serialize};

#[component]
pub fn GlobalAuthMiddleware(children: ChildrenFn) -> impl IntoView {
    // 检查认证状态
    let auth_resource = Resource::new(|| (), |_| async move { check_global_auth().await });

    // 获取当前路由信息
    let location = hooks::use_location();
    let navigate = hooks::use_navigate();

    // 不需要认证的路径白名单
    let public_paths = vec!["/", "/login", "/register"];

    let current_path = move || location.pathname.get();

    view! {
        <Suspense fallback=move || view! { <GlobalAuthLoading/> }>
            {move || {
                let path = current_path();
                let is_public = public_paths.iter().any(|p| path.starts_with(p));

                match auth_resource.get() {
                    None => view! { <Outlet/> }.into_view(),
                    Some(Ok(auth_result)) => {
                        match auth_result {
                            AuthResult::Authenticated(user) => {
                                // 已认证用户，提供用户上下文
                                provide_context(user);

                                // 如果访问登录页，重定向到首页
                                if path == "/login" || path == "/register" {
                                    view! {
                                        <Show when=move || true fallback=|| ()>
                                            {move || {
                                                navigate("/dashboard", Default::default());
                                            }}
                                        </Show>
                                        <Outlet/>
                                    }.into_view()
                                } else {
                                    children().into_view()
                                }
                            }
                            AuthResult::Unauthenticated => {
                                // 未认证用户
                                if is_public {
                                    // 允许访问公开页面
                                    children().into_view()
                                } else {
                                    // 重定向到登录页
                                    view! {
                                        <Show when=move || !is_public fallback=|| ()>
                                            {move || {
                                                navigate(
                                                    &format!("/login?redirect={}", path),
                                                    Default::default()
                                                );
                                            }}
                                        </Show>
                                        <Outlet/>
                                    }.into_view()
                                }
                            }
                        }
                    }
                    Some(Err(e)) => view! {
                        <GlobalAuthError error=e/>
                    }.into_view(),
                }
            }}
        </Suspense>
    }
}

// 全局认证检查
#[server]
pub async fn check_global_auth() -> Result<AuthResult, ServerFnError> {
    use cookie::Cookie;
    use http::HeaderMap;
    use leptos_axum::extract;

    let headers: HeaderMap = extract().await?;

    // 从 cookie 中获取 session token
    let user_session = headers
        .get_all(http::header::COOKIE)
        .iter()
        .flat_map(|value| value.to_str().unwrap_or("").split(';'))
        .filter_map(|s| s.trim().parse::<Cookie>().ok())
        .find(|cookie| cookie.name() == "user_session")
        .and_then(|cookie| cookie.value().parse::<String>().ok());

    match user_session {
        Some(token) => {
            // 验证 token 并获取用户信息
            if let Ok(user) = validate_token_and_get_user(&token).await {
                Ok(AuthResult::Authenticated(user))
            } else {
                Ok(AuthResult::Unauthenticated)
            }
        }
        None => Ok(AuthResult::Unauthenticated),
    }
}

// 认证结果枚举
#[derive(Clone, Deserialize, Serialize)]
pub enum AuthResult {
    Authenticated(User),
    Unauthenticated,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub roles: Vec<String>,
}

// 加载组件
#[component]
fn GlobalAuthLoading() -> impl IntoView {
    view! {
        <div class="global-auth-loading">
            <div class="loader">"加载中..."</div>
            <style>"
                .global-auth-loading {
                    display: flex;
                    justify-content: center;
                    align-items: center;
                    height: 100vh;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                }
                .loader {
                    color: white;
                    font-size: 1.5rem;
                    animation: pulse 1.5s infinite;
                }
                @keyframes pulse {
                    0%, 100% { opacity: 1; }
                    50% { opacity: 0.5; }
                }
            "</style>
        </div>
    }
}

// 错误组件
#[component]
fn GlobalAuthError(error: ServerFnError) -> impl IntoView {
    view! {
        <div class="global-auth-error">
            <h1>"系统错误"</h1>
            <p>"认证服务暂时不可用"</p>
            <p class="error-detail">{format!("错误详情: {:?}", error)}</p>
            <button
                on:click=move |_| {
                    let window = web_sys::window().unwrap();
                    window.location().reload().unwrap();
                }
            >
                "刷新页面"
            </button>
            <A href="/">"返回首页"</A>
        </div>
    }
}
