use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;

use crate::components::ui::toast::{error, success};
use crate::utils::api::call_api;
use shared::admin::{TenantMigrationRequest, TenantMigrationResponse};

pub use crate::server::admin_maintenance_handlers::run_tenant_migrations;

#[component]
pub fn TenantMaintenance() -> impl IntoView {
    let migration_status = RwSignal::new(String::new());
    let migration_merchant_uuid = RwSignal::new(String::new());
    let migration_loading = RwSignal::new(false);
    let migration_result: RwSignal<Option<TenantMigrationResponse>> = RwSignal::new(None);

    let on_run_migrations = move |_| {
        if migration_loading.get() {
            return;
        }
        migration_loading.set(true);

        let status = migration_status.get();
        let merchant_uuid = migration_merchant_uuid.get();

        spawn_local(async move {
            let request = TenantMigrationRequest {
                status: normalize_optional_string(status),
                merchant_uuid: normalize_optional_string(merchant_uuid),
            };

            match call_api(run_tenant_migrations(Some(request))).await {
                Ok(result) => {
                    let message = format!(
                        "租户迁移完成：共 {}，成功 {}，失败 {}",
                        result.total, result.migrated, result.failed
                    );
                    if result.failed > 0 {
                        error(message);
                    } else {
                        success(message);
                    }
                    migration_result.set(Some(result));
                }
                Err(e) => {
                    error(format!("执行失败: {}", e));
                }
            }
            migration_loading.set(false);
        });
    };

    view! {
        <Title text="租户维护 - PicoCRM"/>
        <div class="space-y-6">
            <div class="card bg-base-100 shadow-xl">
                <div class="card-body space-y-4">
                    <div>
                        <h2 class="card-title">"租户维护"</h2>
                        <p class="text-sm text-base-content/60">
                            "执行租户级迁移以补齐缺失表结构（建议低峰操作）。"
                        </p>
                    </div>
                    <div class="grid grid-cols-1 gap-4 lg:grid-cols-3">
                        <div>
                            <label class="label">
                                <span class="label-text">"商户状态（可选）"</span>
                            </label>
                            <select
                                class="select select-bordered w-full"
                                prop:value=move || migration_status.get()
                                on:change=move |ev| {
                                    migration_status.set(event_target_value(&ev));
                                }
                            >
                                <option value="">"全部"</option>
                                <option value="active">"活跃"</option>
                                <option value="inactive">"停用"</option>
                                <option value="suspended">"暂停"</option>
                            </select>
                        </div>
                        <div class="lg:col-span-2">
                            <label class="label">
                                <span class="label-text">"商户 UUID（可选）"</span>
                            </label>
                            <input
                                type="text"
                                class="input input-bordered w-full"
                                placeholder="指定某个商户执行迁移"
                                prop:value=move || migration_merchant_uuid.get()
                                on:input=move |ev| {
                                    migration_merchant_uuid.set(event_target_value(&ev));
                                }
                            />
                        </div>
                    </div>
                    <div class="flex flex-wrap items-center justify-between gap-3">
                        <p class="text-xs text-base-content/50">
                            "状态选择“全部”且商户 UUID 留空时，将对全部商户执行。"
                        </p>
                        <button
                            class="btn btn-warning"
                            on:click=on_run_migrations
                            disabled=move || migration_loading.get()
                        >
                            {move || if migration_loading.get() { "执行中..." } else { "执行租户迁移" }}
                        </button>
                    </div>
                    {move || migration_result.get().map(|result| {
                        let failures = result.failures.clone();
                        view! {
                            <div class="rounded-lg border border-base-300 bg-base-200/50 p-4 text-sm space-y-2">
                                <div>
                                    {format!(
                                        "结果：共 {}，成功 {}，失败 {}",
                                        result.total, result.migrated, result.failed
                                    )}
                                </div>
                                {(!failures.is_empty()).then(|| view! {
                                    <div class="space-y-1">
                                        <For
                                            each=move || failures.clone()
                                            key=|item| item.schema_name.clone()
                                            children=move |item| view! {
                                                <div class="text-xs text-error">
                                                    {format!("{}: {}", item.schema_name, item.error)}
                                                </div>
                                            }
                                        />
                                    </div>
                                })}
                            </div>
                        }
                    })}
                </div>
            </div>
        </div>
    }
}

fn normalize_optional_string(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed == "all" {
        None
    } else {
        Some(trimmed.to_string())
    }
}
