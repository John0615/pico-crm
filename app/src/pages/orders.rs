use crate::components::ui::date_picker::{FlyonDatePicker, FlyonDateTimePicker};
use crate::components::ui::modal::{Modal, DETAIL_MODAL_BOX_CLASS};
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable};
use crate::components::ui::toast::{error, success};
use crate::server::after_sales_handlers::{
    create_after_sales_case, fetch_after_sales_cases, update_after_sales_refund,
};
use crate::server::after_sales_record_handlers::{
    create_after_sales_case_record, fetch_after_sales_case_records,
};
use crate::server::after_sales_rework_handlers::{
    create_after_sales_rework, fetch_after_sales_reworks,
};
use crate::server::contact_handlers::{
    create_contact_follow_record, fetch_contact_follow_records, fetch_contacts, get_contact,
};
use crate::server::order_handlers::{
    cancel_order, fetch_orders, get_order_change_logs, update_order, update_order_settlement,
    update_order_status,
};
use crate::server::user_handlers::fetch_users;
use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use shared::after_sales::{
    AfterSalesCase, AfterSalesCaseRecord, AfterSalesRework, CreateAfterSalesCaseRecordRequest,
    CreateAfterSalesCaseRequest, CreateAfterSalesReworkRequest, UpdateAfterSalesRefundRequest,
};
use shared::contact::{
    Contact, ContactFollowRecord, ContactQuery, CreateContactFollowRecordRequest,
};
use shared::order::{
    CancelOrderRequest, Order, OrderChangeLogDto, OrderQuery, UpdateOrderRequest,
    UpdateOrderSettlement, UpdateOrderStatus,
};
use shared::user::UserListQuery;
use shared::ListResult;
use std::collections::{HashMap, HashSet};

impl Identifiable for Order {
    fn id(&self) -> String {
        format!("{}-{}-{}", self.uuid, self.status, self.updated_at)
    }
}

