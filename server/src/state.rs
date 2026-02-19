use std::sync::Arc;

use anyhow::{Result, anyhow};
use tokio::sync::RwLock;
use tracing::{debug, warn};
use uuid::Uuid;

use crate::db::{ExpenseRow, GroupRow, Store, UserRow};
use common::{
    Expense, Group, GroupId, User, UserId, api::{CreateExpenseRequest, CreateGroupRequest}
};

struct AppStateData {
    store: Box<dyn Store>,
}

impl AppStateData {
    fn new(store: impl Store + 'static) -> Self {
        Self {
            store: Box::new(store),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    data: Arc<RwLock<AppStateData>>,
}

impl AppState {
    pub fn new(store: impl Store + 'static) -> AppState {
        AppState {
            data: Arc::new(RwLock::new(AppStateData::new(store))),
        }
    }

    // pub fn commit_all(&mut self) -> Result<()> {
    //     let guard = self.data.write();
    //     guard.user_db.commit()?;
    //     guard.group_db.commit()?;
    //     guard.expense_db.commit()?;
    //     Ok(())
    // }

    pub async fn register_user(&self, user: User) -> Result<User> {
        let guard = self.data.write().await;
        let stored = guard.store.create_user(user.into()).await?;

        debug!("User added: {:?}", stored);
        Ok(User {
            id: stored.id,
            username: stored.username,
            friends: stored.friends,
        })
    }

    pub async fn get_user(&self, id: UserId) -> Result<User> {
        let guard = self.data.read().await;
        match guard.store.get_user(id).await? {
            Some(u) => Ok(u.into()),
            None => Err(anyhow!("Failed to get user")),
        }
    }

    pub async fn get_users(&self) -> Result<Vec<User>> {
        let guard = self.data.read().await;
        Ok(guard
            .store
            .list_users()
            .await?
            .into_iter()
            .map(|u| u.into())
            .collect())
    }

    pub async fn search_users(&self, query: &str) -> Result<Vec<User>> {
        let guard = self.data.read().await;
        // NOTE: This is could be optimized later.
        let users = guard
            .store
            .list_users()
            .await?;
        let users: Vec<User> = users.into_iter()
            .filter(|u| u.username.contains(query))
            .map(|u| u.into())
            .collect();
        debug!("Search users with query '{}': found {} users", query, users.len());
        Ok(users)
    }

    pub async fn delete_user(&self, id: UserId) -> Result<()> {
        let guard = self.data.write().await;
        guard.store.delete_user(id).await?;
        Ok(())
    }

    // FIXME: This should require some form of confirmation/authentication
    pub async fn edit_user(&self, updated_user: User) -> Result<User> {
        let guard = self.data.write().await;
        let stored = guard.store.update_user(updated_user.into()).await?;
        Ok(User {
            id: stored.id,
            username: stored.username,
            friends: stored.friends,
        })
    }

    // FIXME: Require confirmation on both parties.
    pub async fn add_friend(&self, user_id: UserId, friend_id: UserId) -> Result<()> {
        let guard = self.data.write().await;
        // let Some(mut user) = guard.store.get_user(user_id).await? else {
        //     return Err(anyhow!("User not found"));
        // };
        // let Some(mut friend) = guard.store.get_user(friend_id).await? else {
        //     return Err(anyhow!("Friend not found"));
        // };

        guard.store.add_friend(user_id, friend_id).await?;

        Ok(())
    }

    pub async fn create_expense(&self, expense_req: CreateExpenseRequest) -> Result<Expense> {
        let guard = self.data.write().await;
        let expense = Expense {
            id: Uuid::new_v4(),
            payer: expense_req.payer,
            participants: expense_req.participants,
            amount: expense_req.amount,
            description: expense_req.description,
            group_id: expense_req.group_id,
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
        };
        let stored = guard.store.create_expense(expense.into()).await?;
        Ok(stored.into())
    }

    pub async fn create_group(&self, group_req: CreateGroupRequest) -> Result<Group> {
        let guard = self.data.write().await;
        let group = Group {
            id: Uuid::new_v4(),
            name: group_req.name,
            owner_id: group_req.owner_id,
            members: group_req.members,
        };
        let stored = guard.store.create_group(group.clone().into()).await?;
        Ok(stored.into())
    }

    pub async fn get_group(&self, group_id: GroupId) -> Result<Option<Group>> {
        let guard = self.data.write().await;
        warn!("GET GROUP!!!!");
        guard.store.get_group(group_id).await.map(|g| g.map(|g| g.into()))
    }
}

impl From<User> for UserRow {
    fn from(value: User) -> Self {
        UserRow::new(value.id, value.username, value.friends)
    }
}

impl From<UserRow> for User {
    fn from(user: UserRow) -> Self {
        User {
            id: user.id,
            username: user.username,
            friends: user.friends,
        }
    }
}

impl From<Group> for GroupRow {
    fn from(value: Group) -> Self {
        GroupRow::new(value.id, value.name, value.owner_id, value.members)
    }
}

impl From<GroupRow> for Group {
    fn from(value: GroupRow) -> Self {
        Group {
            id: value.id,
            name: value.name,
            owner_id: value.owner_id,
            members: value.members,
        }
    }
}

impl From<Expense> for ExpenseRow {
    fn from(value: Expense) -> Self {
        ExpenseRow::new(
            value.id,
            value.payer,
            value.participants,
            value.amount,
            value.description,
            value.group_id,
            value.timestamp_ms,
        )
    }
}

impl From<ExpenseRow> for Expense {
    fn from(value: ExpenseRow) -> Self {
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
