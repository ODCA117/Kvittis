use crate::{UserId, GroupId, ExpenseId};
use serde::{Serialize, Deserialize};

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

