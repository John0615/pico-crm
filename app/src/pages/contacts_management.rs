use crate::components::features::{ContactDetail, ContactModal, UpdateContactModal};
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable, SortValue};
use crate::components::ui::{
    message_box::delete_confirm,
    toast::{error, success},
};
use crate::server::contact_handlers::{delete_contact, export_contacts, fetch_contacts};
use crate::utils::api::call_api;
use crate::utils::file_download::download_file;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use shared::{
    contact::{Contact, ContactFilters, ContactQuery, SortField, SortOption, SortOrder},
    ListResult,
};

impl Identifiable for Contact {
    fn id(&self) -> String {
        format!("{}-{}", self.contact_uuid, self.updated_at)
    }
}

#[component]
pub fn ContactsManagement() -> impl IntoView {
    let query = use_query_map();
    let refresh_count = RwSignal::new(0);

    let (sort_ops, set_sort_ops) = signal::<Vec<(String, SortValue)>>(vec![]);
    let (name, set_name) = signal(String::new());
    let (address_keyword, set_address_keyword) = signal(String::new());
    let (tag_keyword, set_tag_keyword) = signal(String::new());
    let (follow_up_status, set_follow_up_status) = signal(String::new());
    let (edit_contact_uuid, set_edit_contact_uuid) = signal(String::new());
    let (detail_contact_uuid, set_detail_contact_uuid) = signal(String::new());

    let show_modal = RwSignal::new(false);
    let show_update_modal = RwSignal::new(false);
    let open_drawer = RwSignal::new(false);

    let data = Resource::new(
        move || {
            (
                sort_ops.get(),
                name.get(),
                address_keyword.get(),
                tag_keyword.get(),
                follow_up_status.get(),
                refresh_count.get(),
                query.with(|value| value.clone()),
            )
        },
        |(sort_ops, name, address_keyword, tag_keyword, follow_up_status, _, query)| async move {
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

            let filters = ContactFilters {
                user_name: normalize_optional(name),
                phone_number: None,
                address_keyword: normalize_optional(address_keyword),
                tag: normalize_optional(tag_keyword),
                follow_up_status: normalize_optional(follow_up_status),
            };

            let params = ContactQuery {
                page,
                page_size,
                sort: Some(sort_options),
                filters: Some(filters),
            };
            let result = call_api(fetch_contacts(params)).await.unwrap_or_else(|e| {
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
        refresh_count.update(|value| *value += 1);
        set_edit_contact_uuid.set(String::new());
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

    let delete_row = move |contact_uuid: String| {
        delete_confirm("删除确认", "确定要删除吗？", move |result| {
            if result {
                let uuid = contact_uuid.clone();
                spawn_local(async move {
                    let result = call_api(delete_contact(uuid)).await;
                    match result {
                        Ok(_) => {
                            success("操作成功".to_string());
                            refresh_count.update(|value| *value += 1);
                        }
                        Err(err) => {
                            logging::error!("Failed to delete contact: {:?}", err);
                            error("操作失败".to_string());
                        }
                    }
                });
            }
        });
    };

    let export_excel = move |_| {
        let sort_options: Vec<SortOption> = sort_ops.with_untracked(|current| {
            current
                .iter()
                .filter_map(|(field, sort_value)| {
                    let field_enum = match field.as_str() {
                        "user_name" => Some(SortField::Name),
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
                .collect()
        });

        let filters = ContactFilters {
            user_name: normalize_optional(name.get_untracked()),
            phone_number: None,
            address_keyword: normalize_optional(address_keyword.get_untracked()),
            tag: normalize_optional(tag_keyword.get_untracked()),
            follow_up_status: normalize_optional(follow_up_status.get_untracked()),
        };

        let params = ContactQuery {
            page: 1,
            page_size: 500,
            sort: Some(sort_options),
            filters: Some(filters),
        };

        spawn_local(async move {
            let result = call_api(export_contacts(params)).await;
            match result {
                Ok(data) => {
                    let _ = download_file(&data, "contacts.xlsx");
                }
                Err(_) => {
                    error("操作失败".to_string());
                }
            }
        });
    };

    view! {
        <Title text="客户管理 - PicoCRM"/>
        <div class="space-y-4">
            <div class="flex flex-col items-start gap-3 text-left md:flex-row md:items-center md:justify-between">
                <div class="text-left">
                    <h1 class="text-2xl font-semibold">"客户管理"</h1>
                    <p class="mt-1 text-sm text-base-content/60">"支持地址、标签、跟进状态和最近服务时间的客户资料管理"</p>
                </div>
                <button
                    class="btn btn-primary"
                    on:click=move |_| {
                        show_modal.set(true);
                    }
                >
                    "新建客户"
                </button>
            </div>

            <div class="grid gap-3 md:grid-cols-2 xl:grid-cols-4">
                <label class="input input-bordered flex items-center gap-2">
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
                    <input
                        type="search"
                        prop:value=move || name.get()
                        on:input=move |ev| set_name.set(event_target_value(&ev))
                        placeholder="搜索客户姓名"
                    />
                </label>

                <label class="input input-bordered flex items-center gap-2">
                    <input
                        type="search"
                        prop:value=move || address_keyword.get()
                        on:input=move |ev| set_address_keyword.set(event_target_value(&ev))
                        placeholder="按地址/小区/门牌筛选"
                    />
                </label>

                <label class="input input-bordered flex items-center gap-2">
                    <input
                        type="search"
                        prop:value=move || tag_keyword.get()
                        on:input=move |ev| set_tag_keyword.set(event_target_value(&ev))
                        placeholder="按标签筛选"
                    />
                </label>

                <select
                    class="select select-bordered"
                    prop:value=move || follow_up_status.get()
                    on:change=move |ev| set_follow_up_status.set(event_target_value(&ev))
                >
                    <option value="">全部跟进状态</option>
                    <option value="pending">待跟进</option>
                    <option value="contacted">已联系</option>
                    <option value="quoted">已报价</option>
                    <option value="scheduled">已预约</option>
                    <option value="completed">已完成</option>
                </select>
            </div>

            <div class="flex flex-wrap items-center gap-2">
                <button on:click=export_excel class="btn btn-sm btn-outline btn-primary">
                    "导出"
                </button>
                <button
                    class="btn btn-sm btn-ghost"
                    on:click=move |_| {
                        set_name.set(String::new());
                        set_address_keyword.set(String::new());
                        set_tag_keyword.set(String::new());
                        set_follow_up_status.set(String::new());
                    }
                >
                    "清空筛选"
                </button>
            </div>

            <ContactModal show=show_modal on_finish=on_contact_modal_finish />
            <UpdateContactModal
                show=show_update_modal
                contact_uuid=edit_contact_uuid
                on_finish=on_contact_modal_finish
            />
            <ContactDetail open_drawer=open_drawer contact_uuid=detail_contact_uuid />

            <div class="overflow-x-auto h-[calc(100vh-220px)] bg-base-100 rounded-lg shadow">
                <DaisyTable data=data on_sort=on_sort>
                    <Column
                        slot:columns
                        freeze=true
                        prop="user_name".to_string()
                        label="姓名".to_string()
                        class="font-bold"
                        sort=true
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <span class="font-medium">
                                    {user.map(|u| u.user_name).unwrap_or_default()}
                                </span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="电话".to_string()
                        prop="phone_number".to_string()
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            view! { <span>{user.map(|u| u.phone_number).unwrap_or_default()}</span> }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="位置".to_string()
                        prop="community".to_string()
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <div class="max-w-48 whitespace-normal text-sm">
                                    {
                                        user.map(|u| {
                                            let community = u.community.unwrap_or_default();
                                            let building = u.building.unwrap_or_default();
                                            match (community.trim().is_empty(), building.trim().is_empty()) {
                                                (true, true) => "-".to_string(),
                                                (false, true) => community,
                                                (true, false) => building,
                                                (false, false) => format!("{} / {}", community, building),
                                            }
                                        }).unwrap_or_else(|| "-".to_string())
                                    }
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="跟进状态".to_string()
                        prop="follow_up_status".to_string()
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            let badge_class = user
                                .as_ref()
                                .map(|u| follow_up_status_badge_class(u.follow_up_status.as_deref()))
                                .unwrap_or("badge-ghost");
                            let label = user
                                .as_ref()
                                .map(|u| {
                                    follow_up_status_label(u.follow_up_status.as_deref()).to_string()
                                })
                                .unwrap_or_else(|| "-".to_string());
                            view! {
                                <span class=format!("badge badge-outline {}", badge_class)>
                                    {label}
                                </span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="标签".to_string()
                        prop="tags".to_string()
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <div class="max-w-48 whitespace-normal text-sm">
                                    {user.map(|u| tags_summary(&u.tags)).unwrap_or_else(|| "-".to_string())}
                                </div>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        label="最近服务".to_string()
                        prop="last_service_at".to_string()
                    >
                        {
                            let user: Option<Contact> = use_context::<Contact>();
                            view! {
                                <span>
                                    {user.and_then(|u| u.last_service_at).unwrap_or_else(|| "-".to_string())}
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
                            let contact_uuid = user.map(|u| u.contact_uuid).unwrap_or_default();
                            let detail_uuid = contact_uuid.clone();
                            let edit_uuid = contact_uuid.clone();
                            let contact_uuid_delete = contact_uuid.clone();
                            view! {
                                <div class="flex justify-end gap-1">
                                    <button
                                        on:click=move |_| {
                                            set_detail_contact_uuid.set(detail_uuid.clone());
                                            open_drawer.set(true);
                                        }
                                        class="btn btn-ghost btn-xs"
                                    >
                                        "查看"
                                    </button>
                                    <button
                                        on:click=move |_| {
                                            set_edit_contact_uuid.set(edit_uuid.clone());
                                            show_update_modal.set(true);
                                        }
                                        class="btn btn-soft btn-warning btn-xs"
                                    >
                                        "修改"
                                    </button>
                                    <button
                                        on:click=move |_| {
                                            delete_row(contact_uuid_delete.clone());
                                        }
                                        class="btn btn-soft btn-error btn-xs"
                                    >
                                        "删除"
                                    </button>
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
        </div>
    }
}

fn follow_up_status_label(value: Option<&str>) -> &'static str {
    match value.unwrap_or("pending") {
        "pending" => "待跟进",
        "contacted" => "已联系",
        "quoted" => "已报价",
        "scheduled" => "已预约",
        "completed" => "已完成",
        _ => "未知",
    }
}

fn follow_up_status_badge_class(value: Option<&str>) -> &'static str {
    match value.unwrap_or("pending") {
        "pending" => "badge-ghost",
        "contacted" => "badge-info",
        "quoted" => "badge-secondary",
        "scheduled" => "badge-primary",
        "completed" => "badge-success",
        _ => "badge-ghost",
    }
}

fn tags_summary(tags: &[String]) -> String {
    if tags.is_empty() {
        "-".to_string()
    } else if tags.len() <= 3 {
        tags.join(" / ")
    } else {
        format!("{} / +{}", tags[..3].join(" / "), tags.len() - 3)
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
