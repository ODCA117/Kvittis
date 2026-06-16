// Balance-related API handlers

use axum::{Json, extract::State, http::StatusCode};
use common::api::{ApiResponse, BalanceRequest};
use tracing::debug;

use super::auth::{json_error, json_success};
use crate::state::AppState;

// ── Balance handler ───────────────────────────────────────────────────────────

pub async fn balance_handler(
    State(state): State<AppState>,
    Json(payload): Json<BalanceRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match payload {
        BalanceRequest::User { user_id } => {
            debug!("Get user balances: {:?}", user_id);
            match state.get_user_non_group_balances(user_id).await {
                Ok(balances) => {
                    json_success(StatusCode::OK, serde_json::to_value(balances).unwrap())
                }
                Err(_) => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to get user balances",
                ),
            }
        }
        BalanceRequest::Group { group_id } => {
            debug!("Get group balances: {:?}", group_id);
            match state.get_group_balance_overview(group_id).await {
                Ok(transfers) => {
                    json_success(StatusCode::OK, serde_json::to_value(transfers).unwrap())
                }
                Err(_) => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to get group balances",
                ),
            }
        }
    }
}
