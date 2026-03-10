// use chrono::{DateTime, FixedOffset};
use uuid::Uuid;
pub mod api;

pub type UserId = Uuid;
pub type GroupId = Uuid;
pub type ExpenseId = Uuid;

#[derive(Clone, Debug)]
pub struct User {
    pub id: UserId,
    pub username: String,
    pub friends: Vec<UserId>,
    // TODO: Add this
    // pub email: String,
    // pub created_at: DateTime<FixedOffset>,
    // pub updated_at: DateTime<FixedOffset>,
}

#[derive(Clone, Debug)]
pub struct Group {
    pub id: GroupId,
    pub name: String,
    pub owner_id: UserId,
    pub members: Vec<UserId>,
}

#[derive(Clone, Debug)]
pub struct Expense {
    pub id: ExpenseId,
    pub payer: UserId,
    pub participants: Vec<UserId>,
    pub amount: i64,
    pub description: Option<String>,
    pub group_id: Option<GroupId>,
    pub timestamp_ms: i64,
}
