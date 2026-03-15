mod api;
mod cli;
mod db;
mod logger;
mod state;
mod types;

use crate::db::db_sqlite::SqliteStore;
use crate::state::AppState;
use crate::{api::unauthorized_user_handler, db::db_file::FileStore};
use axum::{Router, routing::post};
use clap::Parser;
use dotenv::dotenv;
use std::{net::SocketAddr, path::Path};
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::api::{authorized_user_handler, balance_handler, expense_handler, group_handler};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    logger::init();
    let args = cli::Args::parse();
    println!("args: {:?}", args);

    dotenv().ok();

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

    /* Install cryptop provider */
    // CryptoProvider::install_default(&'static self)
    // CryptoProvider::install_default().expect("Failed to install default CryptoProvider");

    /* API routes */
    let app = Router::new()
        .route("/api/auth_user", post(authorized_user_handler)) // TODO: Learn how this just works
        .route("/api/unauth_user", post(unauthorized_user_handler)) // TODO: Learn how this just works
        .route("/api/group", post(group_handler))
        .route("/api/expense", post(expense_handler))
        .route("/api/balance", post(balance_handler))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()))
        .with_state(state.clone());

    let ip = args.ip.parse().expect("Failed to parse IP address");
    let addr = SocketAddr::new(ip, args.port);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    println!("Listening on {addr}");

    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            let _ = tokio::signal::ctrl_c().await;
            info!("Shutdown");
        })
        .await?;

    Ok(())
}
