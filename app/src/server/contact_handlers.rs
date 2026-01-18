use leptos::prelude::*;
use server_fn::ServerFnError;
use shared::{
    contact::{Contact, ContactQuery, UpdateContact},
    ListResult,
};

#[cfg(feature = "ssr")]
mod ssr {
    pub use backend::application::{
        commands::contact_service::ContactAppService,
        queries::contact_service::ContactAppService as ContactQueryService,
    };
    pub use backend::infrastructure::db::Database;
    pub use backend::infrastructure::{
        queries::contact_query_impl::SeaOrmContactQuery,
        repositories::contact_repository_impl::SeaOrmContactRepository,
    };
}

// 获取联系人列表
#[server(
    name = FetchContactsFn,
    prefix = "/api",
    endpoint = "/fetch_contacts",
)]
pub async fn fetch_contacts(params: ContactQuery) -> Result<ListResult<Contact>, ServerFnError> {
    use self::ssr::*;

    // 认证检查已由中间件统一处理，这里可以安全地获取用户信息
    let _user = use_context::<shared::user::User>();

    let pool = expect_context::<Database>();
    let contact_query = SeaOrmContactQuery::new(pool.connection.clone());
    let app_service = ContactQueryService::new(contact_query);

    println!("pool {:?}", pool);
    println!("Fetching contacts...");

    let res = app_service
        .fetch_contacts(params)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    println!("Fetching contacts result {:?}", res);
    Ok(res)
}

// 获取单个联系人
#[server(
    name = GetContactFn,
    prefix = "/api",
    endpoint = "/get_contact",
)]
pub async fn get_contact(uuid: String) -> Result<Option<Contact>, ServerFnError> {
    use self::ssr::*;

    let pool = expect_context::<Database>();
    let contact_query = SeaOrmContactQuery::new(pool.connection.clone());
    let app_service = ContactQueryService::new(contact_query);

    println!("fetch contact uuid: {:?}", uuid);
    let result = app_service
        .fetch_contact(uuid)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    println!("fetch contact result: {:?}", result);

    Ok(result)
}

// 创建联系人
#[server(
    name = CreateContactFn,
    prefix = "/api",
    endpoint = "/create_contact",
)]
pub async fn create_contact(contact: Contact) -> Result<(), ServerFnError> {
    use self::ssr::*;

    let pool = expect_context::<Database>();
    let contact_repository = SeaOrmContactRepository::new(pool.connection.clone());
    let app_service = ContactAppService::new(contact_repository);

    println!("Adding contact: {:?}", contact);
    let result = app_service
        .create_contact(contact)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    println!("Adding contact results: {:?}", result);

    Ok(())
}

// 更新联系人
#[server(
    name = UpdateContactFn,
    prefix = "/api",
    endpoint = "/update_contact",
)]
pub async fn update_contact(contact: UpdateContact) -> Result<(), ServerFnError> {
    use self::ssr::*;

    let pool = expect_context::<Database>();
    let contact_repository = SeaOrmContactRepository::new(pool.connection.clone());
    let app_service = ContactAppService::new(contact_repository);

    println!("editing contact: {:?}", contact);
    let result = app_service
        .update_contact(contact)
        .await
        .map_err(|e| ServerFnError::new(e))?;
    println!("editing contact results: {:?}", result);

    Ok(())
}

// 删除联系人
#[server(
    name = DeleteContactFn,
    prefix = "/api",
    endpoint = "/delete_contact",
)]
pub async fn delete_contact(uuid: String) -> Result<(), ServerFnError> {
    use self::ssr::*;

    // 认证检查已由中间件统一处理
    let _user = use_context::<shared::user::User>();

    let pool = expect_context::<Database>();
    let contact_repository = SeaOrmContactRepository::new(pool.connection.clone());
    let app_service = ContactAppService::new(contact_repository);

    println!("pool {:?}", pool);
    println!("Deleting contact...");

    let res = app_service
        .delete_contact(uuid)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    println!("Deleting contact result {:?}", res);
    Ok(res)
}

// 导出联系人
#[server(
    name = ExportContactsFn,
    prefix = "/api",
    endpoint = "/export_contacts",
)]
pub async fn export_contacts(params: ContactQuery) -> Result<Vec<u8>, ServerFnError> {
    use self::ssr::*;

    // 认证检查已由中间件统一处理
    let _user = use_context::<shared::user::User>();

    let pool = expect_context::<Database>();
    let contact_query = SeaOrmContactQuery::new(pool.connection.clone());
    let app_service = ContactQueryService::new(contact_query);

    println!("pool {:?}", pool);
    println!("Fetching contacts...");

    let excel_data = app_service
        .fetch_contacts_excel_data(params)
        .await
        .map_err(|e| ServerFnError::new(e))?;

    // println!("Fetching contacts excel_data {:?}", excel_data);

    Ok(excel_data)
}