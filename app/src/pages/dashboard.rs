use crate::components::ui::date_picker::FlyonDatePicker;
use crate::server::merchant_dashboard_handlers::fetch_merchant_dashboard;
use crate::utils::api::call_api;
use leptos::prelude::*;
use leptos_meta::Title;
use shared::merchant_dashboard::{MerchantDashboardQuery, MerchantDashboardResponse};

#[component]
pub fn Dashboard() -> impl IntoView {
    let refresh_count = RwSignal::new(0);
    let (preset, set_preset) = signal("last_7_days".to_string());
    let date_start = RwSignal::new(String::new());
    let date_end = RwSignal::new(String::new());

    let data = Resource::new(
        move || {
            (
                preset.get(),
                date_start.get(),
                date_end.get(),
                *refresh_count.read(),
            )
        },
        |(preset, start, end, _)| async move {
            let (preset, start, end) = if preset == "custom" {
                (None, normalize_optional(&start), normalize_optional(&end))
            } else {
                (Some(preset), None, None)
            };

            let query = MerchantDashboardQuery {
                preset,
                start,
                end,
                timezone: Some("Asia/Shanghai".to_string()),
                granularity: Some("day".to_string()),
            };

            call_api(fetch_merchant_dashboard(query)).await
        },
    );

    let on_refresh = move |_| refresh_count.update(|value| *value += 1);
    let on_preset_change = move |ev| {
        let value = event_target_value(&ev);
        set_preset.set(value.clone());
        if value != "custom" {
            date_start.set(String::new());
            date_end.set(String::new());
        }
    };

    view! {
        <Title text="智能看板 - PicoCRM"/>
        <div class="space-y-4">
            <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <h1 class="text-2xl font-semibold">"智能看板"</h1>
                <button class="btn btn-sm btn-outline" on:click=on_refresh>"刷新"</button>
            </div>

            <div class="card bg-base-100 shadow-sm">
                <div class="card-body p-4 flex flex-col gap-3 md:flex-row md:items-end">
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"时间范围"</span>
                        <select
                            class="select select-bordered min-w-[160px]"
                            prop:value=move || preset.get()
                            on:change=on_preset_change
                        >
                            <option value="today">"今日"</option>
                            <option value="last_7_days">"近7天"</option>
                            <option value="last_30_days">"近30天"</option>
                            <option value="custom">"自定义"</option>
                        </select>
                    </div>
                    <Show when=move || preset.get() == "custom">
                        <div class="flex flex-col gap-1">
                            <span class="text-xs text-base-content/60">"开始"</span>
                            <FlyonDatePicker value=date_start class="input input-bordered".to_string() />
                        </div>
                        <div class="flex flex-col gap-1">
                            <span class="text-xs text-base-content/60">"结束"</span>
                            <FlyonDatePicker value=date_end class="input input-bordered".to_string() />
                        </div>
                    </Show>
                </div>
            </div>

            <Suspense fallback=move || view! {
                <div class="text-sm text-base-content/60">"加载中..."</div>
            }>
                {move || {
                    data.get().map(|result| match result {
                        Ok(payload) => render_dashboard(payload),
                        Err(err) => view! {
                            <div class="text-sm text-error">{format!("加载失败: {}", err)}</div>
                        }
                        .into_any(),
                    })
                }}
            </Suspense>
        </div>
    }
}

fn render_dashboard(payload: MerchantDashboardResponse) -> AnyView {
    let overview = payload.overview.clone();
    let trend = payload.trend.clone();
    let todos = payload.todos.clone();
    let meta = payload.meta.clone();

    let trend_rows = trend
        .iter()
        .rev()
        .take(7)
        .cloned()
        .collect::<Vec<_>>();
    let has_trend = !trend_rows.is_empty();
    let has_todos = !todos.is_empty();

    view! {
        <div class="space-y-4">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                {metric_card("新增客户", overview.new_contacts, "badge-success")}
                {metric_card("需求/预约", overview.service_requests, "badge-warning")}
                {metric_card("订单数", overview.orders, "badge-info")}
                {metric_card("已完成订单", overview.completed_orders, "badge-primary")}
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-4">
                <div class="card bg-base-100 shadow lg:col-span-2">
                    <div class="card-body">
                        <div class="flex items-center justify-between gap-2">
                            <h2 class="card-title">"趋势"</h2>
                            <span class="text-xs text-base-content/60">
                                {format!("{} ~ {}", meta.start, meta.end)}
                            </span>
                        </div>
                        {if has_trend {
                            view! {
                                <div class="overflow-x-auto">
                                    <table class="table table-sm">
                                        <thead>
                                            <tr>
                                                <th>"日期"</th>
                                                <th>"新增客户"</th>
                                                <th>"需求"</th>
                                                <th>"订单"</th>
                                                <th>"完成订单"</th>
                                            </tr>
                                        </thead>
                                        <tbody>
                                            {trend_rows.into_iter().map(|item| view! {
                                                <tr>
                                                    <td>{item.bucket}</td>
                                                    <td>{item.new_contacts}</td>
                                                    <td>{item.service_requests}</td>
                                                    <td>{item.orders}</td>
                                                    <td>{item.completed_orders}</td>
                                                </tr>
                                            }).collect::<Vec<_>>()}
                                        </tbody>
                                    </table>
                                </div>
                            }.into_any()
                        } else {
                            view! { <div class="text-sm text-base-content/60">"暂无趋势数据"</div> }.into_any()
                        }}
                    </div>
                </div>

                <div class="card bg-base-100 shadow">
                    <div class="card-body">
                        <h2 class="card-title">"待办"</h2>
                        {if has_todos {
                            view! {
                                <div class="space-y-2">
                                    {todos.into_iter().map(|todo| {
                                        let link = todo_link(&todo.key);
                                        view! {
                                            <a href={link} class="flex items-center justify-between gap-2 rounded-lg border border-base-200 px-3 py-2 hover:bg-base-200">
                                                <span class="text-sm">{todo.label}</span>
                                                <span class="badge badge-outline">{todo.count}</span>
                                            </a>
                                        }
                                    }).collect::<Vec<_>>()}
                                </div>
                            }.into_any()
                        } else {
                            view! { <div class="text-sm text-base-content/60">"暂无待办"</div> }.into_any()
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
    .into_any()
}

fn metric_card(label: &'static str, value: u64, badge_class: &'static str) -> AnyView {
    view! {
        <div class="card bg-base-100 shadow">
            <div class="card-body">
                <h2 class="card-title">{label}</h2>
                <p class="text-3xl font-bold">{value}</p>
                <div class=format!("badge {}", badge_class)>"统计"</div>
            </div>
        </div>
    }
    .into_any()
}

fn normalize_optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn todo_link(key: &str) -> String {
    match key {
        "pending_requests" => "/service-requests?status=new".to_string(),
        "pending_orders" => "/orders?status=pending".to_string(),
        "upcoming_schedules" => "/schedules?upcoming=1".to_string(),
        _ => "#".to_string(),
    }
}
