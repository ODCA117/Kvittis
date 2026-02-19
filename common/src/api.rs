use crate::{Expense, ExpenseId, Group, GroupId, User, UserId};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterRequest {
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponse {
    pub id: UserId,
    pub username: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUserResponse {
    pub id: UserId,
    pub username: String,
    pub friends: Vec<UserId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SearchUserRequest {
    pub query: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FriendRequest {
    pub user_id: UserId,
    pub friend_id: UserId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGroupRequest {
    pub name: String,
    pub owner_id: UserId,
    pub members: Vec<UserId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGroupResponse {
    pub id: GroupId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewGroupMemberRequest {
    pub group_id: GroupId,
    pub new_member: UserId,
}

// #[derive(Serialize, Deserialize, Debug)]
// pub struct GetGroupRequest {
//     pub id: GroupId,
// }

#[derive(Serialize, Deserialize, Debug)]
pub struct GetGroupResponse {
    pub id: GroupId,
    pub name: String,
    pub owner_id: UserId,
    pub members: Vec<UserId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateExpenseRequest {
    pub payer: UserId,
    pub participants: Vec<UserId>,
    pub amount: u64,
    pub description: Option<String>,
    pub group_id: Option<GroupId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateExpenseResponse {
    pub id: ExpenseId,
    pub payer: UserId,
    pub participants: Vec<UserId>,
    pub amount: u64,
    pub description: Option<String>,
    pub group_id: Option<GroupId>,
    pub timestamp_ms: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BalanceEntry {
    pub other: UserId,
    pub amount: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GroupBalance {
    pub from: UserId,
    pub to: UserId,
    pub amount: f64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiResponse<T: Serialize> {
    Success(T),
    Error { message: String },
}

impl From<User> for GetUserResponse {
    fn from(value: User) -> Self {
        GetUserResponse {
            id: value.id,
            username: value.username,
            friends: value.friends.clone(),
        }
    }
}

impl From<GetUserResponse> for User {
    fn from(value: GetUserResponse) -> Self {
        Self {
            id: value.id,
            username: value.username,
            friends: value.friends,
        }
    }
}

impl From<User> for RegisterResponse {
    fn from(value: User) -> Self {
        RegisterResponse {
            id: value.id,
            username: value.username,
        }
    }
}

impl From<RegisterResponse> for User {
    fn from(value: RegisterResponse) -> Self {
        Self {
            id: value.id,
            username: value.username,
            friends: vec![],
        }
    }
}

impl From<GetGroupResponse> for Group {
    fn from(value: GetGroupResponse) -> Self {
        Group {
            id: value.id,
            name: value.name,
            owner_id: value.owner_id,
            members: value.members,
        }
    }
}

impl From<CreateExpenseResponse> for Expense {
    fn from(value: CreateExpenseResponse) -> Self {
        Expense {
            id: value.id,
            payer: value.payer,
            participants: value.participants,
            amount: value.amount,
            description: value.description,
            group_id: value.group_id,
            timestamp_ms: value.timestamp_ms,
        }
    }
}

impl From<Expense> for CreateExpenseResponse {
    fn from(value: Expense) -> Self {
        CreateExpenseResponse {
            id: value.id,
            payer: value.payer,
            participants: value.participants,
            amount: value.amount,
            description: value.description,
            group_id: value.group_id,
            timestamp_ms: value.timestamp_ms,
        }
    }
}
