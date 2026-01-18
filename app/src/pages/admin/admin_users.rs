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
use shared::user::{User, UserListQuery, PagedResult};

// 重新导出server函数
pub use crate::server::user_handlers::{fetch_users, delete_user};

impl Identifiable for User {
    fn id(&self) -> String {
        self.uuid.clone()
    }
}

#[component]
pub fn AdminUsers() -> impl IntoView {
    let query = use_query_map();
    let refresh_count = RwSignal::new(0);
    
    // 简单的筛选状态管理
    let (name, set_name) = signal(String::new());
    let (role, set_role) = signal(String::new());
    let (status, set_status) = signal(String::new());
    let (edit_user_uuid, set_edit_user_uuid) = signal(String::new());
    let show_modal = RwSignal::new(false);
    let show_update_modal = RwSignal::new(false);

    let data = Resource::new(
        move || {
            (
                name.get(),
                role.get(),
                status.get(),
                refresh_count.get(),
                query.get(),
            )
        },
        |(name, role, status, _, query)| async move {
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
                name: (!name.is_empty()).then_some(name),
                role: (!role.is_empty()).then_some(role),
                status: (!status.is_empty()).then_some(status),
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
        refresh_count.set(refresh_count.get_untracked() + 1);
        set_edit_user_uuid.set(String::new());
    };

    let search = move |ev| {
        let value = event_target_value(&ev);
        set_name.set(value);
    };

    let filter_by_role = move |ev| {
        let value = event_target_value(&ev);
        set_role.set(value);
    };

    let filter_by_status = move |ev| {
        let value = event_target_value(&ev);
        set_status.set(value);
    };

    let delete_row = move |user_uuid: String| {
        delete_confirm("删除确认", "确定要删除这个用户吗？", move |result| {
            if result {
                let user_uuid_clone = user_uuid.clone();
                logging::error!("delete row user_uuid: {:?}", user_uuid_clone);
                spawn_local(async move {
                    let result = call_api(delete_user(user_uuid_clone)).await;
                    match result {
                        Ok(_) => {
                            success("删除成功".to_string());
                            refresh_count.set(refresh_count.get_untracked() + 1);
                        }
                        Err(err) => {
                            logging::error!("Failed to delete user: {:?}", err);
                            error("删除失败".to_string());
                        }
                    }
                });
            }
        });
    };

    view! {
        <Title text="用户管理 - PicoCRM"/>
        <div class="">
            // 搜索和筛选栏
            <div class="flex flex-col md:flex-row gap-4 mb-4">
                <label class="input">
                    <svg class="h-[1em] opacity-50" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24">
                        <g
                            stroke-linejoin="round"
                            stroke-linecap="round"
                            stroke-width="2.5"
                            fill="none"
                            stroke="currentColor"
                        >
                            <circle cx="11" cy="11" r="8"></circle>
                            <path d="m21 21-4.3-4.3"></path>
                        </g>
                    </svg>
                    <input type="search" on:input=search required placeholder="搜索用户..." />
                </label>
                <div class="flex gap-2 items-center">
                    <select on:change=filter_by_role class="select select-bordered">
                        <option value="">所有角色</option>
                        <option value="admin">管理员</option>
                        <option value="user">普通用户</option>
                        <option value="guest">访客</option>
                    </select>
                    <select on:change=filter_by_status class="select select-bordered">
                        <option value="">所有状态</option>
                        <option value="active">活跃</option>
                        <option value="inactive">禁用</option>
                        <option value="pending">待激活</option>
                    </select>
                </div>
            </div>

            // 添加用户按钮
            <div class="fixed bottom-8 right-8 z-10">
                <button
                    on:click=move |_|{
                        show_modal.set(true);
                    }
                    class="btn btn-circle btn-primary shadow-lg hover:shadow-xl transition-all"
                    style="width: 56px; height: 56px;"
                >
                    <svg
                        xmlns="http://www.w3.org/2000/svg"
                        class="h-6 w-6"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                    >
                        <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            stroke-width="2"
                            d="M12 4v16m8-8H4"
                        />
                    </svg>
                </button>
            </div>

            // 用户表格
            <div class="overflow-x-auto h-[calc(100vh-200px)] bg-base-100 rounded-lg shadow">
                <DaisyTable data=data>
                    <Column
                        slot:columns
                        freeze=true
                        prop="user_name".to_string()
                        label="用户".to_string()
                        class="font-bold"
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            view! {
                                <div class="flex items-center space-x-3">
                                    <div class="avatar placeholder">
                                        <div class="bg-neutral text-neutral-content rounded-full w-12">
                                            <span class="text-lg">{user.as_ref().map(|u| u.user_name.chars().next().unwrap_or('U')).unwrap_or('U')}</span>
                                        </div>
                                    </div>
                                    <div>
                                        <div class="font-bold">{user.as_ref().map(|u| u.user_name.clone()).unwrap_or_default()}</div>
                                        <div class="text-sm opacity-50">{user.as_ref().and_then(|u| u.email.clone()).unwrap_or_default()}</div>
                                    </div>
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="角色".to_string()
                        prop="is_admin".to_string()
                        class=""
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            view! {
                                <span class=format!("badge {}",
                                    user.as_ref().map(|u| {
                                        match u.is_admin {
                                            Some(true) => "badge-primary",
                                            Some(false) => "badge-secondary",
                                            None => "badge-accent"
                                        }
                                    }).unwrap_or("badge-ghost")
                                )>
                                    {user.as_ref().map(|u| {
                                        match u.is_admin {
                                            Some(true) => "管理员",
                                            Some(false) => "普通用户",
                                            None => "访客"
                                        }
                                    }).unwrap_or("访客")}
                                </span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="状态".to_string()
                        prop="status".to_string()
                        class=""
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            view! {
                                <div class=format!("badge {}",
                                    user.as_ref().map(|u| {
                                        match u.status.as_str() {
                                            "active" => "badge-success",
                                            "inactive" => "badge-error",
                                            "pending" => "badge-warning",
                                            _ => "badge-ghost"
                                        }
                                    }).unwrap_or("badge-ghost")
                                )>
                                    {
                                        user.clone().map(|u| {
                                            match u.status.as_str() {
                                                "active" => "活跃",
                                                "inactive" => "禁用",
                                                "pending" => "待激活",
                                                _ => "未知",
                                            }
                                        }).unwrap_or("未知")
                                    }
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="最后登录".to_string()
                        prop="last_login_at".to_string()
                        class=""
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            view! {
                                <span class="text-sm opacity-70">
                                    {user.as_ref().and_then(|u| u.last_login_at.clone()).unwrap_or("从未登录".to_string())}
                                </span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="创建时间".to_string()
                        prop="inserted_at".to_string()
                        class=""
                    >
                        {
                            let user: Option<User> = use_context::<User>();
                            view! {
                                <span class="text-sm opacity-70">
                                    {user.as_ref().map(|u| u.inserted_at.clone()).unwrap_or_default()}
                                </span>
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
                                    <button on:click=move |_ev| {
                                        set_edit_user_uuid.set(user_uuid.clone());
                                        show_update_modal.set(true);
                                    } class="btn btn-soft btn-warning btn-xs">修改</button>
                                    <button on:click=move |_ev| {
                                        delete_row(user_uuid_delete.clone());
                                    } class="btn btn-soft btn-error btn-xs">删除</button>
                                </div>
                            }
                        }
                    </Column>
                </DaisyTable>
            </div>

            <Transition>
                 {move || data.get().map(|data| view! {
                    <Pagination total_items=data.1 />
                 })}
            </Transition>

            // 模态框组件
            <UserModal show=show_modal on_finish=on_user_modal_finish />
            <UpdateUserModal show=show_update_modal user_uuid=edit_user_uuid on_finish=on_user_modal_finish />
        </div>
    }
}