mod api;
mod cli;
mod db;
mod logger;
mod state;
mod types;

use crate::db::db_file::{FileStore};
use crate::{
    api::{get_user, get_users},
    state::AppState,
};
use axum::{
    Router,
    routing::{get, post},
};
use clap::Parser;
use tracing::info;
use std::{net::SocketAddr, path::Path};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer; // adjust if crate name differs

use crate::api::{
    add_friend, create_expense, create_group, get_group_balances, get_user_balances, register_user,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init();
    let args = cli::Args::parse();
    println!("args: {:?}", args);

    /* Data path */
    // Connect to database
    let data_dir = Path::new(&args.data_dir);
    let store;
    match args.db_type {
        // cli::DbType::Sql => {
            // user_db = UserSqlDB::connect(&data_dir.join("user_db.json")).expect("Cannot open user db");
            // group_db =
            //     GroupSqlDB::connect(&data_dir.join("group_db.json")).expect("Cannot open group db");
            // expense_db =
            //     ExpenseSqlDB::connect(&data_dir.join("expense_db.json")).expect("Cannot open expense db");
        // },
        cli::DbType::File => {
            store =
                FileStore::connect(&data_dir.join("store.json")).await.expect("Cannot open db");
        },
    }

    let state = AppState::new(store);

    let app = Router::new()
        .route("/register", post(register_user))
        .route("/user/{user_id}", get(get_user))
        .route("/users", get(get_users))
        .route("/friend", post(add_friend))
        .route("/group", post(create_group))
        .route("/expense", post(create_expense))
        .route("/balances/{user_id}", get(get_user_balances))
        .route("/group_balances/{group_id}", get(get_group_balances))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(state.clone());

    let ip = args.ip.parse().expect("Failed to parse IP address");
    let addr = SocketAddr::new(ip, args.port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on {addr}");
    // let shutdown_state = state.clone();

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("Shutdown");
            // if let Err(e) = shutdown_state.commit_all() {
            //     eprintln!("Failed to commit databases: {e}");
            // } else {
            //     println!("Databases committed (Ctrl+C).");
            // }
        })
        .await?;

    Ok(())
}
