use crate::components::features::{ContactModal, UpdateContactModal};
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable, SortValue};
use crate::components::ui::toast::{show_toast, ToastType};
use crate::utils::file_download::download_file;
use js_sys::Math::random;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::hooks::use_query_map;
use server_fn::ServerFnError;
use shared::{
    contact::{Contact, ContactFilters, ContactQuery, SortField, SortOption, SortOrder},
    ListResult,
};
#[cfg(feature = "ssr")]
pub mod ssr {
    pub use backend::application::services::contact_service::ContactAppService;
    pub use backend::domain::services::contact_service::ContactService;
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::repositories::contact_repository_impl::SeaOrmContactRepository;
}

#[server]
pub async fn fetch_contacts(params: ContactQuery) -> Result<ListResult<Contact>, ServerFnError> {
    use self::ssr::*;

    let pool = expect_context::<Database>();
    let contact_repository = SeaOrmContactRepository::new(pool.connection.clone());
    let contact_service = ContactService::new(contact_repository);
    let app_service = ContactAppService::new(contact_service);

    println!("pool {:?}", pool);

    println!("Fetching contacts...");

    let res = app_service
        .fetch_contacts(params)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    println!("Fetching contacts result {:?}", res);
    Ok(res)
}

#[server]
pub async fn export_contacts(params: ContactQuery) -> Result<Vec<u8>, ServerFnError> {
    use self::ssr::*;

    let pool = expect_context::<Database>();
    let contact_repository = SeaOrmContactRepository::new(pool.connection.clone());
    let contact_service = ContactService::new(contact_repository);
    let app_service = ContactAppService::new(contact_service);

    println!("pool {:?}", pool);

    println!("Fetching contacts...");

    let excel_data = app_service
        .fetch_contacts_excel_data(params)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    // println!("Fetching contacts excel_data {:?}", excel_data);

    Ok(excel_data)
}

impl Identifiable for Contact {
    fn id(&self) -> String {
        // self.contact_uuid.clone()
        let rand_suffix = (random() * 10000.0) as u32;
        format!("{}-{}", self.contact_uuid, rand_suffix)
    }
}

