use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use shared::user::User;

#[cfg(target_arch = "wasm32")]
fn try_init_flyonui_components() {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;

    let Some(window) = web_sys::window() else {
        return;
    };
    let Some(document) = window.document() else {
        return;
    };
    if document
        .get_element_by_id("collapsible-mini-sidebar")
        .is_none()
    {
        return;
    }
    let overlay_btn_exists = document
        .query_selector("[data-overlay-minifier=\"#collapsible-mini-sidebar\"]")
        .ok()
        .flatten()
        .is_some();
    let dropdown_exists = document
        .query_selector("#admin-dropdown")
        .ok()
        .flatten()
        .is_some();
    if !overlay_btn_exists && !dropdown_exists {
        return;
    }

    let static_methods = js_sys::Reflect::get(&window, &JsValue::from_str("HSStaticMethods"));
    if let Ok(static_methods) = static_methods {
        if !static_methods.is_undefined() && !static_methods.is_null() {
            let auto_init = js_sys::Reflect::get(&static_methods, &JsValue::from_str("autoInit"));
            if let Ok(auto_init) = auto_init {
                if let Ok(auto_init) = auto_init.dyn_into::<js_sys::Function>() {
                    let collections = js_sys::Array::of2(
                        &JsValue::from_str("overlay"),
                        &JsValue::from_str("dropdown"),
                    );
                    let _ = auto_init.call1(&static_methods, &collections);
                    return;
                }
            }
        }
    }

    let overlay = js_sys::Reflect::get(&window, &JsValue::from_str("HSOverlay"));
    if let Ok(overlay) = overlay {
        if !overlay.is_undefined() && !overlay.is_null() {
            if let Ok(auto_init) = js_sys::Reflect::get(&overlay, &JsValue::from_str("autoInit")) {
                if let Ok(auto_init) = auto_init.dyn_into::<js_sys::Function>() {
                    let _ = auto_init.call0(&overlay);
                }
            }
        }
    }

    let dropdown = js_sys::Reflect::get(&window, &JsValue::from_str("HSDropdown"));
    if let Ok(dropdown) = dropdown {
        if !dropdown.is_undefined() && !dropdown.is_null() {
            if let Ok(auto_init) = js_sys::Reflect::get(&dropdown, &JsValue::from_str("autoInit")) {
                if let Ok(auto_init) = auto_init.dyn_into::<js_sys::Function>() {
                    let _ = auto_init.call0(&dropdown);
                }
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn init_flyonui_overlay_with_retry() {
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    try_init_flyonui_components();

    let Some(window) = web_sys::window() else {
        return;
    };

    let cb_1 = Closure::once_into_js(|| {
        try_init_flyonui_components();
    });
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(cb_1.unchecked_ref(), 100);

    let cb_2 = Closure::once_into_js(|| {
        try_init_flyonui_components();
    });
    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(cb_2.unchecked_ref(), 400);
}

#[cfg(not(target_arch = "wasm32"))]
fn init_flyonui_overlay_with_retry() {}

#[server(
    name = Logout,
    prefix = "/api",
    endpoint = "/logout",
)]
pub async fn logout() -> Result<(), ServerFnError> {
    use cookie::{time::Duration, Cookie, SameSite};
    use http::header::SET_COOKIE;
    use leptos_axum::ResponseOptions;

    let response = expect_context::<ResponseOptions>();

    let clear_session_cookie = Cookie::build(("user_session", ""))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax)
        .max_age(Duration::ZERO)
        .expires(cookie::time::OffsetDateTime::UNIX_EPOCH)
        .build();

    // 设置cookie到响应头
    let cookie_str = clear_session_cookie.to_string();
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

#[server(
    name = GetUserInfo,
    prefix = "/api",
    endpoint = "/get_user_info",
)]
pub async fn get_user_info() -> Result<User, ServerFnError> {
    use axum::extract::Extension;
    use leptos::logging::log;
    use leptos_axum::extract;

    let Extension(user): Extension<User> = extract().await?;
    log!("usereeeeeeeee: {:?}", user);
    Ok(user)
}

