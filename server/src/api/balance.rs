// Balance-related API handlers

use axum::{Json, extract::State, http::StatusCode};
use common::api::{ApiResponse, BalanceRequest};
use tracing::debug;

use super::auth::{json_state_error, json_success};
use crate::api::Claims;
use crate::state::AppState;

// ── Balance handler ───────────────────────────────────────────────────────────

pub async fn balance_handler(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<BalanceRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match payload {
        BalanceRequest::User { user_id } => {
            debug!("Get user balances: {:?}", user_id);
            match state.get_user_non_group_balances(claims.sub, user_id).await {
                Ok(balances) => {
                    json_success(StatusCode::OK, serde_json::to_value(balances).unwrap())
                }
                Err(error) => json_state_error(error, "Failed to get user balances"),
            }
        }
        BalanceRequest::Group { group_id } => {
            debug!("Get group balances: {:?}", group_id);
            match state.get_group_balance_overview(claims.sub, group_id).await {
                Ok(transfers) => {
                    json_success(StatusCode::OK, serde_json::to_value(transfers).unwrap())
                }
                Err(error) => json_state_error(error, "Failed to get group balances"),
            }
        }
    }
}
