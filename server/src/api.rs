use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use tracing::debug;
use uuid::Uuid;
use common::{UserId, GroupId, ExpenseId, User, Group, Expense};

use crate::{
    db::{ExpenseRow, GroupRow, UserDB, UserRow}, state::AppState
};

#[derive(Deserialize, Debug)]
pub struct RegisterRequest {
    username: String,
}

#[derive(Serialize)]
pub struct RegisterResponse {
    id: UserId,
    username: String,
}

#[derive(Deserialize, Debug)]
pub struct FriendRequest {
    user_id: UserId,
    friend_id: UserId,
}

#[derive(Deserialize, Debug)]
pub struct CreateGroupRequest {
    name: String,
    owner_id: UserId,
    members: Vec<UserId>,
}

#[derive(Serialize)]
pub struct CreateGroupResponse {
    id: GroupId,
    name: String,
}

#[derive(Deserialize, Debug)]
pub struct CreateExpenseRequest {
    payer: UserId,
    participants: Vec<UserId>,
    amount: f64,
    description: Option<String>,
    group_id: Option<GroupId>,
}

#[derive(Serialize)]
pub struct ExpenseResponse {
    id: ExpenseId,
}

#[derive(Serialize)]
pub struct BalanceEntry {
    other: UserId,
    amount: f64,
}

#[derive(Serialize)]
pub struct GroupBalance {
    from: UserId,
    to: UserId,
    amount: f64,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum ApiResponse<T> {
    Success(T),
    Error { message: String },
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

pub async fn add_friend(
    State(state): State<AppState>,
    Json(payload): Json<FriendRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    debug!("Add friend: {:?}", payload);
    json_success(StatusCode::OK, serde_json::json!({"ok": true}))
}

pub async fn create_group(
    State(state): State<AppState>,
    Json(payload): Json<CreateGroupRequest>,
) -> (StatusCode, Json<ApiResponse<CreateGroupResponse>>) {
    debug!("Create group: {:?}", payload);
    json_success(
        StatusCode::CREATED,
        CreateGroupResponse {
            id: Uuid::new_v4(),
            name: payload.name,
        },
    )
}

pub async fn create_expense(
    State(state): State<AppState>,
    Json(payload): Json<CreateExpenseRequest>,
) -> (StatusCode, Json<ApiResponse<ExpenseResponse>>) {
    debug!("Create expense: {:?}", payload);
    json_success(StatusCode::CREATED, ExpenseResponse { id: Uuid::new_v4() })
}

pub async fn get_user_balances(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Vec<BalanceEntry>>>) {
    debug!("get user balance: {:?}", user_id);
    json_success(
        StatusCode::OK,
        vec![BalanceEntry {
            other: Uuid::new_v4(),
            amount: 0.0,
        }],
    )
}

pub async fn get_group_balances(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Vec<GroupBalance>>>) {
    debug!("get group balance: {:?}", group_id);
    json_success(
        StatusCode::OK,
        vec![GroupBalance {
            from: Uuid::new_v4(),
            to: Uuid::new_v4(),
            amount: 0.0,
        }],
    )
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
