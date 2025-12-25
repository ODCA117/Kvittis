use std::{iter::chain, sync::Arc};

use anyhow::Result;
use parking_lot::RwLock;
use uuid::Uuid;

use crate::db::{ExpenseDB, ExpenseRow, GroupDB, GroupRow, UserDB, UserRow};
use common::{
    Expense, Group, User, UserId,
    api::{CreateExpenseRequest, CreateGroupRequest},
};

struct AppStateData {
    user_db: Box<dyn UserDB>,
    group_db: Box<dyn GroupDB>,
    expense_db: Box<dyn ExpenseDB>,
}

impl AppStateData {
    fn new(
        user_db: impl UserDB + 'static,
        group_db: impl GroupDB + 'static,
        expense_db: impl ExpenseDB + 'static,
    ) -> Self {
        Self {
            user_db: Box::new(user_db),
            group_db: Box::new(group_db),
            expense_db: Box::new(expense_db),
        }
    }
}

#[derive(Clone)]
pub struct AppState {
    data: Arc<RwLock<AppStateData>>,
}

impl AppState {
    pub fn new(
        user_db: impl UserDB + 'static,
        group_db: impl GroupDB + 'static,
        expense_db: impl ExpenseDB + 'static,
    ) -> AppState {
        AppState {
            data: Arc::new(RwLock::new(AppStateData::new(
                user_db, group_db, expense_db,
            ))),
        }
    }

    pub fn commit_all(&mut self) -> Result<()> {
        let guard = self.data.write();
        guard.user_db.commit()?;
        guard.group_db.commit()?;
        guard.expense_db.commit()?;
        Ok(())
    }

    pub fn register_user(&self, user: User) -> Result<User> {
        let mut guard = self.data.write();
        let stored = guard.user_db.register(user.into())?;
        Ok(User {
            id: stored.id(),
            username: stored.username().to_owned(),
            friends: Vec::from(stored.friends()),
        })
    }

    pub fn get_user(&self, id: UserId) -> Result<User> {
        let guard = self.data.read();
        guard.user_db.get_user(id).map(|u| (*u).clone().into())
    }

    pub fn get_users(&self) -> Vec<User> {
        let guard = self.data.read();
        guard
            .user_db
            .get_users()
            .into_iter()
            .map(|u| (*u).clone().into())
            .collect()
    }

    // FIXME: This should require some form of confirmation/authentication
    fn edit_user(&self, updated_user: User) -> Result<User> {
        let mut guard = self.data.write();
        let stored = guard.user_db.update_user(updated_user.into())?;
        Ok(User {
            id: stored.id(),
            username: stored.username().to_owned(),
            friends: Vec::from(stored.friends()),
        })
    }

    // FIXME: Require confirmation on both parties.
    pub fn add_friend(&self, user_id: UserId, friend_id: UserId) -> Result<()> {
        let mut guard = self.data.write();
        let mut user = guard.user_db.get_user(user_id)?.clone();
        let mut friend = guard.user_db.get_user(friend_id)?.clone();

        if !user.friends().contains(&friend_id) {
            let mut new_friends = Vec::from(user.friends());
            new_friends.push(friend_id);
            user = UserRow::new(user.id(), user.username().to_owned(), new_friends);
            guard.user_db.update_user(user)?;
        }

        if !friend.friends().contains(&user_id) {
            let mut new_friends = Vec::from(friend.friends());
            new_friends.push(user_id);
            friend = UserRow::new(friend.id(), friend.username().to_owned(), new_friends);
            guard.user_db.update_user(friend)?;
        }
        Ok(())
    }

    pub fn create_expense(&self, expense_req: CreateExpenseRequest) -> Result<Expense> {
        let mut guard = self.data.write();
        let expense = Expense {
            id: Uuid::new_v4(),
            payer: expense_req.payer,
            participants: expense_req.participants,
            amount: expense_req.amount,
            description: expense_req.description,
            group_id: expense_req.group_id,
            timestamp_ms: chrono::Utc::now().timestamp_millis(),
        };
        let stored = guard.expense_db.create_expense(expense.into())?;
        Ok(Expense {
            id: stored.id(),
            payer: stored.payer(),
            participants: stored.participants().to_owned(),
            amount: stored.amount(),
            description: stored.description().cloned(),
            group_id: stored.group_id(),
            timestamp_ms: stored.timestamp_ms(),
        })
    }

    pub fn create_group(&self, group_req: CreateGroupRequest) -> Result<Group> {
        let mut guard = self.data.write();
        let group = Group {
            id: Uuid::new_v4(),
            name: group_req.name,
            owner_id: group_req.owner_id,
            members: group_req.members,
        };
        let stored = guard.group_db.create_group(group.clone().into())?;
        Ok(Group {
            id: stored.id(),
            name: stored.name().to_owned(),
            owner_id: stored.owner_id(),
            members: stored.members().to_owned(),
        })
    }
}

impl From<User> for UserRow {
    fn from(value: User) -> Self {
        UserRow::new(value.id, value.username, value.friends)
    }
}

impl From<UserRow> for User {
    fn from(r: UserRow) -> Self {
        User {
            id: r.id(),
            username: r.username().to_owned(),
            friends: Vec::from(r.friends()),
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
            id: value.id(),
            name: value.name().to_owned(),
            owner_id: value.owner_id(),
            members: value.members().to_owned(),
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
            id: value.id(),
            payer: value.payer(),
            participants: value.participants().to_owned(),
            amount: value.amount(),
            description: value.description().cloned(),
            group_id: value.group_id(),
            timestamp_ms: value.timestamp_ms(),
        }
    }
}
