use crate::components::ui::date_picker::{FlyonDatePicker, FlyonDateTimePicker};
use crate::components::ui::modal::Modal;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable};
use crate::components::ui::toast::{error, success};
use crate::server::order_handlers::create_order_from_request;
use crate::server::service_request_handlers::{
    create_service_request, fetch_service_requests, update_service_request,
    update_service_request_status,
};
use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use shared::order::CreateOrderFromRequest;
use shared::service_request::{
    CreateServiceRequest, ServiceRequest, ServiceRequestQuery, UpdateServiceRequest,
    UpdateServiceRequestStatus,
};
use shared::ListResult;

impl Identifiable for ServiceRequest {
    fn id(&self) -> String {
        self.uuid.clone()
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

    let data = Resource::new(
        move || {
            (
                status_filter.get(),
                contact_filter.get(),
                date_start.get(),
                date_end.get(),
                *refresh_count.read(),
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

    let change_status = move |uuid: String, status: String| {
        spawn_local(async move {
            let payload = UpdateServiceRequestStatus { status };
            let result = call_api(update_service_request_status(uuid, payload)).await;
            match result {
                Ok(_) => {
                    success("状态已更新".to_string());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => {
                    error(format!("更新失败: {}", err));
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
                        <span class="text-xs text-base-content/60">"客户ID"</span>
                        <input
                            class="input input-bordered"
                            placeholder="输入客户UUID"
                            on:input=move |ev| set_contact_filter.set(event_target_value(&ev))
                        />
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

            <div class="overflow-x-auto bg-base-100 rounded-lg shadow">
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
                    <Column slot:columns prop="contact_uuid".to_string() label="客户ID".to_string()>
                        {
                            let item: Option<ServiceRequest> = use_context::<ServiceRequest>();
                            view! { <span class="text-xs">{item.as_ref().map(|v| v.contact_uuid.clone()).unwrap_or_default()}</span> }
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
                            let uuid = item.as_ref().map(|v| v.uuid.clone()).unwrap_or_default();
                            let status_value = item.as_ref().map(|v| v.status.clone()).unwrap_or_default();
                            view! {
                                <select
                                    class="select select-bordered select-xs"
                                    prop:value=move || status_value.clone()
                                    on:change=move |ev| change_status(uuid.clone(), event_target_value(&ev))
                                >
                                    <option value="new">"新建"</option>
                                    <option value="confirmed">"已确认"</option>
                                    <option value="converted">"已转订单"</option>
                                    <option value="cancelled">"已取消"</option>
                                </select>
                            }
                        }
                    </Column>
                    <Column
                        slot:columns
                        prop="actions".to_string()
                        label="操作".to_string()
                        class="text-right"
                    >
                        <div class="flex items-center justify-end gap-2">
                            {
                                let item: Option<ServiceRequest> = use_context::<ServiceRequest>();
                                let request_id = item.as_ref().map(|v| v.uuid.clone()).unwrap_or_default();
                                let edit_item = item.clone();
                                view! {
                                    <button class="btn btn-ghost btn-xs" on:click=move |_| {
                                        if let Some(value) = edit_item.clone() {
                                            open_edit_modal(value);
                                        }
                                    }>
                                        "编辑"
                                    </button>
                                    <button class="btn btn-outline btn-xs" on:click=move |_| {
                                        if !request_id.is_empty() {
                                            create_order(request_id.clone());
                                        }
                                    }>
                                        "生成订单"
                                    </button>
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
                    <div class="label"><span class="label-text">"客户UUID"</span></div>
                    <input
                        class="input input-bordered w-full"
                        prop:value=move || contact_uuid.get()
                        on:input=move |ev| contact_uuid.set(event_target_value(&ev))
                        disabled=move || editing_request.with(|value| value.is_some())
                    />
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
