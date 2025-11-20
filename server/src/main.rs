mod logger;
mod types;
mod state;
mod db;
mod api;

use axum::{
    Router,
    routing::{get, post},
};
use parking_lot::RwLock;
use std::sync::Arc;

use crate::{api::{
    AppState, add_friend, create_expense, create_group, get_group_balances, get_user_balances,
    register, try_load_state,
}, db::DataBase};

#[tokio::main]
async fn main() {
    logger::init();

    // Connect to database
    let user_db = db::UserFileDB::connect("user_db.json").expect("Cannot open user db");
    let group_db = db::GroupFileDB::connect("group_db.json").expect("Cannot open group db");
    let expense_db = db::ExpenseFileDB::connect("expense_db.json").expect("Cannot open expense db");

    // persist file path (optional)
    let persist_path = Some("kvittis_state.json".to_string());

    // attempt load
    let loaded = persist_path
        .as_ref()
        .and_then(|p| try_load_state(p))
        .unwrap_or_default();

    let state = AppState {
        data: Arc::new(RwLock::new(loaded)),
        persist_path,
    };

    let app = Router::new()
        .route("/register", post(register))
        .route("/friend", post(add_friend))
        .route("/group", post(create_group))
        .route("/expense", post(create_expense))
        .route("/balances/{user_id}", get(get_user_balances))
        .route("/group_balances/{group_id}", get(get_group_balances))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", "0.0.0.0:3000");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}
