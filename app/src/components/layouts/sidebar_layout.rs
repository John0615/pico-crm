use crate::components::features::{Navbar, Sidebar};
use leptos::prelude::*;
use leptos_router::components::Outlet;

#[component]
pub fn SidebarLayout() -> impl IntoView {
    view! {
        <div class="h-screen bg-base-100 relative overflow-hidden">
            <Sidebar />

            <div class="flex h-screen flex-col transition-all duration-300 sm:ps-60 sm:overlay-minified:ps-16">
                <Navbar />

                <main class="p-4 flex-1 overflow-y-auto">
                    <Outlet/>
                </main>
            </div>
        </div>
    }
}
