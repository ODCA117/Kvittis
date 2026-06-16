use crate::{PublicUser, User, UserId};
use serde::{Deserialize, Serialize};

// Response types

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUserResponse {
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchUserResponse {
    pub user: Vec<PublicUser>,
}

// Request enums

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AuthorizedUserRequest {
    Get,
    Delete,
    List,
    Search {
        query: String,
    },
    SendFriendRequest {
        friend_id: UserId,
    },
    HandleFriendRequest {
        request_id: crate::FriendRequestId,
        request_action: crate::FriendRequestAction,
    },
    GetPendingFriendRequests,
    Logout,
}

// From trait implementations

impl From<User> for GetUserResponse {
    fn from(value: User) -> Self {
        GetUserResponse { user: value }
    }
}

impl From<GetUserResponse> for User {
    fn from(value: GetUserResponse) -> Self {
        value.user
    }
}
