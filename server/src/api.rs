use axum::{
    Json,
    extract::{FromRef, FromRequestParts, State},
    http::{self, StatusCode},
    response::{IntoResponse, Response},
};
use common::api::{
    ApiResponse, AuthorizedUserRequest, BalanceRequest, CreateExpenseResponse, CreateGroupResponse,
    ExpenseRequest, GetExpenseResponse, GetGroupResponse, GetUserResponse, GroupRequest,
    LoginResponse, RegisterResponse, SearchUserResponse, UnauthorizedUserRequest,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tracing::{debug, info, warn};
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

#[derive(Debug)]
pub enum AuthError {
    _WrongCredentials,
    _MissingCredentials,
    _TokenCreation,
    InvalidToken,
}

// TODO: Maybe move this?
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub app: String,
    pub exp: usize,
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
    AppState: FromRef<S>,
{
    type Rejection = AuthError; // Need to implement IntoResponse

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        state: &S,
    ) -> Result<Self, Self::Rejection> {
        let state = AppState::from_ref(state);
        match state
            .validate_jwt(parts)
            .await
            .map_err(|_| AuthError::InvalidToken)
        {
            Ok(token_data) => {
                info!("Validation successfull");
                Ok(token_data.claims)
            }
            Err(e) => {
                warn!("Validation failed");
                Err(e)
            }
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            AuthError::_WrongCredentials => (http::StatusCode::UNAUTHORIZED, "Wrong credentials"),
            AuthError::_MissingCredentials => {
                (http::StatusCode::BAD_REQUEST, "Missing credentials")
            }
            AuthError::_TokenCreation => (
                http::StatusCode::INTERNAL_SERVER_ERROR,
                "Token creation error",
            ),
            AuthError::InvalidToken => (http::StatusCode::BAD_REQUEST, "Invalid token"),
        };

        let body = Json(json!({ "error": error_message }));
        (status, body).into_response()
    }
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
pub async fn unauthorized_user_handler(
    State(state): State<AppState>,
    Json(payload): Json<UnauthorizedUserRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match payload {
        UnauthorizedUserRequest::Register { user } => {
            debug!(
                "Register user: Username: {:?}, Email: {:?}",
                &user.username, &user.email
            );
            match state.register_user(user).await {
                Ok(u) => {
                    info!(
                        "Successfully registered user: username: {:?}, email: {:?}",
                        &u.id, &u.email,
                    );
                    let resp = RegisterResponse { user: u };
                    json_success(StatusCode::CREATED, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::BAD_REQUEST, "Failed to register user"),
            }
        }

        UnauthorizedUserRequest::Login { username, password } => {
            match state.login_user(username, password).await {
                Ok((user, token, token_type)) => {
                    info!("User: {:?} successfully logged in", user.id);
                    let resp = LoginResponse {
                        user,
                        token,
                        token_type,
                    };
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::UNAUTHORIZED, "Authentication failed"),
            }
        }
    }
}

#[axum::debug_handler]
pub async fn authorized_user_handler(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<AuthorizedUserRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match payload {
        AuthorizedUserRequest::Logout => {
            warn!("Do not really know how to make this work yet");
            json_not_implemented()
        }

        AuthorizedUserRequest::Get => {
            // NOTE: Only get information if logged in user (or admin)
            info!("Get user: {:?}", claims.sub);
            match state.get_user(claims.sub).await {
                Ok(u) => {
                    let resp: GetUserResponse = u.into();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::NOT_FOUND, "User not found"),
            }
        }

        AuthorizedUserRequest::Delete { user_id } => {
            // NOTE: Only delete if logged in user (or admin)
            info!("Delete user: {:?}", user_id);
            match state.delete_user(user_id).await {
                Ok(_) => json_success(
                    StatusCode::OK,
                    serde_json::json!({"status": "User deleted"}),
                ),
                Err(_) => json_error(StatusCode::NOT_FOUND, "User not found"),
            }
        }

        AuthorizedUserRequest::List => {
            // NOTE: Implement this for admins/privilege users
            info!("List users");
            json_error(StatusCode::FORBIDDEN, "Not allowed to get users")
            // NOTE: Note implemented, Dangerous to be able to get all users in the field.
            // match state.get_users().await {
            //     Ok(users) => {
            //         let resp: Vec<GetUserResponse> = users.into_iter().map(|u| u.into()).collect();
            //         json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
            //     }
            //     Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get users"),
            // }
        }

        AuthorizedUserRequest::Search { query } => {
            // NOTE: Search for all users, only return username and UserId
            debug!("Search users: {:?}", query);
            match state.search_users(&query).await {
                Ok(users) => {
                    let resp = SearchUserResponse { user: users };
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to search users"),
            }
        }

        AuthorizedUserRequest::AddFriend { user_id, friend_id } => {
            // NOTE: Send friend request, need to be accepted
            debug!("Add friend: user={:?} friend={:?}", user_id, friend_id);
            match state.add_friend(user_id, friend_id).await {
                Ok(_) => json_success(
                    StatusCode::OK,
                    serde_json::json!({"status": "friend added"}),
                ),
                Err(_) => json_error(StatusCode::BAD_REQUEST, "user_id or friend_id not found"),
            }
        }
    }
}

// ── Group handler ─────────────────────────────────────────────────────────────

// NOTE: Need to be authorized user
pub async fn group_handler(
    State(state): State<AppState>,
    Json(payload): Json<GroupRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match payload {
        GroupRequest::Create {
            name,
            owner_id,
            members,
        } => {
            debug!("Create group: name={:?}", name);
            let req = common::api::CreateGroupRequest {
                name,
                owner_id,
                members,
            };
            match state.create_group(req).await {
                Ok(g) => {
                    let resp = CreateGroupResponse {
                        id: g.id,
                        name: g.name,
                    };
                    json_success(StatusCode::CREATED, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to create group"),
            }
        }

        GroupRequest::Get { group_id } => {
            // NOTE: Only get groups user is member of
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
            // NOTE: Only delete groups user is member of
            debug!("Delete group: {:?}", group_id);
            match state.delete_group(group_id).await {
                Ok(_) => json_success(
                    StatusCode::OK,
                    serde_json::json!({"status": "Group deleted"}),
                ),
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete group"),
            }
        }

        GroupRequest::Search { query } => {
            // NOTE: Remove
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

        GroupRequest::AddMember {
            group_id,
            new_member,
        } => {
            // NOTE: Only add if admin of group
            debug!(
                "Add group member: group={:?} member={:?}",
                group_id, new_member
            );
            let req = common::api::NewGroupMemberRequest {
                group_id,
                new_member,
            };
            match state.new_group_member(req).await {
                Ok(_) => json_success(
                    StatusCode::OK,
                    serde_json::json!({"status": "member added"}),
                ),
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
