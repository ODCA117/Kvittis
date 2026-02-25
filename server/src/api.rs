use axum::{
    Json,
    extract::State,
    http::StatusCode,
};
use common::{
    User, api::{
        ApiResponse, BalanceEntry, BalanceRequest, CreateExpenseResponse, CreateGroupResponse,
        ExpenseRequest, GetExpenseResponse, GetGroupResponse, GetUserResponse, GroupRequest,
        GroupBalance, RegisterResponse, UserRequest,
    }
};
use serde::Serialize;
use tracing::debug;
use uuid::Uuid;

use crate::state::AppState;

// Helper function for error responses (generic over T)
fn json_error<T: Serialize>(
    status: StatusCode,
    message: &str,
) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        status,
        Json::<ApiResponse<T>>(ApiResponse::Error {
            message: message.to_string(),
        }),
    )
}

// Helper function for success responses
fn json_success<T: Serialize>(status: StatusCode, data: T) -> (StatusCode, Json<ApiResponse<T>>) {
    (status, Json::<ApiResponse<T>>(ApiResponse::Success(data)))
}

fn json_not_implemented<T: Serialize>() -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json::<ApiResponse<T>>(ApiResponse::Error {
            message: "Function not implemented".to_owned(),
        }),
    )
}

// ── User handler ──────────────────────────────────────────────────────────────

