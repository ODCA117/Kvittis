// Expense-related API handlers

use axum::{Json, extract::State, http::StatusCode};
use common::api::{ApiResponse, CreateExpenseResponse, ExpenseRequest, GetExpenseResponse};
use tracing::debug;

use super::auth::{json_state_error, json_success};
use crate::api::Claims;
use crate::state::AppState;

// ── Expense handler ───────────────────────────────────────────────────────────

pub async fn expense_handler(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<ExpenseRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match payload {
        ExpenseRequest::Create {
            payer,
            participants,
            amount,
            description,
            group_id,
        } => {
            debug!("Create expense: payer={:?} amount={:?}", payer, amount);
            let req = common::api::CreateExpenseRequest {
                payer,
                participants,
                amount,
                description,
                group_id,
            };
            match state.create_expense(claims.sub, req).await {
                Ok(e) => {
                    let resp: CreateExpenseResponse = e.into();
                    json_success(StatusCode::CREATED, serde_json::to_value(resp).unwrap())
                }
                Err(error) => json_state_error(error, "Failed to create expense"),
            }
        }

        ExpenseRequest::Get { id } => {
            debug!("Get expense: {:?}", id);
            let req = common::api::GetExpenseRequest { id };
            match state.get_expense(claims.sub, req).await {
                Ok(e) => {
                    let resp: GetExpenseResponse = e.into();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(error) => json_state_error(error, "Failed to get expense"),
            }
        }

        ExpenseRequest::Delete { id } => {
            debug!("Delete expense: {:?}", id);
            let req = common::api::DeleteExpenseRequest { id };
            match state.delete_expense(claims.sub, req).await {
                Ok(_) => json_success(
                    StatusCode::OK,
                    serde_json::json!({"status": "Expense deleted"}),
                ),
                Err(error) => json_state_error(error, "Failed to delete expense"),
            }
        }

        ExpenseRequest::ListForUser { user_id } => {
            debug!("List expenses for user: {:?}", user_id);
            match state.list_expenses_for_user(claims.sub, user_id).await {
                Ok(expenses) => {
                    let resp: Vec<GetExpenseResponse> =
                        expenses.into_iter().map(|e| e.into()).collect();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(error) => json_state_error(error, "Failed to list expenses for user"),
            }
        }

        ExpenseRequest::ListForGroup { group_id } => {
            debug!("List expenses for group: {:?}", group_id);
            match state.list_expenses_for_group(claims.sub, group_id).await {
                Ok(expenses) => {
                    let resp: Vec<GetExpenseResponse> =
                        expenses.into_iter().map(|e| e.into()).collect();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(error) => json_state_error(error, "Failed to list expenses for group"),
            }
        }
    }
}
