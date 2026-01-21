use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn SystemSettings() -> impl IntoView {
    view! {
        <Title text="系统设置 - PicoCRM"/>
        <div class="">
            // 设置选项卡
            <div class="tabs tabs-boxed mb-4">
                <a class="tab tab-active">"常规设置"</a>
                <a class="tab">"安全设置"</a>
                <a class="tab">"通知设置"</a>
                <a class="tab">"备份设置"</a>
            </div>

            // 设置内容
            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-4">
                <GeneralSettings />
                <SecuritySettings />
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6 mb-4">
                <NotificationSettings />
                <BackupSettings />
            </div>

            // 保存按钮
            <div class="flex justify-end space-x-4">
                <button class="btn btn-outline">"重置"</button>
                <button class="btn btn-primary">"保存设置"</button>
            </div>
        </div>
    }
}

#[component]
pub fn GeneralSettings() -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <fieldset class="border border-base-300 rounded-lg p-4">
                    <legend class="text-lg font-semibold px-2">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-5 h-5 stroke-current mr-2">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                        </svg>
                        "常规设置"
                    </legend>
                    <div class="space-y-4">
                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"系统名称"</legend>
                            <input
                                type="text"
                                value="PicoCRM"
                                class="input input-bordered w-full"
                                placeholder="输入系统名称"
                            />
                            <p class="label">"设置系统的显示名称"</p>
                        </fieldset>

                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"系统描述"</legend>
                            <textarea
                                class="textarea textarea-bordered h-24"
                                placeholder="输入系统描述..."
                            ></textarea>
                            <p class="label">"简要描述系统的用途和功能"</p>
                        </fieldset>

                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"系统语言"</legend>
                            <select class="select select-bordered w-full">
                                <option selected>"中文 (简体)"</option>
                                <option>"English"</option>
                                <option>"日本語"</option>
                            </select>
                            <p class="label">"选择系统界面显示语言"</p>
                        </fieldset>

                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"维护模式"</legend>
                            <label class="label cursor-pointer justify-start gap-2">
                                <input type="checkbox" class="checkbox checkbox-primary" />
                                <span class="label-text">"启用维护模式"</span>
                            </label>
                            <p class="label">"启用后系统将进入维护状态，用户无法正常访问"</p>
                        </fieldset>
                    </div>
                </fieldset>
            </div>
        </div>
    }
}

#[component]
pub fn SecuritySettings() -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <fieldset class="border border-base-300 rounded-lg p-4">
                    <legend class="text-lg font-semibold px-2">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-5 h-5 stroke-current mr-2">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z"></path>
                        </svg>
                        "安全设置"
                    </legend>
                    <div class="space-y-4">
                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"密码最小长度"</legend>
                            <input
                                type="range"
                                min="6"
                                max="20"
                                value="8"
                                class="range range-primary"
                                step="1"
                            />
                            <div class="w-full flex justify-between text-xs px-2">
                                <span>"6"</span>
                                <span>"8"</span>
                                <span>"12"</span>
                                <span>"16"</span>
                                <span>"20"</span>
                            </div>
                            <p class="label">"设置用户密码的最小字符长度要求"</p>
                        </fieldset>

                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"双因素认证"</legend>
                            <label class="label cursor-pointer justify-start gap-2">
                                <input type="checkbox" class="checkbox checkbox-secondary" checked />
                                <span class="label-text">"启用双因素认证"</span>
                            </label>
                            <p class="label">"为用户账户提供额外的安全保护"</p>
                        </fieldset>

                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"登录限制"</legend>
                            <label class="label cursor-pointer justify-start gap-2">
                                <input type="checkbox" class="checkbox checkbox-accent" checked />
                                <span class="label-text">"限制登录尝试次数"</span>
                            </label>
                            <p class="label">"防止暴力破解攻击，保护用户账户安全"</p>
                        </fieldset>

                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"会话超时时间"</legend>
                            <input
                                type="number"
                                value="30"
                                min="5"
                                max="480"
                                class="input input-bordered w-full"
                                placeholder="分钟"
                            />
                            <p class="label">"用户无操作后自动退出登录的时间（分钟）"</p>
                        </fieldset>

                        <div class="alert alert-info">
                            <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="stroke-current shrink-0 w-6 h-6">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"></path>
                            </svg>
                            <span>"安全设置更改将在下次登录时生效"</span>
                        </div>
                    </div>
                </fieldset>
            </div>
        </div>
    }
}

