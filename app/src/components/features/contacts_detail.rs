use crate::components::ui::drawer::DaisyDrawer;
use leptos::prelude::*;

#[component]
pub fn ContactDetail(open_drawer: RwSignal<bool>) -> impl IntoView {
    view! {
        <DaisyDrawer id="contact-drawer" width=800 position="right" is_open=open_drawer >
            <div class="h-full bg-base-100 flex flex-col overflow-hidden">
                {/* 头部信息区 */}
                <div class="p-6 border-b border-base-200 flex items-start gap-4">
                    <div class="avatar">
                        <div class="w-16 rounded-full">
                            <img src="https://img.daisyui.com/images/stock/photo-1534528741775-53994a69daeb.webp" />
                        </div>
                    </div>

                    <div class="flex-1">
                        <div class="flex flex-wrap items-center gap-2">
                            <h2 class="text-2xl font-bold">"张伟"</h2>
                            <div class="badge badge-primary">"VIP客户"</div>
                            <div class="ml-auto flex gap-2">
                                <button class="btn btn-square btn-sm">
                                    <i class="ph-note-pencil"></i>
                                </button>
                                <button class="btn btn-square btn-sm">
                                    <i class="ph-chat-centered-text"></i>
                                </button>
                            </div>
                        </div>

                        <div class="mt-2 text-sm flex flex-wrap gap-x-4 gap-y-1">
                            <div class="flex items-center gap-1">
                                <i class="ph-envelope text-gray-500"></i>
                                <span>"zhang@example.com"</span>
                            </div>
                            <div class="flex items-center gap-1">
                                <i class="ph-phone text-gray-500"></i>
                                <span>"138-1234-5678"</span>
                            </div>
                            <div class="flex items-center gap-1">
                                <i class="ph-buildings text-gray-500"></i>
                                <span>"星辰科技"</span>
                            </div>
                        </div>
                    </div>
                </div>

                {/* 主体内容区 */}
                <div class="flex-1 overflow-auto">
                    <div class="tabs tabs-boxed px-4 pt-2 bg-base-100 sticky top-0 z-10">
                        <a class="tab tab-active">"沟通记录"</a>
                        <a class="tab">"待办任务"</a>
                        <a class="tab">"关联订单"</a>
                        <a class="tab">"关系图谱"</a>
                    </div>

                    <div class="p-4 space-y-6">
                        {/* 沟通记录面板 */}
                        <div class="space-y-4">
                            <div class="flex items-center justify-between">
                                <h3 class="font-bold flex items-center gap-2">
                                    <i class="ph-chats text-blue-500"></i>
                                    "最近沟通"
                                </h3>
                                <button class="btn btn-xs btn-primary">
                                    <i class="ph-plus"></i>"新增记录"
                                </button>
                            </div>

                            <div class="space-y-3">
                                <div class="chat chat-start">
                                    <div class="chat-header opacity-70 text-sm">
                                        "今天 14:30 · 电话沟通"
                                    </div>
                                    <div class="chat-bubble bg-blue-50 text-gray-800">
                                        "确认了下周会议时间，客户对AI功能感兴趣"
                                        <div class="mt-1 flex gap-1">
                                            <span class="badge badge-xs badge-success">"已完成"</span>
                                        </div>
                                    </div>
                                </div>

                                <div class="chat chat-start">
                                    <div class="chat-header opacity-70 text-sm">
                                        "昨天 09:15 · 邮件"
                                    </div>
                                    <div class="chat-bubble bg-blue-50 text-gray-800">
                                        "发送了产品报价单（附件PDF）"
                                        <div class="mt-1">
                                            <button class="btn btn-xs btn-ghost">
                                                <i class="ph-paperclip"></i>"报价单_20231102.pdf"
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>

                        {/* 待办任务面板 */}
                        <div class="collapse collapse-arrow border border-base-200">
                            <input type="checkbox" />
                            <div class="collapse-title font-medium">
                                <i class="ph-list-checks text-green-500"></i>
                                "待办任务 (3)"
                            </div>
                            <div class="collapse-content">
                                <div class="space-y-2 pt-2">
                                    <div class="flex items-center gap-2 p-2 hover:bg-base-200 rounded">
                                        <input type="checkbox" class="checkbox checkbox-sm" />
                                        <span>"11月5日前发送合同草案"</span>
                                        <span class="badge badge-warning badge-xs ml-auto">"高优先级"</span>
                                    </div>
                                    <div class="flex items-center gap-2 p-2 hover:bg-base-200 rounded">
                                        <input type="checkbox" class="checkbox checkbox-sm" />
                                        <span>"安排产品演示会议"</span>
                                    </div>
                                </div>
                            </div>
                        </div>

                        {/* 关联订单面板 */}
                        <div class="collapse collapse-arrow border border-base-200">
                            <input type="checkbox" />
                            <div class="collapse-title font-medium">
                                <i class="ph-currency-dollar text-purple-500"></i>
                                "关联订单 (2)"
                            </div>
                            <div class="collapse-content">
                                <table class="table table-zebra table-xs mt-2">
                                    <thead>
                                        <tr>
                                            <th>"订单号"</th>
                                            <th>"金额"</th>
                                            <th>"状态"</th>
                                        </tr>
                                    </thead>
                                    <tbody>
                                        <tr>
                                            <td>"ORD-2023-1102"</td>
                                            <td>"¥28,000"</td>
                                            <td><span class="badge badge-success">"已付款"</span></td>
                                        </tr>
                                        <tr>
                                            <td>"ORD-2023-1015"</td>
                                            <td>"¥15,000"</td>
                                            <td><span class="badge badge-warning">"待确认"</span></td>
                                        </tr>
                                    </tbody>
                                </table>
                            </div>
                        </div>

                        {/* 关系图谱面板（改进版）*/}
                        <div class="collapse collapse-arrow border border-base-200">
                            <input type="checkbox" />
                            <div class="collapse-title font-medium">
                                <i class="ph-graph text-red-500"></i>
                                "关系图谱"
                            </div>
                            <div class="collapse-content p-4">
                                {/* 关系网络可视化容器 */}
                                <div class="bg-base-200 rounded-lg p-4 h-64 flex items-center justify-center">
                                    <div class="text-center">
                                        <div class="avatar mb-4">
                                            <div class="w-16 rounded-full ring ring-primary ring-offset-base-100">
                                                <img src="https://img.daisyui.com/images/stock/photo-1534528741775-53994a69daeb.webp" />
                                            </div>
                                        </div>
                                        <p class="font-bold">"张伟"</p>
                                        <p class="text-sm opacity-70">"星辰科技 CTO"</p>

                                        <div class="flex justify-center gap-8 mt-4">
                                            {/* 关联节点1 */}
                                            <div class="flex flex-col items-center">
                                                <div class="avatar">
                                                    <div class="w-10 rounded-full ring ring-secondary">
                                                        <img src="https://i.pravatar.cc/80?img=5" />
                                                    </div>
                                                </div>
                                                <p class="text-xs mt-1">"李四"</p>
                                                <p class="text-xs opacity-50">"技术总监"</p>
                                            </div>

                                            {/* 关联节点2 */}
                                            <div class="flex flex-col items-center">
                                                <div class="avatar">
                                                    <div class="w-10 rounded-full ring ring-accent">
                                                        <img src="https://i.pravatar.cc/80?img=6" />
                                                    </div>
                                                </div>
                                                <p class="text-xs mt-1">"星辰科技"</p>
                                                <p class="text-xs opacity-50">"任职公司"</p>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                {/* 关系强度说明 */}
                                <div class="mt-4 space-y-2 text-sm">
                                    <div class="flex items-center gap-2">
                                        <div class="w-3 h-3 rounded-full bg-primary"></div>
                                        <span>"直接联系人 (强关系)"</span>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <div class="w-3 h-3 rounded-full bg-secondary"></div>
                                        <span>"技术决策影响者"</span>
                                    </div>
                                    <div class="flex items-center gap-2">
                                        <div class="w-3 h-3 rounded-full bg-accent"></div>
                                        <span>"组织关联"</span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                {/* 底部快速操作区 */}
                <div class="p-3 border-t border-base-200 flex gap-2">
                    <button class="btn btn-sm btn-primary flex-1">
                        <i class="ph-phone"></i>"拨打电话"
                    </button>
                    <button class="btn btn-sm btn-outline flex-1">
                        <i class="ph-envelope"></i>"发送邮件"
                    </button>
                </div>
            </div>
        </DaisyDrawer>
    }
}
