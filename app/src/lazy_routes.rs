use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};

use crate::pages::admin::{AdminUsers, AdminMerchants, SystemSettings};
use crate::pages::{ContactsList, Dashboard, Login};

#[derive(Debug)]
pub struct LoginRoute;

#[lazy_route]
impl LazyRoute for LoginRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;
        view! { <Login/> }.into_any()
    }
}

#[derive(Debug)]
pub struct DashboardRoute;

#[lazy_route]
impl LazyRoute for DashboardRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;
        view! { <Dashboard/> }.into_any()
    }
}

#[derive(Debug)]
pub struct ContactsListRoute;

#[lazy_route]
impl LazyRoute for ContactsListRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;
        view! { <ContactsList/> }.into_any()
    }
}

#[derive(Debug)]
pub struct UsersRoute;

#[lazy_route]
impl LazyRoute for UsersRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;
        view! { <AdminUsers/> }.into_any()
    }
}

#[derive(Debug)]
pub struct AdminMerchantsRoute;

#[lazy_route]
impl LazyRoute for AdminMerchantsRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;
        view! { <AdminMerchants/> }.into_any()
    }
}

#[derive(Debug)]
pub struct SystemSettingsRoute;

#[lazy_route]
impl LazyRoute for SystemSettingsRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;
        view! { <SystemSettings/> }.into_any()
    }
}
