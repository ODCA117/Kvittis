pub mod db_file;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{io::Read, path::Path};

use common::{ExpenseId, GroupId, UserId};

// Object-safe base trait (no Sized)
pub trait DataBase {
    fn commit(&self) -> Result<()>;
}

// Specialised traits (object-safe)
pub trait UserDB: DataBase + Send + Sync {
    fn register(&mut self, user: UserRow) -> Result<&UserRow>;
    fn get_user(&self, id: UserId) -> Result<&UserRow>;
    fn get_users(&self) -> Vec<&UserRow>;
    fn update_user(&mut self, user: UserRow) -> Result<&UserRow>;
}

pub trait GroupDB: DataBase + Send + Sync {
    fn create_group(&mut self, group: GroupRow) -> Result<&GroupRow>;
}

pub trait ExpenseDB: DataBase + Send + Sync {
    // expense operations later
    fn create_expense(&mut self, expense: ExpenseRow) -> Result<&ExpenseRow>;
}

#[derive(Serialize, Deserialize, Clone)]
pub struct UserRow {
    id: UserId,
    username: String,
    friends: Vec<UserId>,
}

impl UserRow {
    pub fn new(id: UserId, username: String, friends: Vec<UserId>) -> Self {
        Self {
            id,
            username,
            friends,
        }
    }
    pub fn id(&self) -> UserId {
        self.id
    }
    pub fn username(&self) -> &str {
        &self.username
    }
    pub fn friends(&self) -> &[UserId] {
        &self.friends
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GroupRow {
    id: GroupId,
    name: String,
    owner_id: UserId,
    members: Vec<UserId>,
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

    pub fn id(&self) -> GroupId {
        self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn owner_id(&self) -> UserId {
        self.owner_id
    }

    pub fn members(&self) -> &Vec<UserId> {
        &self.members
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ExpenseRow {
    id: ExpenseId,
    payer: UserId,
    participants: Vec<UserId>,
    amount: u64,
    description: Option<String>,
    group_id: Option<GroupId>,
    timestamp_ms: i64,
}

impl ExpenseRow {
    pub fn new(
        id: ExpenseId,
        payer: UserId,
        participants: Vec<UserId>,
        amount: u64,
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

    pub fn id(&self) -> ExpenseId {
        self.id
    }

    pub fn payer(&self) -> UserId {
        self.payer
    }

    pub fn participants(&self) -> &Vec<UserId> {
        &self.participants
    }

    pub fn amount(&self) -> u64 {
        self.amount
    }

    pub fn description(&self) -> Option<&String> {
        self.description.as_ref()
    }

    pub fn group_id(&self) -> Option<GroupId> {
        self.group_id
    }

    pub fn timestamp_ms(&self) -> i64 {
        self.timestamp_ms
    }
}

fn read_file_db(path: &Path) -> Result<Option<Vec<u8>>> {
    match std::fs::exists(path) {
        Ok(true) => {
            let mut file = std::fs::OpenOptions::new().read(true).open(path)?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;
            Ok(Some(buf))
        }
        Ok(false) => Ok(None),
        Err(e) => Err(anyhow!(e.to_string())),
    }
}
