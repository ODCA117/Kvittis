use std::sync::Arc;

use anyhow::Result;
use parking_lot::RwLock;

use crate::db::{ExpenseDB, GroupDB, UserDB, UserRow};
use crate::types::{User, UserId};

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

    pub fn commit_all(&mut self) -> Result<()>{
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
        Self::new(value.id, value.username, value.friends)
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
