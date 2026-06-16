// Expense-related API handlers

use axum::{Json, extract::State, http::StatusCode};
use common::api::{ApiResponse, CreateExpenseResponse, ExpenseRequest, GetExpenseResponse};
use tracing::debug;

use super::auth::{json_error, json_success};
use crate::state::AppState;

// ── Expense handler ───────────────────────────────────────────────────────────

pub async fn expense_handler(
    State(state): State<AppState>,
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
            match state.create_expense(req).await {
                Ok(e) => {
                    let resp: CreateExpenseResponse = e.into();
                    json_success(StatusCode::CREATED, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to create expense",
                ),
            }
        }

        ExpenseRequest::Get { id } => {
            debug!("Get expense: {:?}", id);
            let req = common::api::GetExpenseRequest { id };
            match state.get_expense(req).await {
                Ok(e) => {
                    let resp: GetExpenseResponse = e.into();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get expense"),
            }
        }

        ExpenseRequest::Delete { id } => {
            debug!("Delete expense: {:?}", id);
            let req = common::api::DeleteExpenseRequest { id };
            match state.delete_expense(req).await {
                Ok(_) => json_success(
                    StatusCode::OK,
                    serde_json::json!({"status": "Expense deleted"}),
                ),
                Err(_) => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to delete expense",
                ),
            }
        }

        ExpenseRequest::ListForUser { user_id } => {
            debug!("List expenses for user: {:?}", user_id);
            match state.list_expenses_for_user(user_id).await {
                Ok(expenses) => {
                    let resp: Vec<GetExpenseResponse> =
                        expenses.into_iter().map(|e| e.into()).collect();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to list expenses for user",
                ),
            }
        }

        ExpenseRequest::ListForGroup { group_id } => {
            debug!("List expenses for group: {:?}", group_id);
            match state.list_expenses_for_group(group_id).await {
                Ok(expenses) => {
                    let resp: Vec<GetExpenseResponse> =
                        expenses.into_iter().map(|e| e.into()).collect();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to list expenses for group",
                ),
            }
        }
    }
}
