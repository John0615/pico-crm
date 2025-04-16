use axum::Router;
use leptos::prelude::*;
use leptos_axum::{generate_route_list, LeptosRoutes};
use app::*;
use leptos::logging::log;
use std::env;
use migration::{Migrator, MigratorTrait};
use backend::db::Database;



#[tokio::main]
async fn main() {

    // 加载.env文件
    let env = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());
    println!("当前环境: {}", env);
    let env_file = format!(".env.{}", env);
    dotenvy::from_filename(&env_file).unwrap_or_else(|_| panic!("无法读取 {} 文件", env_file));
    // 数据库相关
    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is not set in .env file");
    println!("db_url: {:?}", db_url);
    let db = Database::new().await;
    Migrator::up(db.get_connection(), None).await.unwrap();

    let conf = get_configuration(None).unwrap();
    let addr = conf.leptos_options.site_addr;
    let leptos_options = conf.leptos_options;
    // Generate the list of routes in your Leptos App
    let routes = generate_route_list(App);

    let app = Router::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || {
                // 如果需要，可以在这里提供上下文
                provide_context(db.clone());
                shell(leptos_options.clone())
            }
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    // run our app with hyper
    // `axum::Server` is a re-export of `hyper::Server`
    log!("listening on http://{}", &addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
