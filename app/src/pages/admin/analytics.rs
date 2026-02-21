use leptos::logging;
use leptos::prelude::*;
use leptos_meta::Title;

use crate::components::ui::toast::error;
use crate::utils::api::call_api;
use shared::analytics::{
    AnalyticsBreakdownItem, AnalyticsBreakdownResponse, AnalyticsOverviewResponse, AnalyticsQuery,
    AnalyticsTrendPoint, AnalyticsTrendResponse,
};

pub use crate::server::admin_analytics_handlers::{
    fetch_admin_analytics_breakdown, fetch_admin_analytics_overview, fetch_admin_analytics_trends,
};

#[component]
pub fn AdminAnalytics() -> impl IntoView {
    let (preset, set_preset) = signal("last_7_days".to_string());
    let (granularity, set_granularity) = signal("day".to_string());
    let (dimension, set_dimension) = signal("merchant_status".to_string());

    let overview = Resource::new(
        move || preset.get(),
        |preset| async move {
            call_api(fetch_admin_analytics_overview(AnalyticsQuery {
                preset: Some(preset),
                ..Default::default()
            }))
            .await
            .unwrap_or_else(|e| {
                logging::error!("overview error: {e}");
                error(format!("加载概览失败: {e}"));
                AnalyticsOverviewResponse {
                    meta: Default::default(),
                    overview: Default::default(),
                }
            })
        },
    );

    let trends = Resource::new(
        move || (preset.get(), granularity.get()),
        |(preset, granularity)| async move {
            call_api(fetch_admin_analytics_trends(AnalyticsQuery {
                preset: Some(preset),
                granularity: Some(granularity),
                ..Default::default()
            }))
            .await
            .unwrap_or_else(|e| {
                logging::error!("trend error: {e}");
                error(format!("加载趋势失败: {e}"));
                AnalyticsTrendResponse {
                    meta: Default::default(),
                    series: Vec::new(),
                }
            })
        },
    );

    let breakdown = Resource::new(
        move || (preset.get(), dimension.get()),
        |(preset, dimension)| async move {
            call_api(fetch_admin_analytics_breakdown(AnalyticsQuery {
                preset: Some(preset),
                dimension: Some(dimension),
                ..Default::default()
            }))
            .await
            .unwrap_or_else(|e| {
                logging::error!("breakdown error: {e}");
                error(format!("加载分布失败: {e}"));
                AnalyticsBreakdownResponse {
                    meta: Default::default(),
                    dimension: "merchant_status".to_string(),
                    items: Vec::new(),
                }
            })
        },
    );

    let on_preset_change = move |ev| {
        set_preset.set(event_target_value(&ev));
    };
    let on_granularity_change = move |ev| {
        set_granularity.set(event_target_value(&ev));
    };
    let on_dimension_change = move |ev| {
        set_dimension.set(event_target_value(&ev));
    };

    view! {
        <Title text="平台统计 - PicoCRM"/>
        <div class="space-y-6">
            <div class="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-4">
                <h1 class="text-2xl font-semibold">"平台统计"</h1>
                <div class="card bg-base-100 shadow-sm w-full lg:w-auto">
                    <div class="card-body p-4">
                        <div class="grid grid-cols-1 sm:grid-cols-3 gap-3">
                            <div class="flex flex-col gap-1">
                                <span class="text-xs text-base-content/60">"时间范围"</span>
                                <select
                                    class="select select-bordered w-full min-w-[140px]"
                                    on:change=on_preset_change
                                    prop:value=move || preset.get()
                                >
                                    <option value="today">"今日"</option>
                                    <option value="last_7_days">"最近 7 天"</option>
                                    <option value="last_30_days">"最近 30 天"</option>
                                </select>
                            </div>
                            <div class="flex flex-col gap-1">
                                <span class="text-xs text-base-content/60">"聚合粒度"</span>
                                <select
                                    class="select select-bordered w-full min-w-[140px]"
                                    on:change=on_granularity_change
                                    prop:value=move || granularity.get()
                                >
                                    <option value="day">"按日"</option>
                                    <option value="week">"按周"</option>
                                    <option value="month">"按月"</option>
                                </select>
                            </div>
                            <div class="flex flex-col gap-1">
                                <span class="text-xs text-base-content/60">"分布维度"</span>
                                <select
                                    class="select select-bordered w-full min-w-[160px]"
                                    on:change=on_dimension_change
                                    prop:value=move || dimension.get()
                                >
                                    <option value="merchant_status">"商户状态分布"</option>
                                    <option value="plan_type">"套餐分布"</option>
                                    <option value="merchant_type">"商户类型分布"</option>
                                </select>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <Suspense fallback=move || view! { <div class="text-sm text-base-content/60">"加载概览..."</div> }>
                {move || overview.get().map(|data| {
                    let metrics = &data.overview;
                    view! {
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                            <MetricCard title="商户总数" value=format_number(metrics.total_merchants) subtitle=Some(format!("活跃 {} 家", format_number(metrics.active_merchants))) />
                            <MetricCard title="用户总数" value=format_number(metrics.total_users) subtitle=Some(format!("活跃 {}", format_number(metrics.active_users))) />
                            <MetricCard title="新增" value=format!("商户 {} / 用户 {}", format_number(metrics.new_merchants), format_number(metrics.new_users)) subtitle=None />
                        </div>
                    }
                })}
            </Suspense>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body space-y-4">
                        <h2 class="text-lg font-semibold">"趋势"</h2>
                        <Suspense fallback=move || view! { <div class="text-sm text-base-content/60">"加载趋势..."</div> }>
                            {move || trends.get().map(|data| {
                                if data.series.is_empty() {
                                    view! { <div class="text-sm text-base-content/60">"暂无趋势数据"</div> }.into_any()
                                } else {
                                    view! { <TrendChart series=data.series.clone() /> }.into_any()
                                }
                            })}
                        </Suspense>
                    </div>
                </div>

                <div class="card bg-base-100 shadow-xl">
                    <div class="card-body space-y-4">
                        <h2 class="text-lg font-semibold">"分布"</h2>
                        <Suspense fallback=move || view! { <div class="text-sm text-base-content/60">"加载分布..."</div> }>
                            {move || breakdown.get().map(|data| {
                                if data.items.is_empty() {
                                    view! { <div class="text-sm text-base-content/60">"暂无分布数据"</div> }.into_any()
                                } else {
                                    view! { <BreakdownChart items=data.items.clone() /> }.into_any()
                                }
                            })}
                        </Suspense>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
fn MetricCard(title: &'static str, value: String, subtitle: Option<String>) -> impl IntoView {
    view! {
        <div class="card bg-base-100 shadow-lg">
            <div class="card-body">
                <div class="text-sm text-base-content/60">{title}</div>
                <div class="text-2xl font-semibold">{value}</div>
                {subtitle.map(|text| view! { <div class="text-xs text-base-content/50">{text}</div> })}
            </div>
        </div>
    }
}

#[component]
fn TrendChart(series: Vec<AnalyticsTrendPoint>) -> impl IntoView {
    let max_value = series
        .iter()
        .map(|item| item.new_merchants.max(item.new_users))
        .max()
        .unwrap_or(1);

    view! {
        <div class="space-y-3">
            <For
                each=move || series.clone()
                key=|item| item.bucket.clone()
                children=move |item| {
                    let merchant_ratio = calc_ratio(item.new_merchants, max_value);
                    let user_ratio = calc_ratio(item.new_users, max_value);
                    view! {
                        <div class="space-y-1">
                            <div class="flex items-center justify-between text-xs text-base-content/60">
                                <span>{item.bucket.clone()}</span>
                                <span>{format!("商户 {} / 用户 {}", item.new_merchants, item.new_users)}</span>
                            </div>
                            <div class="h-2 w-full bg-base-200 rounded">
                                <div class="h-2 bg-primary/70 rounded" style=format!("width: {}%", merchant_ratio)></div>
                            </div>
                            <div class="h-2 w-full bg-base-200 rounded">
                                <div class="h-2 bg-secondary/70 rounded" style=format!("width: {}%", user_ratio)></div>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}

#[component]
fn BreakdownChart(items: Vec<AnalyticsBreakdownItem>) -> impl IntoView {
    let max_value = items.iter().map(|item| item.count).max().unwrap_or(1);
    view! {
        <div class="space-y-3">
            <For
                each=move || items.clone()
                key=|item| item.label.clone()
                children=move |item| {
                    let ratio = calc_ratio(item.count, max_value);
                    view! {
                        <div class="space-y-1">
                            <div class="flex items-center justify-between text-xs text-base-content/60">
                                <span>{item.label.clone()}</span>
                                <span>{item.count}</span>
                            </div>
                            <div class="h-2 w-full bg-base-200 rounded">
                                <div class="h-2 bg-accent/70 rounded" style=format!("width: {}%", ratio)></div>
                            </div>
                        </div>
                    }
                }
            />
        </div>
    }
}

fn calc_ratio(value: u64, max_value: u64) -> u64 {
    if max_value == 0 {
        0
    } else {
        ((value as f64 / max_value as f64) * 100.0).round() as u64
    }
}

fn format_number(value: u64) -> String {
    let s = value.to_string();
    let mut result = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i != 0 && i % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result.chars().rev().collect()
}
