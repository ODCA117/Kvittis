use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub type UserId = Uuid;
pub type GroupId = Uuid;
pub type ExpenseId = Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub friends: Vec<UserId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub owner_id: UserId,
    pub members: Vec<UserId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Expense {
    pub id: ExpenseId,
    pub payer: UserId,
    pub participants: Vec<UserId>,
    pub amount: f64,
    pub description: Option<String>,
    pub group_id: Option<GroupId>,
    pub timestamp_ms: i64,
}
