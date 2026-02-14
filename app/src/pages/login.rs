use crate::components::ui::toast::{error, success};
use leptos::prelude::*;

#[cfg(feature = "ssr")]
pub mod login_ssr {
    pub use backend::application::commands::auth::AuthAppService;
    pub use backend::infrastructure::auth::jwt_provider::JwtAuthProvider;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::repositories::user_repository_impl::SeaOrmUserRepository;
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

    let pool = expect_context::<Database>();
    let auth = JwtAuthProvider::new(pool.connection.clone());
    let user_repository = SeaOrmUserRepository::new(pool.connection.clone());
    let auth_app_service = AuthAppService::new(auth, user_repository);

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
    let pending = do_login.pending();
    let result = do_login.value();
    let navigate = leptos_router::hooks::use_navigate();

    Effect::new(move |_| {
        result.with(|current_value| {
            if let Some(action_result) = current_value.as_ref() {
                if action_result.is_ok() {
                    success("登录成功".to_string());
                    navigate("/", Default::default());
                } else if let Err(err) = action_result {
                    let error_message = err.to_string();
                    let clean_error =
                        if error_message.starts_with("error running server function: ") {
                            error_message.replace("error running server function: ", "")
                        } else {
                            error_message
                        };
                    error(clean_error);
                }
            }
        });
    });

    // 密码可见性状态
    let (password_visible, set_password_visible) = signal(false);
    let toggle_password = move |_| {
        set_password_visible.update(|v| *v = !*v);
    };

