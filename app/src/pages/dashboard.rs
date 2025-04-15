use leptos::prelude::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title">今日新增客户</h2>
                    <p class="text-3xl font-bold">24</p >
                    <div class="badge badge-success">+12%</div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title">待跟进</h2>
                    <p class="text-3xl font-bold">8</p >
                    <div class="badge badge-warning">需处理</div>
                </div>
            </div>

            <div class="card bg-base-100 shadow">
                <div class="card-body">
                    <h2 class="card-title">本月成交额</h2>
                    <p class="text-3xl font-bold">"¥56,800"</p >
                    <div class="badge badge-primary">目标达成</div>
                </div>
            </div>
        </div>
    }
}
