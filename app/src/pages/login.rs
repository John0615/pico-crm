use crate::components::ui::toast::{error, success};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
pub mod login_ssr {
    pub use backend::application::commands::auth::AuthAppService;
    pub use backend::infrastructure::auth::jwt_provider::JwtAuthProvider;
    pub use backend::infrastructure::db::Database;
}

#[server(
    name = LoginAction,
    prefix = "/api",
    endpoint = "/login",
)]
pub async fn login_action(user_name: String, password: String) -> Result<(), ServerFnError> {
    use self::login_ssr::*;
    use cookie::{time::Duration, Cookie, SameSite};
    use http::header::SET_COOKIE;
    use leptos::logging::log;
    use leptos_axum::ResponseOptions;

    log!("Login: {:?} {:?}", user_name, password);
    if user_name.is_empty() {
        return Err(ServerFnError::ServerError("用户名不能为空".to_string()));
    }

    if password.len() < 6 {
        return Err(ServerFnError::ServerError("密码长度至少6位".to_string()));
    }

    // 模拟其他验证逻辑
    if user_name != "admin" || password != "123456" {
        return Err(ServerFnError::ServerError("用户名或密码错误".to_string()));
    }

    let pool = expect_context::<Database>();
    let auth = JwtAuthProvider::new(pool.connection.clone());
    let auth_app_service = AuthAppService::new(auth);

    println!("pool {:?}", pool);

    println!("Fetching user...");

    let token = auth_app_service
        .authenticate(&user_name, &password)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    println!("User Token: {:?}", token);

    let response = expect_context::<ResponseOptions>();

    // 使用cookie库构建cookie
    let session_cookie = Cookie::build(("user_session", token))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::hours(2))
        .build();

    // 设置cookie到响应头
    let cookie_str = session_cookie.to_string();
    let header_value: http::HeaderValue =
        cookie_str
            .parse()
            .map_err(|e: http::header::InvalidHeaderValue| {
                ServerFnError::<http::header::InvalidHeaderValue>::ServerError(format!(
                    "Failed to parse cookie: {}",
                    e
                ))
            })?;

    response.insert_header(SET_COOKIE, header_value);

    Ok(())
}

