use leptos::prelude::*;

#[component]
pub fn Pagination(
    current_page: RwSignal<usize>,
    page_size: RwSignal<usize>,
    #[prop(into)]
    total_items: Signal<u64>
) -> impl IntoView {
    let total_pages = move || (total_items.get() as f64 / page_size.get() as f64).ceil() as usize;

    // 处理页码变化
    let on_page_change = move |page: usize| {
        if page >= 1 && page <= total_pages() {
            current_page.set(page);
        }
    };

    // 处理每页数量变化
    let on_page_size_change = move |ev: leptos::ev::Event| {
        let value = event_target_value(&ev).parse::<usize>().unwrap_or(10);
        page_size.set(value);
        current_page.set(1); // 重置到第一页
    };

    // 生成页码按钮的逻辑
    let render_page_buttons = move || {
        let page_count = total_pages();
        let current = current_page.get();

        if page_count == 1 {
            vec![1]
        } else if page_count < 7 {
            (1..=page_count).collect()
        } else if current < 4 {
            let mut pages = vec![1, 2, 3, 4];
            pages.push(0); // 0 表示省略号
            pages.push(page_count);
            pages
        } else if current >= 4 && current < page_count - 3 {
            let mut pages = vec![1];
            pages.push(0); // 0 表示省略号
            pages.extend([current - 1, current, current + 1]);
            pages.push(0); // 0 表示省略号
            pages.push(page_count);
            pages
        } else {
            let mut pages = vec![1];
            pages.push(0); // 0 表示省略号
            pages.extend([page_count - 3, page_count - 2, page_count - 1, page_count]);
            pages
        }
    };

    view! {
        <div class="absolute bottom-4 flex flex-col sm:flex-row justify-between items-center mt-4 gap-4">
            {/* 每页显示数量选择 */}
            <div class="flex items-center gap-2">
                <span class="text-sm shrink-0">每页</span>
                <select
                    class="select select-bordered select-sm min-w-24"
                    on:change=on_page_size_change
                >
                    <option selected=move || page_size.get() == 10 value="10">10</option>
                    <option selected=move || page_size.get() == 20 value="20">20</option>
                    <option selected=move || page_size.get() == 50 value="50">50</option>
                </select>
                <span class="text-sm shrink-0">
                    {move || format!("共 {} 条记录", total_items.get())}
                </span>
            </div>

            {/* 分页导航 */}
            <div class="join">
                <button
                    class="join-item btn btn-sm"
                    on:click=move |_| on_page_change(1)
                    disabled=move || current_page.get() == 1
                >
                    "«"
                </button>
                <button
                    class="join-item btn btn-sm"
                    on:click=move |_| on_page_change(current_page.get() - 1)
                    disabled=move || current_page.get() == 1
                >
                    "‹"
                </button>

                // 显示页码按钮（带省略逻辑）
                {move || {
                    render_page_buttons().into_iter().map(|page| {
                        if page == 0 {
                            view! {
                                <button class="join-item btn btn-sm" disabled=true>
                                    "..."
                                </button>
                            }.into_any()
                        } else {
                            view! {
                                <button
                                    class=format!(
                                        "join-item btn btn-sm {}",
                                        if page == current_page.get() { "btn-active" } else { "" }
                                    )
                                    on:click=move |_| on_page_change(page)
                                >
                                    {page}
                                </button>
                            }.into_any()
                        }
                    }).collect_view()
                }}

                <button
                    class="join-item btn btn-sm"
                    on:click=move |_| on_page_change(current_page.get() + 1)
                    disabled=move || current_page.get() == total_pages()
                >
                    "›"
                </button>
                <button
                    class="join-item btn btn-sm"
                    on:click=move |_| on_page_change(total_pages())
                    disabled=move || current_page.get() == total_pages()
                >
                    "»"
                </button>
            </div>
        </div>
    }
}
