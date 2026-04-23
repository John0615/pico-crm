use crate::components::ui::form::{DaisyForm, FieldType, FormContainer, FormField, ValidationRule};
use crate::components::ui::modal::Modal;
use crate::components::ui::pagination::Pagination;
use crate::components::ui::table::{Column, DaisyTable, Identifiable};
use crate::components::ui::toast::{error, success};
use crate::utils::api::call_api;
use leptos::logging;
use leptos::prelude::*;
use leptos_meta::Title;
use leptos_router::hooks::use_query_map;
use shared::merchant::{
    MerchantListQuery, MerchantSummary, ProvisionMerchantRequest, UpdateMerchantRequest,
};

pub use crate::server::admin_merchant_handlers::{
    create_merchant, fetch_merchants, update_merchant,
};

impl Identifiable for MerchantSummary {
    fn id(&self) -> String {
        format!("{}-{}", self.uuid, self.updated_at)
    }
}

#[component]
pub fn AdminMerchants() -> impl IntoView {
    let query = use_query_map();
    let refresh_count = RwSignal::new(0);

    let (name, set_name) = signal(String::new());
    let (status, set_status) = signal(String::new());
    let (plan_type, set_plan_type) = signal(String::new());
    let (contact_phone, set_contact_phone) = signal(String::new());

    let show_modal = RwSignal::new(false);
    let show_create_modal = RwSignal::new(false);
    let edit_merchant = RwSignal::new(None::<MerchantSummary>);

    let data = Resource::new(
        move || {
            (
                name.with(|value| value.clone()),
                status.with(|value| value.clone()),
                plan_type.with(|value| value.clone()),
                contact_phone.with(|value| value.clone()),
                *refresh_count.read(),
                query.with(|value| value.clone()),
            )
        },
        |(name, status, plan_type, contact_phone, _, query)| async move {
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

            let params = MerchantListQuery {
                page,
                page_size,
                name: (!name.is_empty()).then_some(name),
                status: (!status.is_empty()).then_some(status),
                plan_type: (!plan_type.is_empty()).then_some(plan_type),
                contact_phone: (!contact_phone.is_empty()).then_some(contact_phone),
            };

            let result = call_api(fetch_merchants(params)).await.unwrap_or_else(|e| {
                logging::error!("Error loading merchants: {e}");
                shared::merchant::MerchantPagedResult {
                    items: Vec::new(),
                    total: 0,
                }
            });
            (result.items, result.total)
        },
    );

    let on_edit = move |merchant: MerchantSummary| {
        edit_merchant.set(Some(merchant));
        show_modal.set(true);
    };

    let on_modal_finish = move || {
        refresh_count.update(|value| *value += 1);
        show_modal.set(false);
        edit_merchant.set(None);
    };
    let on_create_finish = move || {
        refresh_count.update(|value| *value += 1);
        show_create_modal.set(false);
    };

    let filter_by_name = move |ev| {
        let value = event_target_value(&ev);
        set_name.set(value);
    };

    let filter_by_status = move |ev| {
        let value = event_target_value(&ev);
        set_status.set(value);
    };

    let filter_by_plan = move |ev| {
        let value = event_target_value(&ev);
        set_plan_type.set(value);
    };

    let filter_by_phone = move |ev| {
        let value = event_target_value(&ev);
        set_contact_phone.set(value);
    };

    view! {
        <Title text="商户管理 - PicoCRM"/>
        <div class="">
            <div class="flex flex-col md:flex-row md:flex-wrap gap-4 mb-4">
                <label class="input w-full md:w-80 md:flex-none">
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
                    <input type="search" on:input=filter_by_name placeholder="搜索商户名称..." />
                </label>
                <label class="input w-full md:w-72 md:flex-none">
                    <span class="icon-[tabler--phone] size-5 text-base-content/50"></span>
                    <input type="search" on:input=filter_by_phone placeholder="联系人手机号..." />
                </label>
                <div class="flex gap-2 items-center md:flex-nowrap">
                    <select on:change=filter_by_status class="select select-bordered">
                        <option value="">所有状态</option>
                        <option value="active">活跃</option>
                        <option value="inactive">停用</option>
                        <option value="suspended">暂停</option>
                    </select>
                    <select on:change=filter_by_plan class="select select-bordered">
                        <option value="">所有套餐</option>
                        <option value="trial">试用</option>
                        <option value="basic">基础版</option>
                        <option value="pro">专业版</option>
                    </select>
                </div>
            </div>

            <div class="fixed bottom-8 right-8 z-10">
                <button
                    on:click=move |_| {
                        show_create_modal.set(true);
                    }
                    class="btn btn-circle btn-primary shadow-lg hover:shadow-xl transition-all"
                    style="width: 56px; height: 56px;"
                    aria-label="新增商户"
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

            <div class="h-[calc(100vh-260px)] bg-base-100 rounded-lg shadow relative flex flex-col">
                <div class="overflow-x-auto overflow-y-auto flex-1 min-h-0">
                    <Suspense fallback=move || view! {
                        <div class="p-6 text-sm text-base-content/60">"加载中..."</div>
                    }>
                        <DaisyTable data=data>
                        <Column
                            slot:columns
                            freeze=true
                            prop="name".to_string()
                            label="商户".to_string()
                            class="font-semibold"
                        >
                            {move || {
                                let merchant: Option<MerchantSummary> = use_context::<MerchantSummary>();
                                match merchant {
                                    Some(item) => view! {
                                        <span>{item.name}</span>
                                    }
                                    .into_any(),
                                    None => view! { <span>-</span> }.into_any(),
                                }
                            }}
                        </Column>
                        <Column
                            slot:columns
                            prop="contact".to_string()
                            label="联系人".to_string()
                        >
                            {move || {
                                let merchant: Option<MerchantSummary> = use_context::<MerchantSummary>();
                                match merchant {
                                    Some(item) => view! {
                                        <div class="flex flex-col">
                                            <span>{item.contact_name}</span>
                                            <span class="text-xs text-base-content/50">{item.contact_phone}</span>
                                        </div>
                                    }
                                    .into_any(),
                                    None => view! { <span>-</span> }.into_any(),
                                }
                            }}
                        </Column>
                        <Column
                            slot:columns
                            prop="plan".to_string()
                            label="套餐".to_string()
                        >
                            {move || {
                                let merchant: Option<MerchantSummary> = use_context::<MerchantSummary>();
                                match merchant {
                                    Some(item) => {
                                        let plan = merchant_plan_label(item.plan_type.as_deref());
                                        view! { <span class="badge badge-ghost">{plan}</span> }.into_any()
                                    }
                                    None => view! { <span>-</span> }.into_any(),
                                }
                            }}
                        </Column>
                        <Column
                            slot:columns
                            prop="status".to_string()
                            label="状态".to_string()
                        >
                            {move || {
                                let merchant: Option<MerchantSummary> = use_context::<MerchantSummary>();
                                match merchant {
                                    Some(item) => {
                                        let badge = match item.status.as_str() {
                                            "active" => "badge badge-success",
                                            "inactive" => "badge badge-ghost",
                                            "suspended" => "badge badge-warning",
                                            _ => "badge badge-ghost",
                                        };
                                        let status = merchant_status_label(item.status.as_str());
                                        view! { <span class=badge>{status}</span> }.into_any()
                                    }
                                    None => view! { <span>-</span> }.into_any(),
                                }
                            }}
                        </Column>
                        <Column
                            slot:columns
                            prop="trial_end_at".to_string()
                            label="试用结束".to_string()
                        >
                            {move || {
                                let merchant: Option<MerchantSummary> = use_context::<MerchantSummary>();
                                match merchant {
                                    Some(item) => view! {
                                        <span>{item.trial_end_at.unwrap_or_else(|| "-".to_string())}</span>
                                    }
                                    .into_any(),
                                    None => view! { <span>-</span> }.into_any(),
                                }
                            }}
                        </Column>
                        <Column
                            slot:columns
                            prop="expired_at".to_string()
                            label="到期时间".to_string()
                        >
                            {move || {
                                let merchant: Option<MerchantSummary> = use_context::<MerchantSummary>();
                                match merchant {
                                    Some(item) => view! {
                                        <span>{item.expired_at.unwrap_or_else(|| "-".to_string())}</span>
                                    }
                                    .into_any(),
                                    None => view! { <span>-</span> }.into_any(),
                                }
                            }}
                        </Column>
                        <Column
                            slot:columns
                            prop="actions".to_string()
                            label="操作".to_string()
                        >
                            {move || {
                                let merchant: Option<MerchantSummary> = use_context::<MerchantSummary>();
                                match merchant {
                                    Some(item) => {
                                        let merchant_clone = item.clone();
                                        view! {
                                            <button
                                                class="btn btn-xs btn-outline"
                                                on:click=move |_| on_edit(merchant_clone.clone())
                                            >
                                                "编辑"
                                            </button>
                                        }
                                        .into_any()
                                    }
                                    None => view! { <span>-</span> }.into_any(),
                                }
                            }}
                        </Column>
                        </DaisyTable>

                    </Suspense>
                </div>
            </div>
            <Transition>
                {move || {
                    data.with(|d| {
                        d.as_ref().map(|(_, total)| {
                            view! { <Pagination total_items=*total /> }
                        })
                    })
                }}
            </Transition>
        </div>

        <MerchantEditModal
            show=show_modal
            merchant=edit_merchant
            on_finish=on_modal_finish
        />
        <MerchantCreateModal
            show=show_create_modal
            on_finish=on_create_finish
        />
    }
}

