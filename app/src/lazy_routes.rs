use leptos::prelude::*;
use leptos_router::{lazy_route, LazyRoute};

use crate::pages::admin::{
    AdminAnalytics, AdminMerchants, AdminUsers, SystemSettings, TenantMaintenance,
};
use crate::pages::{ContactsList, Login, ServiceRequestsPage, OrdersPage, SchedulesPage};

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
pub struct ServiceRequestsRoute;

#[lazy_route]
impl LazyRoute for ServiceRequestsRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;
        view! { <ServiceRequestsPage/> }.into_any()
    }
}

#[derive(Debug)]
pub struct OrdersRoute;

#[lazy_route]
impl LazyRoute for OrdersRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;
        view! { <OrdersPage/> }.into_any()
    }
}

#[derive(Debug)]
pub struct SchedulesRoute;

#[lazy_route]
impl LazyRoute for SchedulesRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;
        view! { <SchedulesPage/> }.into_any()
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

#[derive(Debug)]
pub struct AdminAnalyticsRoute;

#[lazy_route]
impl LazyRoute for AdminAnalyticsRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;
        view! { <AdminAnalytics/> }.into_any()
    }
}

#[derive(Debug)]
pub struct TenantMaintenanceRoute;

#[lazy_route]
impl LazyRoute for TenantMaintenanceRoute {
    fn data() -> Self {
        Self
    }

    fn view(this: Self) -> AnyView {
        let _ = this;
        view! { <TenantMaintenance/> }.into_any()
    }
}
