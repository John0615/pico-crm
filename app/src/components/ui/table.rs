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

pub trait Identifiable {
    fn id(&self) -> String;
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
pub fn DaisyTable<T: Clone + Send + Sync + Identifiable + 'static>(
    #[prop(optional)] _class: Option<String>,
    data: Resource<(Vec<T>, u64)>,
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
            <Transition
                fallback=move || view! {
                    <tbody>
                        <tr class="h-[calc(100vh-300px)]">
                            <td colspan="9" class="h-32 text-center align-middle">
                                <span class="loading loading-bars loading-xl"></span>
                            </td>
                        </tr>
                    </tbody>
                }
            >
                <tbody>
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
                            key=|(_index, item)| item.id()
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
                </tbody>
            </Transition>
        </table>
    }
}