#[component]
fn MerchantEditModal(
    show: RwSignal<bool>,
    merchant: RwSignal<Option<MerchantSummary>>,
    on_finish: impl Fn() + Copy + Send + 'static,
) -> impl IntoView {
    let name_value = ArcRwSignal::new(String::new());
    let short_name_value = ArcRwSignal::new(String::new());
    let contact_name_value = ArcRwSignal::new(String::new());
    let contact_phone_value = ArcRwSignal::new(String::new());
    let merchant_type_value = ArcRwSignal::new(String::new());
    let plan_type_value = ArcRwSignal::new(String::new());
    let status_value = ArcRwSignal::new(String::new());
    let trial_end_at_value = ArcRwSignal::new(String::new());
    let expired_at_value = ArcRwSignal::new(String::new());

    Effect::new({
        let name_value = name_value.clone();
        let short_name_value = short_name_value.clone();
        let contact_name_value = contact_name_value.clone();
        let contact_phone_value = contact_phone_value.clone();
        let merchant_type_value = merchant_type_value.clone();
        let plan_type_value = plan_type_value.clone();
        let status_value = status_value.clone();
        let trial_end_at_value = trial_end_at_value.clone();
        let expired_at_value = expired_at_value.clone();
        move |_| {
            if let Some(current) = merchant.with(|value| value.clone()) {
                name_value.set(current.name.clone());
                short_name_value.set(current.short_name.clone().unwrap_or_default());
                contact_name_value.set(current.contact_name.clone());
                contact_phone_value.set(current.contact_phone.clone());
                merchant_type_value.set(current.merchant_type.clone().unwrap_or_default());
                plan_type_value.set(current.plan_type.clone().unwrap_or_default());
                status_value.set(current.status.clone());
                trial_end_at_value.set(to_datetime_picker(current.trial_end_at.as_ref()));
                expired_at_value.set(to_datetime_picker(current.expired_at.as_ref()));
            } else {
                name_value.set(String::new());
                short_name_value.set(String::new());
                contact_name_value.set(String::new());
                contact_phone_value.set(String::new());
                merchant_type_value.set(String::new());
                plan_type_value.set(String::new());
                status_value.set(String::new());
                trial_end_at_value.set(String::new());
                expired_at_value.set(String::new());
            }
        }
    });

    let initial_fields = vec![
        FormField {
            name: "name".to_string(),
            label: "商户名称".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: name_value,
            placeholder: Some("输入商户名称".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::MinLength(2)),
        },
        FormField {
            name: "short_name".to_string(),
            label: "商户简称".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: short_name_value,
            placeholder: Some("输入商户简称".into()),
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "contact_name".to_string(),
            label: "联系人".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: contact_name_value,
            placeholder: Some("输入联系人姓名".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::MinLength(2)),
        },
        FormField {
            name: "contact_phone".to_string(),
            label: "联系人手机号".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: contact_phone_value,
            placeholder: Some("输入联系人手机号".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::CnMobile),
        },
        FormField {
            name: "merchant_type".to_string(),
            label: "商户类型".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: merchant_type_value,
            placeholder: Some("输入商户类型".into()),
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "plan_type".to_string(),
            label: "套餐".to_string(),
            field_type: FieldType::Select(vec![
                ("".to_string(), "未设置".to_string()),
                ("trial".to_string(), "试用".to_string()),
                ("basic".to_string(), "基础版".to_string()),
                ("pro".to_string(), "专业版".to_string()),
            ]),
            required: false,
            value: plan_type_value,
            placeholder: None,
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "status".to_string(),
            label: "状态".to_string(),
            field_type: FieldType::Select(vec![
                ("active".to_string(), "活跃".to_string()),
                ("inactive".to_string(), "停用".to_string()),
                ("suspended".to_string(), "暂停".to_string()),
            ]),
            required: true,
            value: status_value,
            placeholder: None,
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "trial_end_at".to_string(),
            label: "试用结束时间".to_string(),
            field_type: FieldType::DateTimePicker,
            required: false,
            value: trial_end_at_value,
            placeholder: None,
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "expired_at".to_string(),
            label: "到期时间".to_string(),
            field_type: FieldType::DateTimePicker,
            required: false,
            value: expired_at_value,
            placeholder: None,
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
    ];

    let submit = move |fields: Vec<FormField>| async move {
        let Some(merchant_uuid) =
            merchant.with_untracked(|value| value.as_ref().map(|item| item.uuid.clone()))
        else {
            error("未找到商户信息".to_string());
            return Err(vec!["未找到商户信息".to_string()]);
        };

        let name = fields[0]
            .value
            .with_untracked(|value| value.trim().to_string());
        let short_name = fields[1].value.with_untracked(|value| value.clone());
        let contact_name = fields[2]
            .value
            .with_untracked(|value| value.trim().to_string());
        let contact_phone = fields[3]
            .value
            .with_untracked(|value| value.trim().to_string());
        let merchant_type = fields[4].value.with_untracked(|value| value.clone());
        let plan_type = fields[5].value.with_untracked(|value| value.clone());
        let status = fields[6]
            .value
            .with_untracked(|value| value.trim().to_string());
        let trial_end_at = fields[7].value.with_untracked(|value| value.clone());
        let expired_at = fields[8].value.with_untracked(|value| value.clone());

        let request = UpdateMerchantRequest {
            name: Some(name),
            short_name: normalize_optional(short_name),
            contact_name: Some(contact_name),
            contact_phone: Some(contact_phone),
            merchant_type: normalize_optional(merchant_type),
            status: Some(status),
            plan_type: normalize_optional(plan_type),
            trial_end_at: normalize_optional(trial_end_at),
            expired_at: normalize_optional(expired_at),
        };

        match call_api(update_merchant(merchant_uuid, request)).await {
            Ok(_) => {
                success("更新成功".to_string());
                on_finish();
                Ok(())
            }
            Err(err) => {
                logging::error!("Failed to update merchant: {:?}", err);
                error("更新失败".to_string());
                Err(vec![err.to_string()])
            }
        }
    };

    view! {
        <Modal show=show box_class="max-w-2xl max-h-none overflow-visible px-3 py-4">
            <FormContainer title="更新商户">
                <DaisyForm
                    initial_fields
                    on_submit=submit
                    submit_text="保存".to_string()
                    reset_text="取消".to_string()
                    form_class="max-w-none".to_string()
                />
            </FormContainer>
        </Modal>
    }
}

#[component]
fn MerchantCreateModal(
    show: RwSignal<bool>,
    on_finish: impl Fn() + Copy + Send + 'static,
) -> impl IntoView {
    let name_value = ArcRwSignal::new(String::new());
    let short_name_value = ArcRwSignal::new(String::new());
    let contact_name_value = ArcRwSignal::new(String::new());
    let contact_phone_value = ArcRwSignal::new(String::new());
    let merchant_type_value = ArcRwSignal::new(String::new());
    let plan_type_value = ArcRwSignal::new(String::new());
    let owner_user_name_value = ArcRwSignal::new(String::new());
    let owner_password_value = ArcRwSignal::new(String::new());

    Effect::new({
        let name_value = name_value.clone();
        let short_name_value = short_name_value.clone();
        let contact_name_value = contact_name_value.clone();
        let contact_phone_value = contact_phone_value.clone();
        let merchant_type_value = merchant_type_value.clone();
        let plan_type_value = plan_type_value.clone();
        let owner_user_name_value = owner_user_name_value.clone();
        let owner_password_value = owner_password_value.clone();
        move |_| {
            if !*show.read() {
                name_value.set(String::new());
                short_name_value.set(String::new());
                contact_name_value.set(String::new());
                contact_phone_value.set(String::new());
                merchant_type_value.set(String::new());
                plan_type_value.set(String::new());
                owner_user_name_value.set(String::new());
                owner_password_value.set(String::new());
            }
        }
    });

    let initial_fields = vec![
        FormField {
            name: "name".to_string(),
            label: "商户名称".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: name_value,
            placeholder: Some("输入商户名称".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::MinLength(2)),
        },
        FormField {
            name: "short_name".to_string(),
            label: "商户简称".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: short_name_value,
            placeholder: Some("输入商户简称".into()),
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "contact_name".to_string(),
            label: "联系人".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: contact_name_value,
            placeholder: Some("输入联系人姓名".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::MinLength(2)),
        },
        FormField {
            name: "contact_phone".to_string(),
            label: "联系人手机号".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: contact_phone_value,
            placeholder: Some("输入联系人手机号".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::CnMobile),
        },
        FormField {
            name: "merchant_type".to_string(),
            label: "商户类型".to_string(),
            field_type: FieldType::Text,
            required: false,
            value: merchant_type_value,
            placeholder: Some("输入商户类型".into()),
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "plan_type".to_string(),
            label: "套餐".to_string(),
            field_type: FieldType::Select(vec![
                ("".to_string(), "未设置".to_string()),
                ("trial".to_string(), "试用".to_string()),
                ("basic".to_string(), "基础版".to_string()),
                ("pro".to_string(), "专业版".to_string()),
            ]),
            required: false,
            value: plan_type_value,
            placeholder: None,
            error_message: ArcRwSignal::new(None),
            validation: None,
        },
        FormField {
            name: "owner_user_name".to_string(),
            label: "登录用户名".to_string(),
            field_type: FieldType::Text,
            required: true,
            value: owner_user_name_value,
            placeholder: Some("输入全局唯一登录用户名".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::MinLength(2)),
        },
        FormField {
            name: "owner_password".to_string(),
            label: "初始密码".to_string(),
            field_type: FieldType::Password,
            required: true,
            value: owner_password_value,
            placeholder: Some("输入初始密码".into()),
            error_message: ArcRwSignal::new(None),
            validation: Some(ValidationRule::MinLength(6)),
        },
    ];

    let submit = move |fields: Vec<FormField>| async move {
        let name = fields[0]
            .value
            .with_untracked(|value| value.trim().to_string());
        let short_name = fields[1].value.with_untracked(|value| value.clone());
        let contact_name = fields[2]
            .value
            .with_untracked(|value| value.trim().to_string());
        let contact_phone = fields[3]
            .value
            .with_untracked(|value| value.trim().to_string());
        let merchant_type = fields[4].value.with_untracked(|value| value.clone());
        let plan_type = fields[5].value.with_untracked(|value| value.clone());
        let owner_user_name = fields[6]
            .value
            .with_untracked(|value| value.trim().to_string());
        let owner_password = fields[7].value.with_untracked(|value| value.clone());

        let request = ProvisionMerchantRequest {
            name,
            short_name: normalize_optional(short_name),
            contact_name,
            contact_phone,
            merchant_type: normalize_optional(merchant_type),
            plan_type: normalize_optional(plan_type),
            owner_user_name: Some(owner_user_name),
            owner_password: Some(owner_password),
        };

        match call_api(create_merchant(request)).await {
            Ok(_) => {
                success("创建成功".to_string());
                on_finish();
                Ok(())
            }
            Err(err) => {
                logging::error!("Failed to create merchant: {:?}", err);
                error("创建失败".to_string());
                Err(vec![err.to_string()])
            }
        }
    };

    view! {
        <Modal show=show box_class="max-w-2xl max-h-none overflow-visible px-3 py-4">
            <FormContainer title="新增商户">
                <DaisyForm
                    initial_fields
                    on_submit=submit
                    submit_text="创建".to_string()
                    reset_text="取消".to_string()
                    form_class="max-w-none".to_string()
                />
            </FormContainer>
        </Modal>
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

fn to_datetime_picker(value: Option<&String>) -> String {
    let Some(value) = value else {
        return String::new();
    };
    let mut output = value.replace('T', " ");
    if output.len() >= 16 {
        output.truncate(16);
    }
    output
}

fn merchant_plan_label(plan_type: Option<&str>) -> String {
    match plan_type {
        None | Some("") => "未设置".to_string(),
        Some("trial") => "试用".to_string(),
        Some("basic") => "基础版".to_string(),
        Some("pro") => "专业版".to_string(),
        Some(other) => format!("未知({})", other),
    }
}

fn merchant_status_label(status: &str) -> String {
    match status {
        "active" => "活跃".to_string(),
        "inactive" => "停用".to_string(),
        "suspended" => "暂停".to_string(),
        other => format!("未知({})", other),
    }
}
