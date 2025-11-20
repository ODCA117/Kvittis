use anyhow::{anyhow, Result};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::types::{Expense, ExpenseId, Group, GroupId, User, UserId};

pub trait DataBase {
    fn connect(path: &str) -> Result<impl DataBase>;
    fn disconnect(self) -> Result<()>;
    fn commit(&self) -> Result<()>;
}

pub struct UserRow {
    id: UserId,
    username: String,
    friends: Vec<UserId>,
}

impl From<User> for UserRow {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            username: value.username,
            friends: value.friends,
        }
    }
}

pub struct GroupRow {
    id: GroupId,
    name: String,
    owner_id: UserId,
    members: Vec<UserId>,
}

impl From<Group> for GroupRow {
    fn from(value: Group) -> Self {
        Self {
            id: value.id,
            name: value.name,
            owner_id: value.owner_id,
            members: value.members,
        }
    }
}

pub struct ExpenseRow {
    id: ExpenseId,
    payer: UserId,
    participants: Vec<UserId>,
    amount: f64,
    description: Option<String>,
    group_id: Option<GroupId>,
    timestamp_ms: i64,
}

impl From<Expense> for ExpenseRow {
    fn from(value: Expense) -> Self {
        Self {
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

pub struct UserFileDB {
    path: PathBuf,
    data: BTreeMap<UserId, User>,
}

impl DataBase for UserFileDB {
    fn connect(path: &str) -> Result<impl DataBase> {
        let path = PathBuf::from_str(path)?;
        match read_file_db(&path)? {
            Some(data) => {
                let data = serde_json::from_slice(&data)?;
                Ok(Self { path, data })
            }
            None => Ok(Self {
                path,
                data: BTreeMap::new(),
            }),
        }
    }

    fn disconnect(self) -> Result<()> {
        self.commit()
    }

    fn commit(&self) -> Result<()> {
        let data = serde_json::to_vec(&self.data)?;
        let mut file = std::fs::OpenOptions::new().write(true).create(true).open(&self.path)?;
        file.write_all(&data)?;
        Ok(())
    }
}

pub struct GroupFileDB {
    path: PathBuf,
    data: BTreeMap<GroupId, Group>,
}

impl DataBase for GroupFileDB {
    fn connect(path: &str) -> Result<impl DataBase> {
        let path = PathBuf::from_str(path)?;
        match read_file_db(&path)? {
            Some(data) => {
                let data = serde_json::from_slice(&data)?;
                Ok(Self { path, data })
            }
            None => Ok(Self {
                path,
                data: BTreeMap::new(),
            }),
        }
    }

    fn disconnect(self) -> Result<()> {
        self.commit()
    }

    fn commit(&self) -> Result<()> {
        let data = serde_json::to_vec(&self.data)?;
        let mut file = std::fs::OpenOptions::new().write(true).create(true).open(&self.path)?;
        file.write_all(&data)?;
        Ok(())
    }
}

pub struct ExpenseFileDB {
    path: PathBuf,
    data: BTreeMap<ExpenseId, Expense>,
}

impl DataBase for ExpenseFileDB {
    fn connect(path: &str) -> Result<impl DataBase> {
        let path = PathBuf::from_str(path)?;
        match read_file_db(&path)? {
            Some(data) => {
                let data = serde_json::from_slice(&data)?;
                Ok(Self { path, data })
            }
            None => Ok(Self {
                path,
                data: BTreeMap::new(),
            }),
        }
    }

    fn disconnect(self) -> Result<()> {
        self.commit()
    }

    fn commit(&self) -> Result<()> {
        let data = serde_json::to_vec(&self.data)?;
        let mut file = std::fs::OpenOptions::new().write(true).create(true).open(&self.path)?;
        file.write_all(&data)?;
        Ok(())
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
