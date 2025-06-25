// use leptos::logging;
use leptos::{context::Provider, prelude::*};

#[derive(Clone)]
#[slot]
pub struct Column {
    pub label: String,
    #[prop(default = false)]
    freeze: bool,
    #[prop(default = false)]
    sort: bool,
    #[prop(default = SortValue::Asc)]
    default_sort: SortValue,
    prop: String,
    #[prop(optional, into)]
    pub class: Option<String>,
    pub children: ChildrenFn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortValue {
    Asc,
    Desc,
}

impl SortValue {
    pub fn reverse(&self) -> Self {
        match self {
            SortValue::Asc => SortValue::Desc,
            SortValue::Desc => SortValue::Asc,
        }
    }
}

#[component]
pub fn ColumnSorter(
    /// 初始排序值 (可选，默认为 Asc)
    #[prop(optional)]
    initial_sort: Option<SortValue>,
    /// 排序变化时的回调函数
    #[prop(optional)]
    on_change: Option<impl Fn(SortValue) + 'static>,
) -> impl IntoView {
    // 内部状态管理
    let (sort_value, set_sort_value) = signal(initial_sort.unwrap_or(SortValue::Asc));

    // 点击处理函数
    let handle_click = move |_| {
        let new_value = sort_value.get().reverse();
        set_sort_value.set(new_value);
        if let Some(f) = on_change.as_ref() {
            f(new_value);
        }
    };

    let is_asc = move || sort_value.get() == SortValue::Asc;

    view! {
        <span
            class="ml-1 inline-block cursor-pointer select-none"
            on:click=handle_click
        >
            // 上箭头，当升序时高亮
            <span class:text-blue-500=is_asc class:opacity-30=move || !is_asc()>
                "↑"
            </span>
            // 下箭头，当降序时高亮
            <span class:text-blue-500=move || !is_asc() class:opacity-30=is_asc>
                "↓"
            </span>
        </span>
    }
}

#[component]
pub fn DaisyTable<T: Clone + Send + Sync + 'static>(
    #[prop(optional)] _class: Option<String>,
    // data: Vec<T>,
    data: Resource<(Vec<T>, usize)>,
    columns: Vec<Column>,
    #[prop(optional)] on_sort: Option<Callback<(String, SortValue)>>,
) -> impl IntoView {
    // let data = StoredValue::new(data.clone());
    let columns = StoredValue::new(columns.clone());
    let sort_change = move |prop, sort_value| {
        if let Some(cb) = on_sort {
            cb.run((prop, sort_value));
        }
    };
    view! {
        <table class="table table-pin-rows table-pin-cols whitespace-nowrap">
            <thead>
                <tr class="bg-base-200">
                    <For
                        // 关键修改2：使用 with_value 获取数据
                        each=move || columns.with_value(|c| c.clone().into_iter().enumerate())
                        key=|(index, _col)| *index
                        children=move |(_index, col)| {
                            let cell_content = view! {

                                {col.label.clone()}
                                {col.sort.then(|| view! {
                                    <ColumnSorter
                                        initial_sort=col.default_sort
                                        on_change=move|sort_value: SortValue| {
                                            sort_change(col.prop.clone(), sort_value);
                                        }
                                    />
                                })}

                            };

                            if col.freeze {
                                view! { <th class=col.class.clone().unwrap_or_default()>{cell_content}</th> }.into_any()
                            } else {
                                view! { <td class=col.class.clone().unwrap_or_default()>{cell_content}</td> }.into_any()
                            }

                        }
                    />
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
                        when=move || !data.get().map(|d| d.0.is_empty()).unwrap_or_default()
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
                            each=move || data.get().map(|d| d.0.clone().into_iter().enumerate()).unwrap_or_default()
                            key=|(index, _)| *index
                            children=move |(_index, row)| {
                                view! {
                                    <Provider value=row>
                                        <tr>
                                            <For
                                                each=move || columns.with_value(|cols|
                                                    cols.clone().into_iter().enumerate()
                                                )
                                                key=|(index, _)| *index
                                                children=move |(_, col)| {
                                                    if col.freeze {
                                                        view! {
                                                            <th class=col.class.clone().unwrap_or_default()>
                                                                {(col.children)()}
                                                            </th>
                                                        }.into_any()
                                                    } else {
                                                        view! {
                                                            <td class=col.class.clone().unwrap_or_default()>
                                                                {(col.children)()}
                                                            </td>
                                                        }.into_any()
                                                    }
                                                }
                                            />
                                        </tr>
                                    </Provider>
                                }
                            }
                        />
                    </Show>
                </Transition>
            </tbody>
        </table>
    }
}

