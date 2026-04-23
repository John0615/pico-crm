use leptos::prelude::*;
use leptos_meta::Title;

#[component]
pub fn TenantMaintenance() -> impl IntoView {
    view! {
        <Title text="架构迁移提示 - PicoCRM"/>
        <div class="space-y-6">
            <div class="card bg-base-100 shadow-xl">
                <div class="card-body space-y-4">
                    <div>
                        <h2 class="card-title">"架构迁移提示"</h2>
                        <p class="text-sm text-base-content/60">
                            "系统正在从多 Schema 架构迁移到单库共享表架构。旧的“租户迁移维护”入口已停用。"
                        </p>
                    </div>
                    <div class="rounded-lg border border-warning/30 bg-warning/10 p-4 text-sm text-base-content/80">
                        "后续数据结构调整、回填和验收请统一参照《单库共享表改造实施与验收清单》。"
                    </div>
                </div>
            </div>
        </div>
    }
}
