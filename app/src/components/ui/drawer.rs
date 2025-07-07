use leptos::prelude::*;

#[component]
pub fn DaisyDrawer(
    // 抽屉的唯一ID
    id: &'static str,
    // 抽屉宽度(px)
    width: i32,
    // 位置 ("left"或"right")
    position: &'static str,
    // 控制开关状态的信号
    is_open: RwSignal<bool>,
    // 抽屉内容
    children: Children,
) -> impl IntoView {
    let position_class = match position {
        "left" => "drawer-start",
        "right" => "drawer-end",
        _ => "drawer-end",
    };

    view! {
        <div class=format!("drawer {}", position_class)>
            <input
                id=id
                type="checkbox"
                class="drawer-toggle"
                checked=is_open
                // 这个on:change是为了支持原生点击遮罩层关闭
                on:change=move |ev| is_open.set(event_target_checked(&ev))
            />
            // 抽屉内容容器 (空容器，内容完全由外部控制)
            <div class="drawer-content"></div>

            // 抽屉侧边栏
            <div class="drawer-side z-50">
                <label
                    for=id
                    aria-label="close sidebar"
                    class="drawer-overlay"
                    on:click=move |_| is_open.set(false)
                ></label>
                <ul
                    class="menu bg-base-200 text-base-content min-h-full p-4"
                    style=format!("width: {}px", width)
                >
                    {children()}
                </ul>
            </div>
        </div>
    }
}
