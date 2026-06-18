// User-related API handlers

use axum::{Json, extract::State, http::StatusCode};
use common::api::{
    ApiResponse, AuthorizedUserRequest, GetUserResponse, HandleFriendRequestResponse,
    LoginResponse, PendingFriendRequestResponse, RegisterResponse, SearchUserResponse,
    UnauthorizedUserRequest,
};
use tracing::{debug, info, warn};

use super::auth::{json_error, json_not_implemented, json_success};
use crate::api::Claims;
use crate::state::AppState;

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

        AuthorizedUserRequest::Delete => {
            // NOTE: Only delete if logged in user (or admin)
            info!("Delete user: {:?}", claims.sub);
            match state.delete_user(claims.sub).await {
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

        AuthorizedUserRequest::SendFriendRequest { friend_id } => {
            // NOTE: Send friend request, need to be accepted
            debug!(
                "Send friend request: Sender={:?} Receiver={:?}",
                claims.sub, friend_id
            );
            match state.send_friend_request(claims.sub, friend_id).await {
                Ok(resp) => json_success(StatusCode::OK, serde_json::to_value(resp).unwrap()),
                Err(_) => json_error(StatusCode::BAD_REQUEST, "user_id or friend_id not found"),
            }
        }

        AuthorizedUserRequest::GetPendingFriendRequests => {
            debug!("Get pending friend requests: User={:?}", claims.sub);
            let incoming = state
                .list_pending_incoming_friend_requests(claims.sub)
                .await;
            let outgoing = state
                .list_pending_outgoing_friend_requests(claims.sub)
                .await;
            if let (Ok(incoming), Ok(outgoing)) = (incoming, outgoing) {
                json_success(
                    StatusCode::OK,
                    serde_json::to_value(PendingFriendRequestResponse { incoming, outgoing })
                        .unwrap(),
                )
            } else {
                json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Failed to get friend requests",
                )
            }
        }

        AuthorizedUserRequest::HandleFriendRequest {
            request_id,
            request_action,
        } => {
            debug!(
                "Handle friend request: Request_id={:?}, update={:?}",
                request_id, request_action
            );
            match state
                .handle_friend_request(request_id, request_action)
                .await
            {
                Ok(_) => json_success(
                    StatusCode::OK,
                    serde_json::to_value(HandleFriendRequestResponse {
                        status: "Success".to_string(),
                    })
                    .unwrap(),
                ),
                Err(_e) => json_error(StatusCode::NOT_FOUND, "Friend Request not found"),
            }
        }
    }
}