    view! {
        <div class="min-h-screen crm-login">
            <style>
                {r#"
                @import url("https://fonts.googleapis.com/css2?family=DM+Sans:wght@400;500;700&family=Space+Grotesk:wght@400;500;600;700&display=swap");

                .crm-login {
                    --crm-ink: #0b1220;
                    --crm-muted: #475569;
                    --crm-accent: #0ea5e9;
                    --crm-accent-2: #14b8a6;
                    --crm-warm: #f59e0b;
                    --crm-surface: #f8fafc;
                    font-family: "Space Grotesk", "DM Sans", ui-sans-serif, system-ui, sans-serif;
                    background:
                        radial-gradient(1200px circle at 10% 10%, rgba(14, 165, 233, 0.12), transparent 40%),
                        radial-gradient(1000px circle at 90% -10%, rgba(20, 184, 166, 0.18), transparent 45%),
                        linear-gradient(180deg, #f8fafc 0%, #eef2ff 100%);
                    color: var(--crm-ink);
                }

                .crm-hero {
                    background: linear-gradient(135deg, #0b1d32 0%, #0f3d4c 45%, #0f766e 100%);
                }

                .crm-grid {
                    background-image: radial-gradient(rgba(255, 255, 255, 0.25) 1px, transparent 1px);
                    background-size: 18px 18px;
                }

                .crm-card {
                    background: rgba(255, 255, 255, 0.88);
                    border: 1px solid rgba(148, 163, 184, 0.25);
                    box-shadow: 0 24px 60px -32px rgba(15, 23, 42, 0.6);
                    backdrop-filter: blur(16px);
                }

                .crm-muted {
                    color: var(--crm-muted);
                }

                @keyframes crm-enter {
                    from {
                        opacity: 0;
                        transform: translateY(18px);
                    }
                    to {
                        opacity: 1;
                        transform: translateY(0);
                    }
                }

                @keyframes crm-float {
                    0%, 100% { transform: translateY(0); }
                    50% { transform: translateY(-8px); }
                }

                .crm-enter {
                    animation: crm-enter 0.8s ease both;
                }

                .crm-enter-delay {
                    animation: crm-enter 0.9s ease both;
                    animation-delay: 0.12s;
                }

                .crm-float {
                    animation: crm-float 8s ease-in-out infinite;
                }
                "#}
            </style>

            <div class="mx-auto max-w-6xl px-4 py-6 sm:px-6 lg:px-10 lg:py-12">
                <div class="grid items-stretch gap-6 lg:grid-cols-[1.1fr_0.9fr]">
                    <section class="crm-hero relative overflow-hidden rounded-3xl p-8 lg:p-10 crm-enter">
                        <div class="crm-grid absolute inset-0 opacity-30"></div>
                        <div class="absolute -top-24 -right-16 h-56 w-56 rounded-full bg-white/10 blur-3xl"></div>

                        <div class="relative z-10 flex h-full flex-col justify-between gap-10">
                            <div class="space-y-6">
                                <div class="inline-flex items-center gap-2 rounded-full border border-white/15 bg-white/10 px-3 py-1 text-xs uppercase tracking-[0.35em] text-white/80">
                                    <span class="icon-[tabler--sparkles] size-4"></span>
                                    <span>"PicoCRM"</span>
                                </div>
                                <div class="space-y-3">
                                    <h1 class="text-4xl font-semibold text-white leading-tight">
                                        "让客户运营更有节奏"
                                    </h1>
                                    <p class="max-w-md text-sm text-white/75 leading-relaxed">
                                        "把线索、商机与续费链路拉通，让团队在同一节奏里协作与成交。"
                                    </p>
                                </div>

                                <div class="grid gap-4 sm:grid-cols-2">
                                    <div class="crm-float rounded-2xl border border-white/15 bg-white/10 p-4 backdrop-blur-sm">
                                        <div class="flex items-center gap-2 text-white">
                                            <span class="icon-[tabler--users-group] size-5"></span>
                                            <span class="text-sm font-semibold">"统一客户视图"</span>
                                        </div>
                                        <p class="mt-2 text-sm text-white/75 leading-relaxed">
                                            "客户资料、跟进记录与关键文件一处汇总，协作更顺畅。"
                                        </p>
                                        <div class="mt-3 flex items-center gap-2 text-xs text-white/70">
                                            <span class="badge badge-sm badge-outline border-white/30 text-white/80">"全程留痕"</span>
                                            <span>"减少重复触达"</span>
                                        </div>
                                    </div>

                                    <div
                                        class="rounded-2xl border border-white/15 bg-white/10 p-4 backdrop-blur-sm crm-enter-delay"
                                        style="animation-delay: 0.2s;"
                                    >
                                        <div class="flex items-center gap-2 text-white">
                                            <span class="icon-[tabler--bell-ringing] size-5"></span>
                                            <span class="text-sm font-semibold">"关键节点提醒"</span>
                                        </div>
                                        <p class="mt-2 text-sm text-white/75 leading-relaxed">
                                            "跟进与SLA自动提醒，机会不再悄悄流失。"
                                        </p>
                                        <div class="mt-3 flex items-center gap-2 text-xs text-white/70">
                                            <span class="badge badge-sm badge-outline border-white/30 text-white/80">"智能节奏"</span>
                                            <span>"更快成交"</span>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div class="grid gap-3 text-sm text-white/75 sm:grid-cols-3">
                                <div class="flex items-center gap-2">
                                    <span class="icon-[tabler--shield-check] size-5"></span>
                                    <span>"登录安全"</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <span class="icon-[tabler--cloud-lock] size-5"></span>
                                    <span>"数据加密"</span>
                                </div>
                                <div class="flex items-center gap-2">
                                    <span class="icon-[tabler--headset] size-5"></span>
                                    <span>"7x12 支持"</span>
                                </div>
                            </div>
                        </div>
                    </section>

                    <section class="flex items-center justify-center crm-enter-delay">
                        <div class="card crm-card w-full max-w-md">
                            <div class="card-body space-y-6">
                                <div class="rounded-2xl border border-slate-200/70 bg-white/80 p-5">
                                    <div class="flex items-center gap-4">
                                        <div class="flex h-12 w-12 items-center justify-center rounded-2xl bg-sky-100 text-sky-600">
                                            <span class="icon-[tabler--shield-lock] size-6"></span>
                                        </div>
                                        <div class="space-y-1">
                                            <p class="text-xs uppercase tracking-[0.35em] text-slate-400">"登录入口"</p>
                                            <h2 class="text-2xl font-semibold text-slate-900">"进入 PicoCRM"</h2>
                                            <p class="text-sm text-slate-500">"使用工作账户继续"</p>
                                        </div>
                                    </div>
                                </div>

                                <ActionForm action=do_login>
                                    <div class="space-y-4 text-left">
                                        <div class="form-control">
                                        <label class="label">
                                            <span class="label-text font-medium">"用户名"</span>
                                        </label>
                                        <label class="input input-bordered flex items-center gap-2 bg-white">
                                            <span class="icon-[tabler--user] size-5 text-slate-400"></span>
                                            <input
                                                type="text"
                                                name="user_name"
                                                placeholder="请输入用户名"
                                                class="grow"
                                                required
                                            />
                                        </label>
                                        </div>

                                        <div class="form-control">
                                        <label class="label">
                                            <span class="label-text font-medium">"密码"</span>
                                            <a href="#" class="label-text-alt link link-hover text-sky-600">"忘记密码？"</a>
                                        </label>
                                        <label class="input input-bordered flex items-center gap-2 bg-white">
                                            <span class="icon-[tabler--lock] size-5 text-slate-400"></span>
                                            <input
                                                type=move || if *password_visible.read() { "text" } else { "password" }
                                                name="password"
                                                placeholder="请输入密码"
                                                class="grow"
                                                required
                                            />
                                            <button
                                                type="button"
                                                class="btn btn-ghost btn-sm h-7 min-h-0 px-2 text-slate-500"
                                                aria-label="切换密码可见"
                                                on:click=toggle_password
                                            >
                                                <span class=move || if *password_visible.read() { "icon-[tabler--eye] size-5" } else { "icon-[tabler--eye-off] size-5" }></span>
                                            </button>
                                        </label>
                                        </div>

                                        <div class="flex items-center justify-between text-sm">
                                        <label class="flex cursor-pointer items-center gap-2">
                                            <input type="checkbox" class="checkbox checkbox-sm border-slate-300" />
                                            <span class="crm-muted">"保持登录状态"</span>
                                        </label>
                                        <span class="text-xs text-slate-400">"登录后可直接进入仪表盘"</span>
                                        </div>

                                        <button
                                            type="submit"
                                            class="btn w-full border-none text-white"
                                            style="background: linear-gradient(135deg, var(--crm-accent), var(--crm-accent-2));"
                                        >
                                            <span class="loading loading-spinner" class:hidden=move || !*pending.read()></span>
                                            <span class="ml-2">"登录进入"</span>
                                        </button>
                                    </div>
                                </ActionForm>

                                <div class="divider text-slate-300">"or"</div>

                                <div class="flex items-center justify-between text-sm">
                                    <span class="crm-muted">"还没有账号？"</span>
                                    <a href="#" class="link link-hover text-sky-700 font-medium">"申请试用"</a>
                                </div>

                                <div class="flex items-center gap-2 text-xs text-slate-400">
                                    <span class="icon-[tabler--shield-lock] size-4"></span>
                                    <span>"登录即同意安全协议与隐私条款"</span>
                                </div>
                            </div>
                        </div>
                    </section>
                </div>
            </div>
        </div>
    }
}
