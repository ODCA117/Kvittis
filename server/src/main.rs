mod api;
mod cli;
mod db;
mod logger;
mod state;
mod types;

use crate::db::db_sqlite::SqliteStore;
use crate::state::AppState;
use crate::{api::delete_user, db::db_file::FileStore};
use axum::{
    routing::{delete, get, post},
    Router,
};
use clap::Parser;
use std::{net::SocketAddr, path::Path};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info; // adjust if crate name differs

use crate::api::{
    add_friend, create_expense, create_group, delete_group, get_group, get_group_balances, get_user, get_user_balances, get_users, new_group_member, register_user, search_groups, search_users
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init();
    let args = cli::Args::parse();
    println!("args: {:?}", args);

    /* Data path */
    // Connect to database
    let data_dir = Path::new(&args.data_dir);
    let state = match args.db_type {
        cli::DbType::Sql => {
            let db = SqliteStore::connect(data_dir.join("store.db").to_string_lossy().as_ref())
                .await
                .expect("failed to open db");
            AppState::new(db)
        }
        cli::DbType::File => {
            let db = FileStore::connect(&data_dir.join("store.json"))
                .await
                .expect("Cannot open db");
            AppState::new(db)
        }
    };

    let app = Router::new()
        .route("/register", post(register_user))
        .route("/user/{user_id}", get(get_user))
        .route("/user/{user_id}", delete(delete_user))
        .route("/users", get(get_users))
        .route("/search_user", post(search_users))
        .route("/friend", post(add_friend))
        .route("/create_group", post(create_group))
        .route("/group/{group_id}", get(get_group))
        .route("/search_group", post(search_groups))
        .route("/new_group_member", post(new_group_member))
        .route("/group/{group_id}", delete(delete_group))
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
