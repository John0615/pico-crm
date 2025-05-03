use leptos::{context::Provider, prelude::*};

#[derive(Clone)]
#[slot]
pub struct Column {
    pub label: String,
    #[prop(default = false)]
    freeze: bool,
    #[prop(optional, into)]
    pub class: Option<String>,
    pub children: ChildrenFn,
}

#[component]
pub fn DaisyTable<T: Clone + Send + Sync + 'static>(
    #[prop(optional)] _class: Option<String>,
    data: Vec<T>,
    columns: Vec<Column>,
) -> impl IntoView {
    let data = StoredValue::new(data.clone());
    let columns = StoredValue::new(columns.clone());
    view! {
        <table class="table table-pin-rows table-pin-cols whitespace-nowrap">
            <thead>
                <tr class="bg-base-200">
                    <For
                        // 关键修改2：使用 with_value 获取数据
                        each=move || columns.with_value(|c| c.clone().into_iter().enumerate())
                        key=|(index, _col)| *index
                        children=move |(_index, col)| {
                            if col.freeze {
                                view! {
                                    <th class=col.class.clone().unwrap_or_default()>
                                        {col.label.clone()}
                                    </th>
                                }.into_any()
                            } else {
                                view! {
                                    <td class=col.class.clone().unwrap_or_default()>
                                        {col.label.clone()}
                                    </td>
                                }.into_any()
                            }
                        }
                    />
                </tr>
            </thead>
            <tbody>
                <For
                    each=move || data.with_value(|d| d.clone().into_iter().enumerate())
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
            </tbody>
        </table>
    }
}

#[derive(Clone)]
struct User {
    id: u64,
    name: String,
    email: String,
}

#[component]
pub fn Demo() -> impl IntoView {
    let users = vec![
        User {
            id: 1,
            name: "Alice".into(),
            email: "alice@example.com".into(),
        },
        User {
            id: 2,
            name: "Bob".into(),
            email: "bob@example.com".into(),
        },
    ];

    view! {
        <DaisyTable data=users>
            <Column
                slot:columns
                label="ID".to_string()
            >
                {
                    let user: Option<User> = use_context::<User>();
                    view! {
                        <span>
                            {user.map(|u| u.id).unwrap_or(1)}
                        </span>
                    }
                }
            </Column>

            <Column
                slot:columns
                label="Name".to_string()
                class="font-bold"
            >
                {
                    let user: Option<User> = use_context::<User>();
                    view! {
                        <span>
                            {user.map(|u| u.name).unwrap_or("Guest".to_string())}
                        </span>
                    }
                }
            </Column>
            <Column
                slot:columns
                label="Name".to_string()
                class="font-bold"
            >
                {
                    let user: Option<User> = use_context::<User>();
                    view! {
                        <span>
                            {user.map(|u| u.email).unwrap_or("Guest".to_string())}
                        </span>
                    }
                }
            </Column>
        </DaisyTable>
    }
}
