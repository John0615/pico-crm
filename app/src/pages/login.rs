use leptos::prelude::*;

#[component]
pub fn Login() -> impl IntoView {
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

            <div class="bubble w-80 h-80 -left-20 -top-20"></div>
            <div class="bubble w-64 h-64 right-0 bottom-1/3"></div>
            <div class="bubble w-96 h-96 -right-20 top-1/4"></div>

            <div class="card w-full max-w-md bg-white/90 backdrop-blur-sm border border-white/20 rounded-soft overflow-hidden floating-card">
                <div class="bg-gradient-to-r from-purple-600 to-pink-500 text-white p-6 text-center rounded-t-soft">
                    <h1 class="text-3xl font-bold">"欢迎登录PicoCRM"</h1>
                    <p class="opacity-90 mt-1 text-sm">"开启您的专属体验"</p>
                </div>

                <div class="card-body p-8">
                    <form class="space-y-4">
                        <div class="form-control">
                            <label class="input-label font-medium text-gray-700">"用户名"</label>
                            <div class="input-container">
                                <span class="input-icon">
                                    <i class="fas fa-user text-lg"></i>
                                </span>
                                <input
                                    type="text"
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
                    </form>

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
