use crate::components::ui::toast::{error, success};
use leptos::prelude::*;
use leptos_meta::Title;
use shared::auth::LoginResponse;

#[cfg(feature = "ssr")]
pub mod login_ssr {
    pub use backend::domain::identity::auth::AuthProvider;
    pub use backend::infrastructure::auth::jwt_provider::JwtAuthProvider;
    pub use backend::infrastructure::db::Database;
}

#[server(
    name = UserLoginAction,
    prefix = "/api",
    endpoint = "/login",
)]
pub async fn user_login_action(
    user_name: String,
    password: String,
) -> Result<LoginResponse, ServerFnError> {
    use self::login_ssr::*;

    let (user_name, password) = validate_login_request(user_name, password)?;

    let pool = expect_context::<Database>();
    let auth = JwtAuthProvider::new(pool.connection.clone());

    let token = auth
        .authenticate(&user_name, &password)
        .await
        .map_err(ServerFnError::new)?
        .0;

    set_session_cookie(&token)?;

    let claims = auth.get_claims(&token).map_err(ServerFnError::new)?;

    Ok(LoginResponse {
        role: claims.role,
        redirect_to: "/".to_string(),
    })
}

#[cfg(feature = "ssr")]
pub(crate) fn validate_login_request(
    user_name: String,
    password: String,
) -> Result<(String, String), ServerFnError> {
    let user_name = user_name.trim().to_string();
    if user_name.is_empty() {
        return Err(ServerFnError::ServerError("用户名不能为空".to_string()));
    }

    if password.len() < 6 {
        return Err(ServerFnError::ServerError("密码长度至少6位".to_string()));
    }

    Ok((user_name, password))
}

