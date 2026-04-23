use crate::components::features::{UpdateUserModal, UserModal};
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable};
use crate::components::ui::{
    message_box::delete_confirm,
    toast::{error, success},
};
use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use shared::user::{PagedResult, User, UserListQuery};

pub use crate::server::user_handlers::{delete_user, fetch_users, toggle_user_status};

impl Identifiable for User {
    fn id(&self) -> String {
        format!("{}-{}", self.uuid, self.updated_at)
    }
}

#[component]
pub fn AdminUsers() -> impl IntoView {
    let query = use_query_map();
    let refresh_count = RwSignal::new(0);

    let (name, set_name) = signal(String::new());
    let (employment_status, set_employment_status) = signal(String::new());
    let (skill, set_skill) = signal(String::new());
    let (account_status, set_account_status) = signal(String::new());
    let (edit_user_uuid, set_edit_user_uuid) = signal(String::new());
    let show_modal = RwSignal::new(false);
    let show_update_modal = RwSignal::new(false);

    let data = Resource::new(
        move || {
            (
                name.get(),
                employment_status.get(),
                skill.get(),
                account_status.get(),
                refresh_count.get(),
                query.with(|value| value.clone()),
            )
        },
        |(name, employment_status, skill, account_status, _, query)| async move {
            let page = query
                .get("page")
                .unwrap_or_default()
                .parse::<u64>()
                .unwrap_or(1);
            let page_size = query
                .get("page_size")
                .unwrap_or_default()
                .parse::<u64>()
                .unwrap_or(10);

            let params = UserListQuery {
                page,
                page_size,
                name: normalize_optional(name),
                role: Some("user".to_string()),
                status: normalize_optional(account_status),
                employment_status: normalize_optional(employment_status),
                skill: normalize_optional(skill),
                dispatchable_only: None,
            };

            let result = call_api(fetch_users(params)).await.unwrap_or_else(|e| {
                logging::error!("Error loading users: {e}");
                PagedResult {
                    items: Vec::new(),
                    total: 0,
                }
            });
            (result.items, result.total)
        },
    );

    let on_user_modal_finish = move || {
        refresh_count.update(|value| *value += 1);
        set_edit_user_uuid.set(String::new());
    };

    let delete_row = move |user_uuid: String| {
        delete_confirm(
            "删除确认",
            "确定要删除这个员工吗？",
            move |result| {
                if result {
                    let user_uuid_clone = user_uuid.clone();
                    spawn_local(async move {
                        let result = call_api(delete_user(user_uuid_clone)).await;
                        match result {
                            Ok(_) => {
                                success("删除成功".to_string());
                                refresh_count.update(|value| *value += 1);
                            }
                            Err(err) => {
                                logging::error!("Failed to delete user: {:?}", err);
                                error("删除失败".to_string());
                            }
                        }
                    });
                }
            },
        );
    };

    let toggle_status = move |user_uuid: String, current_status: String| {
        let action_text = if current_status == "active" {
            "禁用"
        } else {
            "激活"
        };

        let user_uuid_clone = user_uuid.clone();
        spawn_local(async move {
            let result = call_api(toggle_user_status(user_uuid_clone)).await;
            match result {
                Ok(_) => {
                    success(format!("{}成功", action_text));
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => {
                    logging::error!("Failed to toggle user status: {:?}", err);
                    error(format!("{}失败", action_text));
                }
            }
        });
    };

    view! {
        <Title text="员工管理 - PicoCRM"/>
        <div class="space-y-4">
            <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <div>
                    <h1 class="text-2xl font-semibold text-left">"员工管理"</h1>
                    <p class="mt-1 text-sm text-base-content/60">"维护员工技能、服务范围和在岗状态，离职员工默认不出现在列表中"</p>
                </div>
                <button
                    class="btn btn-primary"
                    on:click=move |_| {
                        show_modal.set(true);
                    }
                >
                    "新建员工"
                </button>
            </div>

            <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                <label class="input input-bordered flex items-center gap-2">
                    <input
                        type="search"
                        prop:value=move || name.get()
                        on:input=move |ev| set_name.set(event_target_value(&ev))
                        placeholder="搜索员工姓名"
                    />
                </label>
                <select
                    class="select select-bordered"
                    prop:value=move || employment_status.get()
                    on:change=move |ev| set_employment_status.set(event_target_value(&ev))
                >
                    <option value="">全部员工状态</option>
                    <option value="active">在岗</option>
                    <option value="on_leave">休假</option>
                    <option value="resigned">离职</option>
                </select>
                <label class="input input-bordered flex items-center gap-2">
                    <input
                        type="search"
                        prop:value=move || skill.get()
                        on:input=move |ev| set_skill.set(event_target_value(&ev))
                        placeholder="按技能筛选"
                    />
                </label>
                <select
                    class="select select-bordered"
                    prop:value=move || account_status.get()
                    on:change=move |ev| set_account_status.set(event_target_value(&ev))
                >
                    <option value="">全部账号状态</option>
                    <option value="active">启用</option>
                    <option value="inactive">禁用</option>
                </select>
            </div>

            <div class="overflow-x-auto h-[calc(100vh-220px)] bg-base-100 rounded-lg shadow">
                <DaisyTable data=data>
                    <Column
                        slot:columns
                        freeze=true
                        prop="user_name".to_string()
                        label="员工".to_string()
                        class="font-bold"
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            view! {
                                <div class="flex items-center space-x-3">
                                    <div class="avatar">
                                        {if let Some(user) = &user {
                                            if let Some(avatar_url) = &user.avatar_url {
                                                view! {
                                                    <div class="w-12 h-12 rounded-full">
                                                        <img src=avatar_url.clone() alt="头像" class="w-full h-full object-cover" />
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! {
                                                    <div class="bg-neutral text-neutral-content rounded-full w-12 placeholder">
                                                        <span class="text-lg">{user.user_name.chars().next().unwrap_or('U')}</span>
                                                    </div>
                                                }.into_any()
                                            }
                                        } else {
                                            view! {
                                                <div class="bg-neutral text-neutral-content rounded-full w-12 placeholder">
                                                    <span class="text-lg">U</span>
                                                </div>
                                            }.into_any()
                                        }}
                                    </div>
                                    <div>
                                        <div class="font-bold">{user.as_ref().map(|u| u.user_name.clone()).unwrap_or_default()}</div>
                                        <div class="text-sm opacity-50">{user.as_ref().and_then(|u| u.phone_number.clone()).unwrap_or_else(|| user.as_ref().and_then(|u| u.email.clone()).unwrap_or_default())}</div>
                                    </div>
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="员工状态".to_string()
                        prop="employment_status".to_string()
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            let badge_class = user.as_ref().map(|u| employment_status_badge_class(&u.employment_status)).unwrap_or("badge-ghost");
                            let label = user.as_ref().map(|u| employment_status_label(&u.employment_status).to_string()).unwrap_or_else(|| "-".to_string());
                            view! { <span class=format!("badge {}", badge_class)>{label}</span> }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="技能".to_string()
                        prop="skills".to_string()
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            view! {
                                <div class="max-w-56 whitespace-normal text-sm">
                                    {user.as_ref().map(|u| summary_list(&u.skills)).unwrap_or_else(|| "-".to_string())}
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="服务范围".to_string()
                        prop="service_areas".to_string()
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            view! {
                                <div class="max-w-48 whitespace-normal text-sm">
                                    {user.as_ref().map(|u| summary_list(&u.service_areas)).unwrap_or_else(|| "-".to_string())}
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="资质档案".to_string()
                        prop="credentials".to_string()
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            let summary = user.as_ref().map(credential_summary).unwrap_or_else(|| "-".to_string());
                            view! {
                                <div class="max-w-56 whitespace-normal text-sm opacity-80">
                                    {summary}
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="入职时间".to_string()
                        prop="joined_at".to_string()
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            view! {
                                <span class="text-sm opacity-70">
                                    {user.as_ref().and_then(|u| u.joined_at.clone()).unwrap_or_else(|| "-".to_string())}
                                </span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="服务表现".to_string()
                        prop="performance".to_string()
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            let summary = user
                                .as_ref()
                                .map(service_performance_summary)
                                .unwrap_or_else(|| "-".to_string());
                            view! {
                                <div class="max-w-48 whitespace-normal text-sm opacity-80">
                                    {summary}
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="售后影响".to_string()
                        prop="after_sales".to_string()
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            let summary = user
                                .as_ref()
                                .map(after_sales_impact_summary)
                                .unwrap_or_else(|| "-".to_string());
                            view! {
                                <div class="max-w-56 whitespace-normal text-sm opacity-80">
                                    {summary}
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="账号状态".to_string()
                        prop="status".to_string()
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            let user_uuid = user.as_ref().map(|u| u.uuid.clone()).unwrap_or_default();
                            let current_status = user.as_ref().map(|u| u.status.clone()).unwrap_or_default();
                            let is_active = current_status == "active";
                            let status_for_closure = current_status.clone();
                            let status_for_display = current_status.clone();
                            view! {
                                <div class="flex items-center gap-2">
                                    <input
                                        type="checkbox"
                                        checked=is_active
                                        class=format!("toggle {}",
                                            if is_active { "toggle-success" } else { "toggle-error" }
                                        )
                                        on:change=move |_| {
                                            toggle_status(user_uuid.clone(), status_for_closure.clone());
                                        }
                                    />
                                    <span class="text-sm">
                                        {
                                            match status_for_display.as_str() {
                                                "active" => "启用",
                                                "inactive" => "禁用",
                                                _ => "未知",
                                            }
                                        }
                                    </span>
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="备注".to_string()
                        prop="employee_note".to_string()
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            view! {
                                <div class="max-w-56 whitespace-normal text-sm opacity-70">
                                    {user.as_ref().and_then(|u| u.employee_note.clone()).unwrap_or_else(|| "-".to_string())}
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        freeze=true
                        label="操作".to_string()
                        prop="".to_string()
                        class="font-bold"
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            let user_uuid = user.as_ref().map(|u| u.uuid.clone()).unwrap_or_default();
                            let user_uuid_delete = user_uuid.clone();
                            view! {
                                <div class="flex justify-end gap-1">
                                    <button on:click=move |_| {
                                        set_edit_user_uuid.set(user_uuid.clone());
                                        show_update_modal.set(true);
                                    } class="btn btn-soft btn-warning btn-xs">修改</button>
                                    <button on:click=move |_| {
                                        delete_row(user_uuid_delete.clone());
                                    } class="btn btn-soft btn-error btn-xs">删除</button>
                                </div>
                            }
                        }
                    </Column>
                </DaisyTable>
            </div>

            <Transition>
                {move || {
                    data.with(|data| {
                        data.as_ref().map(|data| view! { <Pagination total_items=data.1 /> })
                    })
                }}
            </Transition>

            <UserModal show=show_modal on_finish=on_user_modal_finish />
            <UpdateUserModal show=show_update_modal user_uuid=edit_user_uuid on_finish=on_user_modal_finish />
        </div>
    }
}

fn employment_status_label(value: &str) -> &'static str {
    match value {
        "active" => "在岗",
        "on_leave" => "休假",
        "resigned" => "离职",
        _ => "未知",
    }
}

fn employment_status_badge_class(value: &str) -> &'static str {
    match value {
        "active" => "badge-success",
        "on_leave" => "badge-warning",
        "resigned" => "badge-error",
        _ => "badge-ghost",
    }
}

fn summary_list(values: &[String]) -> String {
    if values.is_empty() {
        "-".to_string()
    } else if values.len() <= 3 {
        values.join(" / ")
    } else {
        format!("{} / +{}", values[..3].join(" / "), values.len() - 3)
    }
}

fn service_performance_summary(user: &User) -> String {
    let completed = user.completed_service_count.unwrap_or(0);
    let feedbacks = user.feedback_count.unwrap_or(0);
    let rating = user
        .average_rating
        .map(|value| format!("{value:.1}"))
        .unwrap_or_else(|| "-".to_string());
    format!(
        "已完成 {} 单 / 反馈 {} 条 / 平均评分 {}",
        completed, feedbacks, rating
    )
}

fn after_sales_impact_summary(user: &User) -> String {
    let total = user.after_sales_case_count.unwrap_or(0);
    let complaint = user.complaint_case_count.unwrap_or(0);
    let refund = user.refund_case_count.unwrap_or(0);
    let rework = user.rework_count.unwrap_or(0);
    format!(
        "售后 {} / 投诉 {} / 退款 {} / 返工 {}",
        total, complaint, refund, rework
    )
}

fn credential_summary(user: &User) -> String {
    let health = health_status_label(&user.health_status);
    let training = if user.training_records.is_empty() {
        "培训 0".to_string()
    } else {
        format!("培训 {}", user.training_records.len())
    };
    let certificates = if user.certificates.is_empty() {
        "证书 0".to_string()
    } else {
        format!("证书 {}", user.certificates.len())
    };
    format!("{} / {} / {}", health, training, certificates)
}

fn health_status_label(value: &str) -> &'static str {
    match value {
        "healthy" => "健康",
        "attention" => "需关注",
        "expired" => "已过期",
        _ => "未知",
    }
}

fn normalize_optional(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}
