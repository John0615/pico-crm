// use leptos::logging::log;
use leptos::prelude::*;
use leptos_router::hooks::use_location;

#[component]
pub fn Sidebar() -> impl IntoView {
    let location = use_location();
    // 判断当前路径是否匹配菜单项
    let is_active = move |path: &str| location.pathname.with(|current| current == path);
    view! {
        <div class="drawer-side">
            <label for="sidebar-toggle" class="drawer-overlay"></label>
            <div class="menu p-4 w-64 h-full bg-base-200 text-base-content space-y-2">
                <div class="flex items-center p-4 rounded-lg bg-gradient-to-r from-primary/10 to-secondary/10 border border-base-300 mb-4">
                    <div class="ml-3">
                        <p class="font-bold">管理员</p>
                        <p class="text-xs opacity-70">admin@picocrm.com</p>
                    </div>
                </div>

                <ul class="space-y-1">
                    <li>
                        <a href="/" class:menu-active=move || is_active("/") class="group hover:bg-primary/10 hover:text-primary rounded-lg transition-all duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 group-[.menu-active]:text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 12l2-2m0 0l7-7 7 7M5 10v10a1 1 0 001 1h3m10-11l2 2m-2-2v10a1 1 0 01-1 1h-3m-6 0a1 1 0 001-1v-4a1 1 0 011-1h2a1 1 0 011 1v4a1 1 0 001 1m-6 0h6" />
                            </svg>
                            <span class="group-[.menu-active]:font-semibold">仪表盘</span>
                            <div class="group-[.menu-active]:block hidden absolute right-4 top-1/2 transform -translate-y-1/2 w-2 h-2 rounded-full bg-primary"></div>
                        </a>
                    </li>

                    <li>
                        <a href="/contacts" class:menu-active=move || is_active("/contacts") class="group hover:bg-primary/10 hover:text-primary rounded-lg transition-all duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5 group-[.menu-active]:text-primary" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 4.354a4 4 0 110 5.292M15 21H3v-1a6 6 0 0112 0v1zm0 0h6v-1a6 6 0 00-9-5.197M13 7a4 4 0 11-8 0 4 4 0 018 0z" />
                            </svg>
                            <span class="group-[.menu-active]:font-semibold">客户管理</span>
                            <div class="group-[.menu-active]:block hidden absolute right-4 top-1/2 transform -translate-y-1/2 w-2 h-2 rounded-full bg-primary"></div>
                            <span class="badge badge-primary badge-sm ml-auto">12</span>
                        </a>
                    </li>

                    <li>
                        <a class="group hover:bg-primary/10 hover:text-primary rounded-lg transition-all duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M8 7V3m8 4V3m-9 8h10M5 21h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v12a2 2 0 002 2z" />
                            </svg>
                            <span>销售机会</span>
                            <span class="badge badge-secondary badge-sm ml-auto">5</span>
                        </a>
                    </li>

                    <li>
                        <a class="group hover:bg-primary/10 hover:text-primary rounded-lg transition-all duration-200">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2" />
                            </svg>
                            <span>任务</span>
                        </a>
                    </li>
                </ul>

                <div class="absolute bottom-0 left-0 right-0 p-4 border-t border-base-300">
                    <div class="flex justify-between items-center">
                        <button class="btn btn-ghost btn-sm hover:bg-base-300">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.065 2.572c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.572 1.065c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.065-2.572c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                            </svg>
                        </button>
                        <button class="btn btn-ghost btn-sm hover:bg-base-300">
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.354 15.354A9 9 0 018.646 3.646 9.003 9.003 0 0012 21a9.003 9.003 0 008.354-5.646z" />
                            </svg>
                        </button>
                    </div>
                </div>
            </div>
        </div>
    }
}
