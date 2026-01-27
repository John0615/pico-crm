use leptos::prelude::*;
use leptos_meta::{provide_meta_context, MetaTags, Stylesheet, Title};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    path, Lazy, StaticSegment,
};

use components::layouts::*;
use components::ui::{message_box::MessageBox, toast::Toast};
use lazy_routes::{
    AdminUsersRoute, ContactsListRoute, DashboardRoute, LoginRoute,
    SystemSettingsRoute,
};
use pages::NotFoundPage;

pub mod components;
mod lazy_routes;
pub mod pages;
pub mod server;
pub mod utils;

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/start-axum-workspace.css"/>
        <Stylesheet href="https://cdnjs.cloudflare.com/ajax/libs/font-awesome/6.4.0/css/all.min.css" />

        // sets the document title
        <Title text="PicoCRM"/>
        <Toast/>
        <MessageBox/>
        // content for this welcome page
        <Router>
            <main>
                <Routes fallback=|| view! { <NotFoundPage /> }>
                    <Route path=path!("/login") view={Lazy::<LoginRoute>::new()}/>
                    <ParentRoute path=StaticSegment("") view=SidebarLayout>
                        <Route path=StaticSegment("") view={Lazy::<DashboardRoute>::new()}/>
                        <ParentRoute path=path!("/contacts") view=|| view! {
                            <Outlet/>
                        }>
                            <Route path=path!("") view={Lazy::<ContactsListRoute>::new()}/>
                        </ParentRoute>
                        <ParentRoute path=path!("/admin") view=|| view! {
                            <Outlet/>
                        }>
                            <Route path=path!("") view={Lazy::<SystemSettingsRoute>::new()}/>
                            <Route path=path!("/users") view={Lazy::<AdminUsersRoute>::new()}/>
                            <Route path=path!("/settings") view={Lazy::<SystemSettingsRoute>::new()}/>
                        </ParentRoute>
                    </ParentRoute>
                </Routes>
            </main>
        </Router>
    }
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <h1 class="bg-blue-500">"Welcome to Leptos!"</h1>
        <button on:click=on_click>"点击： " {count}</button>
    }
}
