use crate::components::ui::date_picker::{FlyonDatePicker, FlyonDateTimePicker};
use crate::components::ui::modal::Modal;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable};
use crate::components::ui::toast::{error, success};
use crate::server::contact_handlers::fetch_contacts;
use crate::server::order_handlers::create_order_from_request;
use crate::server::service_request_handlers::{
    create_service_request, fetch_service_requests, update_service_request,
    update_service_request_status,
};
use crate::server::user_handlers::get_user;
use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use std::collections::{HashMap, HashSet};
use shared::order::CreateOrderFromRequest;
use shared::contact::{Contact, ContactQuery};
use shared::service_request::{
    CreateServiceRequest, ServiceRequest, ServiceRequestQuery, UpdateServiceRequest,
    UpdateServiceRequestStatus,
};
use shared::user::User;
use shared::ListResult;

impl Identifiable for ServiceRequest {
    fn id(&self) -> String {
        format!("{}-{}-{}", self.uuid, self.status, self.updated_at)
    }
}

#[component]
pub fn ServiceRequestsPage() -> impl IntoView {
    let query = use_query_map();
    let refresh_count = RwSignal::new(0);
    let (status_filter, set_status_filter) = signal(String::new());
    let (contact_filter, set_contact_filter) = signal(String::new());
    let date_start = RwSignal::new(String::new());
    let date_end = RwSignal::new(String::new());

    let show_modal = RwSignal::new(false);
    let editing_request: RwSignal<Option<ServiceRequest>> = RwSignal::new(None);
    let contact_uuid = RwSignal::new(String::new());
    let service_content = RwSignal::new(String::new());
    let appointment_start_at = RwSignal::new(String::new());
    let appointment_end_at = RwSignal::new(String::new());
    let notes = RwSignal::new(String::new());
    let contact_labels: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let pending_contacts: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());
    let user_labels: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let pending_users: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());

    let contacts = Resource::new(
        move || (),
        |_| async move {
            let params = ContactQuery {
                page: 1,
                page_size: 100,
                sort: None,
                filters: None,
            };
            match call_api(fetch_contacts(params)).await {
                Ok(result) => result.items,
                Err(err) => {
                    logging::error!("Error loading contacts: {err}");
                    Vec::new()
                }
            }
        },
    );

    Effect::new(move || {
        if let Some(items) = contacts.get() {
            let mut map = HashMap::new();
            for contact in items {
                map.insert(contact.contact_uuid.clone(), contact_display_label(&contact));
            }
            contact_labels.set(map);
        }
    });

    let data = Resource::new(
        move || {
            (
                status_filter.get(),
                contact_filter.get(),
                date_start.get(),
                date_end.get(),
                refresh_count.get(),
                query.with(|value| value.clone()),
            )
        },
        |(status, contact, start, end, _, query)| async move {
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

            let params = ServiceRequestQuery {
                page,
                page_size,
                status: (!status.is_empty()).then_some(status),
                contact_uuid: (!contact.is_empty()).then_some(contact),
                start_date: (!start.is_empty()).then_some(start),
                end_date: (!end.is_empty()).then_some(end),
            };

            let result = call_api(fetch_service_requests(params)).await.unwrap_or_else(|e| {
                logging::error!("Error loading service requests: {e}");
                ListResult {
                    items: Vec::new(),
                    total: 0,
                }
            });
            (result.items, result.total)
        },
    );

    Effect::new(move || {
        let map = query.with(|value| value.clone());
        if let Some(status) = map.get("status") {
            if status_filter.get() != status {
                set_status_filter.set(status);
            }
        }
    });

    Effect::new(move || {
        if cfg!(feature = "ssr") {
            return;
        }
        let Some((items, _)) = data.get() else {
            return;
        };

        let existing = contact_labels.get();
        let mut pending = pending_contacts.get();
        let mut missing_ids: Vec<String> = Vec::new();
        for request in items {
            if request.contact_uuid.is_empty() {
                continue;
            }
            if existing.contains_key(&request.contact_uuid) || pending.contains(&request.contact_uuid) {
                continue;
            }
            pending.insert(request.contact_uuid.clone());
            missing_ids.push(request.contact_uuid);
        }
        if missing_ids.is_empty() {
            return;
        }
        pending_contacts.set(pending);

        for contact_id in missing_ids {
            let contact_labels = contact_labels;
            let pending_contacts = pending_contacts;
            spawn_local(async move {
                let label = match call_api(crate::server::contact_handlers::get_contact(
                    contact_id.clone(),
                ))
                .await
                {
                    Ok(Some(contact)) => contact_display_label(&contact),
                    _ => "未知客户".to_string(),
                };
                contact_labels.update(|map| {
                    map.insert(contact_id.clone(), label);
                });
                pending_contacts.update(|set| {
                    set.remove(&contact_id);
                });
            });
        }
    });

    Effect::new(move || {
        if cfg!(feature = "ssr") {
            return;
        }
        let Some((items, _)) = data.get() else {
            return;
        };

        let existing = user_labels.get();
        let mut pending = pending_users.get();
        let mut missing_ids: Vec<String> = Vec::new();
        for request in &items {
            let user_id = request.creator_uuid.clone();
            if user_id.is_empty() {
                continue;
            }
            if existing.contains_key(&user_id) || pending.contains(&user_id) {
                continue;
            }
            pending.insert(user_id.clone());
            missing_ids.push(user_id);
        }
        if missing_ids.is_empty() {
            return;
        }
        pending_users.set(pending);

        for user_id in missing_ids {
            let user_labels = user_labels;
            let pending_users = pending_users;
            spawn_local(async move {
                let label = match call_api(get_user(user_id.clone())).await {
                    Ok(user) => user_display_label(&user),
                    _ => "未知员工".to_string(),
                };
                user_labels.update(|map| {
                    map.insert(user_id.clone(), label);
                });
                pending_users.update(|set| {
                    set.remove(&user_id);
                });
            });
        }
    });

    let open_create_modal = move |_| {
        editing_request.set(None);
        contact_uuid.set(String::new());
        service_content.set(String::new());
        appointment_start_at.set(String::new());
        appointment_end_at.set(String::new());
        notes.set(String::new());
        show_modal.set(true);
    };

    let open_edit_modal = move |request: ServiceRequest| {
        contact_uuid.set(request.contact_uuid.clone());
        service_content.set(request.service_content.clone());
        appointment_start_at.set(to_datetime_local(request.appointment_start_at.clone()));
        appointment_end_at.set(to_datetime_local(request.appointment_end_at.clone()));
        notes.set(request.notes.clone().unwrap_or_default());
        editing_request.set(Some(request));
        show_modal.set(true);
    };

    let submit_request = move |_| {
        if contact_uuid.get().trim().is_empty() {
            error("请选择客户".to_string());
            return;
        }
        if is_end_before_start(&appointment_start_at.get(), &appointment_end_at.get()) {
            error("预约结束时间必须晚于开始时间".to_string());
            return;
        }

        let editing = editing_request.get();
        let payload = if let Some(editing) = editing {
            UpdateServiceRequest {
                uuid: editing.uuid,
                service_content: service_content.get(),
                appointment_start_at: normalize_datetime_local(&appointment_start_at.get()),
                appointment_end_at: normalize_datetime_local(&appointment_end_at.get()),
                notes: normalize_optional(&notes.get()),
            }
            .into()
        } else {
            CreateServiceRequest {
                contact_uuid: contact_uuid.get(),
                service_content: service_content.get(),
                appointment_start_at: normalize_datetime_local(&appointment_start_at.get()),
                appointment_end_at: normalize_datetime_local(&appointment_end_at.get()),
                notes: normalize_optional(&notes.get()),
            }
            .into()
        };

        spawn_local(async move {
            let result = match payload {
                RequestPayload::Create(payload) => call_api(create_service_request(payload)).await,
                RequestPayload::Update(payload) => call_api(update_service_request(payload)).await,
            };

            match result {
                Ok(_) => {
                    success("保存成功".to_string());
                    show_modal.set(false);
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => {
                    error(format!("保存失败: {}", err));
                }
            }
        });
    };

    let confirm_request = move |uuid: String| {
        spawn_local(async move {
            let payload = UpdateServiceRequestStatus {
                status: "confirmed".to_string(),
            };
            let result = call_api(update_service_request_status(uuid, payload)).await;
            match result {
                Ok(_) => {
                    success("需求已确认".to_string());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => {
                    error(format!("确认失败: {}", err));
                }
            }
        });
    };

    let create_order = move |request_id: String| {
        spawn_local(async move {
            let payload = CreateOrderFromRequest {
                request_id,
                notes: None,
            };
            let result = call_api(create_order_from_request(payload)).await;
            match result {
                Ok(_) => {
                    success("订单已生成".to_string());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => {
                    error(format!("生成订单失败: {}", err));
                }
            }
        });
    };

    view! {
        <Title text="需求/预约 - PicoCRM"/>
        <div class="space-y-4">
            <div class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between">
                <h1 class="text-2xl font-semibold">"预约/需求单"</h1>
                <button class="btn btn-primary" on:click=open_create_modal>
                    "新建需求"
                </button>
            </div>

            <div class="card bg-base-100 shadow-sm">
                <div class="card-body p-4 flex flex-col gap-3 md:flex-row md:items-end">
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"状态"</span>
                        <select
                            class="select select-bordered min-w-[160px]"
                            prop:value=move || status_filter.get()
                            on:change=move |ev| set_status_filter.set(event_target_value(&ev))
                        >
                            <option value="">"全部"</option>
                            <option value="new">"新建"</option>
                            <option value="confirmed">"已确认"</option>
                            <option value="converted">"已转订单"</option>
                            <option value="cancelled">"已取消"</option>
                        </select>
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"客户"</span>
                        <Transition fallback=move || view! {
                            <select
                                class="select select-bordered min-w-[200px]"
                                prop:value=move || contact_filter.get()
                                on:change=move |ev| set_contact_filter.set(event_target_value(&ev))
                            >
                                <option value="">"全部"</option>
                            </select>
                        }>
                            {move || {
                                let items = contacts.get().unwrap_or_default();
                                let options = items
                                    .into_iter()
                                    .map(|contact| {
                                        let label = contact_display_label(&contact);
                                        view! { <option value={contact.contact_uuid}>{label}</option> }
                                    })
                                    .collect::<Vec<_>>();
                                view! {
                                    <select
                                        class="select select-bordered min-w-[200px]"
                                        prop:value=move || contact_filter.get()
                                        on:change=move |ev| set_contact_filter.set(event_target_value(&ev))
                                    >
                                        <option value="">"全部"</option>
                                        {options}
                                    </select>
                                }
                            }}
                        </Transition>
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"预约开始"</span>
                        <FlyonDatePicker value=date_start class="input input-bordered".to_string() />
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"预约结束"</span>
                        <FlyonDatePicker value=date_end class="input input-bordered".to_string() />
                    </div>
                </div>
            </div>

            <div class="overflow-x-auto overflow-y-auto h-[calc(100vh-260px)] bg-base-100 rounded-lg shadow">
                <DaisyTable data=data>
                    <Column
                        slot:columns
                        prop="service_content".to_string()
                        label="服务内容".to_string()
                        class="font-semibold"
                    >
                        {
                            let item: Option<ServiceRequest> = use_context::<ServiceRequest>();
                            view! { <span>{item.as_ref().map(|v| v.service_content.clone()).unwrap_or_default()}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="contact_uuid".to_string() label="客户".to_string()>
                        {
                            let item: Option<ServiceRequest> = use_context::<ServiceRequest>();
                            let contact_id = item.as_ref().map(|v| v.contact_uuid.clone()).unwrap_or_default();
                            let label = contact_labels
                                .get()
                                .get(&contact_id)
                                .cloned()
                                .unwrap_or_else(|| {
                                    if pending_contacts.get().contains(&contact_id) {
                                        "加载中...".to_string()
                                    } else {
                                        "未知客户".to_string()
                                    }
                                });
                            view! { <span class="text-xs">{label}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="creator_uuid".to_string() label="创建人".to_string()>
                        {
                            let item: Option<ServiceRequest> = use_context::<ServiceRequest>();
                            let user_id = item.as_ref().map(|v| v.creator_uuid.clone()).unwrap_or_default();
                            let label = user_labels
                                .get()
                                .get(&user_id)
                                .cloned()
                                .unwrap_or_else(|| {
                                    if pending_users.get().contains(&user_id) {
                                        "加载中...".to_string()
                                    } else {
                                        "未知员工".to_string()
                                    }
                                });
                            view! { <span class="text-xs">{label}</span> }
                        }
                    </Column>
                    <Column
                        slot:columns
                        prop="appointment".to_string()
                        label="预约时间".to_string()
                    >
                        {
                            let item: Option<ServiceRequest> = use_context::<ServiceRequest>();
                            let start = item.as_ref().and_then(|v| v.appointment_start_at.clone()).unwrap_or_default();
                            let end = item.as_ref().and_then(|v| v.appointment_end_at.clone()).unwrap_or_default();
                            view! { <span class="text-xs">{format!("{} - {}", start, end)}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="status".to_string() label="状态".to_string()>
                        {
                            let item: Option<ServiceRequest> = use_context::<ServiceRequest>();
                            let status_value = item.as_ref().map(|v| v.status.clone()).unwrap_or_default();
                            let label = service_request_status_label(&status_value);
                            let badge_class = service_request_status_badge_class(&status_value);
                            view! {
                                <span class=format!("badge {}", badge_class)>{label}</span>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        prop="actions".to_string()
                        label="操作".to_string()
                        class="text-right"
                    >
                        <div class="flex justify-end gap-1">
                            {
                                let item: Option<ServiceRequest> = use_context::<ServiceRequest>();
                                let request = StoredValue::new(item);
                                let can_confirm = request
                                    .with_value(|value| {
                                        value
                                            .as_ref()
                                            .map(|value| value.status == "new")
                                            .unwrap_or(false)
                                    });
                                let can_edit = can_confirm;
                                let can_create = request
                                    .with_value(|value| {
                                        value
                                            .as_ref()
                                            .map(|value| value.status == "confirmed")
                                            .unwrap_or(false)
                                    });
                                view! {
                                    <Show when=move || can_confirm>
                                        <button class="btn btn-soft btn-primary btn-xs" on:click=move |_| {
                                            if let Some(value) = request.with_value(|value| value.clone()) {
                                                confirm_request(value.uuid.clone());
                                            }
                                        }>
                                            "确认"
                                        </button>
                                    </Show>
                                    <Show when=move || can_edit>
                                        <button class="btn btn-soft btn-warning btn-xs" on:click=move |_| {
                                            if let Some(value) = request.with_value(|value| value.clone()) {
                                                open_edit_modal(value);
                                            }
                                        }>
                                            "编辑"
                                        </button>
                                    </Show>
                                    <Show when=move || can_create>
                                        <button
                                            class="btn btn-soft btn-success btn-xs"
                                            on:click=move |_| {
                                                if let Some(value) = request.with_value(|value| value.clone()) {
                                                    create_order(value.uuid.clone());
                                                }
                                            }
                                        >
                                            "生成订单"
                                        </button>
                                    </Show>
                                }
                            }
                        </div>
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

        <Modal show=show_modal>
            <h3 class="text-lg font-semibold mb-4">"需求单"</h3>
            <div class="space-y-3">
                <label class="form-control w-full">
                    <div class="label"><span class="label-text">"客户"</span></div>
                    <Transition fallback=move || view! {
                        <select
                            class="select select-bordered w-full"
                            prop:value=move || contact_uuid.get()
                            on:change=move |ev| contact_uuid.set(event_target_value(&ev))
                            disabled=move || editing_request.with(|value| value.is_some())
                        >
                            <option value="">"请选择客户"</option>
                        </select>
                    }>
                        {move || {
                            let items = contacts.get().unwrap_or_default();
                            let options = items
                                .into_iter()
                                .map(|contact| {
                                    let label = contact_display_label(&contact);
                                    view! { <option value={contact.contact_uuid}>{label}</option> }
                                })
                                .collect::<Vec<_>>();
                            view! {
                                <select
                                    class="select select-bordered w-full"
                                    prop:value=move || contact_uuid.get()
                                    on:change=move |ev| contact_uuid.set(event_target_value(&ev))
                                    disabled=move || editing_request.with(|value| value.is_some())
                                >
                                    <option value="">"请选择客户"</option>
                                    {options}
                                </select>
                            }
                        }}
                    </Transition>
                </label>
                <label class="form-control w-full">
                    <div class="label"><span class="label-text">"服务内容"</span></div>
                    <textarea
                        class="textarea textarea-bordered w-full"
                        rows="3"
                        prop:value=move || service_content.get()
                        on:input=move |ev| service_content.set(event_target_value(&ev))
                    ></textarea>
                </label>
                <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                    <label class="form-control w-full">
                        <div class="label"><span class="label-text">"预约开始"</span></div>
                        <FlyonDateTimePicker
                            value=appointment_start_at
                            class="input input-bordered".to_string()
                        />
                    </label>
                    <label class="form-control w-full">
                        <div class="label"><span class="label-text">"预约结束"</span></div>
                        <FlyonDateTimePicker
                            value=appointment_end_at
                            class="input input-bordered".to_string()
                        />
                    </label>
                </div>
                <label class="form-control w-full">
                    <div class="label"><span class="label-text">"备注"</span></div>
                    <textarea
                        class="textarea textarea-bordered w-full"
                        rows="2"
                        prop:value=move || notes.get()
                        on:input=move |ev| notes.set(event_target_value(&ev))
                    ></textarea>
                </label>
                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_modal.set(false)>
                        "取消"
                    </button>
                    <button class="btn btn-primary" on:click=submit_request>
                        "保存"
                    </button>
                </div>
            </div>
        </Modal>
    }
}

enum RequestPayload {
    Create(CreateServiceRequest),
    Update(UpdateServiceRequest),
}

impl From<CreateServiceRequest> for RequestPayload {
    fn from(value: CreateServiceRequest) -> Self {
        RequestPayload::Create(value)
    }
}

impl From<UpdateServiceRequest> for RequestPayload {
    fn from(value: UpdateServiceRequest) -> Self {
        RequestPayload::Update(value)
    }
}

fn contact_display_label(contact: &Contact) -> String {
    let name = contact.user_name.trim();
    let company = contact.company.trim();
    let mut label = String::new();
    if !name.is_empty() {
        label.push_str(name);
    }
    if !company.is_empty() {
        if !label.is_empty() {
            label.push_str(" / ");
        }
        label.push_str(company);
    }
    if label.is_empty() {
        label = "未命名客户".to_string();
    }
    let extra = contact.phone_number.trim();
    if !extra.is_empty() {
        format!("{} ({})", label, extra)
    } else {
        let email = contact.email.trim();
        if !email.is_empty() {
            format!("{} ({})", label, email)
        } else {
            label
        }
    }
}

fn user_display_label(user: &User) -> String {
    if let Some(phone) = user.phone_number.clone().filter(|value| !value.is_empty()) {
        format!("{} ({})", user.user_name, phone)
    } else if let Some(email) = user.email.clone().filter(|value| !value.is_empty()) {
        format!("{} ({})", user.user_name, email)
    } else {
        user.user_name.clone()
    }
}

fn normalize_optional(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_string())
    }
}

fn normalize_datetime_local(value: &str) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return None;
    }
    let normalized = trimmed.replace('T', " ");
    if normalized.len() == 16 {
        Some(format!("{}:00", normalized))
    } else {
        Some(normalized)
    }
}

fn to_datetime_local(value: Option<String>) -> String {
    let Some(value) = value else { return String::new() };
    if value.trim().is_empty() {
        return String::new();
    }
    let replaced = value.replace('T', " ");
    if replaced.len() >= 16 {
        replaced[..16].to_string()
    } else {
        replaced
    }
}

fn is_end_before_start(start: &str, end: &str) -> bool {
    match (normalize_datetime_local(start), normalize_datetime_local(end)) {
        (Some(start), Some(end)) => end < start,
        _ => false,
    }
}

fn service_request_status_label(status: &str) -> &'static str {
    match status {
        "new" => "新建",
        "confirmed" => "已确认",
        "converted" => "已转订单",
        "cancelled" => "已取消",
        _ => "未知",
    }
}

fn service_request_status_badge_class(status: &str) -> &'static str {
    match status {
        "new" => "badge-warning",
        "confirmed" => "badge-info",
        "converted" => "badge-success",
        "cancelled" => "badge-error",
        _ => "badge-info",
    }
}
