// TODO: Put these behind features
pub mod db_file;
pub mod db_sqlite;
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, FixedOffset};
use serde::{Deserialize, Serialize};

use common::{ExpenseId, FriendRequestId, GroupId, UserId};

#[async_trait]
pub trait Store: Send + Sync {
    // --- Users ---
    async fn create_user(&self, user: UserRow) -> Result<UserRow>;
    async fn get_user_by_id(&self, id: UserId) -> Result<Option<UserRow>>;
    async fn get_user_by_name(&self, username: String) -> Result<Option<UserRow>>;
    async fn delete_user(&self, id: UserId) -> Result<()>;
    async fn list_users(&self) -> Result<Vec<UserRow>>;
    async fn _update_user(&self, user: UserRow) -> Result<UserRow>;
    async fn create_friend_request(&self, request: FriendRequestRow) -> Result<FriendRequestRow>;
    async fn get_friend_request(&self, request: FriendRequestId) -> Result<FriendRequestRow>;
    async fn get_outgoing_requests(&self, user: UserId) -> Result<Vec<FriendRequestRow>>;
    async fn get_incoming_requests(&self, user: UserId) -> Result<Vec<FriendRequestRow>>;
    async fn update_friend_request(&self, request: FriendRequestRow) -> Result<()>;
    async fn delete_friend_requests_from_user(&self, user: UserId) -> Result<()>;
    async fn add_friendship(&self, user1: UserId, user2: UserId) -> Result<()>;
    async fn remove_friendship(&self, user: UserId) -> Result<()>;

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
    async fn list_expenses_for_user(&self, user_id: UserId) -> Result<Vec<ExpenseRow>>;
    async fn list_expenses_for_group(&self, group_id: GroupId) -> Result<Vec<ExpenseRow>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserRow {
    pub id: UserId,
    pub username: String,
    // pub friends: Vec<UserId>, Does not exist in the DB.
    pub email: String,
    pub password_hash: String,
    pub created_at: DateTime<FixedOffset>,
    pub updated_at: DateTime<FixedOffset>,
    pub deleted_at: Option<DateTime<FixedOffset>>,
}

impl UserRow {
    pub fn new(
        id: UserId,
        username: String,
        // friends: Vec<UserId>,
        email: String,
        password_hash: String,
        created_at: DateTime<FixedOffset>,
        updated_at: DateTime<FixedOffset>,
        deleted_at: Option<DateTime<FixedOffset>>,
    ) -> Self {
        Self {
            id,
            username,
            // friends,
            email,
            password_hash,
            created_at,
            updated_at,
            deleted_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct FriendRequestRow {
    pub id: FriendRequestId,
    pub sender_id: UserId,
    pub receiver_id: UserId,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
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
