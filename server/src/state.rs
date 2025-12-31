use std::sync::Arc;

use anyhow::{anyhow, Result};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::db::{ExpenseRow, GroupRow, Store, UserRow};
use common::{
    Expense, Group, User, UserId,
    api::{CreateExpenseRequest, CreateGroupRequest},
};

struct AppStateData {
    store: Box<dyn Store>,
}

impl AppStateData {
    fn new(
        store: impl Store + 'static,
    ) -> Self {
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
    pub fn new(
        store: impl Store + 'static,
    ) -> AppState {
        AppState {
            data: Arc::new(RwLock::new(AppStateData::new(
                store,
            ))),
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
        let mut guard = self.data.write().await;
        let Some(mut user) = guard.store.get_user(user_id).await? else {
            return Err(anyhow!("User not found"));
        };
        let Some(mut friend) = guard.store.get_user(friend_id).await? else {
            return Err(anyhow!("Friend not found"));
        };

        if !user.friends.contains(&friend_id) {
            let mut new_friends = Vec::from(user.friends);
            new_friends.push(friend_id);
            user = UserRow::new(user.id, user.username, new_friends);
            guard.store.update_user(user.into()).await?;
        }

        if !friend.friends.contains(&user_id) {
            let mut new_friends = Vec::from(friend.friends);
            new_friends.push(user_id);
            friend = UserRow::new(friend.id, friend.username, new_friends);
            guard.store.update_user(friend).await?;
        }
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
