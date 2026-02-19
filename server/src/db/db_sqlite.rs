use std::{collections::HashMap, path::Path};

use anyhow::{Result, anyhow};
use common::{ExpenseId, Group, GroupId, UserId};
use sqlx::{Row, SqlitePool, sqlite::SqlitePoolOptions};
use tracing::{debug, info, warn};

use crate::db::{ExpenseRow, GroupRow, Store, UserRow};

pub struct SqliteStore {
    pool: SqlitePool,
}

impl SqliteStore {
    pub async fn connect(path: impl AsRef<Path> + std::fmt::Display) -> Result<Self> {
        let url = format!("sqlite:{}?mode=rwc", path);
        debug!("url: {:?}", url);
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect(&url)
            .await?;
        sqlx::migrate!("./migrations/").run(&pool).await?;
        Ok(Self { pool })
    }
}

#[async_trait::async_trait]
impl Store for SqliteStore {
    async fn create_user(&self, user: UserRow) -> Result<UserRow> {
        let res = sqlx::query(
            r#"
            INSERT INTO users (id, username)
            VALUES ($1, $2)
            "#,
        )
        .bind(user.id)
        .bind(&user.username)
        .execute(&self.pool)
        .await;

        match res {
            Ok(sql_res) => {
                info!("User added: {:?}", sql_res);
                Ok(user)
            }
            Err(e) => {
                warn!("Error: {:?}", e.to_string());
                Err(anyhow!("Failed to add User to DB"))
            }
        }
    }

