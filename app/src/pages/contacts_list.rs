use crate::components::features::ContactModal;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable};
use leptos::logging;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use server_fn::ServerFnError;
use shared::{contact::Contact, ListResult};

#[server]
pub async fn fetch_contacts(
    page: u64,
    page_size: u64,
) -> Result<ListResult<Contact>, ServerFnError> {
    use backend::application::services::contact_service::ContactAppService;
    use backend::domain::services::contact_service::ContactService;
    use backend::infrastructure::db::Database;
    use backend::infrastructure::repositories::contact_repository_impl::SeaOrmContactRepository;

    let pool = expect_context::<Database>();

    let contact_repository = SeaOrmContactRepository::new(pool.connection.clone());
    let contact_service = ContactService::new(contact_repository);
    let app_service = ContactAppService::new(contact_service);

    println!("pool {:?}", pool);

    println!("Fetching contacts...");

    let res = app_service
        .fetch_contacts(page, page_size)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    println!("Fetching contacts result {:?}", res);
    Ok(res)
}

impl Identifiable for Contact {
    fn id(&self) -> String {
        self.contact_uuid.clone()
    }
}

#[component]
pub fn ContactsList() -> impl IntoView {
    let (sort_name_asc, set_sort_name_asc) = signal(true);
    let show_modal = RwSignal::new(false);
    let refresh_count = RwSignal::new(0);
    let query = use_query_map();

    let sort_name = move || {
        set_sort_name_asc.update(|a| *a = !*a);
    };

    let data = Resource::new(
        move || (sort_name_asc.get(), refresh_count.get(), query.get()),
        // every time `count` changes, this will run
        |(_, _, query)| async move {
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
            // logging::error!("Fetching contacts with query: {:?} {:?}", page, page_size);
            let result = fetch_contacts(page, page_size).await.unwrap_or_else(|e| {
                logging::error!("Error loading contacts: {e}");
                ListResult {
                    items: Vec::new(),
                    total: 0,
                }
            });
            (result.items, result.total)
        },
    );

    let on_contact_modal_finish = move || {
        refresh_count.set(refresh_count.get_untracked() + 1);
    };

    view! {
        <div class="">
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
                <input type="search" required placeholder="搜索客户..." />
                </label>
                <div class="flex gap-2 items-center">
                <select class="select select-bordered">
                    <option disabled selected>状态筛选</option>
                    <option>全部</option>
                    <option>已签约</option>
                    <option>待跟进</option>
                    <option>已流失</option>
                </select>
                <button class="btn btn-ghost">
                    <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                    </svg>
                    更多筛选
                </button>
                    <button class="btn btn-sm btn-ghost">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                        </svg>
                        导出
                    </button>
                </div>
            </div>
            // 添加客户按钮
            <div class="fixed bottom-8 right-8 z-10">
                <button
                    on:click=move |_| show_modal.set(true)
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
            <ContactModal show=show_modal on_finish=on_contact_modal_finish  />
            <div class="overflow-x-auto h-[calc(100vh-200px)] bg-base-100 rounded-lg shadow">
                <DaisyTable data=data>
                    <Column
                        slot:columns
                        freeze=true
                        sort=true
                        prop="user_name".to_string()
                        label="姓名".to_string()
                        class="font-bold"
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <span class="font-medium">
                                    {user.map(|u| u.user_name).unwrap_or("".to_string())}
                                </span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="公司".to_string()
                        prop="company".to_string()
                        class=""
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <span>
                                    {user.map(|u| u.company).unwrap_or("".to_string())}
                                </span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="职位".to_string()
                        prop="position".to_string()
                        class=""
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <span>
                                    {user.map(|u| u.position).unwrap_or("".to_string())}
                                </span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="电话".to_string()
                        prop="phone_number".to_string()
                        class=""
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <span>
                                    {user.map(|u| u.phone_number).unwrap_or("".to_string())}
                                </span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="邮箱".to_string()
                        prop="email".to_string()
                        class=""
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <span>
                                    {user.map(|u| u.email).unwrap_or("".to_string())}
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
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <span class=format!("badge {}",
                                    user.map(|u| {
                                        match u.status {
                                            1 => "badge-success",
                                            2 => "badge-warning",
                                            3 => "badge-error",
                                            _ => "badge-info"
                                        }
                                    }).unwrap_or("")

                                )>
                                    {
                                        user.clone().map(|u| {
                                            match u.status {
                                                1 => "已签约",
                                                2 => "待跟进",
                                                3 => "已流失",
                                                _ => "未知",
                                            }
                                        }).unwrap_or("")
                                    }
                                </span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="最后联系".to_string()
                        prop="last_contact".to_string()
                        class=""
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <span>
                                    {user.map(|u| u.last_contact).unwrap_or("".to_string())}
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
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <span class=format!("badge {}",
                                    user.map(|u| {
                                        match u.value_level {
                                            1 => "badge-success",
                                            2 => "badge-warning",
                                            3 => "badge-error",
                                            _ => "badge-info"
                                        }
                                    }).unwrap_or("")

                                )>
                                    {
                                        user.clone().map(|u| {
                                            match u.value_level {
                                                1 => "活跃客户",
                                                2 => "潜在客户",
                                                3 => "不活跃客户",
                                                _ => "未知",
                                            }
                                        }).unwrap_or("")
                                    }
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
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <div class="flex justify-end gap-1">
                                    <a href=format!("/contacts/{}", user.map(|u| u.contact_uuid).unwrap_or("".to_string())) class="btn btn-ghost btn-xs">查看</a>
                                    <button class="btn btn-soft btn-warning btn-xs">修改</button>
                                    <button class="btn btn-soft btn-error btn-xs">删除</button>
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
        </div>
    }
}
