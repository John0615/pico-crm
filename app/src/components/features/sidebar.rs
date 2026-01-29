// use leptos::logging::log;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();

    // 判断当前路径是否匹配菜单项
    let is_active = move |path: &str| location.pathname.with(|current| current == path);
    let is_admin_section = move || {
        location
            .pathname
            .with(|current| current.starts_with("/admin"))
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
                <ul class="menu p-0 w-full">
                    <li>
                        <a
                            href="/"
                            class=move || if is_active("/") { "menu-active w-full" } else { "w-full" }
                        >
                            <span class="icon-[tabler--home] size-5"></span>
                            <span class="overlay-minified:hidden">"智能看板"</span>
                        </a>
                    </li>

                    <li>
                        <a
                            href="/contacts"
                            class=move || if is_active("/contacts") { "menu-active w-full" } else { "w-full" }
                        >
                            <span class="icon-[tabler--user] size-5"></span>
                            <span class="overlay-minified:hidden">"交互中心"</span>
                            <span class="badge badge-primary badge-sm overlay-minified:hidden">"12"</span>
                        </a>
                    </li>

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
                            aria-label="Admin"
                        >
                            <span class="icon-[tabler--shield-check] size-5"></span>
                            <span class="overlay-minified:hidden">"管理中心"</span>
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
                                    href="/admin/users"
                                    class=move || if is_active("/admin/users") { "menu-active w-full" } else { "w-full" }
                                >
                                    <span class="icon-[tabler--users] size-5"></span>
                                    "用户管理"
                                </a>
                            </li>
                            <li>
                                <a
                                    href="/admin/settings"
                                    class=move || if is_active("/admin/settings") || is_active("/admin") { "menu-active w-full" } else { "w-full" }
                                >
                                    <span class="icon-[tabler--settings] size-5"></span>
                                    "系统设置"
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
                            aria-label="管理中心"
                        >
                            <span class="icon-[tabler--shield-check] size-5"></span>
                            <span class="sr-only">"管理中心"</span>
                        </button>

                        <ul
                            class="dropdown-menu absolute left-full top-0 ms-3 shadow-md shadow-base-300/20 hidden min-w-60 w-60 dropdown-open:opacity-100 ms-0 ps-0 before:hidden before:content-none"
                            role="menu"
                            aria-orientation="vertical"
                        >
                            <li>
                                <a
                                    href="/admin/users"
                                    class=move || if is_active("/admin/users") { "menu-active w-full" } else { "w-full" }
                                >
                                    <span class="icon-[tabler--users] size-5"></span>
                                    "用户管理"
                                </a>
                            </li>
                            <li>
                                <a
                                    href="/admin/settings"
                                    class=move || if is_active("/admin/settings") || is_active("/admin") { "menu-active w-full" } else { "w-full" }
                                >
                                    <span class="icon-[tabler--settings] size-5"></span>
                                    "系统设置"
                                </a>
                            </li>
                        </ul>
                    </li>

                </ul>
            </div>
        </aside>
    }
}
