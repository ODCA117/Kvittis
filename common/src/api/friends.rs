use crate::FriendRequest;
use serde::{Deserialize, Serialize};

// Response types

#[derive(Serialize, Deserialize, Debug)]
pub struct FriendRequestResponse {
    pub request: FriendRequest,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PendingFriendRequestResponse {
    pub incoming: Vec<FriendRequest>,
    pub outgoing: Vec<FriendRequest>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HandleFriendRequestResponse {
    pub status: String,
}
