use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::contact::Contact;
use leptos::logging;
use crate::components::features::ContactModal;


#[server]
pub async fn fetch_contacts() -> Result<Vec<Contact>, ServerFnError> {
    use backend::db::Database;
    let pool = expect_context::<Database>();
    println!("pool {:?}", pool);

    println!("Fetching contacts...");
    let contacts = backend::operations::contacts::fetch_contacts(&pool.connection).await.map_err(|e| ServerFnError::new(e))?;
    println!("Fetching contacts result {:?}", contacts);
    Ok(contacts)
}

#[component]
pub fn ContactsList() -> impl IntoView {
    let (sort_name_asc, set_sort_name_asc) = signal(true);
    let show_modal =  RwSignal::new(false);

    let sort_name = move || {
        set_sort_name_asc.update(|a| *a = !*a);
    };

    let data = Resource::new(
        move || sort_name_asc.get(),
        // every time `count` changes, this will run
        |_| async move {
            fetch_contacts()
                .await
                .unwrap_or_else(|e| {
                    logging::error!("Error loading contacts: {e}");
                    Vec::new()
                })
        }
    );

    view! {
        <div class="p-6">
          <div class="flex flex-col md:flex-row gap-4 mb-6">
            <div class="flex-1">
              <div class="join w-full">
                <input type="text" placeholder="搜索客户..." class="input input-bordered join-item w-full" />
                <button class="btn btn-primary join-item">
                  <svg xmlns="http://www.w3.org/2000/svg" class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 21l-6-6m2-5a7 7 0 11-14 0 7 7 0 0114 0z" />
                  </svg>
                </button>
              </div>
            </div>
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
        <ContactModal show=show_modal />
          <div class="overflow-x-auto h-[calc(100vh-250px)] bg-base-100 rounded-lg shadow">
            <table class="table table-pin-rows">
              <thead>
                <tr class="bg-base-200">
                  <th class="cursor-pointer hover:bg-base-300" on:click=move |_| sort_name()>
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
                  <th>公司</th>
                  <th>职位</th>
                  <th>电话</th>
                  <th>邮箱</th>
                  <th class="cursor-pointer hover:bg-base-300" onclick="sortTable(5)">
                    状态
                    <span class="ml-1 inline-block">"↑↓"</span>
                  </th>
                  <th class="cursor-pointer hover:bg-base-300" onclick="sortTable(6)">
                    最后联系
                    <span class="ml-1 inline-block">"↑↓"</span>
                  </th>
                  <th>价值等级</th>
                  <th class="text-right">操作</th>
                </tr>
              </thead>
              <tbody>
                <Suspense
                    fallback=move || view! {
                        <tr class="h-[calc(100vh-300px)]">
                            <td colspan="9" class="h-32 text-center align-middle">
                                <span class="loading loading-bars loading-xl"></span>
                            </td>
                        </tr>
                    }
                >
                    <Show
                        when=move || !data.with(|d| d.as_ref().map_or(true, |v| v.is_empty()))
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
                        each=move || data.get().unwrap_or_default()
                        key=|contact| contact.contact_uuid.clone()
                        children=move |contact: Contact| {
                            let stars = contact.value_level;
                            let status = contact.status.clone();
                            view! {
                                <tr class="hover:bg-base-100">
                                    <td class="font-medium">{contact.user_name.clone()}</td>
                                    <td>{contact.company}</td>
                                    <td>{contact.position}</td>
                                    <td>{contact.phone_number}</td>
                                    <td>{contact.email}</td>
                                    <td>
                                        <span class=format!("badge {}",
                                            match status {
                                                1 => "badge-success",
                                                0 => "badge-warning",
                                                _ => "badge-error",
                                            }
                                        )>
                                            {status.clone()}
                                        </span>
                                    </td>
                                    <td>{contact.last_contact}</td>
                                    <td>
                                        <div class="rating rating-sm">
                                            {(0..5).map(|i| {
                                                view! {
                                                    <input
                                                        type="radio"
                                                        name=format!("rating-{}", contact.user_name)
                                                        class="mask mask-star bg-yellow-400"
                                                        checked=i < stars
                                                    />
                                                }
                                            }).collect_view()}
                                        </div>
                                    </td>
                                    <td>
                                        <div class="flex justify-end gap-1">
                                            <a href=format!("/contacts/{}", contact.user_name) class="btn btn-ghost btn-xs">查看</a>
                                            <button class="btn btn-ghost btn-xs">
                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 5v.01M12 12v.01M12 19v.01M12 6a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2zm0 7a1 1 0 110-2 1 1 0 010 2z" />
                                                </svg>
                                            </button>
                                        </div>
                                    </td>
                                </tr>
                            }
                        }
                    />
                    </Show>

                </Suspense>


              </tbody>
            </table>
          </div>

          <div class="absolute bottom-4 flex flex-col sm:flex-row justify-between items-center mt-4 gap-4">
            <div class="flex items-center gap-2">
              <span class="text-sm shrink-0">每页</span>
              <select class="select select-bordered select-sm min-w-24">
                <option>10</option>
                <option selected>20</option>
                <option>50</option>
              </select>
              <span class="text-sm shrink-0">共 128 条记录</span>
            </div>

            <div class="join">
              <button class="join-item btn btn-sm">"«"</button>
              <button class="join-item btn btn-sm">1</button>
              <button class="join-item btn btn-sm btn-active">2</button>
              <button class="join-item btn btn-sm">3</button>
              <button class="join-item btn btn-sm">"»"</button>
            </div>


          </div>
        </div>
    }
}
