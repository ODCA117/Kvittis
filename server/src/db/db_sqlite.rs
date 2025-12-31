use common::UserId;
use sqlx::{SqlitePool, Row};
use anyhow::Result;

use crate::db::{DataBase, UserDB, UserRow};

pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait::async_trait]
impl Store for SqliteStore {
    async fn create_user(&self, user: UserRow) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO users (id, username, friends)
            VALUES (?, ?, ?)
            "#,
        )
        .bind(user.id)
        .bind(user.username)
        .bind(serde_json::to_string(&user.friends)?)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_user(&self, id: UserId) -> Result<Option<UserRow>> {
        let row = sqlx::query(
            r#"
            SELECT id, username, friends
            FROM users
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            return Ok(None);
        };

        Ok(Some(UserRow {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            friends: serde_json::from_str(row.try_get::<String, _>("friends")?)?,
        }))
    }

    async fn list_users(&self) -> Result<Vec<UserRow>> {
        let rows = sqlx::query(
            r#"
            SELECT id, username, friends
            FROM users
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let users = rows
            .into_iter()
            .map(|row| {
                Ok(UserRow {
                    id: row.try_get("id")?,
                    username: row.try_get("username")?,
                    friends: serde_json::from_str(row.try_get::<String, _>("friends")?)?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok(users)
    }

    async fn update_user(&self, user: UserRow) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE users
            SET username = ?, friends = ?
            WHERE id = ?
            "#,
        )
        .bind(user.username)
        .bind(serde_json::to_string(&user.friends)?)
        .bind(user.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // Groups / Expenses omitted for brevity but same pattern
}
