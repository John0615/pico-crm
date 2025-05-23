use crate::components::features::ContactModal;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::Demo;
use leptos::logging;
use leptos::prelude::*;
use leptos_router::hooks::use_query_map;
use server_fn::ServerFnError;
use shared::contact::{Contact, ContactsResult};

#[server]
pub async fn fetch_contacts(page: u64, page_size: u64) -> Result<ContactsResult, ServerFnError> {
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
            fetch_contacts(page, page_size).await.unwrap_or_else(|e| {
                logging::error!("Error loading contacts: {e}");
                ContactsResult {
                    contacts: Vec::new(),
                    total: 0,
                }
            })
        },
    );

    let on_contact_modal_finish = move || {
        refresh_count.set(refresh_count.get_untracked() + 1);
    };

    view! {
        <div class="">
            <Demo />
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
                <table class="table table-pin-rows table-pin-cols whitespace-nowrap">
                    <thead>
                    <tr class="bg-base-200">
                        <th class="cursor-pointer" on:click=move |_| sort_name()>
                            姓名
                            <span class="ml-1 inline-block">
                                {move || if sort_name_asc.get() {
                                    "↑"
                                } else {
                                    "↓"
                                }
                                }
                            </span>
                        </th>
                        <td>公司</td>
                        <td>职位</td>
                        <td>电话</td>
                        <td>邮箱</td>
                        <td class="cursor-pointer hover:bg-base-300">
                            状态
                            <span class="ml-1 inline-block">"↑↓"</span>
                        </td>
                        <td class="cursor-pointer hover:bg-base-300">
                            最后联系
                            <span class="ml-1 inline-block">"↑↓"</span>
                        </td>
                        <td>价值等级</td>
                        <th>操作</th>
                    </tr>
                    </thead>
                    <tbody>
                    <Transition
                        fallback=move || view! {
                            <tr class="h-[calc(100vh-300px)]">
                                <td colspan="9" class="h-32 text-center align-middle">
                                    <span class="loading loading-bars loading-xl"></span>
                                </td>
                            </tr>
                        }
                    >
                        <Show
                            when=move || !data.get().map(|d| d.contacts.is_empty()).unwrap_or_default()
                            fallback=move || view! {
                                <tr class="hover:bg-transparent h-[calc(100vh-300px)]">
                                    <td colspan="9" class="py-12 text-center align-middle">
                                        <div class="inline-flex flex-col items-center">
                                            <span class="text-gray-500 font-medium">暂无数据</span>
                                        </div>
                                    </td>
                                </tr>
                            }
                        >
                        <For
                            each=move || data.get().map(|d| d.contacts).unwrap_or_default()
                            key=|contact| contact.contact_uuid.clone()
                            children=move |contact: Contact| {
                                let status = contact.status.clone();
                                let value_level = contact.value_level.clone();
                                view! {
                                    <tr class="hover:bg-base-100">
                                        <th class="font-medium">{contact.user_name.clone()}</th>
                                        <td>{contact.company}</td>
                                        <td>{contact.position}</td>
                                        <td>{contact.phone_number}</td>
                                        <td>{contact.email}</td>
                                        <td>
                                            <span class=format!("badge {}",
                                                match status {
                                                    1 => "badge-success",
                                                    2 => "badge-warning",
                                                    3 => "badge-error",
                                                    _ => "badge-info"
                                                }
                                            )>
                                                {
                                                    match status.clone() {
                                                        1 => "已签约",
                                                        2 => "待跟进",
                                                        3 => "已流失",
                                                        _ => "未知",
                                                    }
                                                }
                                            </span>
                                        </td>
                                        <td>{contact.last_contact}</td>
                                        <td>
                                            <span class=format!("badge {}",
                                                match value_level {
                                                    1 => "badge-success",
                                                    2 => "badge-warning",
                                                    3 => "badge-error",
                                                    _ => "badge-info"
                                                }
                                            )>
                                                {
                                                    match value_level.clone() {
                                                        1 => "活跃客户",
                                                        2 => "潜在客户",
                                                        3 => "不活跃客户",
                                                        _ => "未知",
                                                    }
                                                }
                                            </span>
                                        </td>
                                        <th>
                                            <div class="flex justify-end gap-1">
                                                <a href=format!("/contacts/{}", contact.contact_uuid) class="btn btn-ghost btn-xs">查看</a>
                                                <button class="btn btn-soft btn-warning btn-xs">修改</button>
                                                <button class="btn btn-soft btn-error btn-xs">删除</button>
                                            </div>
                                        </th>
                                    </tr>
                                }
                            }
                        />
                        </Show>

                    </Transition>


                    </tbody>
                </table>
            </div>

            <Transition>
                 {move || data.get().map(|data| view! {
                    <Pagination total_items=data.total />
                 })}
            </Transition>
        </div>
    }
}
