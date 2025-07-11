use crate::components::ui::drawer::DaisyDrawer;
use leptos::prelude::*;

#[component]
pub fn ContactDetail(open_drawer: RwSignal<bool>) -> impl IntoView {
    view! {
        <DaisyDrawer id="contact-drawer" width=800 position="right" is_open=open_drawer >
            <div class="min-h-screen bg-base-200 p-4 md:p-8">
              <div class="max-w-6xl mx-auto">
                <div class="card bg-base-100 shadow-xl">
                  <div class="card-body p-6 md:p-8">

                    <div class="flex flex-col md:flex-row gap-8">
                      <div class="avatar self-start">
                        <div class="avatar">
                          <div class="w-24 rounded-full">
                            <img src="https://img.daisyui.com/images/stock/photo-1534528741775-53994a69daeb.webp" />
                          </div>
                        </div>
                      </div>

                      <div class="flex-1">
                        <div class="flex flex-col md:flex-row md:items-center gap-4">
                          <h1 class="text-3xl font-bold">
                            张伟
                            <span class="badge badge-primary ml-3">VIP客户</span>
                          </h1>

                          <div class="flex gap-2 md:ml-auto">
                            <button class="btn btn-primary btn-sm md:btn-md">
                              <i class="ph-note-pencil"></i>
                              编辑
                            </button>
                            <button class="btn btn-outline btn-sm md:btn-md">
                              <i class="ph-chat-centered-text"></i>
                              发送消息
                            </button>
                          </div>
                        </div>

                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mt-6">
                          <div class="flex items-center gap-3">
                            <i class="ph-envelope text-xl text-gray-500"></i>
                            <a href="mailto:zhangwei@example.com" class="link link-primary">
                              zhangwei@example.com
                            </a>
                          </div>

                          <div class="flex items-center gap-3">
                            <i class="ph-phone text-xl text-gray-500"></i>
                            <span>138-1234-5678</span>
                          </div>

                          <div class="flex items-center gap-3">
                            <i class="ph-buildings text-xl text-gray-500"></i>
                            <span>星辰科技有限公司</span>
                          </div>

                          <div class="flex items-center gap-3">
                            <i class="ph-calendar text-xl text-gray-500"></i>
                            <span>最后联系: 2023-10-20</span>
                          </div>
                        </div>

                        <div class="mt-6 flex flex-wrap gap-2 items-center">
                          <span class="badge badge-outline gap-1">
                            <i class="ph-star-fill text-yellow-400"></i>
                            重要客户
                          </span>
                          <span class="badge badge-outline gap-1">
                            <i class="ph-handshake text-blue-500"></i>
                            长期合作
                          </span>
                          <button class="btn btn-xs btn-ghost">
                            <i class="ph-plus"></i>
                            添加标签
                          </button>
                        </div>
                      </div>
                    </div>

                    <div class="divider my-2"></div>

                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                      <div>
                        <div class="card bg-base-50 border">
                          <div class="card-body p-5">
                            <h2 class="card-title text-lg mb-4">
                              <i class="ph-notebook text-primary"></i>
                              客户备注
                            </h2>

                            <div class="prose max-w-none">
                              <p>"对新产品线非常感兴趣，需要11月跟进报价。客户偏好蓝色系设计风格。"</p>
                              <p class="text-sm text-gray-500 mt-2">"最后更新: 2023-10-18 14:30"</p>
                            </div>

                            <textarea
                              class="textarea textarea-bordered w-full mt-4"
                              placeholder="添加新备注..."
                              rows="3"
                            ></textarea>

                            <div class="flex justify-end mt-2">
                              <button class="btn btn-primary btn-sm">保存备注</button>
                            </div>
                          </div>
                        </div>

                        <div class="card bg-base-50 border mt-4">
                          <div class="card-body p-5">
                            <h2 class="card-title text-lg mb-4">
                              <i class="ph-info text-secondary"></i>
                              附加信息
                            </h2>

                            <div class="space-y-3">
                              <div>
                                <label class="label-text">客户来源</label>
                                <p>2023年行业展会</p>
                              </div>
                              <div>
                                <label class="label-text">负责销售</label>
                                <p>李经理</p>
                              </div>
                              <div>
                                <label class="label-text">预计价值</label>
                                <p class="text-green-600 font-medium">"¥50,000+"</p>
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>

                      <div>
                        <div class="card bg-base-50 border">
                          <div class="card-body p-5">
                            <h2 class="card-title text-lg mb-4">
                              <i class="ph-clock-counter-clockwise text-accent"></i>
                              互动记录
                            </h2>

                            <ul class="contact-timeline space-y-6 pl-6 relative">
                              <li class="relative pl-4">
                                <div class="flex flex-col sm:flex-row sm:items-start gap-2">
                                  <div class="badge badge-primary badge-sm mt-1 shrink-0">
                                    2023-10-20
                                  </div>
                                  <div class="bg-base-100 p-4 rounded-lg shadow-sm flex-1">
                                    <div class="flex items-center gap-2">
                                      <i class="ph-phone text-primary"></i>
                                      <h3 class="font-medium">电话沟通</h3>
                                      <span class="text-sm text-gray-500 ml-auto">15分钟</span>
                                    </div>
                                    <p class="mt-2 text-sm">
                                      "讨论了新产品的功能需求，客户对AI模块特别感兴趣，要求下周发送详细报价。"
                                    </p>
                                    <div class="mt-2 flex gap-2">
                                      <span class="badge badge-ghost badge-xs">重要</span>
                                    </div>
                                  </div>
                                </div>
                              </li>

                              <li class="relative pl-4">
                                <div class="flex flex-col sm:flex-row sm:items-start gap-2">
                                  <div class="badge badge-secondary badge-sm mt-1 shrink-0">
                                    2023-10-15
                                  </div>
                                  <div class="bg-base-100 p-4 rounded-lg shadow-sm flex-1">
                                    <div class="flex items-center gap-2">
                                      <i class="ph-envelope text-secondary"></i>
                                      <h3 class="font-medium">邮件往来</h3>
                                      <span class="text-sm text-gray-500 ml-auto">报价单</span>
                                    </div>
                                    <p class="mt-2 text-sm">
                                      "发送了产品目录和初步报价，客户回复会在一周内确认需求细节。"
                                    </p>
                                    <div class="mt-2">
                                      <button class="btn btn-xs btn-ghost">
                                        <i class="ph-paperclip"></i>
                                        查看附件
                                      </button>
                                    </div>
                                  </div>
                                </div>
                              </li>
                            </ul>

                            <div class="mt-6">
                              <button class="btn btn-block btn-outline">
                                <i class="ph-plus"></i>
                                添加互动记录
                              </button>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>
        </DaisyDrawer>

    }
}
