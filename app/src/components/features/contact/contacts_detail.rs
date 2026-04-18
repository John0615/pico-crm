use crate::components::ui::drawer::DaisyDrawer;
use crate::server::contact_handlers::get_contact;
use crate::utils::api::call_api;
use leptos::prelude::*;
use shared::contact::Contact;

#[component]
pub fn ContactDetail(
    open_drawer: RwSignal<bool>,
    contact_uuid: ReadSignal<String>,
) -> impl IntoView {
    let detail = Resource::new(
        move || contact_uuid.with(|value| value.clone()),
        |uuid| async move {
            if uuid.trim().is_empty() {
                None
            } else {
                call_api(get_contact(uuid)).await.ok().flatten()
            }
        },
    );

    view! {
        <DaisyDrawer id="contact-drawer" width=560 position="right" is_open=open_drawer>
            <div class="h-full bg-base-100 flex flex-col overflow-hidden">
                <Transition
                    fallback=move || view! {
                        <div class="flex h-full items-center justify-center">
                            <span class="loading loading-bars loading-xl"></span>
                        </div>
                    }
                >
                    {move || {
                        detail
                            .get()
                            .map(|contact| {
                                contact
                                    .map(|contact| {
                                        view! { <ContactDetailContent contact /> }.into_any()
                                    })
                                    .unwrap_or_else(|| {
                                        view! {
                                            <div class="flex h-full items-center justify-center text-sm text-base-content/60">
                                                "未找到客户详情"
                                            </div>
                                        }
                                        .into_any()
                                    })
                            })
                    }}
                </Transition>
            </div>
        </DaisyDrawer>
    }
}

#[component]
fn ContactDetailContent(contact: Contact) -> impl IntoView {
    let address = build_contact_address(&contact);
    let service_need = contact
        .service_need
        .clone()
        .unwrap_or_else(|| "暂无记录".to_string());
    let tags = if contact.tags.is_empty() {
        vec!["未设置".to_string()]
    } else {
        contact.tags.clone()
    };

    view! {
        <div class="space-y-6">
            <div class="space-y-4 border-b border-base-200 pb-5">
                <div class="flex items-start justify-between gap-4">
                    <div class="space-y-2">
                        <div class="flex flex-wrap items-center gap-2">
                            <h2 class="text-2xl font-bold">{contact.user_name.clone()}</h2>
                            <span class=format!(
                                "badge badge-outline {}",
                                follow_up_status_badge_class(contact.follow_up_status.as_deref())
                            )>
                                {follow_up_status_label(contact.follow_up_status.as_deref())}
                            </span>
                        </div>
                        <div class="text-sm text-base-content/70">
                            {format!("电话：{}", empty_dash(&contact.phone_number))}
                        </div>
                    </div>
                    <div class="text-right text-xs text-base-content/60">
                        <div>{format!(
                            "最近服务：{}",
                            contact.last_service_at.as_deref().unwrap_or("-")
                        )}</div>
                    </div>
                </div>

                <div class="grid gap-3 sm:grid-cols-2">
                    <InfoCard
                        label="房屋面积"
                        value=contact
                            .house_area_sqm
                            .map(|value| format!("{} ㎡", value))
                            .unwrap_or_else(|| "-".to_string())
                    />
                    <InfoCard
                        label="小区/社区"
                        value=contact.community.clone().unwrap_or_else(|| "-".to_string())
                    />
                    <InfoCard
                        label="楼栋/门牌"
                        value=contact.building.clone().unwrap_or_else(|| "-".to_string())
                    />
                </div>
            </div>

            <section class="space-y-3">
                <div class="text-sm font-semibold text-base-content/70">"地址信息"</div>
                <div class="rounded-box border border-base-200 bg-base-50 p-4 text-sm">
                    {address}
                </div>
            </section>

            <section class="space-y-3">
                <div class="text-sm font-semibold text-base-content/70">"服务需求"</div>
                <div class="rounded-box border border-base-200 bg-base-50 p-4 text-sm whitespace-pre-wrap">
                    {service_need}
                </div>
            </section>

            <section class="space-y-3">
                <div class="text-sm font-semibold text-base-content/70">"客户标签"</div>
                <div class="flex flex-wrap gap-2">
                    <For
                        each=move || tags.clone().into_iter().enumerate()
                        key=|(idx, tag)| format!("{}-{}", idx, tag)
                        children=move |(_, tag)| view! { <span class="badge badge-outline">{tag}</span> }
                    />
                </div>
            </section>
        </div>
    }
}

#[component]
fn InfoCard(label: &'static str, value: String) -> impl IntoView {
    view! {
        <div class="rounded-box border border-base-200 bg-base-50 p-4">
            <div class="text-xs uppercase tracking-wide text-base-content/50">{label}</div>
            <div class="mt-2 text-sm font-medium">{value}</div>
        </div>
    }
}

fn build_contact_address(contact: &Contact) -> String {
    let mut parts = Vec::new();
    if let Some(address) = &contact.address {
        if !address.trim().is_empty() {
            parts.push(address.trim().to_string());
        }
    }
    if let Some(community) = &contact.community {
        if !community.trim().is_empty() {
            parts.push(community.trim().to_string());
        }
    }
    if let Some(building) = &contact.building {
        if !building.trim().is_empty() {
            parts.push(building.trim().to_string());
        }
    }

    if parts.is_empty() {
        "暂无地址信息".to_string()
    } else {
        parts.join(" / ")
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

fn empty_dash(value: &str) -> String {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        "-".to_string()
    } else {
        trimmed.to_string()
    }
}