// #[derive(Clone)]
// struct User {
//     contact_uuid: String,
//     user_name: String,
//     company: String,
//     position: String,
//     phone_number: String,
//     email: String,
//     last_contact: String,
//     value_level: i32,
//     status: i32,
//     inserted_at: String,
//     updated_at: String,
// }

// #[component]
// pub fn Demo() -> impl IntoView {
//     let users: Vec<User> = vec![
//         User {
//             contact_uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
//             user_name: "张三".to_string(),
//             company: "阿里巴巴".to_string(),
//             position: "高级工程师".to_string(),
//             phone_number: "13800138000".to_string(),
//             email: "zhangsan@alibaba.com".to_string(),
//             last_contact: "2023-10-15T09:30:00Z".to_string(),
//             value_level: 3,
//             status: 1,
//             inserted_at: "2023-09-01T08:00:00Z".to_string(),
//             updated_at: "2023-10-15T10:00:00Z".to_string(),
//         },
//         User {
//             contact_uuid: "6ba7b810-9dad-11d1-80b4-00c04fd430c8".to_string(),
//             user_name: "李四".to_string(),
//             company: "腾讯科技".to_string(),
//             position: "产品经理".to_string(),
//             phone_number: "13900139000".to_string(),
//             email: "lisi@tencent.com".to_string(),
//             last_contact: "2023-11-20T14:15:00Z".to_string(),
//             value_level: 2,
//             status: 0,
//             inserted_at: "2023-08-15T10:30:00Z".to_string(),
//             updated_at: "2023-11-20T15:30:00Z".to_string(),
//         },
//         User {
//             contact_uuid: "1b9d6bcd-bbfd-4b2d-9b5d-ab8dfbbd4bed".to_string(),
//             user_name: "王五".to_string(),
//             company: "字节跳动".to_string(),
//             position: "数据分析师".to_string(),
//             phone_number: "13700137000".to_string(),
//             email: "wangwu@bytedance.com".to_string(),
//             last_contact: "2023-12-05T16:45:00Z".to_string(),
//             value_level: 4,
//             status: 1,
//             inserted_at: "2023-10-10T13:20:00Z".to_string(),
//             updated_at: "2023-12-05T17:00:00Z".to_string(),
//         },
//     ];

//     view! {
//         <DaisyTable data=users>
//             <Column
//                 slot:columns
//                 freeze=true
//                 sort=true
//                 prop="user_name".to_string()
//                 label="姓名".to_string()
//                 class="font-bold"
//             >
//                 {
//                     let user: Option<User> = use_context::<User>();
//                     view! {
//                         <span>
//                             {user.map(|u| u.user_name).unwrap_or("Guest".to_string())}
//                         </span>
//                     }
//                 }
//             </Column>
//             <Column
//                 slot:columns
//                 label="公司".to_string()
//                 prop="company".to_string()
//                 class="font-bold"
//             >
//                 {
//                     let user: Option<User> = use_context::<User>();
//                     view! {
//                         <span>
//                             {user.map(|u| u.company).unwrap_or("Guest".to_string())}
//                         </span>
//                     }
//                 }
//             </Column>
//             <Column
//                 slot:columns
//                 freeze=true
//                 label="操作".to_string()
//                 prop="".to_string()
//                 class="font-bold"
//             >
//                 {
//                     let user: Option<User> = use_context::<User>();
//                     view! {
//                         <span>
//                             查看
//                         </span>
//                     }
//                 }
//             </Column>
//         </DaisyTable>
//     }
// }
