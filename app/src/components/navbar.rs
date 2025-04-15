use leptos::prelude::*;

#[component]
pub fn Navbar() -> impl IntoView {
    view! {
        <div class="navbar bg-base-100 sticky top-0 z-50 border-b border-base-200 shadow-sm">
            <div class="flex-none lg:hidden">
                <label for="sidebar-toggle" class="btn btn-square btn-ghost hover:bg-base-200">
                    <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-6 h-6 stroke-current">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16"></path>
                    </svg>
                </label>
            </div>

            <div class="flex-1 px-2 mx-2">
                <span class="font-bold text-xl bg-gradient-to-r from-primary to-secondary bg-clip-text text-transparent">
                    PicoCRM
                </span>
            </div>

            <div class="flex-none gap-2">
                <div class="dropdown dropdown-end mr-4">
                    <button class="btn btn-ghost btn-circle hover:bg-base-200">
                        <div class="indicator">
                            <span class="indicator-item badge badge-primary badge-sm">3</span>
                            <svg xmlns="http://www.w3.org/2000/svg" class="h-6 w-6" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-1.405-1.405A2.032 2.032 0 0118 14.158V11a6.002 6.002 0 00-4-5.659V5a2 2 0 10-4 0v.341C7.67 6.165 6 8.388 6 11v3.159c0 .538-.214 1.055-.595 1.436L4 17h5m6 0v1a3 3 0 11-6 0v-1m6 0H9" />
                            </svg>
                        </div>
                    </button>
                    <div tabindex="0" class="mt-3 z-[1] card card-compact dropdown-content w-72 bg-base-100 shadow">
                        <div class="card-body">
                            <span class="font-bold text-lg">3 条新通知</span>
                            <div class="flex flex-col gap-2">
                                <a class="hover:bg-base-200 p-2 rounded">用户 A 提交了新订单</a>
                                <a class="hover:bg-base-200 p-2 rounded">系统将于今晚进行维护</a>
                                <a class="hover:bg-base-200 p-2 rounded">您收到了新消息</a>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="dropdown dropdown-end">
                    <label tabindex="0" class="btn btn-ghost btn-circle avatar hover:bg-base-200">
                        <div class="avatar">
                          <div class="w-8 rounded-full">
                            <img src="https://img.daisyui.com/images/stock/photo-1534528741775-53994a69daeb.webp" />
                          </div>
                        </div>
                    </label>

                    <ul tabindex="0" class="mt-3 z-[1] p-2 shadow menu menu-sm dropdown-content bg-base-100 rounded-box w-52 border border-base-200">
                        <li>
                            <a class="hover:bg-base-200">
                                <i class="fas fa-user-circle"></i>
                                个人中心
                            </a>
                        </li>
                        <li>
                            <a class="hover:bg-base-200">
                                <i class="fas fa-cog"></i>
                                设置
                            </a>
                        </li>
                        <li>
                            <a class="hover:bg-error hover:text-error-content">
                                <i class="fas fa-sign-out-alt"></i>
                                退出
                            </a>
                        </li>
                    </ul>
                </div>
            </div>
        </div>
    }
}
