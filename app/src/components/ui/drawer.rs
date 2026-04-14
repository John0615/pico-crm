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
    children: ChildrenFn,
) -> impl IntoView {
    let is_left = position == "left";
    let panel_position = if is_left { "left-0" } else { "right-0" };
    let panel_translate = if is_left {
        "-translate-x-full"
    } else {
        "translate-x-full"
    };

    view! {
        <div class="relative">
            <div
                class=move || {
                    if *is_open.read() {
                        "fixed inset-0 z-[60] bg-black/30 opacity-100 transition-opacity"
                    } else {
                        "fixed inset-0 z-[60] bg-black/30 opacity-0 pointer-events-none transition-opacity"
                    }
                }
                on:click=move |_| is_open.set(false)
            ></div>

            <aside
                id=id
                class=move || {
                    let base = format!(
                        "fixed top-0 {} z-[70] h-screen transition-transform duration-300 ease-in-out",
                        panel_position
                    );
                    if *is_open.read() {
                        format!("{base} translate-x-0")
                    } else {
                        format!("{base} {panel_translate}")
                    }
                }
                style=format!("width: {}px", width)
                aria-hidden=move || (!*is_open.read()).to_string()
            >
                <div class="h-full bg-base-100 shadow-lg overflow-y-auto">
                    <div class="p-4">
                        {children()}
                    </div>
                </div>
            </aside>
        </div>
    }
}
