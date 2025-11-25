use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;

use crate::db::{ExpenseDB, ExpenseRow, GroupDB, GroupRow, UserDB, UserRow};
use common::{Expense, Group, User, UserId};

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
        }
        .into())
    }

    pub fn get_user(&self, id: UserId) -> Result<User> {
        let guard = self.data.read();
        guard.user_db.get_user(id).map(|u| (*u).clone().into())
    }

    pub fn edit_user(&self, updated_user: User) -> Result<User> {
        let mut guard = self.data.write();
        let stored = guard.user_db.update_user(updated_user.into())?;
        Ok(User {
            id: stored.id(),
            username: stored.username().to_owned(),
            friends: Vec::from(stored.friends()),
        }
        .into())
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