#[component]
pub fn ContactsList() -> impl IntoView {
    let (sort_ops, set_sort_ops) = signal::<Vec<(String, SortValue)>>(vec![]);
    let (name, set_name) = signal(String::new());
    let (status, set_status) = signal(String::new());
    let (edit_contact_uuid, set_edit_contact_uuid) = signal(String::new());
    let show_modal = RwSignal::new(false);
    let show_update_modal = RwSignal::new(false);
    let refresh_count = RwSignal::new(0);
    let query = use_query_map();

    let data = Resource::new(
        move || {
            (
                sort_ops.get(),
                name.get(),
                status.get(),
                refresh_count.get(),
                query.get(),
            )
        },
        |(sort_ops, name, status, _, query)| async move {
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

            let sort_options: Vec<SortOption> = sort_ops
                .iter()
                .filter_map(|(field, sort_value)| {
                    let field_enum = match field.as_str() {
                        "user_name" => Some(SortField::Name),
                        "last_contact" => Some(SortField::LastContact),
                        _ => None, // 忽略无效字段
                    }?;

                    let order = match sort_value {
                        SortValue::Asc => SortOrder::Asc,
                        SortValue::Desc => SortOrder::Desc,
                    };

                    Some(SortOption {
                        field: field_enum,
                        order,
                    })
                })
                .collect();

            // logging::error!("Generated sort options: {:?}", sort_options);
            // logging::error!("Fetching contacts with query: {:?} {:?}", page, page_size);
            let filters = ContactFilters {
                user_name: (!name.is_empty()).then_some(name),
                status: (!status.is_empty()).then_some(status),
                email: None,
                phone_number: None,
            };
            // logging::error!("Fetching contacts with filters: {:?} ", filters);

            let params = ContactQuery {
                page,
                page_size,
                sort: Some(sort_options),
                filters: Some(filters),
            };
            let result = fetch_contacts(params).await.unwrap_or_else(|e| {
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

    let on_sort = Callback::new(move |(field, sort_value): (String, SortValue)| {
        set_sort_ops.update(|current| {
            let mut new_ops = current
                .iter()
                .filter(|(f, _)| f != &field)
                .cloned()
                .collect::<Vec<_>>();

            new_ops.push((field, sort_value));
            *current = new_ops;
        });
    });

    let search = move |ev| {
        let value = event_target_value(&ev);
        set_name.set(value);
        // logging::error!("Fetching contacts with name: {:?}", value);
    };

    let filter_by_status = move |ev| {
        let value = event_target_value(&ev);
        set_status.set(value);
    };

    let export_excel = move |_ev| {
        let sort_options: Vec<SortOption> = sort_ops
            .get_untracked()
            .iter()
            .filter_map(|(field, sort_value)| {
                let field_enum = match field.as_str() {
                    "user_name" => Some(SortField::Name),
                    "last_contact" => Some(SortField::LastContact),
                    _ => None,
                }?;

                let order = match sort_value {
                    SortValue::Asc => SortOrder::Asc,
                    SortValue::Desc => SortOrder::Desc,
                };

                Some(SortOption {
                    field: field_enum,
                    order,
                })
            })
            .collect();

        // logging::error!("Generated sort options: {:?}", sort_options);
        let filters = ContactFilters {
            user_name: (!name.get_untracked().is_empty()).then_some(name.get_untracked()),
            status: (!status.get_untracked().is_empty()).then_some(status.get_untracked()),
            email: None,
            phone_number: None,
        };

        let params = ContactQuery {
            page: 1,
            page_size: 10,
            sort: Some(sort_options),
            filters: Some(filters),
        };

        // logging::error!("export contacts with params: {:?} ", &params);

        spawn_local(async move {
            let result = export_contacts(params).await;
            // logging::error!("export result: {:?}", result);

            match result {
                Ok(data) => {
                    let _ = download_file(&data, "contacts.xlsx");
                }
                Err(_e) => {
                    show_toast("操作失败".to_string(), ToastType::Success);
                }
            }
        });
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
                    <input type="search" on:input=search required placeholder="搜索客户..." />
                </label>
                <div class="flex gap-2 items-center">
                    <select on:change=filter_by_status class="select select-bordered">
                        <option disabled selected>状态筛选</option>
                        <option value="">全部</option>
                        <option value="1">已签约</option>
                        <option value="2">待跟进</option>
                        <option value="3">已流失</option>
                    </select>
                    <button class="btn btn-ghost">
                        <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M3 4a1 1 0 011-1h16a1 1 0 011 1v2.586a1 1 0 01-.293.707l-6.414 6.414a1 1 0 00-.293.707V17l-4 4v-6.586a1 1 0 00-.293-.707L3.293 7.293A1 1 0 013 6.586V4z" />
                        </svg>
                        更多筛选
                    </button>
                    <button on:click=export_excel class="btn btn-sm btn-ghost">
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
                    on:click=move |_|{
                        set_edit_contact_uuid.set(String::new());
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
            <ContactModal show=show_modal on_finish=on_contact_modal_finish  />
            <UpdateContactModal show=show_update_modal contact_uuid=edit_contact_uuid on_finish=on_contact_modal_finish />
            <div class="overflow-x-auto h-[calc(100vh-200px)] bg-base-100 rounded-lg shadow">
                <DaisyTable data=data on_sort=on_sort>
                    <Column
                        slot:columns
                        freeze=true
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
                        sort=true
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
                        label="客户价值".to_string()
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
                                    <a href=format!("/contacts/{}", user.clone().map(|u| u.contact_uuid).unwrap_or("".to_string())) class="btn btn-ghost btn-xs">查看</a>
                                    <button on:click=move |_ev| {
                                        let contact_uuid = user.clone().map(|u| u.contact_uuid).unwrap_or("".to_string());
                                        set_edit_contact_uuid.set(contact_uuid);
                                        show_update_modal.set(true);
                                    } class="btn btn-soft btn-warning btn-xs">修改</button>
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
