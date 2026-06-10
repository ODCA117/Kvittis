use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;
use tracing::debug;

use common::{ExpenseId, FriendRequestId, GroupId, UserId};

use crate::db::{ExpenseRow, FriendRequestRow, GroupRow, Store, UserRow};

/// ===============================
/// File-backed persistent state
/// ===============================
#[derive(Debug, Default, Serialize, Deserialize)]
struct FileState {
    users: BTreeMap<UserId, UserRow>,
    groups: BTreeMap<GroupId, GroupRow>,
    expenses: BTreeMap<ExpenseId, ExpenseRow>,
}

/// ===============================
/// FileStore
/// ===============================
pub struct FileStore {
    path: PathBuf,
    state: RwLock<FileState>,
}

impl FileStore {
    /// Load or create a file-backed store
    pub async fn connect(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();

        let state = if tokio::fs::try_exists(&path).await? {
            debug!("Reading FileStore from {:?}", path);
            let raw = tokio::fs::read(&path).await?;
            serde_json::from_slice(&raw)?
        } else {
            FileState::default()
        };

        Ok(Self {
            path,
            state: RwLock::new(state),
        })
    }

    /// Persist state atomically
    async fn persist(&self) -> Result<()> {
        debug!("Persisting FileStore to {:?}", self.path);

        if let Some(parent) = self.path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let tmp = self.path.with_extension("tmp");
        let state = self.state.read().await;

        let data = serde_json::to_vec_pretty(&*state)?;
        tokio::fs::write(&tmp, data).await?;
        tokio::fs::rename(&tmp, &self.path).await?;

        Ok(())
    }
}

/// ===============================
/// Store implementation
/// ===============================

#[async_trait]
impl Store for FileStore {
    // -------- Users --------

    async fn create_user(&self, user: UserRow) -> Result<UserRow> {
        let mut state = self.state.write().await;
        state.users.insert(user.id, user.clone());
        drop(state);
        self.persist().await?;
        Ok(user)
    }

    async fn get_user_by_id(&self, id: UserId) -> Result<Option<UserRow>> {
        Ok(self.state.read().await.users.get(&id).cloned())
    }

    async fn get_user_by_name(&self, name: String) -> Result<Option<UserRow>> {
        let guard = self.state.read().await;
        Ok(guard.users.values().find(|u| u.username.eq(&name)).cloned())
    }

    async fn delete_user(&self, id: UserId) -> Result<()> {
        let mut state = self.state.write().await;
        state.users.remove(&id);
        drop(state);
        self.persist().await?;
        Ok(())
    }

    async fn list_users(&self) -> Result<Vec<UserRow>> {
        Ok(self.state.read().await.users.values().cloned().collect())
    }

    async fn _update_user(&self, user: UserRow) -> Result<UserRow> {
        let mut state = self.state.write().await;
        state.users.insert(user.id, user.clone());
        drop(state);
        self.persist().await?;
        Ok(user)
    }
    async fn create_friend_request(&self, request: FriendRequestRow) -> Result<FriendRequestRow> {
        Ok(request)
    }

    async fn get_friend_request(&self, _request: FriendRequestId) -> Result<FriendRequestRow> {
        todo!()
    }

    async fn get_outgoing_requests(&self, _user: UserId) -> Result<Vec<FriendRequestRow>> {
        todo!()
    }
    async fn get_incoming_requests(&self, _user: UserId) -> Result<Vec<FriendRequestRow>> {
        todo!()
    }

    async fn update_friend_request(&self, _request: FriendRequestRow) -> Result<()> {
        todo!()
    }

    async fn delete_friend_requests_from_user(&self, _user: UserId) -> Result<()> {
        todo!()
    }

    async fn add_friendship(&self, _user1: UserId, _user2: UserId) -> Result<()> {
        todo!()
    }

    async fn remove_friendship(&self, _user: UserId) -> Result<()> {
        todo!()
    }
    // -------- Groups --------

    async fn create_group(&self, group: GroupRow) -> Result<GroupRow> {
        let mut state = self.state.write().await;
        state.groups.insert(group.id, group.clone());
        drop(state);
        self.persist().await?;
        Ok(group)
    }

    async fn delete_group(&self, id: UserId) -> Result<()> {
        let mut state = self.state.write().await;
        state.groups.remove(&id);
        drop(state);
        self.persist().await?;
        Ok(())
    }

    async fn get_group(&self, id: GroupId) -> Result<Option<GroupRow>> {
        Ok(self.state.read().await.groups.get(&id).cloned())
    }

    async fn get_groups(&self) -> Result<Vec<GroupRow>> {
        Ok(self.state.read().await.groups.values().cloned().collect())
    }

    async fn update_group(&self, group: GroupRow) -> Result<GroupRow> {
        let mut state = self.state.write().await;
        state.groups.insert(group.id, group.clone());
        drop(state);
        self.persist().await?;
        Ok(group)
    }

    // -------- Expenses --------

    async fn create_expense(&self, expense: ExpenseRow) -> Result<ExpenseRow> {
        let mut state = self.state.write().await;
        state.expenses.insert(expense.id, expense.clone());
        drop(state);
        self.persist().await?;
        Ok(expense)
    }

    async fn delete_expense(&self, id: ExpenseId) -> Result<()> {
        let mut state = self.state.write().await;
        state.expenses.remove(&id);
        drop(state);
        self.persist().await?;
        Ok(())
    }

    async fn get_expense(&self, id: ExpenseId) -> Result<Option<ExpenseRow>> {
        Ok(self.state.read().await.expenses.get(&id).cloned())
    }

    async fn list_expenses_for_user(&self, user_id: UserId) -> Result<Vec<ExpenseRow>> {
        let state = self.state.read().await;
        Ok(state
            .expenses
            .values()
            .filter(|e| e.payer == user_id || e.participants.contains(&user_id))
            .cloned()
            .collect())
    }

    async fn list_expenses_for_group(&self, group_id: GroupId) -> Result<Vec<ExpenseRow>> {
        let state = self.state.read().await;
        Ok(state
            .expenses
            .values()
            .filter(|e| e.group_id == Some(group_id))
            .cloned()
            .collect())
    }
}
