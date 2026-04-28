use crate::components::ui::toast::{error, success};
use crate::pages::login::{admin_auth_copy, clean_login_error_message, AuthFrame};
use leptos::prelude::*;
use leptos_meta::Title;
use shared::auth::LoginResponse;

#[cfg(feature = "ssr")]
pub mod admin_login_ssr {
    pub use backend::application::commands::platform::admin_auth::AdminAuthAppService;
    pub use backend::infrastructure::auth::jwt_provider::JwtAuthProvider;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::repositories::platform::admin_user_repository_impl::SeaOrmAdminUserRepository;
}

#[server(
    name = AdminLoginAction,
    prefix = "/api/admin",
    endpoint = "/login",
)]
pub async fn admin_login_action(
    user_name: String,
    password: String,
) -> Result<LoginResponse, ServerFnError> {
    use self::admin_login_ssr::*;
    use crate::pages::login::{set_session_cookie, validate_login_request};

    let (user_name, password) = validate_login_request(user_name, password)?;

    let pool = expect_context::<Database>();
    let auth = JwtAuthProvider::new(pool.connection.clone());
    let admin_repo = SeaOrmAdminUserRepository::new(pool.connection.clone());
    let admin_service = AdminAuthAppService::new(admin_repo, auth);

    let token = admin_service
        .authenticate(&user_name, &password)
        .await
        .map_err(ServerFnError::new)?;

    set_session_cookie(&token)?;

    Ok(LoginResponse {
        role: "admin".to_string(),
        redirect_to: "/admin/merchants".to_string(),
    })
}

#[component]
pub fn AdminLogin() -> impl IntoView {
    let do_login = ServerAction::<AdminLoginAction>::new();
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
        <Title text="管理员登录 - PicoCRM"/>
        <AuthFrame copy=admin_auth_copy()>
            <ActionForm action=do_login>
                <div class="space-y-3 text-left sm:space-y-4">
                    <div class="form-control">
                        <label class="label">
                            <span class="label-text font-medium">"管理员用户名"</span>
                        </label>
                        <label class="input input-bordered flex items-center gap-2 bg-white">
                            <span class="icon-[tabler--user-shield] size-5 text-slate-400"></span>
                            <input
                                type="text"
                                name="user_name"
                                placeholder="请输入管理员用户名"
                                class="grow"
                                required
                            />
                        </label>
                    </div>

                    <div class="form-control">
                        <label class="label">
                            <span class="label-text font-medium">"密码"</span>
                        </label>
                        <label class="input input-bordered flex items-center gap-2 bg-white">
                            <span class="icon-[tabler--lock] size-5 text-slate-400"></span>
                            <input
                                type=move || if *password_visible.read() { "text" } else { "password" }
                                name="password"
                                placeholder="请输入管理员密码"
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
                        <span class="crm-muted">"仅限平台管理员使用"</span>
                        <span class="text-xs text-slate-400">"登录后进入平台管理后台"</span>
                    </div>

                    <button
                        type="submit"
                        class="btn w-full border-none text-white"
                        style="background: linear-gradient(135deg, var(--crm-accent), var(--crm-accent-2));"
                    >
                        <span class="loading loading-spinner" class:hidden=move || !*pending.read()></span>
                        <span class="ml-2">"登录进入平台管理"</span>
                    </button>
                </div>
            </ActionForm>

            <div class="divider my-1 text-slate-300">"or"</div>

            <div class="flex flex-col gap-2 text-xs sm:flex-row sm:items-center sm:justify-between sm:text-sm">
                <span class="crm-muted">"需要平台支持？"</span>
                <span class="text-sky-700 font-medium">"联系系统负责人"</span>
            </div>

            <div class="flex items-center gap-2 text-xs text-slate-400">
                <span class="icon-[tabler--shield-lock] size-4"></span>
                <span>"平台管理员登录受统一权限和审计策略约束"</span>
            </div>
        </AuthFrame>
    }
}
