use leptos::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq)]
struct TableUser {
    username: String,
    company: String,
    position: String,
    phone_number: String,
    email: String,
    status: String,
    last_contact: String,
    value_level: String,
}

#[component]
pub fn ContactsList() -> impl IntoView {
    let (data, set_data) = signal(vec![
        TableUser {
            username: "张三".to_string(),
            company: "ABC科技".to_string(),
            position: "技术总监".to_string(),
            phone_number: "13800138000".to_string(),
            email: "zhangsan@abc.com".to_string(),
            status: "已签约".to_string(),
            last_contact: "2023-10-20".to_string(),
            value_level: "3".to_string(), // 改为数字表示星级
        },
        TableUser {
            username: "李四".to_string(),
            company: "XYZ贸易".to_string(),
            position: "销售经理".to_string(),
            phone_number: "13900139000".to_string(),
            email: "lisi@xyz.com".to_string(),
            status: "待跟进".to_string(),
            last_contact: "2023-10-18".to_string(),
            value_level: "2".to_string(), // 改为数字表示星级
        },
    ]);

    let (sort_name_asc, set_sort_name_asc) = signal(true);
    let sort_name = move || {
        set_sort_name_asc.update(|a| *a = !*a);
    };

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
                  <For
                    each=move || data.get()
                    key=|contact| contact.username.clone()
                    children=move |contact: TableUser| {
                        let stars = contact.value_level.parse::<usize>().unwrap_or(0);
                        let status = contact.status.clone();
                        view! {
                            <tr class="hover:bg-base-100">
                                <td class="font-medium">{contact.username.clone()}</td>
                                <td>{contact.company}</td>
                                <td>{contact.position}</td>
                                <td>{contact.phone_number}</td>
                                <td>{contact.email}</td>
                                <td>
                                    <span class=format!("badge {}",
                                        match status.as_str() {
                                            "已签约" => "badge-success",
                                            "待跟进" => "badge-warning",
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
                                                    name=format!("rating-{}", contact.username)
                                                    class="mask mask-star bg-yellow-400"
                                                    checked=i < stars
                                                />
                                            }
                                        }).collect_view()}
                                    </div>
                                </td>
                                <td>
                                    <div class="flex justify-end gap-1">
                                        <a href=format!("/contacts/{}", contact.username) class="btn btn-ghost btn-xs">查看</a>
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
