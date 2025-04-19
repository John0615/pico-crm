use crate::components::features::{Navbar, Sidebar};
use leptos::prelude::*;
use leptos_router::components::Outlet;

#[component]
pub fn SidebarLayout() -> impl IntoView {
    view! {
        <div class="drawer lg:drawer-open">
            <input id="sidebar-toggle" type="checkbox" class="drawer-toggle" />

            <div class="drawer-content flex flex-col">
               <Navbar />

                <main class="p-4 flex-1">
                    <Outlet/>
                </main>
            </div>

            <Sidebar />
        </div>
    }
}