    async fn get_user(&self, id: UserId) -> Result<Option<UserRow>> {
        let row = sqlx::query(
            r#"
            SELECT id, username
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = row else {
            warn!("NO user found");
            return Ok(None);
        };
        info!("User found");

        let friends: Vec<UserId> = sqlx::query_scalar(
            r#"
            SELECT
                CASE
                    WHEN user1_id = $1 THEN user2_id
                    ELSE user1_id
                END AS friend_id
            FROM
                friendships
            WHERE
                user1_id = $1 OR user2_id = $1
            "#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;
        let user = UserRow {
            id: row.try_get("id")?,
            username: row.try_get("username")?,
            friends,
        };

        info!(" USER: {:?}", &user);
        Ok(Some(user))
    }

    async fn delete_user(&self, id: UserId) -> Result<()> {
        print_sql_result(
            sqlx::query(
                r#"
            DELETE FROM users
            WHERE id = $1
            "#,
            )
            .bind(id)
            .execute(&self.pool)
            .await,
        )?;

        info!("User deleted: {:?}", id);
        Ok(())
    }

    async fn add_friend(&self, user1: UserId, user2: UserId) -> Result<()> {
        let (id1, id2) = if user1 < user2 {
            (user1, user2)
        } else {
            (user2, user1)
        };

        let exists: bool = print_sql_result(
            sqlx::query_scalar(
                r#"
            SELECT EXISTS (
                SELECT 1
                FROM friendships
                WHERE user1_id = $1 AND user2_id = $2
            )"#,
            )
            .bind(id1)
            .bind(id2)
            .fetch_one(&self.pool)
            .await,
        )?;

        if exists {
            warn!("Friendship exists");
            return Err(anyhow!("Friendship already exists"));
        }

        print_sql_result(
            sqlx::query(
                r#"
            INSERT INTO friendships (user1_id, user2_id)
            VALUES ($1, $2)
            "#,
            )
            .bind(id1)
            .bind(id2)
            .execute(&self.pool)
            .await,
        )?;

        info!("Add friendship between {:?} and {:?}", user1, user2);

        Ok(())
    }

    async fn list_users(&self) -> Result<Vec<UserRow>> {
        /* Returns a vec with users that contans user_id, user_name, friend_id, and friend_username */
        let friendships = print_sql_result(
            sqlx::query(
                r#"
            SELECT
                u.id          AS user_id,
                u.username    AS user_username,
                f.id          AS friend_id,
                f.username    AS friend_username
            FROM users u
            LEFT JOIN friendships fs
                ON u.id = fs.user1_id OR u.id = fs.user2_id
            LEFT JOIN users f
                ON f.id = CASE
                    WHEN fs.user1_id = u.id THEN fs.user2_id
                    ELSE fs.user1_id
                END
            ORDER BY u.id;
            "#,
            )
            .fetch_all(&self.pool)
            .await,
        )?;

        /* Merge all the friends to one user */
        let mut map = HashMap::new();
        for r in friendships.iter() {
            let user_id: UserId = r.get("user_id");
            let username: String = r.get("user_username");
            let friend_id: Option<UserId> = r.try_get("friend_id").ok();
            let user = map.entry(user_id).or_insert(UserRow {
                id: user_id,
                username,
                friends: vec![],
            });
            if let Some(friend_id) = friend_id {
                user.friends.push(friend_id);
            }

            // let friend_username: Option<String> = r.try_get("friend_username").ok();
            // println!("user: {}, {}, friend: {:?}, {:?}", user_id, username, friend_id, friend_username);
        }

        let users: Vec<UserRow> = map.values().cloned().collect();
        // dbg!(&users);
        Ok(users)
    }

    async fn update_user(&self, user: UserRow) -> Result<UserRow> {
        print_sql_result(
            sqlx::query(
                r#"
                    UPDATE users
                    SET username = $1
                    WHERE id = $2
                "#,
            )
            .bind(&user.username)
            .bind(user.id)
            .execute(&self.pool)
            .await
        )?;

        Ok(user)
    }

    // --- Groups ---
    async fn create_group(&self, group: GroupRow) -> Result<GroupRow> {
        print_sql_result(
            sqlx::query(
                r#"
                    INSERT INTO groups (id, name, owner_id)
                    VALUES ($1, $2, $3)
                "#,
            )
            .bind(group.id)
            .bind(&group.name)
            .bind(group.owner_id)
            .execute(&self.pool)
            .await
        )?;

        print_sql_result(
            sqlx::query(
                r#"
                    INSERT INTO group_members (group_id, user_id)
                    VALUES ($1, $2)
                "#,

            )
            .bind(group.id)
            .bind(group.owner_id)
            .execute(&self.pool)
            .await
        )?;

        Ok(group)
    }

    async fn get_group(&self, id: GroupId) -> Result<Option<GroupRow>> {
        warn!("Get Group!!!!!");
        let group = print_sql_result(
            sqlx::query(
                r#"
                    SELECT
                        g.id                AS group_id,
                        g.name              AS group_name,
                        g.owner_id          AS group_owner,
                        u.id                AS user_id,
                        u.username          AS username
                    FROM groups g
                    LEFT JOIN group_members gm
                        ON g.id = gm.group_id
                    LEFT JOIN users u
                        ON gm.user_id = u.id
                    WHERE g.id = $1
                    ORDER BY g.id;
                "#,
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await,
        )?;

        let mut map = HashMap::new();
        for r in group.iter() {
            let id: GroupId = r.get("group_id");
            let name: String = r.get("group_name");
            let owner_id: UserId = r.get("group_owner");
            let user_id: UserId = r.get("user_id");
            let username: String = r.get("username");
            let g = map.entry(id).or_insert( GroupRow { id, name, owner_id, members: vec![] });
            g.members.push(user_id);

            // let user = map.entry(user_id).or_insert(UserRow {
            //     id: user_id,
            //     username,
            //     friends: vec![],
            // });
            // if let Some(friend_id) = friend_id {
            //     user.friends.push(friend_id);
            // }

            // let friend_username: Option<String> = r.try_get("friend_username").ok();
            // println!("group: {}, {}, {}, user: {:?}, {:?}", group_id, group_name, group_owner, user_id, username);
        }

        Ok(Some(map.values().cloned().collect::<Vec<GroupRow>>()[0].clone()))
    }

    // --- Expenses ---
    async fn create_expense(&self, _expense: ExpenseRow) -> Result<ExpenseRow> {
        todo!();
    }

    async fn get_expense(&self, _id: ExpenseId) -> Result<Option<ExpenseRow>> {
        todo!();
    }
}

fn print_sql_result<T>(res: Result<T, sqlx::Error>) -> Result<T> {
    match res {
        Ok(res) => {
            debug!("SQL query successful:");
            Ok(res)
        }
        Err(e) => {
            warn!("SQL query failed: {:?}", e.to_string());
            Err(anyhow!("Failed to SQL query"))
        }
    }
}