#[cfg(feature = "ssr")]
pub(crate) fn set_session_cookie(token: &str) -> Result<(), ServerFnError> {
    use cookie::{time::Duration, Cookie, SameSite};
    use http::header::SET_COOKIE;
    use leptos_axum::ResponseOptions;

    let response = expect_context::<ResponseOptions>();
    let session_cookie = Cookie::build(("user_session", token.to_string()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::hours(2))
        .build();

    let header_value =
        session_cookie
            .to_string()
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

pub fn clean_login_error_message(err: &ServerFnError) -> String {
    let error_message = err.to_string();
    if error_message.starts_with("error running server function: ") {
        error_message.replace("error running server function: ", "")
    } else {
        error_message
    }
}

#[derive(Clone, Copy)]
pub struct AuthHighlight {
    icon: &'static str,
    title: &'static str,
    description: &'static str,
    badge: &'static str,
    note: &'static str,
    class_name: &'static str,
    style: Option<&'static str>,
}

#[derive(Clone, Copy)]
pub struct AuthPageCopy {
    portal_label: &'static str,
    hero_title: &'static str,
    hero_description: &'static str,
    primary_highlight: AuthHighlight,
    secondary_highlight: AuthHighlight,
    footer_points: [&'static str; 3],
    panel_tag: &'static str,
    panel_title: &'static str,
    panel_caption: &'static str,
}

pub fn user_auth_copy() -> AuthPageCopy {
    AuthPageCopy {
        portal_label: "工作台入口",
        hero_title: "让客户运营更有节奏",
        hero_description: "把线索、商机与续费链路拉通，让团队在同一节奏里协作与成交。",
        primary_highlight: AuthHighlight {
            icon: "icon-[tabler--users-group] size-5",
            title: "统一客户视图",
            description: "客户资料、跟进记录与关键文件一处汇总，协作更顺畅。",
            badge: "全程留痕",
            note: "减少重复触达",
            class_name: "crm-float rounded-2xl border border-white/15 bg-white/10 p-4 backdrop-blur-sm",
            style: None,
        },
        secondary_highlight: AuthHighlight {
            icon: "icon-[tabler--bell-ringing] size-5",
            title: "关键节点提醒",
            description: "跟进与 SLA 自动提醒，机会不再悄悄流失。",
            badge: "智能节奏",
            note: "更快成交",
            class_name: "rounded-2xl border border-white/15 bg-white/10 p-4 backdrop-blur-sm crm-enter-delay",
            style: Some("animation-delay: 0.2s;"),
        },
        footer_points: ["登录安全", "数据加密", "7x12 支持"],
        panel_tag: "用户登录",
        panel_title: "进入工作台",
        panel_caption: "使用工作账户继续",
    }
}

pub fn admin_auth_copy() -> AuthPageCopy {
    AuthPageCopy {
        portal_label: "平台治理入口",
        hero_title: "把商户管理和平台治理放到同一张图上",
        hero_description: "集中维护商户、系统配置与平台统计，减少平台侧运营与交付切换成本。",
        primary_highlight: AuthHighlight {
            icon: "icon-[tabler--building-store] size-5",
            title: "商户全局视图",
            description: "统一查看商户开通状态、试用周期与平台侧运营情况。",
            badge: "全局编排",
            note: "减少切换成本",
            class_name: "crm-float rounded-2xl border border-white/15 bg-white/10 p-4 backdrop-blur-sm",
            style: None,
        },
        secondary_highlight: AuthHighlight {
            icon: "icon-[tabler--settings-cog] size-5",
            title: "配置与审计",
            description: "把系统配置、通知参数与平台侧操作入口集中到统一后台。",
            badge: "统一治理",
            note: "便于审计追踪",
            class_name: "rounded-2xl border border-white/15 bg-white/10 p-4 backdrop-blur-sm crm-enter-delay",
            style: Some("animation-delay: 0.2s;"),
        },
        footer_points: ["权限隔离", "审计留痕", "平台支持"],
        panel_tag: "管理员登录",
        panel_title: "进入平台管理",
        panel_caption: "使用平台管理员账户继续",
    }
}

#[component]
pub fn AuthFrame(copy: AuthPageCopy, children: Children) -> impl IntoView {
    let content = children();

    view! {
        <div class="min-h-[100dvh] crm-login overflow-x-hidden lg:flex lg:items-center">
            <style>{AUTH_PAGE_STYLES}</style>

            <div class="mx-auto max-w-6xl px-4 py-3 sm:px-6 sm:py-6 lg:w-full lg:px-10 lg:py-4 xl:py-6">
                <div class="grid items-stretch gap-4 sm:gap-6 lg:gap-5 xl:gap-6 lg:grid-cols-[1.1fr_0.9fr]">
                    <section class="order-1 flex items-center justify-center crm-enter-delay lg:order-2">
                        <div class="card crm-card w-full max-w-none sm:max-w-md">
                            <div class="card-body space-y-4 p-4 sm:space-y-6 sm:p-6 lg:space-y-5 lg:p-5 xl:space-y-6 xl:p-6">
                                <div class="rounded-2xl border border-slate-200/70 bg-white/80 p-4 sm:p-5 lg:p-4 xl:p-5">
                                    <div class="flex flex-col items-start gap-3 sm:flex-row sm:items-center sm:gap-4">
                                        <div class="flex h-12 w-12 items-center justify-center rounded-2xl bg-sky-100 text-sky-600">
                                            <span class="icon-[tabler--shield-lock] size-6"></span>
                                        </div>
                                        <div class="space-y-1">
                                            <p class="text-xs uppercase tracking-[0.28em] text-slate-400 sm:tracking-[0.35em]">
                                                {copy.panel_tag}
                                            </p>
                                            <h2 class="text-lg font-semibold text-slate-900 sm:text-2xl lg:text-xl xl:text-2xl">
                                                {copy.panel_title}
                                            </h2>
                                            <p class="text-sm text-slate-500">
                                                {copy.panel_caption}
                                            </p>
                                        </div>
                                    </div>
                                </div>

                                {content}
                            </div>
                        </div>
                    </section>

                    <section class="order-2 crm-hero relative overflow-hidden rounded-[28px] p-5 sm:rounded-3xl sm:p-8 lg:order-1 lg:p-8 xl:p-10 crm-enter">
                        <div class="crm-grid absolute inset-0 opacity-30"></div>
                        <div class="absolute -top-24 -right-16 h-56 w-56 rounded-full bg-white/10 blur-3xl"></div>

                        <div class="relative z-10 flex h-full flex-col justify-between gap-5 sm:gap-10 lg:gap-7 xl:gap-10">
                            <div class="space-y-4 sm:space-y-6 lg:space-y-5 xl:space-y-6">
                                <div class="inline-flex max-w-full flex-wrap items-center gap-2 rounded-full border border-white/15 bg-white/10 px-3 py-1 text-[11px] uppercase tracking-[0.2em] text-white/80 sm:text-xs sm:tracking-[0.35em]">
                                    <span class="icon-[tabler--sparkles] size-4"></span>
                                    <span>{copy.portal_label}</span>
                                </div>
                                <div class="space-y-3">
                                    <h1 class="text-[1.85rem] font-semibold leading-tight text-white sm:text-4xl lg:text-[2.7rem] xl:text-4xl">
                                        {copy.hero_title}
                                    </h1>
                                    <p class="max-w-md text-[13px] leading-relaxed text-white/75 sm:text-sm">
                                        {copy.hero_description}
                                    </p>
                                </div>

                                <div class="grid gap-4 sm:grid-cols-2">
                                    <div class=copy.primary_highlight.class_name>
                                        <div class="flex items-center gap-2 text-white">
                                            <span class=copy.primary_highlight.icon></span>
                                            <span class="text-sm font-semibold">
                                                {copy.primary_highlight.title}
                                            </span>
                                        </div>
                                        <p class="mt-2 text-sm text-white/75 leading-relaxed">
                                            {copy.primary_highlight.description}
                                        </p>
                                        <div class="mt-3 flex flex-wrap items-center gap-2 text-xs text-white/70">
                                            <span class="badge badge-sm badge-outline border-white/30 text-white/80">
                                                {copy.primary_highlight.badge}
                                            </span>
                                            <span>{copy.primary_highlight.note}</span>
                                        </div>
                                    </div>

                                    <div
                                        class=copy.secondary_highlight.class_name
                                        style=copy.secondary_highlight.style.unwrap_or("")
                                    >
                                        <div class="flex items-center gap-2 text-white">
                                            <span class=copy.secondary_highlight.icon></span>
                                            <span class="text-sm font-semibold">
                                                {copy.secondary_highlight.title}
                                            </span>
                                        </div>
                                        <p class="mt-2 text-sm text-white/75 leading-relaxed">
                                            {copy.secondary_highlight.description}
                                        </p>
                                        <div class="mt-3 flex flex-wrap items-center gap-2 text-xs text-white/70">
                                            <span class="badge badge-sm badge-outline border-white/30 text-white/80">
                                                {copy.secondary_highlight.badge}
                                            </span>
                                            <span>{copy.secondary_highlight.note}</span>
                                        </div>
                                    </div>
                                </div>
                            </div>

                            <div class="grid grid-cols-2 gap-x-3 gap-y-2 text-xs text-white/75 min-[520px]:grid-cols-3 sm:gap-3 sm:text-sm">
                                <div class="flex items-center gap-1.5 sm:gap-2">
                                    <span class="icon-[tabler--shield-check] size-5"></span>
                                    <span>{copy.footer_points[0]}</span>
                                </div>
                                <div class="flex items-center gap-1.5 sm:gap-2">
                                    <span class="icon-[tabler--cloud-lock] size-5"></span>
                                    <span>{copy.footer_points[1]}</span>
                                </div>
                                <div class="col-span-2 flex items-center gap-1.5 min-[520px]:col-span-1 sm:gap-2">
                                    <span class="icon-[tabler--headset] size-5"></span>
                                    <span>{copy.footer_points[2]}</span>
                                </div>
                            </div>
                        </div>
                    </section>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Login() -> impl IntoView {
    let do_login = ServerAction::<UserLoginAction>::new();
    let pending = do_login.pending();
    let result = do_login.value();
    let navigate = leptos_router::hooks::use_navigate();

    Effect::new(move |_| {
        result.with(|current_value| {
            if let Some(action_result) = current_value.as_ref() {
                match action_result {
                    Ok(response) => {
                        success("登录成功".to_string());
                        navigate(&response.redirect_to, Default::default());
                    }
                    Err(err) => error(clean_login_error_message(err)),
                }
            }
        });
    });

    let (password_visible, set_password_visible) = signal(false);
    let toggle_password = move |_| {
        set_password_visible.update(|visible| *visible = !*visible);
    };

    view! {
        <Title text="用户登录 - PicoCRM"/>
        <AuthFrame copy=user_auth_copy()>
            <ActionForm action=do_login>
                <div class="space-y-3 text-left sm:space-y-4">
                    <div class="form-control">
                        <label class="label">
                            <span class="label-text font-medium">"登录用户名"</span>
                        </label>
                        <label class="input input-bordered flex items-center gap-2 bg-white">
                            <span class="icon-[tabler--user] size-5 text-slate-400"></span>
                            <input
                                type="text"
                                name="user_name"
                                placeholder="请输入登录用户名"
                                class="grow"
                                required
                            />
                        </label>
                    </div>

                    <div class="form-control">
                        <label class="label">
                            <span class="label-text font-medium">"密码"</span>
                            <a href="#" class="label-text-alt link link-hover text-sky-600 text-xs sm:text-sm">
                                "忘记密码？"
                            </a>
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
                                <span
                                    class=move || {
                                        if *password_visible.read() {
                                            "icon-[tabler--eye] size-5"
                                        } else {
                                            "icon-[tabler--eye-off] size-5"
                                        }
                                    }
                                ></span>
                            </button>
                        </label>
                    </div>

                    <div class="flex flex-col gap-2 text-sm sm:flex-row sm:items-center sm:justify-between">
                        <label class="flex cursor-pointer items-center gap-2">
                            <input type="checkbox" class="checkbox checkbox-sm border-slate-300" />
                            <span class="crm-muted">"保持登录状态"</span>
                        </label>
                        <span class="text-xs text-slate-400">"登录后可直接进入工作台"</span>
                    </div>

                    <button
                        type="submit"
                        class="btn w-full border-none text-white"
                        style="background: linear-gradient(135deg, var(--crm-accent), var(--crm-accent-2));"
                    >
                        <span class="loading loading-spinner" class:hidden=move || !*pending.read()></span>
                        <span class="ml-2">"登录进入工作台"</span>
                    </button>
                </div>
            </ActionForm>

            <div class="divider my-1 text-slate-300">"or"</div>

            <div class="flex flex-col gap-2 text-xs sm:flex-row sm:items-center sm:justify-between sm:text-sm">
                <span class="crm-muted">"还没有账号？"</span>
                <a href="#" class="link link-hover text-sky-700 font-medium">"申请试用"</a>
            </div>

            <div class="flex items-center gap-2 text-xs text-slate-400">
                <span class="icon-[tabler--shield-lock] size-4"></span>
                <span>"登录即同意安全协议与隐私条款"</span>
            </div>
        </AuthFrame>
    }
}

const AUTH_PAGE_STYLES: &str = r#"
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

@media (max-width: 639px) {
    .crm-card {
        background: rgba(255, 255, 255, 0.94);
        box-shadow: 0 18px 40px -30px rgba(15, 23, 42, 0.45);
        backdrop-filter: blur(10px);
    }
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
"#;
