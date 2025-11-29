use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use common::{
    ExpenseId, GroupId, User, UserId,
    api::{
        CreateExpenseRequest, CreateExpenseResponse, CreateGroupRequest, CreateGroupResponse,
        FriendRequest, GetUserResponse, RegisterRequest, RegisterResponse,
    },
};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;

use crate::state::AppState;

#[derive(Serialize)]
pub struct BalanceEntry {
    other: UserId,
    amount: u64,
}

#[derive(Serialize)]
pub struct GroupBalance {
    from: UserId,
    to: UserId,
    amount: u64,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success(T),
    Error { message: String },
}

fn json_not_implemented<T>() -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ApiResponse::Error {
            message: "Function not implemented".to_owned(),
        }),
    )
}

// Helper function for error responses (generic over T)
fn json_error<T>(status: StatusCode, message: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (
        status,
        Json(ApiResponse::Error {
            message: message.to_string(),
        }),
    )
}

// Helper function for success responses
fn json_success<T>(status: StatusCode, data: T) -> (StatusCode, Json<ApiResponse<T>>) {
    (status, Json(ApiResponse::Success(data)))
}

pub async fn register_user(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, Json<ApiResponse<RegisterResponse>>) {
    let id = Uuid::new_v4();
    let user = User {
        id,
        username: payload.username.clone(),
        friends: vec![],
    };

    debug!("Register user: {:?}", payload);

    match state.register_user(user) {
        Ok(_) => json_success(
            StatusCode::CREATED,
            RegisterResponse {
                id,
                username: payload.username,
            },
        ),
        Err(_) => json_error(StatusCode::BAD_REQUEST, "user_id not found"),
    }
}

pub async fn get_user(
    State(state): State<AppState>,
    Path(user_id): Path<UserId>,
) -> (StatusCode, Json<ApiResponse<GetUserResponse>>) {
    debug!("Get user: {:?}", user_id);
    match state.get_user(user_id) {
        Ok(user) => json_success(StatusCode::OK, user.into()),
        Err(_) => json_error(StatusCode::NOT_FOUND, "User not found"),
    }
}

pub async fn get_users(
    State(state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<Vec<GetUserResponse>>>) {
    debug!("Get users:");
    let users = state.get_users();
    json_success(
        StatusCode::OK,
        users.into_iter().map(|u| u.into()).collect(),
    )
}

pub async fn add_friend(
    State(state): State<AppState>,
    Json(payload): Json<FriendRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    debug!("Add friend: {:?}", payload);
    match state.add_friend(payload.user_id, payload.friend_id) {
        Ok(_) => json_success(
            StatusCode::OK,
            serde_json::json!({"status": "friend added"}),
        ),
        Err(_) => json_error(StatusCode::BAD_REQUEST, "user_id or friend_id not found"),
    }
}

pub async fn create_group(
    State(state): State<AppState>,
    Json(payload): Json<CreateGroupRequest>,
) -> (StatusCode, Json<ApiResponse<CreateGroupResponse>>) {
    debug!("Create group: {:?}", payload);
    json_not_implemented()
}

pub async fn create_expense(
    State(state): State<AppState>,
    Json(payload): Json<CreateExpenseRequest>,
) -> (StatusCode, Json<ApiResponse<CreateExpenseResponse>>) {
    debug!("Create expense: {:?}", payload);
    match state.create_expense(payload) {
        Ok(e) => json_success(StatusCode::CREATED, e.into()),
        Err(_) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create expense",
        ),
    }
}

pub async fn get_user_balances(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Vec<BalanceEntry>>>) {
    debug!("get user balance: {:?}", user_id);
    json_not_implemented()
}

pub async fn get_group_balances(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Vec<GroupBalance>>>) {
    debug!("get group balance: {:?}", group_id);
    json_not_implemented()
}

// fn compute_debts(
//     expenses: &Vec<Expense>,
//     filter_group: Option<GroupId>,
// ) -> HashMap<(Uuid, Uuid), f64> {
//     // map (from, to) => amount (from owes to)
//     let mut map: HashMap<(Uuid, Uuid), f64> = HashMap::new();
//     for exp in expenses.iter() {
//         if let Some(gid) = filter_group {
//             if exp.group_id != Some(gid) {
//                 continue;
//             }
//         }
//         if exp.participants.is_empty() {
//             continue;
//         }
//
//         let share = exp.amount / (exp.participants.len() as f64);
//         for p in exp.participants.iter() {
//             if *p == exp.payer {
//                 continue;
//             }
//             *map.entry((*p, exp.payer)).or_default() += share;
//         }
//     }
//     map
// }