#[component]
pub fn Login() -> impl IntoView {
    let do_login = ServerAction::<LoginAction>::new();
    let result = do_login.value();
    let navigate = leptos_router::hooks::use_navigate();

    Effect::new(move |_| {
        let current_value = result.get(); // 得到 Option<Result<(), ServerFnError>>

        if let Some(action_result) = current_value {
            // 现在 action_result 是 Result<(), ServerFnError>
            if action_result.is_ok() {
                // 登录成功后跳转到主页
                success("登录成功".to_string());
                navigate("/", Default::default());
            }

            if action_result.is_err() {
                // 登录失败后显示错误信息
                let error_message = action_result.err().unwrap().to_string();
                let clean_error = if error_message.starts_with("error running server function: ") {
                    error_message.replace("error running server function: ", "")
                } else {
                    error_message
                };
                error(clean_error);
            }
        }
    });

    // 密码可见性状态
    let (password_visible, set_password_visible) = signal(false);
    let toggle_password = move |_| {
        set_password_visible.update(|v| *v = !*v);
    };

    view! {
        <div
            class="min-h-screen flex items-center justify-center p-4 relative bg-gradient-to-br from-blue-50 to-purple-100"
        >
            <style>
                {r#"
                /* 自定义圆角 */
                .rounded-soft {
                    border-radius: 12px;
                }
                .rounded-btn {
                    border-radius: 10px;
                }
                .rounded-input {
                    border-radius: 8px;
                }

                /* 卡片浮动动画 */
                @keyframes card-float {
                    0%, 100% { transform: translateY(0) rotate(-0.5deg); }
                    50% { transform: translateY(-10px) rotate(0.5deg); }
                }
                .floating-card {
                    animation: card-float 6s ease-in-out infinite;
                    box-shadow: 0 15px 30px -10px rgba(109, 40, 217, 0.2);
                }
                .floating-card:hover {
                    animation-play-state: paused;
                }

                /* 输入框样式 */
                .input-container {
                    position: relative;
                    margin-top: 0.5rem;  /* 调整标签和输入框间距 */
                }
                .input-label {
                    display: block;
                    text-align: left;  /* 左对齐标签 */
                    margin-bottom: 0.25rem;  /* 标签和输入框间距 */
                }
                .input-icon {
                    position: absolute;
                    left: 12px;
                    top: 50%;
                    transform: translateY(-50%);
                    pointer-events: none;
                    color: #9CA3AF;
                    z-index: 10;
                }
                .input-with-icon {
                    padding-left: 40px !important;
                }

                /* 按钮样式 */
                .btn-comfort {
                    height: 3.25rem;
                    font-size: 1.05rem;
                    display: inline-flex;
                    align-items: center;
                    justify-content: center;
                }

                /* 装饰元素 */
                .bubble {
                    position: absolute;
                    border-radius: 50%;
                    background: rgba(216, 180, 254, 0.3);
                    filter: blur(40px);
                    z-index: -1;
                }
                "#}
            </style>

            <div class="card w-full max-w-md bg-white/90 backdrop-blur-sm border border-white/20 rounded-soft overflow-hidden floating-card">
                <div class="bg-gradient-to-r from-purple-600 to-pink-500 text-white p-6 text-center rounded-t-soft">
                    <h1 class="text-3xl font-bold">"欢迎登录PicoCRM"</h1>
                    <p class="opacity-90 mt-1 text-sm">"开启您的专属体验"</p>
                </div>

                <div class="card-body p-8 space-y-4">
                    <ActionForm action=do_login>
                        <div class="form-control">
                            <label class="input-label font-medium text-gray-700">"用户名"</label>
                            <div class="input-container">
                                <span class="input-icon">
                                    <i class="fas fa-user text-lg"></i>
                                </span>
                                <input
                                    type="text"
                                    name="user_name"
                                    placeholder="请输入用户名"
                                    class="input input-bordered w-full rounded-input bg-white/70 h-12 input-with-icon hover:bg-white/90 transition-colors"
                                    required
                                />
                            </div>
                        </div>

                        <div class="form-control">
                            <label class="input-label font-medium text-gray-700">"密码"</label>
                            <div class="input-container">
                                <span class="input-icon">
                                    <i class="fas fa-lock text-lg"></i>
                                </span>
                                <input
                                    type=move || if password_visible.get() { "text" } else { "password" }
                                    name="password"
                                    placeholder="请输入密码"
                                    class="input input-bordered w-full rounded-input bg-white/70 h-12 input-with-icon pr-10 hover:bg-white/90 transition-colors"
                                    required
                                />
                                <button
                                    type="button"
                                    class="absolute right-3 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-purple-500 transition-colors"
                                    on:click=toggle_password
                                >
                                    <i class=move || if password_visible.get() { "fas fa-eye text-lg" } else { "fas fa-eye-slash text-lg" }></i>
                                </button>
                            </div>
                            <div class="text-right mt-1">
                                <a href="#" class="text-sm text-purple-500 hover:text-purple-600 font-medium">"忘记密码？"</a>
                            </div>
                        </div>

                        <div class="flex items-center justify-between mt-2">
                            <label class="cursor-pointer flex items-center gap-2">
                                <input type="checkbox" class="checkbox checkbox-sm border-gray-300 [--chkbg:theme(colors.purple.500)] rounded-sm" />
                                <span class="text-gray-600 text-sm">"保持登录状态"</span>
                            </label>
                        </div>

                        <button
                            type="submit"
                            class="btn w-full mt-6 bg-gradient-to-r from-purple-500 to-pink-500 border-none text-white hover:from-purple-600 hover:to-pink-600 hover:shadow-lg transition-all rounded-btn btn-comfort"
                        >
                            "立即登录" <i class="fas fa-arrow-right-long ml-2 text-lg"></i>
                        </button>
                    </ActionForm>

                    <div class="text-center mt-6 pt-5 border-t border-gray-100/50">
                        <p class="text-gray-500 text-sm">"新用户？"
                            <a href="#" class="font-medium text-purple-500 hover:text-purple-600 transition-colors">"点击注册"</a>
                        </p>
                    </div>
                </div>
            </div>

        </div>
    }
}
