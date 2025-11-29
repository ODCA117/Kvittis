use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};
use tracing::debug;

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
    // group operations later
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

pub struct UserFileDB {
    path: PathBuf,
    data: BTreeMap<UserId, UserRow>,
}

impl UserFileDB {
    pub fn connect(path: &str) -> Result<Self> {
        let path = PathBuf::from_str(path)?;
        match read_file_db(&path)? {
            Some(raw) => {
                debug!("Read file UserDB");
                let data = serde_json::from_slice(&raw)?;
                Ok(Self { path, data })
            }
            None => Ok(Self {
                path,
                data: BTreeMap::new(),
            }),
        }
    }
    pub fn disconnect(self) -> Result<()> {
        self.commit()
    }
}

impl DataBase for UserFileDB {
    fn commit(&self) -> Result<()> {
        debug!("Commit database: UserDB");
        let data = serde_json::to_vec(&self.data)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)?;
        file.write_all(&data)?;
        Ok(())
    }
}

impl UserDB for UserFileDB {
    fn register(&mut self, user: UserRow) -> Result<&UserRow> {
        let id = user.id;
        self.data.insert(id, user);
        self.data.get(&id).ok_or(anyhow!("User not found"))
    }

    fn get_user(&self, id: UserId) -> Result<&UserRow> {
        self.data.get(&id).ok_or(anyhow!("User not found"))
    }

    fn get_users(&self) -> Vec<&UserRow> {
        self.data.values().collect::<Vec<&UserRow>>()
    }

    fn update_user(&mut self, user: UserRow) -> Result<&UserRow> {
        let id = user.id;
        self.data.insert(id, user);
        self.data.get(&id).ok_or(anyhow!("User not found"))
    }
}

pub struct GroupFileDB {
    path: PathBuf,
    data: BTreeMap<GroupId, GroupRow>,
}

impl GroupFileDB {
    pub fn connect(path: &str) -> Result<Self> {
        let path = PathBuf::from_str(path)?;
        match read_file_db(&path)? {
            Some(raw) => {
                let data = serde_json::from_slice(&raw)?;
                Ok(Self { path, data })
            }
            None => Ok(Self {
                path,
                data: BTreeMap::new(),
            }),
        }
    }
    pub fn disconnect(self) -> Result<()> {
        self.commit()
    }
}

impl DataBase for GroupFileDB {
    fn commit(&self) -> Result<()> {
        debug!("Commit database: GroupDB");
        let data = serde_json::to_vec(&self.data)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)?;
        file.write_all(&data)?;
        Ok(())
    }
}

impl GroupDB for GroupFileDB {}

pub struct ExpenseFileDB {
    path: PathBuf,
    data: BTreeMap<ExpenseId, ExpenseRow>,
}

impl ExpenseFileDB {
    pub fn connect(path: &str) -> Result<Self> {
        let path = PathBuf::from_str(path)?;
        match read_file_db(&path)? {
            Some(raw) => {
                let data = serde_json::from_slice(&raw)?;
                Ok(Self { path, data })
            }
            None => Ok(Self {
                path,
                data: BTreeMap::new(),
            }),
        }
    }

    pub fn disconnect(self) -> Result<()> {
        self.commit()
    }
}

impl DataBase for ExpenseFileDB {
    fn commit(&self) -> Result<()> {
        debug!("Commit database: ExpenseDB");
        let data = serde_json::to_vec(&self.data)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(&self.path)?;
        file.write_all(&data)?;
        Ok(())
    }
}

impl ExpenseDB for ExpenseFileDB {
    fn create_expense(&mut self, expense: ExpenseRow) -> Result<&ExpenseRow> {
        let id = expense.id;
        self.data.insert(id, expense);
        self.data.get(&id).ok_or(anyhow!("Expense not found"))
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