#[axum::debug_handler]
pub async fn user_handler(
    State(state): State<AppState>,
    Json(payload): Json<UserRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match payload {
        UserRequest::Register { username } => {
            let id = Uuid::new_v4();
            let user = User { id, username, friends: vec![] };
            debug!("Register user: {:?}", user);
            match state.register_user(user).await {
                Ok(u) => {
                    let resp = RegisterResponse { id: u.id, username: u.username };
                    json_success(StatusCode::CREATED, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::BAD_REQUEST, "Failed to register user"),
            }
        }

        UserRequest::Get { user_id } => {
            debug!("Get user: {:?}", user_id);
            match state.get_user(user_id).await {
                Ok(u) => {
                    let resp: GetUserResponse = u.into();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::NOT_FOUND, "User not found"),
            }
        }

        UserRequest::Delete { user_id } => {
            debug!("Delete user: {:?}", user_id);
            match state.delete_user(user_id).await {
                Ok(_) => json_success(StatusCode::OK, serde_json::json!({"status": "User deleted"})),
                Err(_) => json_error(StatusCode::NOT_FOUND, "User not found"),
            }
        }

        UserRequest::List => {
            debug!("List users");
            match state.get_users().await {
                Ok(users) => {
                    let resp: Vec<GetUserResponse> = users.into_iter().map(|u| u.into()).collect();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get users"),
            }
        }

        UserRequest::Search { query } => {
            debug!("Search users: {:?}", query);
            match state.search_users(&query).await {
                Ok(users) => {
                    let resp: Vec<GetUserResponse> = users.into_iter().map(|u| u.into()).collect();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to search users"),
            }
        }

        UserRequest::AddFriend { user_id, friend_id } => {
            debug!("Add friend: user={:?} friend={:?}", user_id, friend_id);
            match state.add_friend(user_id, friend_id).await {
                Ok(_) => json_success(StatusCode::OK, serde_json::json!({"status": "friend added"})),
                Err(_) => json_error(StatusCode::BAD_REQUEST, "user_id or friend_id not found"),
            }
        }
    }
}

// ── Group handler ─────────────────────────────────────────────────────────────

pub async fn group_handler(
    State(state): State<AppState>,
    Json(payload): Json<GroupRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match payload {
        GroupRequest::Create { name, owner_id, members } => {
            debug!("Create group: name={:?}", name);
            let req = common::api::CreateGroupRequest { name, owner_id, members };
            match state.create_group(req).await {
                Ok(g) => {
                    let resp = CreateGroupResponse { id: g.id, name: g.name };
                    json_success(StatusCode::CREATED, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create group"),
            }
        }

        GroupRequest::Get { group_id } => {
            debug!("Get group: {:?}", group_id);
            match state.get_group(group_id).await {
                Ok(Some(g)) => {
                    let resp = GetGroupResponse {
                        id: g.id,
                        name: g.name,
                        owner_id: g.owner_id,
                        members: g.members,
                    };
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Ok(None) => json_error(StatusCode::NOT_FOUND, "Group not found"),
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get group"),
            }
        }

        GroupRequest::Delete { group_id } => {
            debug!("Delete group: {:?}", group_id);
            match state.delete_group(group_id).await {
                Ok(_) => json_success(StatusCode::OK, serde_json::json!({"status": "Group deleted"})),
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete group"),
            }
        }

        GroupRequest::Search { query } => {
            debug!("Search groups: {:?}", query);
            match state.search_groups(&query).await {
                Ok(groups) => {
                    let resp: Vec<GetGroupResponse> = groups
                        .into_iter()
                        .map(|g| GetGroupResponse {
                            id: g.id,
                            name: g.name,
                            owner_id: g.owner_id,
                            members: g.members,
                        })
                        .collect();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to search groups"),
            }
        }

        GroupRequest::AddMember { group_id, new_member } => {
            debug!("Add group member: group={:?} member={:?}", group_id, new_member);
            let req = common::api::NewGroupMemberRequest { group_id, new_member };
            match state.new_group_member(req).await {
                Ok(_) => json_success(StatusCode::OK, serde_json::json!({"status": "member added"})),
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to add member"),
            }
        }
    }
}

// ── Expense handler ───────────────────────────────────────────────────────────

pub async fn expense_handler(
    State(state): State<AppState>,
    Json(payload): Json<ExpenseRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match payload {
        ExpenseRequest::Create { payer, participants, amount, description, group_id } => {
            debug!("Create expense: payer={:?} amount={:?}", payer, amount);
            let req = common::api::CreateExpenseRequest { payer, participants, amount, description, group_id };
            match state.create_expense(req).await {
                Ok(e) => {
                    let resp: CreateExpenseResponse = e.into();
                    json_success(StatusCode::CREATED, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create expense"),
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
                Ok(_) => json_success(StatusCode::OK, serde_json::json!({"status": "Expense deleted"})),
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete expense"),
            }
        }

        ExpenseRequest::ListForUser { user_id } => {
            debug!("List expenses for user: {:?}", user_id);
            match state.list_expenses_for_user(user_id).await {
                Ok(expenses) => {
                    let resp: Vec<GetExpenseResponse> = expenses.into_iter().map(|e| e.into()).collect();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to list expenses for user"),
            }
        }

        ExpenseRequest::ListForGroup { group_id } => {
            debug!("List expenses for group: {:?}", group_id);
            match state.list_expenses_for_group(group_id).await {
                Ok(expenses) => {
                    let resp: Vec<GetExpenseResponse> = expenses.into_iter().map(|e| e.into()).collect();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to list expenses for group"),
            }
        }
    }
}

// ── Balance handler ───────────────────────────────────────────────────────────

pub async fn balance_handler(
    State(state): State<AppState>,
    Json(payload): Json<BalanceRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match payload {
        BalanceRequest::User { user_id } => {
            debug!("Get user balances: {:?}", user_id);
            match state.get_user_non_group_balances(user_id).await {
                Ok(balances) => json_success(StatusCode::OK, serde_json::to_value(balances).unwrap()),
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get user balances"),
            }
        }
        BalanceRequest::Group { group_id } => {
            debug!("Get group balances: {:?}", group_id);
            match state.get_group_balance_overview(group_id).await {
                Ok(transfers) => json_success(StatusCode::OK, serde_json::to_value(transfers).unwrap()),
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get group balances"),
            }
        }
    }
}
