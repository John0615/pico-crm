use crate::components::features::get_user_info;
use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use leptos_router::hooks::use_location;
use shared::user::User;

#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();
    let user_data = Resource::new(
        move || (),
        |_| async move {
            let result = call_api(get_user_info()).await.unwrap_or_else(|e| {
                logging::error!("Error loading user info: {e}");
                User::default()
            });
            result
        },
    );
    let user_info = move || {
        user_data.with(|value| {
            value
                .as_ref()
                .map(|user| {
                    let is_admin = user.is_admin.unwrap_or(false) || user.role == "admin";
                    let role = user.role.clone();
                    (is_admin, role)
                })
        })
    };
    let show_admin_menu = move || matches!(user_info(), Some((true, _)));
    let show_merchant_menu = move || {
        matches!(user_info(), Some((false, role)) if role == "operator" || role == "merchant")
    };
    let show_staff_menu = move || {
        matches!(user_info(), Some((false, role)) if role != "operator" && role != "merchant" && role != "admin")
    };

    // 判断当前路径是否匹配菜单项
    let is_active = move |path: &str| location.pathname.with(|current| current == path);
    let is_admin_section = move || {
        location
            .pathname
            .with(|current| current.starts_with("/admin"))
    };
    let is_merchant_section = move || {
        location
            .pathname
            .with(|current| {
                current.starts_with("/contacts")
                    || current.starts_with("/service-requests")
                    || current.starts_with("/orders")
                    || current.starts_with("/schedules")
                    || current.starts_with("/users")
            })
    };
    let admin_dropdown_class = move || {
        let base = "dropdown relative [--adaptive:none] [--strategy:static] [--auto-close:false]";
        if is_admin_section() {
            format!("{base} open")
        } else {
            base.to_string()
        }
    };
    let admin_menu_class = move || {
        let base = "dropdown-menu mt-0 shadow-none dropdown-open:opacity-100 min-w-full w-[calc(100%-0.25rem)] max-w-[calc(100%-0.25rem)] me-1 box-border ms-0 ps-0 before:hidden before:content-none";
        if is_admin_section() {
            format!("{base} block")
        } else {
            format!("{base} hidden")
        }
    };
    let merchant_dropdown_class = move || {
        let base = "dropdown relative [--adaptive:none] [--strategy:static] [--auto-close:false]";
        if is_merchant_section() {
            format!("{base} open")
        } else {
            base.to_string()
        }
    };
    let merchant_menu_class = move || {
        let base = "dropdown-menu mt-0 shadow-none dropdown-open:opacity-100 min-w-full w-[calc(100%-0.25rem)] max-w-[calc(100%-0.25rem)] me-1 box-border ms-0 ps-0 before:hidden before:content-none";
        if is_merchant_section() {
            format!("{base} block")
        } else {
            format!("{base} hidden")
        }
    };

    view! {
        <aside
            id="collapsible-mini-sidebar"
            class="overlay [--auto-close:sm] overlay-minified:w-16 sm:shadow-none overlay-open:translate-x-0 drawer drawer-start hidden w-60 sm:fixed sm:inset-y-0 sm:left-0 sm:z-30 sm:flex sm:translate-x-0 border-e border-base-content/20 bg-base-100 h-screen flex flex-col overflow-hidden"
            role="dialog"
            tabindex="-1"
        >
            <div class="drawer-header overlay-minified:px-3.75 py-3 w-full flex items-center justify-between gap-3 shrink-0">
                <div class="flex items-center">
                    <span class="text-lg font-bold tracking-tight text-primary overlay-minified:hidden">"PicoCRM"</span>
                    <span class="text-lg font-bold tracking-tight text-primary hidden overlay-minified:block">"P"</span>
                </div>
            </div>

            <div class="drawer-body px-2 pt-4 flex-1 overflow-y-auto w-full">
                <Suspense fallback=move || view! {
                    <div class="p-3 text-sm text-base-content/60">"加载中..."</div>
                }>
                <ul class="menu p-0 w-full">
                    <Show when=show_merchant_menu>
                        <li>
                            <a
                                href="/"
                                class=move || if is_active("/") { "menu-active w-full" } else { "w-full" }
                            >
                                <span class="icon-[tabler--home] size-5"></span>
                                <span class="overlay-minified:hidden">"智能看板"</span>
                            </a>
                        </li>

                        <li class=move || format!("overlay-minified:hidden {}", merchant_dropdown_class())>
                            <button
                                id="merchant-dropdown"
                                type="button"
                                class=move || {
                                    if is_merchant_section() {
                                        "dropdown-toggle menu-active w-full"
                                    } else {
                                        "dropdown-toggle w-full"
                                    }
                                }
                                aria-haspopup="menu"
                                aria-expanded=move || if is_merchant_section() { "true" } else { "false" }
                                aria-label="商户业务中心"
                            >
                                <span class="icon-[tabler--briefcase] size-5"></span>
                                <span class="overlay-minified:hidden">"业务中心"</span>
                                <span class="icon-[tabler--chevron-down] dropdown-open:rotate-180 size-4 overlay-minified:hidden"></span>
                            </button>

                            <ul
                                class=merchant_menu_class
                                role="menu"
                                aria-orientation="vertical"
                                aria-labelledby="merchant-dropdown"
                            >
                                <li>
                                    <a
                                        href="/contacts"
                                        class=move || if is_active("/contacts") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--user] size-5"></span>
                                        "客户列表"
                                        <span class="badge badge-primary badge-sm ms-2">"12"</span>
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/service-requests"
                                        class=move || if is_active("/service-requests") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--calendar-event] size-5"></span>
                                        "预约/需求"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/orders"
                                        class=move || if is_active("/orders") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--file-invoice] size-5"></span>
                                        "订单管理"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/schedules"
                                        class=move || if is_active("/schedules") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--calendar-time] size-5"></span>
                                        "排班管理"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/users"
                                        class=move || if is_active("/users") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--users] size-5"></span>
                                        "用户管理"
                                    </a>
                                </li>
                            </ul>
                        </li>
                        <li class="hidden overlay-minified:block dropdown relative [--adaptive:adaptive] [--strategy:fixed] [--offset:12] [--placement:right-start]">
                            <button
                                type="button"
                                class="dropdown-toggle w-full"
                                aria-haspopup="menu"
                                aria-expanded="false"
                                aria-label="业务中心"
                            >
                                <span class="icon-[tabler--briefcase] size-5"></span>
                                <span class="sr-only">"业务中心"</span>
                            </button>

                            <ul
                                class="dropdown-menu absolute left-full top-0 ms-3 shadow-md shadow-base-300/20 hidden min-w-60 w-60 dropdown-open:opacity-100 ms-0 ps-0 before:hidden before:content-none"
                                role="menu"
                                aria-orientation="vertical"
                            >
                                <li>
                                    <a
                                        href="/contacts"
                                        class=move || if is_active("/contacts") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--user] size-5"></span>
                                        "客户列表"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/service-requests"
                                        class=move || if is_active("/service-requests") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--calendar-event] size-5"></span>
                                        "预约/需求"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/orders"
                                        class=move || if is_active("/orders") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--file-invoice] size-5"></span>
                                        "订单管理"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/schedules"
                                        class=move || if is_active("/schedules") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--calendar-time] size-5"></span>
                                        "排班管理"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/users"
                                        class=move || if is_active("/users") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--users] size-5"></span>
                                        "用户管理"
                                    </a>
                                </li>
                            </ul>
                        </li>
                    </Show>

                    <Show when=show_staff_menu>
                        <li>
                            <a
                                href="/"
                                class=move || if is_active("/") { "menu-active w-full" } else { "w-full" }
                            >
                                <span class="icon-[tabler--home] size-5"></span>
                                <span class="overlay-minified:hidden">"智能看板"</span>
                            </a>
                        </li>

                        <li class=move || format!("overlay-minified:hidden {}", merchant_dropdown_class())>
                            <button
                                id="staff-dropdown"
                                type="button"
                                class=move || {
                                    if is_merchant_section() {
                                        "dropdown-toggle menu-active w-full"
                                    } else {
                                        "dropdown-toggle w-full"
                                    }
                                }
                                aria-haspopup="menu"
                                aria-expanded=move || if is_merchant_section() { "true" } else { "false" }
                                aria-label="工作中心"
                            >
                                <span class="icon-[tabler--briefcase] size-5"></span>
                                <span class="overlay-minified:hidden">"工作中心"</span>
                                <span class="icon-[tabler--chevron-down] dropdown-open:rotate-180 size-4 overlay-minified:hidden"></span>
                            </button>

                            <ul
                                class=merchant_menu_class
                                role="menu"
                                aria-orientation="vertical"
                                aria-labelledby="staff-dropdown"
                            >
                                <li>
                                    <a
                                        href="/contacts"
                                        class=move || if is_active("/contacts") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--user] size-5"></span>
                                        "客户列表"
                                        <span class="badge badge-primary badge-sm ms-2">"12"</span>
                                    </a>
                                </li>
                            </ul>
                        </li>
                        <li class="hidden overlay-minified:block dropdown relative [--adaptive:adaptive] [--strategy:fixed] [--offset:12] [--placement:right-start]">
                            <button
                                type="button"
                                class="dropdown-toggle w-full"
                                aria-haspopup="menu"
                                aria-expanded="false"
                                aria-label="工作中心"
                            >
                                <span class="icon-[tabler--briefcase] size-5"></span>
                                <span class="sr-only">"工作中心"</span>
                            </button>

                            <ul
                                class="dropdown-menu absolute left-full top-0 ms-3 shadow-md shadow-base-300/20 hidden min-w-60 w-60 dropdown-open:opacity-100 ms-0 ps-0 before:hidden before:content-none"
                                role="menu"
                                aria-orientation="vertical"
                            >
                                <li>
                                    <a
                                        href="/contacts"
                                        class=move || if is_active("/contacts") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--user] size-5"></span>
                                        "客户列表"
                                    </a>
                                </li>
                            </ul>
                        </li>
                    </Show>

                    <Show when=show_admin_menu>
                        <li class=move || format!("overlay-minified:hidden {}", admin_dropdown_class())>
                            <button
                                id="admin-dropdown"
                                type="button"
                                class=move || {
                                    if is_admin_section() {
                                        "dropdown-toggle menu-active w-full"
                                    } else {
                                        "dropdown-toggle w-full"
                                    }
                                }
                                aria-haspopup="menu"
                                aria-expanded=move || if is_admin_section() { "true" } else { "false" }
                                aria-label="平台管理"
                            >
                                <span class="icon-[tabler--shield-check] size-5"></span>
                                <span class="overlay-minified:hidden">"平台管理"</span>
                                <span class="icon-[tabler--chevron-down] dropdown-open:rotate-180 size-4 overlay-minified:hidden"></span>
                            </button>

                            <ul
                                class=admin_menu_class
                                role="menu"
                                aria-orientation="vertical"
                                aria-labelledby="admin-dropdown"
                            >
                                <li>
                                    <a
                                        href="/admin/merchants"
                                        class=move || {
                                            if is_active("/admin/merchants") || is_active("/admin") {
                                                "menu-active w-full"
                                            } else {
                                                "w-full"
                                            }
                                        }
                                    >
                                        <span class="icon-[tabler--building-store] size-5"></span>
                                        "商户管理"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/admin/analytics"
                                        class=move || if is_active("/admin/analytics") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--chart-bar] size-5"></span>
                                        "平台统计"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/admin/settings"
                                        class=move || if is_active("/admin/settings") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--settings] size-5"></span>
                                        "系统设置"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/admin/maintenance"
                                        class=move || {
                                            if is_active("/admin/maintenance") {
                                                "menu-active w-full"
                                            } else {
                                                "w-full"
                                            }
                                        }
                                    >
                                        <span class="icon-[tabler--tool] size-5"></span>
                                        "租户维护"
                                    </a>
                                </li>
                            </ul>
                        </li>
                        <li class="hidden overlay-minified:block dropdown relative [--adaptive:adaptive] [--strategy:fixed] [--offset:12] [--placement:right-start]">
                            <button
                                type="button"
                                class="dropdown-toggle w-full"
                                aria-haspopup="menu"
                                aria-expanded="false"
                                aria-label="平台管理"
                            >
                                <span class="icon-[tabler--shield-check] size-5"></span>
                                <span class="sr-only">"平台管理"</span>
                            </button>

                            <ul
                                class="dropdown-menu absolute left-full top-0 ms-3 shadow-md shadow-base-300/20 hidden min-w-60 w-60 dropdown-open:opacity-100 ms-0 ps-0 before:hidden before:content-none"
                                role="menu"
                                aria-orientation="vertical"
                            >
                                <li>
                                    <a
                                        href="/admin/merchants"
                                        class=move || {
                                            if is_active("/admin/merchants") || is_active("/admin") {
                                                "menu-active w-full"
                                            } else {
                                                "w-full"
                                            }
                                        }
                                    >
                                        <span class="icon-[tabler--building-store] size-5"></span>
                                        "商户管理"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/admin/analytics"
                                        class=move || if is_active("/admin/analytics") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--chart-bar] size-5"></span>
                                        "平台统计"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/admin/settings"
                                        class=move || if is_active("/admin/settings") { "menu-active w-full" } else { "w-full" }
                                    >
                                        <span class="icon-[tabler--settings] size-5"></span>
                                        "系统设置"
                                    </a>
                                </li>
                                <li>
                                    <a
                                        href="/admin/maintenance"
                                        class=move || {
                                            if is_active("/admin/maintenance") {
                                                "menu-active w-full"
                                            } else {
                                                "w-full"
                                            }
                                        }
                                    >
                                        <span class="icon-[tabler--tool] size-5"></span>
                                        "租户维护"
                                    </a>
                                </li>
                            </ul>
                        </li>
                    </Show>

                </ul>
                </Suspense>
            </div>
        </aside>
    }
}