#[component]
pub fn Navbar() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    Effect::new(move |_| {
        init_flyonui_overlay_with_retry();
    });

    let do_logout = ServerAction::<Logout>::new();
    let result = do_logout.value();

    Effect::new(move || {
        result.with(|current_value| {
            if let Some(action_result) = current_value.as_ref() {
                if action_result.is_ok() {
                    navigate("/login", Default::default());
                }
            }
        });
    });

    let data = Resource::new(
        move || (),
        |_| async move {
            let result = call_api(get_user_info()).await.unwrap_or_else(|e| {
                logging::error!("Error loading user: {e}");
                User::default()
            });
            result
        },
    );
    view! {
        <div class="navbar bg-base-100 sticky top-0 z-50 border-b border-base-200 shadow-sm h-16 min-h-16 w-full px-4">
            <div class="navbar-start gap-2 items-center">
                <div class="sm:hidden">
                    <button
                        type="button"
                        class="btn btn-square btn-ghost"
                        aria-haspopup="dialog"
                        aria-expanded="false"
                        aria-controls="collapsible-mini-sidebar"
                        data-overlay="#collapsible-mini-sidebar"
                    >
                        <span class="icon-[tabler--menu-2] size-5"></span>
                    </button>
                </div>

                <button
                    type="button"
                    class="hidden sm:inline-flex items-center justify-center rounded-md p-2 text-base-content hover:bg-base-200"
                    aria-haspopup="dialog"
                    aria-expanded="false"
                    aria-controls="collapsible-mini-sidebar"
                    aria-label="Toggle sidebar width"
                    data-overlay-minifier="#collapsible-mini-sidebar"
                >
                    <span class="icon-[tabler--menu-2] size-5"></span>
                </button>

                <span class="sr-only">"PicoCRM"</span>
            </div>

            <div class="navbar-end gap-2 items-center">
                <div class="dropdown dropdown-end [--trigger:click] mr-4">
                    <button
                        type="button"
                        class="btn btn-text btn-circle text-base-content hover:bg-base-200 dropdown-toggle"
                        aria-haspopup="menu"
                        aria-expanded="false"
                        aria-label="Notifications"
                    >
                        <div class="indicator">
                            <span class="indicator-item badge badge-primary badge-sm">3</span>
                            <span class="icon-[tabler--bell] size-5"></span>
                        </div>
                    </button>
                    <div class="dropdown-menu mt-3 w-72 bg-base-100 shadow hidden dropdown-open:opacity-100">
                        <div class="p-4">
                            <span class="font-bold text-lg">3 条新通知</span>
                            <div class="mt-2 flex flex-col gap-2">
                                <a class="dropdown-item hover:bg-base-200">用户 A 提交了新订单</a>
                                <a class="dropdown-item hover:bg-base-200">系统将于今晚进行维护</a>
                                <a class="dropdown-item hover:bg-base-200">您收到了新消息</a>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="dropdown dropdown-end [--trigger:click]">
                    <button
                        type="button"
                        class="btn btn-text btn-circle text-base-content hover:bg-base-200 dropdown-toggle"
                        aria-haspopup="menu"
                        aria-expanded="false"
                        aria-label="User menu"
                    >
                        <div class="avatar">
                            <div class="w-8 rounded-full">
                                <Suspense>
                                    <img src=move || data.with(|value| value.clone().unwrap_or_default().avatar_url) />
                                </Suspense>
                            </div>
                        </div>
                    </button>

                    <div class="dropdown-menu mt-3 w-52 bg-base-100 shadow hidden dropdown-open:opacity-100">
                        <a class="dropdown-item hover:bg-base-200">
                            <i class="fas fa-user-circle"></i>
                            个人中心
                        </a>
                        <a class="dropdown-item hover:bg-base-200">
                            <i class="fas fa-cog"></i>
                            设置
                        </a>
                        <button
                            type="button"
                            class="dropdown-item text-error hover:bg-error/10"
                            on:click=move |_| {
                                do_logout.dispatch(Logout{});
                            }
                        >
                            <i class="fas fa-sign-out-alt"></i>
                            退出
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
