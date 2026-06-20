use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self},
    str::FromStr,
};
use uuid::Uuid;

pub mod api;

pub type UserId = Uuid;
pub type GroupId = Uuid;
pub type ExpenseId = Uuid;
pub type FriendRequestId = Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub friends: Vec<UserId>,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicUser {
    pub id: UserId,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FriendRequest {
    pub id: FriendRequestId,
    pub from: UserId,
    pub to: UserId,
    pub status: FriendRequestState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FriendRequestState {
    Pending,
    Rejected,
    Accepted,
}

impl FromStr for FriendRequestState {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Pending" => Ok(FriendRequestState::Pending),
            "Rejected" => Ok(FriendRequestState::Rejected),
            "Accepted" => Ok(FriendRequestState::Accepted),
            _ => Err(format!("{s} is not a valid string")),
        }
    }
}

impl fmt::Display for FriendRequestState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum FriendRequestAction {
    Accept,
    Reject,
    Cancel,
}

#[derive(Clone, Debug)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub members: Vec<(UserId, GroupRole)>,
    pub last_settled: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GroupRole {
    Admin,
    Member,
}

impl FromStr for GroupRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Admin" => Ok(GroupRole::Admin),
            "Member" => Ok(GroupRole::Member),
            _ => Err(format!("{s} is not a valid string")),
        }
    }
}

impl fmt::Display for GroupRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
pub struct Expense {
    pub id: ExpenseId,
    pub payer: UserId,
    pub participants: Vec<UserId>,
    pub amount: i64,
    pub description: Option<String>,
    pub group_id: Option<GroupId>,
    pub created_at: DateTime<Utc>,
}
