use std::sync::Arc;

use anyhow::{Result, anyhow};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::db::{ExpenseRow, GroupRow, Store, UserRow};
use std::collections::HashMap;

use common::{
    Expense, Group, GroupId, User, UserId,
    api::{
        BalanceEntry, CreateExpenseRequest, CreateGroupRequest, DeleteExpenseRequest,
        GetExpenseRequest, GroupBalance, NewGroupMemberRequest,
    },
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

    pub async fn get_expense(&self, expense_req: GetExpenseRequest) -> Result<Expense> {
        let guard = self.data.write().await;
        let stored = guard.store.get_expense(expense_req.id).await?.ok_or_else(|| anyhow!("Expense not found"))?;
        Ok(stored.into())
    }

    pub async fn delete_expense(&self, expense_req: DeleteExpenseRequest) -> Result<()> {
        let guard = self.data.write().await;
        guard.store.delete_expense(expense_req.id).await?;
        Ok(())
    }

    pub async fn list_expenses_for_user(&self, user_id: UserId) -> Result<Vec<Expense>> {
        let guard = self.data.read().await;
        let rows = guard.store.list_expenses_for_user(user_id).await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    pub async fn list_expenses_for_group(&self, group_id: GroupId) -> Result<Vec<Expense>> {
        let guard = self.data.read().await;
        let rows = guard.store.list_expenses_for_group(group_id).await?;
        Ok(rows.into_iter().map(|r| r.into()).collect())
    }

    /// Returns per-counterparty net balances for non-group expenses only.
    /// Positive amount → counterparty owes `user_id`.
    /// Negative amount → `user_id` owes counterparty.
    pub async fn get_user_non_group_balances(&self, user_id: UserId) -> Result<Vec<BalanceEntry>> {
        let expenses = self.list_expenses_for_user(user_id).await?;
        let non_group: Vec<Expense> = expenses.into_iter().filter(|e| e.group_id.is_none()).collect();

        let mut net: HashMap<UserId, i64> = HashMap::new();

        for expense in &non_group {
            let n = expense.participants.len() as i64;
            if n == 0 { continue; }

            // Deterministic integer split: sort participants by UserId ascending;
            // first `rem` get base+1, rest get base.
            let mut sorted_participants = expense.participants.clone();
            sorted_participants.sort();

            let base = expense.amount / n;
            let rem = (expense.amount % n) as usize;

            let shares: Vec<i64> = sorted_participants
                .iter()
                .enumerate()
                .map(|(i, _)| if i < rem { base + 1 } else { base })
                .collect();

            for (participant, share) in sorted_participants.iter().zip(shares.iter()) {
                if participant == &expense.payer {
                    continue; // payer doesn't owe themselves
                }
                if expense.payer == user_id {
                    // We are the payer: participant owes us their share
                    *net.entry(*participant).or_insert(0) += *share;
                } else if participant == &user_id {
                    // We are a participant: we owe the payer our share
                    *net.entry(expense.payer).or_insert(0) -= *share;
                }
            }
        }

        Ok(net
            .into_iter()
            .filter(|(_, amount)| *amount != 0)
            .map(|(other, amount)| BalanceEntry { other, amount })
            .collect())
    }

    /// Returns the minimal settlement transfers for a group.
    /// All group expenses are included regardless of which members are involved.
    pub async fn get_group_balance_overview(&self, group_id: GroupId) -> Result<Vec<GroupBalance>> {
        let expenses = self.list_expenses_for_group(group_id).await?;

        // Compute net position per member: net = paid − owed_share
        let mut net: HashMap<UserId, i64> = HashMap::new();

        for expense in &expenses {
            let n = expense.participants.len() as i64;
            if n == 0 { continue; }

            let mut sorted_participants = expense.participants.clone();
            sorted_participants.sort();

            let base = expense.amount / n;
            let rem = (expense.amount % n) as usize;

            let shares: Vec<i64> = sorted_participants
                .iter()
                .enumerate()
                .map(|(i, _)| if i < rem { base + 1 } else { base })
                .collect();

            // Payer gets the full amount credited
            *net.entry(expense.payer).or_insert(0) += expense.amount;

            // Each participant has their share debited
            for (participant, share) in sorted_participants.iter().zip(shares.iter()) {
                *net.entry(*participant).or_insert(0) -= share;
            }
        }

        // Greedy settlement: sort debtors and creditors by UserId for determinism
        let mut debtors: Vec<(UserId, i64)> = net
            .iter()
            .filter(|(_, v)| **v < 0)
            .map(|(&id, &v)| (id, -v)) // store as positive "amount to pay"
            .collect();
        let mut creditors: Vec<(UserId, i64)> = net
            .iter()
            .filter(|(_, v)| **v > 0)
            .map(|(&id, &v)| (id, v))
            .collect();

        debtors.sort_by_key(|(id, _)| *id);
        creditors.sort_by_key(|(id, _)| *id);

        let mut transfers: Vec<GroupBalance> = Vec::new();
        let mut di = 0;
        let mut ci = 0;
        while di < debtors.len() && ci < creditors.len() {
            let paid = debtors[di].1.min(creditors[ci].1);
            transfers.push(GroupBalance { from: debtors[di].0, to: creditors[ci].0, amount: paid });
            debtors[di].1 -= paid;
            creditors[ci].1 -= paid;
            if debtors[di].1 == 0 { di += 1; }
            if creditors[ci].1 == 0 { ci += 1; }
        }

        Ok(transfers)
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

    pub async fn delete_group(&self, group_req: GroupId) -> Result<()> {
        let guard = self.data.write().await;
        guard.store.delete_group(group_req).await?;
        Ok(())
    }

    pub async fn get_group(&self, group_id: GroupId) -> Result<Option<Group>> {
        let guard = self.data.write().await;
        debug!("GET GROUP!!!!");
        guard.store.get_group(group_id).await.map(|g| g.map(|g| g.into()))
    }

    pub async fn search_groups(&self, query: &str) -> Result<Vec<Group>> {
        let guard = self.data.write().await;
        debug!("Search gropu");
        let groups = guard.store.get_groups().await?;
        let groups: Vec<Group> = groups.into_iter()
            .filter(|g| g.name.contains(query))
            .map(|g| g.into())
            .collect();
        debug!("Search groups with query '{}': found {} groups", query, groups.len());
        Ok(groups)
    }

    pub async fn new_group_member(&self, req: NewGroupMemberRequest) -> Result<Group> {
        let guard = self.data.write().await;
        let mut group = guard.store.get_group(req.group_id).await?.ok_or_else(|| anyhow!("Group not found"))?;
        group.members.push(req.new_member);
        let stored = guard.store.update_group(group.clone().into()).await?;
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
