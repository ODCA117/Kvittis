use crate::db::{
    DataBase, ExpenseDB, ExpenseRow, GroupDB, GroupRow, UserDB, UserRow, read_file_db,
};
use anyhow::{Result, anyhow};
use common::{ExpenseId, GroupId, UserId};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use tracing::debug;

pub struct UserFileDB {
    path: PathBuf,
    data: BTreeMap<UserId, UserRow>,
}

impl UserFileDB {
    pub fn connect(path: &Path) -> Result<Self> {
        match read_file_db(path)? {
            Some(raw) => {
                debug!("Read file UserDB");
                let data = serde_json::from_slice(&raw)?;
                Ok(Self {
                    path: path.to_path_buf(),
                    data,
                })
            }
            None => Ok(Self {
                path: path.to_path_buf(),
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
        debug!("Commit database: UserDB to {:?}", &self.path);
        let data = serde_json::to_vec(&self.data)?;

        if let Some(parent) = self.path.parent() {
            debug!("Create parent dir: {:?}", &parent);
            fs::create_dir_all(parent)?;
        }

        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
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
    pub fn connect(path: &Path) -> Result<Self> {
        match read_file_db(path)? {
            Some(raw) => {
                let data = serde_json::from_slice(&raw)?;
                Ok(Self {
                    path: path.to_path_buf(),
                    data,
                })
            }
            None => Ok(Self {
                path: path.to_path_buf(),
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
        debug!("Commit database: GroupDB to {:?}", &self.path);

        if let Some(parent) = self.path.parent() {
            debug!("Create parent dir: {:?}", &parent);
            fs::create_dir_all(parent)?;
        }

        let data = serde_json::to_vec(&self.data)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.path)?;
        file.write_all(&data)?;
        Ok(())
    }
}

impl GroupDB for GroupFileDB {
    fn create_group(&mut self, group: GroupRow) -> Result<&GroupRow> {
        let id = group.id;
        self.data.insert(id, group);
        self.data.get(&id).ok_or(anyhow!("Group not found"))
    }
}

pub struct ExpenseFileDB {
    path: PathBuf,
    data: BTreeMap<ExpenseId, ExpenseRow>,
}

impl ExpenseFileDB {
    pub fn connect(path: &Path) -> Result<Self> {
        match read_file_db(path)? {
            Some(raw) => {
                let data = serde_json::from_slice(&raw)?;
                Ok(Self {
                    path: path.to_path_buf(),
                    data,
                })
            }
            None => Ok(Self {
                path: path.to_path_buf(),
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
        debug!("Commit database: ExpenseDB to {:?}", &self.path);

        if let Some(parent) = self.path.parent() {
            debug!("Create parent dir: {:?}", &parent);
            fs::create_dir_all(parent)?;
        }

        let data = serde_json::to_vec(&self.data)?;
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
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