#[component]
pub fn NotificationSettings() -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <fieldset class="border border-base-300 rounded-lg p-4">
                    <legend class="text-lg font-semibold px-2">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-5 h-5 stroke-current mr-2">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 17h5l-5 5v-5zM4.828 4.828A4 4 0 015.5 4H9v1H5.5a3 3 0 00-2.121.879l-.707.707A1 1 0 002 7.414V16a2 2 0 002 2h8a2 2 0 002-2v-5a1 1 0 00-.293-.707l-8-8z"></path>
                        </svg>
                        "通知设置"
                    </legend>
                    <div class="space-y-4">
                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"邮件通知"</legend>
                            <label class="label cursor-pointer justify-start gap-2">
                                <input type="checkbox" class="checkbox checkbox-success" checked />
                                <span class="label-text">"启用邮件通知"</span>
                            </label>
                            <p class="label">"接收系统重要事件的邮件通知"</p>
                        </fieldset>

                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"短信通知"</legend>
                            <label class="label cursor-pointer justify-start gap-2">
                                <input type="checkbox" class="checkbox checkbox-warning" />
                                <span class="label-text">"启用短信通知"</span>
                            </label>
                            <p class="label">"接收重要事件的短信提醒"</p>
                        </fieldset>

                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"推送通知"</legend>
                            <label class="label cursor-pointer justify-start gap-2">
                                <input type="checkbox" class="checkbox checkbox-info" checked />
                                <span class="label-text">"启用推送通知"</span>
                            </label>
                            <p class="label">"接收浏览器推送通知"</p>
                        </fieldset>

                        <div class="divider">"通知时间"</div>

                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"工作时间通知"</legend>
                            <div class="flex space-x-2">
                                <input type="time" value="09:00" class="input input-bordered input-sm flex-1" />
                                <span class="self-center">"至"</span>
                                <input type="time" value="18:00" class="input input-bordered input-sm flex-1" />
                            </div>
                            <p class="label">"设置接收通知的工作时间范围"</p>
                        </fieldset>
                    </div>
                </fieldset>
            </div>
        </div>
    }
}

#[component]
pub fn BackupSettings() -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow-xl">
            <div class="card-body">
                <fieldset class="border border-base-300 rounded-lg p-4">
                    <legend class="text-lg font-semibold px-2">
                        <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-5 h-5 stroke-current mr-2">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M20.25 6.375c0 2.278-3.694 4.125-8.25 4.125S3.75 8.653 3.75 6.375m16.5 0c0-2.278-3.694-4.125-8.25-4.125S3.75 4.097 3.75 6.375m16.5 0v11.25c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125V6.375m16.5 0v3.75m-16.5-3.75v3.75m16.5 0v3.75C20.25 16.153 16.556 18 12 18s-8.25-1.847-8.25-4.125v-3.75m16.5 0c0 2.278-3.694 4.125-8.25 4.125s-8.25-1.847-8.25-4.125"></path>
                        </svg>
                        "备份设置"
                    </legend>
                    <div class="space-y-4">
                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"自动备份频率"</legend>
                            <select class="select select-bordered w-full">
                                <option>"每日"</option>
                                <option selected>"每周"</option>
                                <option>"每月"</option>
                                <option>"禁用"</option>
                            </select>
                            <p class="label">"设置系统自动备份的执行频率"</p>
                        </fieldset>

                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"备份保留天数"</legend>
                            <input
                                type="number"
                                value="30"
                                min="1"
                                max="365"
                                class="input input-bordered w-full"
                                placeholder="天数"
                            />
                            <p class="label">"超过此天数的备份将被自动删除"</p>
                        </fieldset>

                        <fieldset class="fieldset">
                            <legend class="fieldset-legend text-left">"压缩备份文件"</legend>
                            <label class="label cursor-pointer justify-start gap-2">
                                <input type="checkbox" class="checkbox checkbox-primary" checked />
                                <span class="label-text">"启用压缩"</span>
                            </label>
                            <p class="label">"压缩备份文件以节省存储空间"</p>
                        </fieldset>

                        <div class="divider">"备份操作"</div>

                        <div class="flex space-x-2">
                            <button class="btn btn-primary flex-1">
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-4 h-4 stroke-current">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4"></path>
                                </svg>
                                "立即备份"
                            </button>
                            <button class="btn btn-outline flex-1">
                                <svg xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24" class="inline-block w-4 h-4 stroke-current">
                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-8l-4-4m0 0L8 8m4-4v12"></path>
                                </svg>
                                "恢复备份"
                            </button>
                        </div>

                        <div class="stats shadow">
                            <div class="stat">
                                <div class="stat-title">"最后备份"</div>
                                <div class="stat-value text-sm">"2024-01-15 10:30"</div>
                                <div class="stat-desc">"备份大小: 2.3 GB"</div>
                            </div>
                        </div>
                    </div>
                </fieldset>
            </div>
        </div>
    }
}
