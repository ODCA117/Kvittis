use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use common::{
    GroupId, User, UserId, api::{
        ApiResponse, BalanceEntry, CreateExpenseRequest, CreateExpenseResponse, CreateGroupRequest, CreateGroupResponse, FriendRequest, GetGroupResponse, GetUserResponse, GroupBalance, RegisterRequest, RegisterResponse
    }
};
use serde::Serialize;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::state::AppState;

/// NOTE: This could maybe be replaced by a impl IntoResponse to be even more generic?
fn json_not_implemented<T: Serialize>() -> (StatusCode, Json<ApiResponse<T>>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json::<ApiResponse<T>>(ApiResponse::Error {
            message: "Function not implemented".to_owned(),
        }),
    )
}

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

#[axum::debug_handler]
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

    debug!("Register user: {:?}, {:?}", payload, user);

    match state.register_user(user).await {
        Ok(u) => json_success(
            StatusCode::CREATED,
            RegisterResponse {
                id: u.id,
                username: u.username,
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
    match state.get_user(user_id).await {
        Ok(user) => json_success(StatusCode::OK, user.into()),
        Err(_) => json_error(StatusCode::NOT_FOUND, "User not found"),
    }
}

pub async fn get_users(
    State(state): State<AppState>,
) -> (StatusCode, Json<ApiResponse<Vec<GetUserResponse>>>) {
    debug!("Get users:");
    match state.get_users().await {
        Ok(users) => json_success(
            StatusCode::OK,
            users.into_iter().map(|u| u.into()).collect(),
        ),
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get users"),
    }
}

pub async fn add_friend(
    State(state): State<AppState>,
    Json(payload): Json<FriendRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    debug!("Add friend: {:?}", payload);
    match state.add_friend(payload.user_id, payload.friend_id).await {
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
    match state.create_group(payload).await {
        Ok(g) => json_success(
            StatusCode::CREATED,
            CreateGroupResponse {
                id: g.id,
                name: g.name,
            },
        ),
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create group"),
    }
}

pub async fn get_group(
    State(state): State<AppState>,
    Path(group_id): Path<GroupId>,
) -> (StatusCode, Json<ApiResponse<GetGroupResponse>>) {
    warn!("Get group: {:?}", group_id);
    match state.get_group(group_id).await {
        Ok(g) => {
            if let Some(g) = g {
                debug!("Found group");
                json_success(
                    StatusCode::CREATED,
                    GetGroupResponse {
                        id: g.id,
                        name: g.name,
                        owner_id: g.owner_id,
                        members: g.members,
                    },
                )
            } else {
                json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create group")
            }
        },
        Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create group"),
    }
}


pub async fn create_expense(
    State(state): State<AppState>,
    Json(payload): Json<CreateExpenseRequest>,
) -> (StatusCode, Json<ApiResponse<CreateExpenseResponse>>) {
    debug!("Create expense: {:?}", payload);
    match state.create_expense(payload).await {
        Ok(e) => json_success(StatusCode::CREATED, e.into()),
        Err(_) => json_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "Failed to create expense",
        ),
    }
}

pub async fn get_user_balances(
    State(_state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Vec<BalanceEntry>>>) {
    debug!("get user balance: {:?}", user_id);
    json_not_implemented()
}

pub async fn get_group_balances(
    State(_state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Vec<GroupBalance>>>) {
    debug!("get group balance: {:?}", group_id);
    json_not_implemented()
}