#[component]
pub fn OrdersPage() -> impl IntoView {
    let query = use_query_map();
    let refresh_count = RwSignal::new(0);
    let (status_filter, set_status_filter) = signal(String::new());
    let (settlement_filter, set_settlement_filter) = signal(String::new());
    let (contact_filter, set_contact_filter) = signal(String::new());
    let date_start = RwSignal::new(String::new());
    let date_end = RwSignal::new(String::new());

    let show_detail_modal = RwSignal::new(false);
    let show_edit_modal = RwSignal::new(false);
    let show_cancel_modal = RwSignal::new(false);
    let show_after_sales_modal = RwSignal::new(false);
    let detail_order: RwSignal<Option<Order>> = RwSignal::new(None);
    let active_after_sales_case: RwSignal<Option<AfterSalesCase>> = RwSignal::new(None);
    let editing_order_uuid = RwSignal::new(String::new());
    let cancelling_order_uuid = RwSignal::new(String::new());

    let form_customer_uuid = RwSignal::new(String::new());
    let form_amount_cents = RwSignal::new(String::new());
    let form_notes = RwSignal::new(String::new());
    let cancel_reason = RwSignal::new(String::new());
    let settlement_status = RwSignal::new(String::new());
    let settlement_note = RwSignal::new(String::new());
    let paid_amount_cents = RwSignal::new(String::new());
    let payment_method = RwSignal::new(String::new());
    let paid_at = RwSignal::new(String::new());
    let follow_up_content = RwSignal::new(String::new());
    let next_follow_up_at = RwSignal::new(String::new());
    let creating_follow_record = RwSignal::new(false);
    let after_sales_case_type = RwSignal::new(String::new());
    let after_sales_description = RwSignal::new(String::new());
    let creating_after_sales_case = RwSignal::new(false);
    let after_sales_record_status = RwSignal::new("processing".to_string());
    let after_sales_record_content = RwSignal::new(String::new());
    let creating_after_sales_record = RwSignal::new(false);
    let refund_amount_cents = RwSignal::new(String::new());
    let refund_reason = RwSignal::new(String::new());
    let saving_after_sales_refund = RwSignal::new(false);
    let rework_assigned_user_uuid = RwSignal::new(String::new());
    let rework_start_at = RwSignal::new(String::new());
    let rework_end_at = RwSignal::new(String::new());
    let rework_note = RwSignal::new(String::new());
    let creating_rework = RwSignal::new(false);

    let contact_labels: RwSignal<HashMap<String, String>> = RwSignal::new(HashMap::new());
    let pending_contacts: RwSignal<HashSet<String>> = RwSignal::new(HashSet::new());

    let data = Resource::new(
        move || {
            (
                status_filter.get(),
                settlement_filter.get(),
                contact_filter.get(),
                date_start.get(),
                date_end.get(),
                refresh_count.get(),
                query.with(|value| value.clone()),
            )
        },
        |(status, settlement, contact, start, end, _, query)| async move {
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

            let params = OrderQuery {
                page,
                page_size,
                status: (!status.is_empty()).then_some(status),
                statuses: None,
                settlement_status: (!settlement.is_empty()).then_some(settlement),
                customer_uuid: (!contact.is_empty()).then_some(contact),
                start_date: (!start.is_empty()).then_some(start),
                end_date: (!end.is_empty()).then_some(end),
            };

            let result = call_api(fetch_orders(params)).await.unwrap_or_else(|e| {
                logging::error!("Error loading orders: {e}");
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
        if let Some(start) = map.get("start_date") {
            if date_start.get() != start {
                date_start.set(start);
            }
        }
        if let Some(end) = map.get("end_date") {
            if date_end.get() != end {
                date_end.set(end);
            }
        }
    });

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

    let users = Resource::new(
        move || (),
        |_| async move {
            let params = UserListQuery {
                page: 1,
                page_size: 200,
                name: None,
                role: None,
                status: None,
                employment_status: None,
                skill: None,
                dispatchable_only: Some(true),
            };

            match call_api(fetch_users(params)).await {
                Ok(result) => result.items,
                Err(err) => {
                    logging::error!("Error loading users: {err}");
                    Vec::new()
                }
            }
        },
    );

    Effect::new(move || {
        if let Some(items) = contacts.get() {
            let mut map = HashMap::new();
            for contact in items {
                map.insert(
                    contact.contact_uuid.clone(),
                    contact_display_label(&contact),
                );
            }
            contact_labels.set(map);
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
        for order in &items {
            let Some(contact_id) = order.customer_uuid.clone() else {
                continue;
            };
            if contact_id.is_empty() {
                continue;
            }
            if existing.contains_key(&contact_id) || pending.contains(&contact_id) {
                continue;
            }
            pending.insert(contact_id.clone());
            missing_ids.push(contact_id);
        }
        if missing_ids.is_empty() {
            return;
        }
        pending_contacts.set(pending);

        for contact_id in missing_ids {
            let contact_labels = contact_labels;
            let pending_contacts = pending_contacts;
            spawn_local(async move {
                let label = match call_api(get_contact(contact_id.clone())).await {
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

    let order_logs = Resource::new(
        move || detail_order.get().map(|order| order.uuid.clone()),
        |uuid| async move {
            match uuid {
                Some(uuid) => call_api(get_order_change_logs(uuid))
                    .await
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        },
    );

    let follow_records = Resource::new(
        move || {
            (
                detail_order
                    .get()
                    .and_then(|order| order.customer_uuid.clone()),
                refresh_count.get(),
            )
        },
        |(contact_uuid, _)| async move {
            match contact_uuid {
                Some(contact_uuid) if !contact_uuid.trim().is_empty() => {
                    call_api(fetch_contact_follow_records(contact_uuid))
                        .await
                        .unwrap_or_default()
                }
                _ => Vec::new(),
            }
        },
    );

    let after_sales_cases = Resource::new(
        move || {
            (
                detail_order.get().map(|order| order.uuid),
                refresh_count.get(),
            )
        },
        |(order_uuid, _)| async move {
            match order_uuid {
                Some(order_uuid) => call_api(fetch_after_sales_cases(order_uuid))
                    .await
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        },
    );

    let after_sales_case_records = Resource::new(
        move || {
            (
                active_after_sales_case.get().map(|item| item.uuid),
                refresh_count.get(),
            )
        },
        |(case_uuid, _)| async move {
            match case_uuid {
                Some(case_uuid) => call_api(fetch_after_sales_case_records(case_uuid))
                    .await
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        },
    );

    let after_sales_reworks = Resource::new(
        move || {
            (
                active_after_sales_case.get().map(|item| item.uuid),
                refresh_count.get(),
            )
        },
        |(case_uuid, _)| async move {
            match case_uuid {
                Some(case_uuid) => call_api(fetch_after_sales_reworks(case_uuid))
                    .await
                    .unwrap_or_default(),
                None => Vec::new(),
            }
        },
    );

    let open_edit = move |order: Order| {
        editing_order_uuid.set(order.uuid.clone());
        form_customer_uuid.set(order.customer_uuid.unwrap_or_default());
        form_amount_cents.set(order.amount_cents.to_string());
        form_notes.set(order.notes.unwrap_or_default());
        show_edit_modal.set(true);
    };

    let open_cancel = move |order: Order| {
        cancelling_order_uuid.set(order.uuid.clone());
        cancel_reason.set(order.cancellation_reason.unwrap_or_default());
        show_cancel_modal.set(true);
    };

    let open_detail = move |order: Order| {
        settlement_status.set(order.settlement_status.clone());
        settlement_note.set(order.settlement_note.clone().unwrap_or_default());
        paid_amount_cents.set(
            order
                .paid_amount_cents
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        payment_method.set(order.payment_method.clone().unwrap_or_default());
        paid_at.set(order.paid_at.clone().unwrap_or_default());
        follow_up_content.set(String::new());
        next_follow_up_at.set(String::new());
        after_sales_case_type.set(String::new());
        after_sales_description.set(String::new());
        detail_order.set(Some(order));
        show_detail_modal.set(true);
    };

    let open_after_sales_case = move |item: AfterSalesCase| {
        after_sales_record_status.set(item.status.clone());
        after_sales_record_content.set(String::new());
        refund_amount_cents.set(
            item.refund_amount_cents
                .map(|value| value.to_string())
                .unwrap_or_default(),
        );
        refund_reason.set(item.refund_reason.clone().unwrap_or_default());
        rework_assigned_user_uuid.set(String::new());
        rework_start_at.set(String::new());
        rework_end_at.set(String::new());
        rework_note.set(String::new());
        active_after_sales_case.set(Some(item));
        show_after_sales_modal.set(true);
    };

    let save_order_details = move |_| {
        let uuid = editing_order_uuid.get();
        if uuid.trim().is_empty() {
            error("缺少订单ID".to_string());
            return;
        }
        let amount_cents = match form_amount_cents.get().trim().parse::<i64>() {
            Ok(value) if value >= 0 => value,
            _ => {
                error("金额必须是非负整数".to_string());
                return;
            }
        };

        let payload = UpdateOrderRequest {
            customer_uuid: form_customer_uuid.get().trim().to_string(),
            amount_cents,
            notes: normalize_optional(&form_notes.get()),
        };

        spawn_local(async move {
            match call_api(update_order(uuid, payload)).await {
                Ok(_) => {
                    success("订单已更新".to_string());
                    show_edit_modal.set(false);
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("更新失败: {}", err)),
            }
        });
    };

    let submit_cancel_order = move |_| {
        let uuid = cancelling_order_uuid.get();
        if uuid.trim().is_empty() {
            error("缺少订单ID".to_string());
            return;
        }
        let reason = cancel_reason.get();
        if reason.trim().is_empty() {
            error("取消订单必须填写原因".to_string());
            return;
        }

        spawn_local(async move {
            match call_api(cancel_order(
                uuid,
                CancelOrderRequest {
                    reason: reason.trim().to_string(),
                },
            ))
            .await
            {
                Ok(_) => {
                    success("订单已取消".to_string());
                    show_cancel_modal.set(false);
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("取消失败: {}", err)),
            }
        });
    };

    let confirm_order = move |uuid: String| {
        spawn_local(async move {
            let payload = UpdateOrderStatus {
                status: "confirmed".to_string(),
            };
            let result = call_api(update_order_status(uuid, payload)).await;
            match result {
                Ok(_) => {
                    success("订单已确认".to_string());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("确认失败: {}", err)),
            }
        });
    };

    let save_settlement = move |_| {
        let Some(order) = detail_order.get() else {
            return;
        };

        let paid_amount_value = paid_amount_cents.get();
        let parsed_paid_amount = if paid_amount_value.trim().is_empty() {
            None
        } else {
            match paid_amount_value.trim().parse::<i64>() {
                Ok(value) if value >= 0 => Some(value),
                _ => {
                    error("实收金额必须是非负整数".to_string());
                    return;
                }
            }
        };

        let payload = UpdateOrderSettlement {
            settlement_status: settlement_status.get(),
            settlement_note: normalize_optional(&settlement_note.get()),
            paid_amount_cents: parsed_paid_amount,
            payment_method: normalize_optional(&payment_method.get()),
            paid_at: normalize_optional(&paid_at.get()),
        };

        spawn_local(async move {
            match call_api(update_order_settlement(order.uuid.clone(), payload)).await {
                Ok(updated) => {
                    success("结算状态已更新".to_string());
                    detail_order.set(Some(updated));
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("更新结算失败: {}", err)),
            }
        });
    };

    let save_follow_record = move |_| {
        let Some(order) = detail_order.get() else {
            return;
        };
        if order.status != "completed" {
            error("仅已完成订单支持记录回访结果".to_string());
            return;
        }
        let Some(contact_uuid) = order.customer_uuid.clone() else {
            error("订单缺少客户信息，无法记录回访".to_string());
            return;
        };
        if creating_follow_record.get() {
            return;
        }
        let content = follow_up_content.get();
        if content.trim().is_empty() {
            error("请填写回访结果".to_string());
            return;
        }

        let payload = CreateContactFollowRecordRequest {
            contact_uuid,
            content,
            next_follow_up_at: normalize_optional(&next_follow_up_at.get()),
        };

        creating_follow_record.set(true);
        spawn_local(async move {
            match call_api(create_contact_follow_record(payload)).await {
                Ok(_) => {
                    success("回访结果已保存".to_string());
                    follow_up_content.set(String::new());
                    next_follow_up_at.set(String::new());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("保存回访失败: {}", err)),
            }
            creating_follow_record.set(false);
        });
    };

    let save_after_sales_case = move |_| {
        let Some(order) = detail_order.get() else {
            return;
        };
        if creating_after_sales_case.get() {
            return;
        }
        if after_sales_case_type.get().trim().is_empty() {
            error("请选择售后类型".to_string());
            return;
        }
        if after_sales_description.get().trim().is_empty() {
            error("请填写问题描述".to_string());
            return;
        }

        let payload = CreateAfterSalesCaseRequest {
            case_type: after_sales_case_type.get(),
            description: after_sales_description.get(),
        };

        creating_after_sales_case.set(true);
        spawn_local(async move {
            match call_api(create_after_sales_case(order.uuid, payload)).await {
                Ok(_) => {
                    success("售后工单已创建".to_string());
                    after_sales_case_type.set(String::new());
                    after_sales_description.set(String::new());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("创建售后工单失败: {}", err)),
            }
            creating_after_sales_case.set(false);
        });
    };

    let save_after_sales_record = move |_| {
        let Some(item) = active_after_sales_case.get() else {
            return;
        };
        if creating_after_sales_record.get() {
            return;
        }
        if after_sales_record_content.get().trim().is_empty() {
            error("请填写处理过程".to_string());
            return;
        }

        let payload = CreateAfterSalesCaseRecordRequest {
            content: after_sales_record_content.get(),
            status: Some(after_sales_record_status.get()),
        };

        creating_after_sales_record.set(true);
        spawn_local(async move {
            match call_api(create_after_sales_case_record(item.uuid.clone(), payload)).await {
                Ok(_) => {
                    success("处理记录已保存".to_string());
                    after_sales_record_content.set(String::new());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("保存处理记录失败: {}", err)),
            }
            creating_after_sales_record.set(false);
        });
    };

    let save_after_sales_refund = move |_| {
        let Some(item) = active_after_sales_case.get() else {
            return;
        };
        if saving_after_sales_refund.get() {
            return;
        }

        let refund_amount = if refund_amount_cents.get().trim().is_empty() {
            None
        } else {
            match refund_amount_cents.get().trim().parse::<i64>() {
                Ok(value) if value >= 0 => Some(value),
                _ => {
                    error("退款金额必须是非负整数".to_string());
                    return;
                }
            }
        };

        let payload = UpdateAfterSalesRefundRequest {
            refund_amount_cents: refund_amount,
            refund_reason: normalize_optional(&refund_reason.get()),
        };

        saving_after_sales_refund.set(true);
        spawn_local(async move {
            match call_api(update_after_sales_refund(item.uuid.clone(), payload)).await {
                Ok(updated) => {
                    success("退款信息已保存".to_string());
                    active_after_sales_case.set(Some(updated));
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("保存退款信息失败: {}", err)),
            }
            saving_after_sales_refund.set(false);
        });
    };

    let save_after_sales_rework = move |_| {
        let Some(item) = active_after_sales_case.get() else {
            return;
        };
        if creating_rework.get() {
            return;
        }
        if rework_assigned_user_uuid.get().trim().is_empty() {
            error("请选择返工人员".to_string());
            return;
        }
        if rework_start_at.get().trim().is_empty() || rework_end_at.get().trim().is_empty() {
            error("请填写返工时间".to_string());
            return;
        }

        let payload = CreateAfterSalesReworkRequest {
            assigned_user_uuid: rework_assigned_user_uuid.get(),
            scheduled_start_at: rework_start_at.get(),
            scheduled_end_at: rework_end_at.get(),
            note: normalize_optional(&rework_note.get()),
        };

        creating_rework.set(true);
        spawn_local(async move {
            match call_api(create_after_sales_rework(item.uuid.clone(), payload)).await {
                Ok(_) => {
                    success("返工安排已保存".to_string());
                    rework_assigned_user_uuid.set(String::new());
                    rework_start_at.set(String::new());
                    rework_end_at.set(String::new());
                    rework_note.set(String::new());
                    refresh_count.update(|value| *value += 1);
                }
                Err(err) => error(format!("保存返工安排失败: {}", err)),
            }
            creating_rework.set(false);
        });
    };

    view! {
        <Title text="订单管理 - PicoCRM"/>
        <div class="space-y-4">
            <div class="flex flex-col items-start gap-3 text-left md:flex-row md:items-center md:justify-between">
                <div class="text-left">
                    <h1 class="text-2xl font-semibold">"订单管理"</h1>
                    <p class="mt-1 text-sm text-base-content/60">"订单只能从已确认的需求单转化生成，这里仅支持查看、编辑、取消、收款更新和变更记录"</p>
                </div>
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
                            <option value="pending">"待确认"</option>
                            <option value="confirmed">"已确认"</option>
                            <option value="dispatching">"派工中"</option>
                            <option value="in_service">"服务中"</option>
                            <option value="completed">"已完成"</option>
                            <option value="cancelled">"已取消"</option>
                        </select>
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"结算"</span>
                        <select
                            class="select select-bordered min-w-[160px]"
                            prop:value=move || settlement_filter.get()
                            on:change=move |ev| set_settlement_filter.set(event_target_value(&ev))
                        >
                            <option value="">"全部"</option>
                            <option value="unsettled">"未结算"</option>
                            <option value="settled">"已结算"</option>
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
                        <span class="text-xs text-base-content/60">"服务开始"</span>
                        <FlyonDatePicker value=date_start class="input input-bordered".to_string() />
                    </div>
                    <div class="flex flex-col gap-1">
                        <span class="text-xs text-base-content/60">"服务结束"</span>
                        <FlyonDatePicker value=date_end class="input input-bordered".to_string() />
                    </div>
                </div>
            </div>

            <div class="overflow-x-auto overflow-y-auto h-[calc(100vh-260px)] bg-base-100 rounded-lg shadow">
                <DaisyTable data=data>
                    <Column
                        slot:columns
                        freeze=true
                        prop="uuid".to_string()
                        label="订单ID".to_string()
                        class="font-semibold"
                    >
                        {
                            let item: Option<Order> = use_context::<Order>();
                            view! { <span class="text-xs">{item.as_ref().map(|v| v.uuid.clone()).unwrap_or_default()}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="customer_uuid".to_string() label="客户".to_string()>
                        {
                            let item: Option<Order> = use_context::<Order>();
                            let contact_id = item.as_ref().and_then(|v| v.customer_uuid.clone());
                            let fallback = item.as_ref().and_then(|v| v.customer_name.clone());
                            let label = contact_id
                                .as_ref()
                                .map(|id| {
                                    contact_labels
                                        .get()
                                        .get(id)
                                        .cloned()
                                        .or(fallback.clone())
                                        .unwrap_or_else(|| {
                                            if pending_contacts.get().contains(id) {
                                                "加载中...".to_string()
                                            } else {
                                                "未知客户".to_string()
                                            }
                                        })
                                })
                                .unwrap_or_else(|| "-".to_string());
                            view! { <span class="text-xs">{label}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="service_catalog".to_string() label="服务项目".to_string()>
                        {
                            let item: Option<Order> = use_context::<Order>();
                            let label = item
                                .as_ref()
                                .and_then(|v| v.service_catalog_name.clone())
                                .unwrap_or_else(|| "-".to_string());
                            view! { <span class="text-xs">{label}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="status".to_string() label="状态".to_string()>
                        {
                            let item: Option<Order> = use_context::<Order>();
                            let status_value = item.as_ref().map(|v| v.status.clone()).unwrap_or_default();
                            let label = order_status_label(&status_value);
                            let badge_class = order_status_badge_class(&status_value);
                            view! { <span class=format!("badge {}", badge_class)>{label}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="settlement".to_string() label="结算".to_string()>
                        {
                            let item: Option<Order> = use_context::<Order>();
                            let settlement_value = item.as_ref().map(|v| v.settlement_status.clone()).unwrap_or_default();
                            let label = settlement_status_label(&settlement_value);
                            let badge_class = settlement_status_badge_class(&settlement_value);
                            view! { <span class=format!("badge {}", badge_class)>{label}</span> }
                        }
                    </Column>
                    <Column slot:columns prop="completed_at".to_string() label="完成时间".to_string()>
                        {
                            let item: Option<Order> = use_context::<Order>();
                            view! { <span class="text-xs">{item.as_ref().and_then(|v| v.completed_at.clone()).unwrap_or_else(|| "-".to_string())}</span> }
                        }
                    </Column>
                    <Column
                        slot:columns
                        freeze=true
                        prop="actions".to_string()
                        label="操作".to_string()
                        class="text-right"
                    >
                        <div class="flex justify-end gap-1">
                            {
                                let item: Option<Order> = use_context::<Order>();
                                let order = StoredValue::new(item);
                                let can_confirm = order
                                    .with_value(|value| value.as_ref().map(|value| value.status == "pending").unwrap_or(false));
                                let can_edit = order
                                    .with_value(|value| value.as_ref().map(|value| value.status != "completed" && value.status != "cancelled").unwrap_or(false));
                                let can_cancel = order
                                    .with_value(|value| value.as_ref().map(|value| value.status != "completed" && value.status != "cancelled").unwrap_or(false));
                                view! {
                                    <Show when=move || can_confirm>
                                        <button class="btn btn-soft btn-primary btn-xs" on:click=move |_| {
                                            if let Some(value) = order.with_value(|value| value.clone()) {
                                                confirm_order(value.uuid.clone());
                                            }
                                        }>
                                            "确认"
                                        </button>
                                    </Show>
                                    <Show when=move || can_edit>
                                        <button class="btn btn-soft btn-warning btn-xs" on:click=move |_| {
                                            if let Some(value) = order.with_value(|value| value.clone()) {
                                                open_edit(value);
                                            }
                                        }>
                                            "编辑"
                                        </button>
                                    </Show>
                                    <Show when=move || can_cancel>
                                        <button class="btn btn-soft btn-error btn-xs" on:click=move |_| {
                                            if let Some(value) = order.with_value(|value| value.clone()) {
                                                open_cancel(value);
                                            }
                                        }>
                                            "取消"
                                        </button>
                                    </Show>
                                    <button class="btn btn-ghost btn-xs" on:click=move |_| {
                                        if let Some(value) = order.with_value(|value| value.clone()) {
                                            open_detail(value);
                                        }
                                    }>
                                        "详情"
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

        <Modal show=show_edit_modal box_class=DETAIL_MODAL_BOX_CLASS>
            <div class="space-y-4">
                <h3 class="text-lg font-semibold">"编辑订单"</h3>
                <OrderForm
                    contacts=contacts
                    customer_uuid=form_customer_uuid
                    amount_cents=form_amount_cents
                    notes=form_notes
                />
                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_edit_modal.set(false)>"取消"</button>
                    <button class="btn btn-primary" on:click=save_order_details>"保存修改"</button>
                </div>
            </div>
        </Modal>

        <Modal show=show_cancel_modal>
            <div class="space-y-4">
                <h3 class="text-lg font-semibold">"取消订单"</h3>
                <label class="form-control">
                    <span class="label-text text-sm">"取消原因"</span>
                    <textarea
                        class="textarea textarea-bordered min-h-32"
                        prop:value=move || cancel_reason.get()
                        on:input=move |ev| cancel_reason.set(event_target_value(&ev))
                        placeholder="必须填写取消原因"
                    />
                </label>
                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_cancel_modal.set(false)>"返回"</button>
                    <button class="btn btn-error" on:click=submit_cancel_order>"确认取消"</button>
                </div>
            </div>
        </Modal>

        <Modal show=show_detail_modal box_class=DETAIL_MODAL_BOX_CLASS>
            <div class="space-y-4">
                <h3 class="text-lg font-semibold">"订单详情"</h3>
                {move || {
                    if let Some(order) = detail_order.get() {
                        let customer_name = order
                            .customer_name
                            .clone()
                            .filter(|value| !value.trim().is_empty())
                            .unwrap_or_else(|| "未知客户".to_string());
                        view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                {detail_item("订单ID", order.uuid.clone())}
                                {detail_item("需求单ID", display_optional(order.request_id.clone()))}
                                {detail_item("服务项目", display_optional(order.service_catalog_name.clone()))}
                                {detail_item("客户", customer_name)}
                                {detail_item("客户UUID", display_optional(order.customer_uuid.clone()))}
                                {detail_item("状态", order_status_label(&order.status).to_string())}
                                {detail_item("取消原因", display_optional(order.cancellation_reason.clone()))}
                                {detail_item("完成时间", display_optional(order.completed_at.clone()))}
                                {detail_item("结算状态", settlement_status_label(&order.settlement_status).to_string())}
                                {detail_item("订单金额(分)", order.amount_cents.to_string())}
                                {detail_item("实收金额(分)", display_optional_i64(order.paid_amount_cents))}
                                {detail_item("支付方式", payment_method_label(order.payment_method.as_deref()).to_string())}
                                {detail_item("收款时间", display_optional(order.paid_at.clone()))}
                                {detail_item("服务开始", display_optional(order.scheduled_start_at.clone()))}
                                {detail_item("服务结束", display_optional(order.scheduled_end_at.clone()))}
                                {detail_item("派工备注", display_optional(order.dispatch_note.clone()))}
                                {detail_item("备注", display_optional(order.notes.clone()))}
                                {detail_item("结算备注", display_optional(order.settlement_note.clone()))}
                                {detail_item("创建时间", order.inserted_at.clone())}
                                {detail_item("更新时间", order.updated_at.clone())}
                            </div>
                        }.into_any()
                    } else {
                        view! { <div class="text-sm text-base-content/60">"暂无详情"</div> }.into_any()
                    }
                }}

                <div class="rounded-box border border-base-200 p-4 space-y-3">
                    <div class="font-medium">"收款更新"</div>
                    <div class="grid gap-3 md:grid-cols-2">
                        <select
                            class="select select-bordered"
                            prop:value=move || settlement_status.get()
                            on:change=move |ev| settlement_status.set(event_target_value(&ev))
                        >
                            <option value="unsettled">"未结算"</option>
                            <option value="settled">"已结算"</option>
                        </select>
                        <select
                            class="select select-bordered"
                            prop:value=move || payment_method.get()
                            on:change=move |ev| payment_method.set(event_target_value(&ev))
                        >
                            <option value="">"请选择支付方式"</option>
                            <option value="cash">"现金"</option>
                            <option value="wechat">"微信"</option>
                            <option value="alipay">"支付宝"</option>
                            <option value="bank_transfer">"银行转账"</option>
                            <option value="other">"其他"</option>
                        </select>
                        <input
                            class="input input-bordered"
                            type="number"
                            min="0"
                            prop:value=move || paid_amount_cents.get()
                            on:input=move |ev| paid_amount_cents.set(event_target_value(&ev))
                            placeholder="填写实收金额(分)"
                        />
                        <input
                            class="input input-bordered"
                            prop:value=move || paid_at.get()
                            on:input=move |ev| paid_at.set(event_target_value(&ev))
                            placeholder="收款时间，例如 2026-04-21T10:20:00Z"
                        />
                        <input
                            class="input input-bordered md:col-span-2"
                            prop:value=move || settlement_note.get()
                            on:input=move |ev| settlement_note.set(event_target_value(&ev))
                            placeholder="填写结算备注"
                        />
                    </div>
                    <div class="flex justify-end">
                        <button class="btn btn-sm btn-primary" on:click=save_settlement>
                            "保存结算状态"
                        </button>
                    </div>
                </div>

                <div class="rounded-box border border-base-200 p-4 space-y-3">
                    <div class="font-medium">"变更记录"</div>
                    <Transition fallback=move || view! { <div class="text-sm text-base-content/60">"加载中..."</div> }>
                        {move || {
                            order_logs.get().map(|logs| {
                                if logs.is_empty() {
                                    view! { <div class="text-sm text-base-content/60">"暂无变更记录"</div> }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-3">
                                            <For
                                            each=move || logs.clone().into_iter().enumerate()
                                            key=|(idx, item)| format!("{}-{}", idx, item.uuid)
                                            children=move |(_, item)| {
                                                let diff_items = order_log_diff_items(&item);
                                                let has_diff = !diff_items.is_empty();
                                                let diff_rows = diff_items
                                                    .iter()
                                                    .cloned()
                                                    .map(|field| {
                                                        view! {
                                                            <div class="grid gap-2 rounded bg-base-100 p-2 text-xs md:grid-cols-[120px,1fr,1fr]">
                                                                <div class="font-medium text-base-content/70">{field.0}</div>
                                                                <div>
                                                                    <div class="text-[11px] text-base-content/50">"变更前"</div>
                                                                    <div class="whitespace-pre-wrap break-all">{field.1}</div>
                                                                </div>
                                                                <div>
                                                                    <div class="text-[11px] text-base-content/50">"变更后"</div>
                                                                    <div class="whitespace-pre-wrap break-all">{field.2}</div>
                                                                </div>
                                                            </div>
                                                        }
                                                    })
                                                    .collect_view();
                                                view! {
                                                    <div class="rounded-box bg-base-200/50 p-3 space-y-2">
                                                        <div class="flex items-center justify-between gap-2">
                                                            <span class="font-medium text-sm">{order_log_action_label(&item.action)}</span>
                                                            <span class="text-xs text-base-content/60">{item.created_at.clone()}</span>
                                                        </div>
                                                        <div class="text-xs text-base-content/60">
                                                            {format!(
                                                                "操作人UUID: {}",
                                                                item.operator_uuid.clone().unwrap_or_else(|| "-".to_string())
                                                            )}
                                                        </div>
                                                        {if has_diff {
                                                            view! {
                                                                <div class="space-y-2">
                                                                    {diff_rows}
                                                                </div>
                                                            }.into_any()
                                                        } else {
                                                            view! {
                                                                <div class="text-xs text-base-content/60">
                                                                    "该操作没有记录到具体字段差异"
                                                                </div>
                                                            }.into_any()
                                                        }}
                                                    </div>
                                                }
                                            }
                                            />
                                        </div>
                                    }.into_any()
                                }
                            })
                        }}
                    </Transition>
                </div>

                <div class="rounded-box border border-base-200 p-4 space-y-4">
                    <div class="font-medium">"售后工单"</div>
                    <div class="space-y-4">
                        <div class="grid gap-3">
                            <select
                                class="select select-bordered"
                                prop:value=move || after_sales_case_type.get()
                                on:change=move |ev| after_sales_case_type.set(event_target_value(&ev))
                            >
                                <option value="">"请选择售后类型"</option>
                                <option value="complaint">"投诉"</option>
                                <option value="rework">"返工"</option>
                                <option value="refund">"退款"</option>
                                <option value="other">"其他"</option>
                            </select>
                            <textarea
                                class="textarea textarea-bordered min-h-24"
                                prop:value=move || after_sales_description.get()
                                on:input=move |ev| after_sales_description.set(event_target_value(&ev))
                                placeholder="填写售后问题描述"
                            />
                            <div class="flex justify-end">
                                <button
                                    class=move || {
                                        if creating_after_sales_case.get() {
                                            "btn btn-sm btn-primary btn-disabled"
                                        } else {
                                            "btn btn-sm btn-primary"
                                        }
                                    }
                                    disabled=move || creating_after_sales_case.get()
                                    on:click=save_after_sales_case
                                >
                                    {move || if creating_after_sales_case.get() { "创建中..." } else { "创建售后工单" }}
                                </button>
                            </div>
                        </div>

                        <Transition fallback=move || view! {
                            <div class="text-sm text-base-content/60">"加载中..."</div>
                        }>
                            {move || {
                                after_sales_cases.get().map(|items| {
                                    if items.is_empty() {
                                        view! {
                                            <div class="text-sm text-base-content/60">"暂无售后工单"</div>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="space-y-3">
                                                <For
                                                    each=move || items.clone().into_iter()
                                                    key=|item: &AfterSalesCase| item.uuid.clone()
                                                    children=move |item| {
                                                        let operator_label = item
                                                            .operator_name
                                                            .clone()
                                                            .filter(|value| !value.trim().is_empty())
                                                            .or_else(|| item.operator_uuid.clone())
                                                            .unwrap_or_else(|| "系统".to_string());
                                                        view! {
                                                            <div class="rounded-box bg-base-200/50 p-3 space-y-2">
                                                                <div class="flex items-center justify-between gap-2">
                                                                    <span class="text-sm font-medium">
                                                                        {after_sales_type_label(&item.case_type)}
                                                                    </span>
                                                                    <span class="text-xs text-base-content/60">{item.created_at.clone()}</span>
                                                                </div>
                                                                <div class="text-xs text-base-content/60">
                                                                    {format!("创建人：{} / 状态：{}", operator_label, after_sales_status_label(&item.status))}
                                                                </div>
                                                                <div class="text-sm whitespace-pre-wrap break-all">{item.description.clone()}</div>
                                                                <div class="flex justify-end">
                                                                    <button class="btn btn-soft btn-xs" on:click=move |_| open_after_sales_case(item.clone())>
                                                                        "处理记录"
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        }
                                                    }
                                                />
                                            </div>
                                        }.into_any()
                                    }
                                })
                            }}
                        </Transition>
                    </div>
                </div>

                <div class="rounded-box border border-base-200 p-4 space-y-4">
                    <div class="font-medium">"客户回访"</div>
                    {move || {
                        if let Some(order) = detail_order.get() {
                            if order.status != "completed" {
                                view! {
                                    <div class="text-sm text-base-content/60">
                                        "订单未完成，暂不支持记录回访结果"
                                    </div>
                                }.into_any()
                            } else if order.customer_uuid.as_deref().unwrap_or_default().is_empty() {
                                view! {
                                    <div class="text-sm text-base-content/60">
                                        "订单缺少客户信息，无法记录回访结果"
                                    </div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="space-y-4">
                                        <div class="grid gap-3">
                                            <textarea
                                                class="textarea textarea-bordered min-h-24"
                                                prop:value=move || follow_up_content.get()
                                                on:input=move |ev| follow_up_content.set(event_target_value(&ev))
                                                placeholder="填写本次回访结果、客户评价或后续安排"
                                            />
                                            <FlyonDateTimePicker
                                                value=next_follow_up_at
                                                class="input input-bordered".to_string()
                                            />
                                            <div class="flex justify-end">
                                                <button
                                                    class=move || {
                                                        if creating_follow_record.get() {
                                                            "btn btn-sm btn-primary btn-disabled"
                                                        } else {
                                                            "btn btn-sm btn-primary"
                                                        }
                                                    }
                                                    disabled=move || creating_follow_record.get()
                                                    on:click=save_follow_record
                                                >
                                                    {move || {
                                                        if creating_follow_record.get() {
                                                            "保存中..."
                                                        } else {
                                                            "保存回访"
                                                        }
                                                    }}
                                                </button>
                                            </div>
                                        </div>

                                        <Transition fallback=move || view! {
                                            <div class="text-sm text-base-content/60">"加载中..."</div>
                                        }>
                                            {move || {
                                                follow_records.get().map(|items| {
                                                    if items.is_empty() {
                                                        view! {
                                                            <div class="text-sm text-base-content/60">
                                                                "暂无回访记录"
                                                            </div>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <div class="space-y-3">
                                                                <For
                                                                    each=move || items.clone().into_iter()
                                                                    key=|item: &ContactFollowRecord| item.uuid.clone()
                                                                    children=move |item| {
                                                                        let has_next_follow_up = item.next_follow_up_at.is_some();
                                                                        let next_follow_up_text = item
                                                                            .next_follow_up_at
                                                                            .clone()
                                                                            .unwrap_or_default();
                                                                        view! {
                                                                            <div class="rounded-box bg-base-200/50 p-3 space-y-2">
                                                                                <div class="flex items-center justify-between gap-2">
                                                                                    <span class="text-sm font-medium">
                                                                                        {follow_record_operator_label(&item)}
                                                                                    </span>
                                                                                    <span class="text-xs text-base-content/60">
                                                                                        {item.created_at.clone()}
                                                                                    </span>
                                                                                </div>
                                                                                <div class="text-sm whitespace-pre-wrap break-all">
                                                                                    {item.content.clone()}
                                                                                </div>
                                                                                <Show when=move || has_next_follow_up>
                                                                                    <div class="text-xs text-base-content/60">
                                                                                        {format!(
                                                                                            "下次跟进：{}",
                                                                                            next_follow_up_text.clone()
                                                                                        )}
                                                                                    </div>
                                                                                </Show>
                                                                            </div>
                                                                        }
                                                                    }
                                                                />
                                                            </div>
                                                        }.into_any()
                                                    }
                                                })
                                            }}
                                        </Transition>
                                    </div>
                                }.into_any()
                            }
                        } else {
                            view! {
                                <div class="text-sm text-base-content/60">"暂无订单详情"</div>
                            }.into_any()
                        }
                    }}
                </div>

                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_detail_modal.set(false)>
                        "关闭"
                    </button>
                </div>
            </div>
        </Modal>

        <Modal show=show_after_sales_modal box_class=DETAIL_MODAL_BOX_CLASS>
            <div class="space-y-4">
                <h3 class="text-lg font-semibold">"售后处理记录"</h3>
                {move || {
                    if let Some(item) = active_after_sales_case.get() {
                        view! {
                            <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
                                {detail_item("工单ID", item.uuid.clone())}
                                {detail_item("售后类型", after_sales_type_label(&item.case_type).to_string())}
                                {detail_item("状态", after_sales_status_label(&item.status).to_string())}
                                {detail_item("退款金额(分)", display_optional_i64(item.refund_amount_cents))}
                                {detail_item("退款原因", display_optional(item.refund_reason.clone()))}
                                {detail_item("创建时间", item.created_at.clone())}
                            </div>
                        }.into_any()
                    } else {
                        view! { <div class="text-sm text-base-content/60">"暂无售后工单"</div> }.into_any()
                    }
                }}

                <div class="rounded-box border border-base-200 p-4 space-y-3">
                    <div class="font-medium">"退款信息"</div>
                    <div class="grid gap-3">
                        <input
                            class="input input-bordered"
                            type="number"
                            min="0"
                            prop:value=move || refund_amount_cents.get()
                            on:input=move |ev| refund_amount_cents.set(event_target_value(&ev))
                            placeholder="填写退款金额(分)"
                        />
                        <textarea
                            class="textarea textarea-bordered min-h-24"
                            prop:value=move || refund_reason.get()
                            on:input=move |ev| refund_reason.set(event_target_value(&ev))
                            placeholder="填写退款原因"
                        />
                        <div class="flex justify-end">
                            <button
                                class=move || {
                                    if saving_after_sales_refund.get() {
                                        "btn btn-sm btn-primary btn-disabled"
                                    } else {
                                        "btn btn-sm btn-primary"
                                    }
                                }
                                disabled=move || saving_after_sales_refund.get()
                                on:click=save_after_sales_refund
                            >
                                {move || if saving_after_sales_refund.get() { "保存中..." } else { "保存退款信息" }}
                            </button>
                        </div>
                    </div>
                </div>

                <div class="rounded-box border border-base-200 p-4 space-y-3">
                    <div class="font-medium">"新增处理记录"</div>
                    <div class="grid gap-3">
                        <select
                            class="select select-bordered"
                            prop:value=move || after_sales_record_status.get()
                            on:change=move |ev| after_sales_record_status.set(event_target_value(&ev))
                        >
                            <option value="processing">"处理中"</option>
                            <option value="resolved">"已解决"</option>
                            <option value="closed">"已关闭"</option>
                        </select>
                        <textarea
                            class="textarea textarea-bordered min-h-24"
                            prop:value=move || after_sales_record_content.get()
                            on:input=move |ev| after_sales_record_content.set(event_target_value(&ev))
                            placeholder="填写本次处理过程、沟通结果或处理结论"
                        />
                        <div class="flex justify-end">
                            <button
                                class=move || {
                                    if creating_after_sales_record.get() {
                                        "btn btn-sm btn-primary btn-disabled"
                                    } else {
                                        "btn btn-sm btn-primary"
                                    }
                                }
                                disabled=move || creating_after_sales_record.get()
                                on:click=save_after_sales_record
                            >
                                {move || if creating_after_sales_record.get() { "保存中..." } else { "保存处理记录" }}
                            </button>
                        </div>
                    </div>
                </div>

                <div class="rounded-box border border-base-200 p-4 space-y-3">
                    <div class="font-medium">"历史处理记录"</div>
                    <Transition fallback=move || view! {
                        <div class="text-sm text-base-content/60">"加载中..."</div>
                    }>
                        {move || {
                            after_sales_case_records.get().map(|items| {
                                if items.is_empty() {
                                    view! {
                                        <div class="text-sm text-base-content/60">"暂无处理记录"</div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-3">
                                            <For
                                                each=move || items.clone().into_iter()
                                                key=|item: &AfterSalesCaseRecord| item.uuid.clone()
                                                children=move |item| {
                                                    let operator_label = item
                                                        .operator_name
                                                        .clone()
                                                        .filter(|value| !value.trim().is_empty())
                                                        .or_else(|| item.operator_uuid.clone())
                                                        .unwrap_or_else(|| "系统".to_string());
                                                    view! {
                                                        <div class="rounded-box bg-base-200/50 p-3 space-y-2">
                                                            <div class="flex items-center justify-between gap-2">
                                                                <span class="text-sm font-medium">{operator_label}</span>
                                                                <span class="text-xs text-base-content/60">{item.created_at.clone()}</span>
                                                            </div>
                                                            <div class="text-xs text-base-content/60">
                                                                {format!("状态：{}", after_sales_status_label(&item.status))}
                                                            </div>
                                                            <div class="text-sm whitespace-pre-wrap break-all">{item.content.clone()}</div>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>
                                    }.into_any()
                                }
                            })
                        }}
                    </Transition>
                </div>

                <div class="rounded-box border border-base-200 p-4 space-y-3">
                    <div class="font-medium">"返工安排"</div>
                    <div class="grid gap-3">
                        <Transition fallback=move || view! {
                            <select class="select select-bordered w-full" disabled=true>
                                <option value="">"加载中..."</option>
                            </select>
                        }>
                            {move || {
                                let items = users.get().unwrap_or_default();
                                let options = items
                                    .into_iter()
                                    .map(|user| view! {
                                        <option value={user.uuid}>{user.user_name}</option>
                                    })
                                    .collect::<Vec<_>>();
                                view! {
                                    <select
                                        class="select select-bordered w-full"
                                        prop:value=move || rework_assigned_user_uuid.get()
                                        on:change=move |ev| rework_assigned_user_uuid.set(event_target_value(&ev))
                                    >
                                        <option value="">"请选择返工人员"</option>
                                        {options}
                                    </select>
                                }.into_any()
                            }}
                        </Transition>
                        <FlyonDateTimePicker
                            value=rework_start_at
                            class="input input-bordered".to_string()
                        />
                        <FlyonDateTimePicker
                            value=rework_end_at
                            class="input input-bordered".to_string()
                        />
                        <textarea
                            class="textarea textarea-bordered min-h-24"
                            prop:value=move || rework_note.get()
                            on:input=move |ev| rework_note.set(event_target_value(&ev))
                            placeholder="填写返工说明或补充安排"
                        />
                        <div class="flex justify-end">
                            <button
                                class=move || {
                                    if creating_rework.get() {
                                        "btn btn-sm btn-primary btn-disabled"
                                    } else {
                                        "btn btn-sm btn-primary"
                                    }
                                }
                                disabled=move || creating_rework.get()
                                on:click=save_after_sales_rework
                            >
                                {move || if creating_rework.get() { "保存中..." } else { "保存返工安排" }}
                            </button>
                        </div>
                    </div>

                    <Transition fallback=move || view! {
                        <div class="text-sm text-base-content/60">"加载中..."</div>
                    }>
                        {move || {
                            after_sales_reworks.get().map(|items| {
                                if items.is_empty() {
                                    view! { <div class="text-sm text-base-content/60">"暂无返工安排"</div> }.into_any()
                                } else {
                                    view! {
                                        <div class="space-y-3">
                                            <For
                                                each=move || items.clone().into_iter()
                                                key=|item: &AfterSalesRework| item.uuid.clone()
                                                children=move |item| {
                                                    let assigned_user_name = item
                                                        .assigned_user_name
                                                        .clone()
                                                        .unwrap_or_else(|| item.assigned_user_uuid.clone());
                                                    let has_note = item.note.as_ref().map(|v| !v.trim().is_empty()).unwrap_or(false);
                                                    let note_text = item.note.clone().unwrap_or_default();
                                                    view! {
                                                        <div class="rounded-box bg-base-200/50 p-3 space-y-2">
                                                            <div class="flex items-center justify-between gap-2">
                                                                <span class="text-sm font-medium">{assigned_user_name}</span>
                                                                <span class="text-xs text-base-content/60">{item.created_at.clone()}</span>
                                                            </div>
                                                            <div class="text-xs text-base-content/60">
                                                                {format!("返工时间：{} ~ {}", item.scheduled_start_at, item.scheduled_end_at)}
                                                            </div>
                                                            <div class="text-xs text-base-content/60">
                                                                {format!("状态：{}", after_sales_rework_status_label(&item.status))}
                                                            </div>
                                                            <Show when=move || has_note>
                                                                <div class="text-sm whitespace-pre-wrap break-all">
                                                                    {note_text.clone()}
                                                                </div>
                                                            </Show>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>
                                    }.into_any()
                                }
                            })
                        }}
                    </Transition>
                </div>

                <div class="flex justify-end gap-2">
                    <button class="btn" on:click=move |_| show_after_sales_modal.set(false)>
                        "关闭"
                    </button>
                </div>
            </div>
        </Modal>
    }
}

#[component]
fn OrderForm(
    contacts: Resource<Vec<Contact>>,
    customer_uuid: RwSignal<String>,
    amount_cents: RwSignal<String>,
    notes: RwSignal<String>,
) -> impl IntoView {
    view! {
        <div class="grid grid-cols-1 gap-4 md:grid-cols-2">
            <label class="form-control">
                <span class="label-text text-sm">"客户"</span>
                <select
                    class="select select-bordered"
                    prop:value=move || customer_uuid.get()
                    on:change=move |ev| customer_uuid.set(event_target_value(&ev))
                >
                    <option value="">"请选择客户"</option>
                    <Transition fallback=move || view! { <option value="">"加载中..."</option> }>
                        {move || contacts.get().map(|items| {
                            view! {
                                <For
                                    each=move || items.clone().into_iter()
                                    key=|contact| contact.contact_uuid.clone()
                                    children=move |contact| {
                                        let label = contact_display_label(&contact);
                                        view! { <option value={contact.contact_uuid}>{label}</option> }
                                    }
                                />
                            }
                        })}
                    </Transition>
                </select>
            </label>
            <label class="form-control">
                <span class="label-text text-sm">"订单金额(分)"</span>
                <input
                    class="input input-bordered"
                    type="number"
                    min="0"
                    prop:value=move || amount_cents.get()
                    on:input=move |ev| amount_cents.set(event_target_value(&ev))
                    placeholder="例如：29900"
                />
            </label>
            <label class="form-control md:col-span-2">
                <span class="label-text text-sm">"订单备注"</span>
                <textarea
                    class="textarea textarea-bordered min-h-24"
                    prop:value=move || notes.get()
                    on:input=move |ev| notes.set(event_target_value(&ev))
                    placeholder="补充订单说明"
                />
            </label>
        </div>
    }
}

fn contact_display_label(contact: &Contact) -> String {
    let name = contact.user_name.trim();
    let mut label = String::new();
    if !name.is_empty() {
        label.push_str(name);
    }
    if label.is_empty() {
        label = "未命名客户".to_string();
    }
    let extra = contact.phone_number.trim();
    if !extra.is_empty() {
        format!("{} ({})", label, extra)
    } else {
        label
    }
}

fn display_optional(value: Option<String>) -> String {
    value
        .and_then(|v| {
            let trimmed = v.trim().to_string();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed)
            }
        })
        .unwrap_or_else(|| "-".to_string())
}

fn detail_item(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-1">
            <span class="text-xs text-base-content/60">{label}</span>
            <span class="text-sm break-all">{value}</span>
        </div>
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

fn order_status_label(status: &str) -> &'static str {
    match status {
        "pending" => "待确认",
        "confirmed" => "已确认",
        "dispatching" => "派工中",
        "in_service" => "服务中",
        "completed" => "已完成",
        "cancelled" => "已取消",
        _ => "未知",
    }
}

fn settlement_status_label(status: &str) -> &'static str {
    match status {
        "unsettled" => "未结算",
        "settled" => "已结算",
        _ => "未知",
    }
}

fn order_status_badge_class(status: &str) -> &'static str {
    match status {
        "pending" => "badge-warning",
        "confirmed" => "badge-info",
        "dispatching" => "badge-warning",
        "in_service" => "badge-warning",
        "completed" => "badge-success",
        "cancelled" => "badge-error",
        _ => "badge-info",
    }
}

fn settlement_status_badge_class(status: &str) -> &'static str {
    match status {
        "unsettled" => "badge-warning",
        "settled" => "badge-success",
        _ => "badge-info",
    }
}

fn order_log_action_label(action: &str) -> &'static str {
    match action {
        "created" => "创建订单",
        "details_updated" => "编辑订单",
        "status_changed" => "状态变更",
        "cancelled" => "取消订单",
        "assignment_updated" => "派工更新",
        "settlement_updated" => "结算更新",
        _ => "订单变更",
    }
}

fn order_log_diff_items(item: &OrderChangeLogDto) -> Vec<(String, String, String)> {
    let mut result = Vec::new();
    let before = item
        .before_data
        .as_ref()
        .and_then(|value| value.as_object());
    let after = item.after_data.as_ref().and_then(|value| value.as_object());

    let keys = [
        "customer_uuid",
        "status",
        "cancellation_reason",
        "completed_at",
        "settlement_status",
        "amount_cents",
        "paid_amount_cents",
        "payment_method",
        "paid_at",
        "notes",
        "dispatch_note",
        "settlement_note",
        "scheduled_start_at",
        "scheduled_end_at",
    ];

    for key in keys {
        let before_value = before.and_then(|map| map.get(key));
        let after_value = after.and_then(|map| map.get(key));
        if before_value == after_value {
            continue;
        }
        result.push((
            order_log_field_label(key),
            format_json_field(before_value),
            format_json_field(after_value),
        ));
    }

    result
}

fn order_log_field_label(key: &str) -> String {
    match key {
        "customer_uuid" => "客户UUID".to_string(),
        "status" => "订单状态".to_string(),
        "cancellation_reason" => "取消原因".to_string(),
        "completed_at" => "完成时间".to_string(),
        "settlement_status" => "结算状态".to_string(),
        "amount_cents" => "订单金额(分)".to_string(),
        "paid_amount_cents" => "实收金额(分)".to_string(),
        "payment_method" => "支付方式".to_string(),
        "paid_at" => "收款时间".to_string(),
        "notes" => "订单备注".to_string(),
        "dispatch_note" => "派工备注".to_string(),
        "settlement_note" => "结算备注".to_string(),
        "scheduled_start_at" => "服务开始".to_string(),
        "scheduled_end_at" => "服务结束".to_string(),
        _ => key.to_string(),
    }
}

fn display_optional_i64(value: Option<i64>) -> String {
    value
        .map(|value| value.to_string())
        .unwrap_or_else(|| "-".to_string())
}

fn payment_method_label(value: Option<&str>) -> &'static str {
    match value {
        Some("cash") => "现金",
        Some("wechat") => "微信",
        Some("alipay") => "支付宝",
        Some("bank_transfer") => "银行转账",
        Some("other") => "其他",
        Some(_) => "未知",
        None => "-",
    }
}

fn after_sales_type_label(value: &str) -> &'static str {
    match value {
        "complaint" => "投诉",
        "rework" => "返工",
        "refund" => "退款",
        "other" => "其他",
        _ => "未知",
    }
}

fn after_sales_status_label(value: &str) -> &'static str {
    match value {
        "open" => "新建",
        "processing" => "处理中",
        "resolved" => "已解决",
        "closed" => "已关闭",
        _ => "未知",
    }
}

fn after_sales_rework_status_label(value: &str) -> &'static str {
    match value {
        "planned" => "已安排",
        "done" => "已完成",
        "cancelled" => "已取消",
        _ => "未知",
    }
}

fn format_json_field(value: Option<&serde_json::Value>) -> String {
    match value {
        None | Some(serde_json::Value::Null) => "-".to_string(),
        Some(serde_json::Value::String(value)) => {
            if value.trim().is_empty() {
                "-".to_string()
            } else {
                value.clone()
            }
        }
        Some(serde_json::Value::Number(value)) => value.to_string(),
        Some(serde_json::Value::Bool(value)) => {
            if *value {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Some(other) => serde_json::to_string_pretty(other).unwrap_or_else(|_| other.to_string()),
    }
}

fn follow_record_operator_label(record: &ContactFollowRecord) -> String {
    record
        .operator_name
        .clone()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| record.operator_uuid.clone())
        .unwrap_or_else(|| "系统".to_string())
}
