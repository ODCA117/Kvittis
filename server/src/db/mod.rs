// TODO: Put these behind features
pub mod db_file;
pub mod db_sqlite;
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use common::{ExpenseId, GroupId, UserId};

#[async_trait]
pub trait Store: Send + Sync {
    // --- Users ---
    async fn create_user(&self, user: UserRow) -> Result<UserRow>;
    async fn get_user(&self, id: UserId) -> Result<Option<UserRow>>;
    async fn delete_user(&self, id: UserId) -> Result<()>;
    async fn list_users(&self) -> Result<Vec<UserRow>>;
    async fn add_friend(&self, user1: UserId, user2: UserId) -> Result<()>;
    async fn update_user(&self, user: UserRow) -> Result<UserRow>;

    // --- Groups ---
    async fn create_group(&self, group: GroupRow) -> Result<GroupRow>;
    async fn get_group(&self, id: GroupId) -> Result<Option<GroupRow>>;
    async fn get_groups(&self) -> Result<Vec<GroupRow>>;
    async fn delete_group(&self, id: GroupId) -> Result<()>;
    async fn update_group(&self, group: GroupRow) -> Result<GroupRow>;

    // --- Expenses ---
    async fn create_expense(&self, expense: ExpenseRow) -> Result<ExpenseRow>;
    async fn delete_expense(&self, id: ExpenseId) -> Result<()>;
    async fn get_expense(&self, id: ExpenseId) -> Result<Option<ExpenseRow>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRow {
    pub id: UserId,
    pub username: String,
    pub friends: Vec<UserId>,
}

impl UserRow {
    pub fn new(id: UserId, username: String, friends: Vec<UserId>) -> Self {
        Self {
            id,
            username,
            friends,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRow {
    pub id: GroupId,
    pub name: String,
    pub owner_id: UserId,
    pub members: Vec<UserId>,
}

impl GroupRow {
    pub fn new(id: GroupId, name: String, owner_id: UserId, members: Vec<UserId>) -> Self {
        Self {
            id,
            name,
            owner_id,
            members,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpenseRow {
    pub id: ExpenseId,
    pub payer: UserId,
    pub participants: Vec<UserId>,
    pub amount: i64,
    pub description: Option<String>,
    pub group_id: Option<GroupId>,
    pub timestamp_ms: i64,
}

impl ExpenseRow {
    pub fn new(
        id: ExpenseId,
        payer: UserId,
        participants: Vec<UserId>,
        amount: i64,
        description: Option<String>,
        group_id: Option<GroupId>,
        timestamp_ms: i64,
    ) -> Self {
        Self {
            id,
            payer,
            participants,
            amount,
            description,
            group_id,
            timestamp_ms,
        }
    }
}
