use std::{collections::HashMap, path::Path};

use anyhow::{Result, anyhow};
use chrono::DateTime;
use common::{ExpenseId, GroupId, UserId};
use sqlx::{
    FromRow, Row, SqlitePool,
    sqlite::{SqlitePoolOptions, SqliteRow},
};
use tracing::{debug, info, warn};

use crate::db::{ExpenseRow, GroupRow, Store, UserRow};

pub struct SqliteStore {
    pool: SqlitePool,
}

// FIXME: User query_as instead?
impl FromRow<'_, SqliteRow> for UserRow {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let id: UserId = row.get("id");
        let username: String = row.get("username");
        let email: String = row.get("email");
        let password_hash: String = row.get("password_hash");
        let created_at: DateTime<chrono::FixedOffset> = DateTime::parse_from_rfc3339(
            row.get::<String, _>("created_at").as_str(),
        )
        .map_err(|e| sqlx::Error::ColumnDecode {
            index: "created_at".into(),
            source: Box::new(e),
        })?;
        let updated_at: DateTime<chrono::FixedOffset> = DateTime::parse_from_rfc3339(
            row.get::<String, _>("updated_at").as_str(),
        )
        .map_err(|e| sqlx::Error::ColumnDecode {
            index: "updated_at".into(),
            source: Box::new(e),
        })?;
        let deleted_at: Option<DateTime<chrono::FixedOffset>> = row
            .try_get::<String, _>("deleted_at")
            .ok()
            .and_then(|s| DateTime::parse_from_rfc3339(s.as_str()).ok());

        Ok(UserRow {
            id,
            username,
            email,
            password_hash,
            created_at,
            updated_at,
            deleted_at,
        })
    }
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
        // find if old email already exists but is deleted.
        // Allow for complete reinstatiation
        let old_user: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at, deleted_at
            FROM users
            WHERE email = $1 AND deleted_at IS NOT NULL
            "#,
        )
        .bind(user.email.clone())
        .fetch_optional(&self.pool)
        .await?;

        // FIXME: Can add an duplicate email here...
        let res = if let Some(_u) = old_user {
            sqlx::query(
                r#"
                    UPDATE users
                    SET
                        id = $1,
                        username = $2,
                        password_hash = $3,
                        created_at = $4,
                        updated_at = $5,
                        deleted_at = $6
                    WHERE email = $7;
                "#,
            )
            .bind(user.id)
            .bind(&user.username)
            .bind(&user.password_hash)
            .bind(user.created_at.to_rfc3339())
            .bind(user.updated_at.to_rfc3339())
            .bind(user.deleted_at.map(|dt| dt.to_rfc3339()))
            .bind(&user.email)
            .execute(&self.pool)
            .await
        } else {
            sqlx::query(
                r#"
                INSERT INTO users (id, username, email, password_hash, created_at, updated_at, deleted_at)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(user.id)
            .bind(&user.username)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(user.created_at.to_rfc3339())
            .bind(user.updated_at.to_rfc3339())
            .bind(user.deleted_at.map(|dt| dt.to_rfc3339()))
            .execute(&self.pool)
            .await
        };

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
        let user: Option<UserRow> = sqlx::query_as(
            r#"
            SELECT id, username, email, password_hash, created_at, updated_at, deleted_at
            FROM users
            WHERE id = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        info!(" USER: {:?}", &user);
        Ok(user)
    }

    /* Should not delete. Do soft delete instead */
    async fn delete_user(&self, id: UserId) -> Result<()> {
        let mut user = self
            .get_user(id)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;
        user.deleted_at = // TODO: Move to state
            Some(chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()));

        print_sql_result(
            sqlx::query(
                r#"
            UPDATE users
            SET deleted_at = $1
            WHERE id = $2
            "#,
            )
            .bind(user.deleted_at.unwrap().to_rfc3339())
            .bind(user.id)
            .execute(&self.pool)
            .await,
        )?;

        info!("User deleted: {:?}", id);
        Ok(())
    }

    // FIXME: Fix in future. Friendship should not be instant.
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
        /* Returns a vec with users */
        let users: Vec<UserRow> = print_sql_result(
            sqlx::query_as(
                r#"
            SELECT
                id,
                username,
                email,
                password_hash,
                created_at,
                updated_at,
                deleted_at
            FROM users
            WHERE deleted_at is NULL
            "#,
            )
            .fetch_all(&self.pool)
            .await,
        )?;

        Ok(users)
    }

    // Should be able to update username, email, password_hash, updated_at is automatic.
    // created_at does not update ever, deleted_at should not update unless deleted.
    async fn update_user(&self, mut user: UserRow) -> Result<UserRow> {
        // TODO: Check the timezone stuff...
        user.updated_at =
            chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
        print_sql_result(
            sqlx::query(
                r#"
                    UPDATE users
                    SET username = $1
                    SET email = $2
                    SET password_hash = $3
                    SET updated_at = $4
                    WHERE id = $5
                "#,
            )
            .bind(&user.username)
            .bind(&user.email)
            .bind(&user.password_hash)
            .bind(user.updated_at.to_rfc3339())
            .bind(user.id)
            .execute(&self.pool)
            .await,
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
            .await,
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
            .await,
        )?;

        Ok(group)
    }

    async fn get_group(&self, id: GroupId) -> Result<Option<GroupRow>> {
        info!("Get Group!!!!!");
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
            let _username: String = r.get("username");
            let g = map.entry(id).or_insert(GroupRow {
                id,
                name,
                owner_id,
                members: vec![],
            });
            g.members.push(user_id);
        }

        Ok(map.into_values().next())
    }

    async fn get_groups(&self) -> Result<Vec<GroupRow>> {
        let groups = print_sql_result(
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
                    ORDER BY g.id;
                "#,
            )
            .fetch_all(&self.pool)
            .await,
        )?;

        let mut map = HashMap::new();
        for r in groups.iter() {
            let id: GroupId = r.get("group_id");
            let name: String = r.get("group_name");
            let owner_id: UserId = r.get("group_owner");
            let user_id: UserId = r.get("user_id");
            let _username: String = r.get("username");
            let g = map.entry(id).or_insert(GroupRow {
                id,
                name,
                owner_id,
                members: vec![],
            });
            g.members.push(user_id);
        }

        Ok(map.values().cloned().collect::<Vec<GroupRow>>())
    }

    async fn delete_group(&self, id: GroupId) -> Result<()> {
        // Remove the members first due to foreign key constraint
        print_sql_result(
            sqlx::query(
                r#"
                    DELETE FROM group_members WHERE group_id = $1
                "#,
            )
            .bind(id)
            .execute(&self.pool)
            .await,
        )?;

        // Remove the group
        print_sql_result(
            sqlx::query(
                r#"
            DELETE FROM groups
            WHERE id = $1
            "#,
            )
            .bind(id)
            .execute(&self.pool)
            .await,
        )?;

        info!("group deleted: {:?}", id);
        Ok(())
    }

    async fn update_group(&self, group: GroupRow) -> Result<GroupRow> {
        print_sql_result(
            sqlx::query(
                r#"
                    UPDATE groups
                    SET name = $1, owner_id = $2
                    WHERE id = $3
                "#,
            )
            .bind(&group.name)
            .bind(group.owner_id)
            .bind(group.id)
            .execute(&self.pool)
            .await,
        )?;

        // Update members: remove all and add again (simpler than calculating the diff)
        print_sql_result(
            sqlx::query(
                r#"
                    DELETE FROM group_members WHERE group_id = $1
                "#,
            )
            .bind(group.id)
            .execute(&self.pool)
            .await,
        )?;

        for member in &group.members {
            print_sql_result(
                sqlx::query(
                    r#"
                        INSERT INTO group_members (group_id, user_id)
                        VALUES ($1, $2)
                    "#,
                )
                .bind(group.id)
                .bind(member)
                .execute(&self.pool)
                .await,
            )?;
        }

        Ok(group)
    }

    // --- Expenses ---
    async fn create_expense(&self, expense: ExpenseRow) -> Result<ExpenseRow> {
        // Insert the expense in the expenses table.
        print_sql_result(
            sqlx::query(
                r#"
                    INSERT INTO expenses (id, payer_id, amount, description, group_id, timestamp_ms)
                    VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(expense.id)
            .bind(expense.payer)
            .bind(expense.amount)
            .bind(&expense.description)
            .bind(expense.group_id)
            .bind(expense.timestamp_ms)
            .execute(&self.pool)
            .await,
        )?;

        // Insert the participants in the expense_participants table.
        for participant in &expense.participants {
            print_sql_result(
                sqlx::query(
                    r#"
                        INSERT INTO expense_participants (expense_id, user_id)
                        VALUES ($1, $2)
                    "#,
                )
                .bind(expense.id)
                .bind(participant)
                .execute(&self.pool)
                .await,
            )?;
        }

        Ok(expense)
    }

    async fn delete_expense(&self, id: ExpenseId) -> Result<()> {
        // Remove the participants first due to foreign key constraint

        print_sql_result(
            sqlx::query(
                r#"
                    DELETE FROM expense_participants WHERE expense_id = $1
                "#,
            )
            .bind(id)
            .execute(&self.pool)
            .await,
        )?;

        // Remove the expense
        print_sql_result(
            sqlx::query(
                r#"
                    DELETE FROM expenses
                    WHERE id = $1
                "#,
            )
            .bind(id)
            .execute(&self.pool)
            .await,
        )?;

        Ok(())
    }

    async fn get_expense(&self, id: ExpenseId) -> Result<Option<ExpenseRow>> {
        // Get the expense and all the participants in one query using LEFT JOIN.
        let expense = print_sql_result(
            sqlx::query(
                r#"
                    SELECT
                        e.id                AS expense_id,
                        e.payer_id          AS payer,
                        e.amount            AS amount,
                        e.description       AS description,
                        e.group_id          AS group_id,
                        e.timestamp_ms      AS timestamp_ms,
                        u.id                AS user_id,
                        u.username          AS username
                    FROM expenses e
                    LEFT JOIN expense_participants ep
                        ON e.id = ep.expense_id
                    LEFT JOIN users u
                        ON ep.user_id = u.id
                    WHERE e.id = $1
                    ORDER BY e.id;
                "#,
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await,
        )?;

        let mut map = HashMap::new();
        for r in expense.iter() {
            let id: ExpenseId = r.get("expense_id");
            let payer: UserId = r.get("payer");
            let amount: i64 = r.get("amount");
            let description: Option<String> = r.try_get("description").ok();
            let group_id: Option<GroupId> = r.try_get("group_id").ok();
            let timestamp_ms: i64 = r.get("timestamp_ms");
            let user_id: UserId = r.get("user_id");
            let _username: String = r.get("username");
            let e = map.entry(id).or_insert(ExpenseRow {
                id,
                payer,
                participants: vec![],
                amount,
                description,
                group_id,
                timestamp_ms,
            });
            e.participants.push(user_id);
        }

        Ok(map.into_values().next())
    }

    async fn list_expenses_for_user(&self, user_id: UserId) -> Result<Vec<ExpenseRow>> {
        let rows = print_sql_result(
            sqlx::query(
                r#"
                    SELECT
                        e.id                AS expense_id,
                        e.payer_id          AS payer,
                        e.amount            AS amount,
                        e.description       AS description,
                        e.group_id          AS group_id,
                        e.timestamp_ms      AS timestamp_ms,
                        ep2.user_id         AS user_id
                    FROM expenses e
                    LEFT JOIN expense_participants ep2
                        ON e.id = ep2.expense_id
                    WHERE e.payer_id = $1
                       OR EXISTS (
                           SELECT 1 FROM expense_participants ep
                           WHERE ep.expense_id = e.id AND ep.user_id = $1
                       )
                    ORDER BY e.id
                "#,
            )
            .bind(user_id)
            .fetch_all(&self.pool)
            .await,
        )?;

        Ok(build_expense_rows(rows))
    }

    async fn list_expenses_for_group(&self, group_id: GroupId) -> Result<Vec<ExpenseRow>> {
        let rows = print_sql_result(
            sqlx::query(
                r#"
                    SELECT
                        e.id                AS expense_id,
                        e.payer_id          AS payer,
                        e.amount            AS amount,
                        e.description       AS description,
                        e.group_id          AS group_id,
                        e.timestamp_ms      AS timestamp_ms,
                        ep.user_id          AS user_id
                    FROM expenses e
                    LEFT JOIN expense_participants ep
                        ON e.id = ep.expense_id
                    WHERE e.group_id = $1
                    ORDER BY e.id
                "#,
            )
            .bind(group_id)
            .fetch_all(&self.pool)
            .await,
        )?;

        Ok(build_expense_rows(rows))
    }
}

fn build_expense_rows(rows: Vec<sqlx::sqlite::SqliteRow>) -> Vec<ExpenseRow> {
    let mut map: HashMap<ExpenseId, ExpenseRow> = HashMap::new();
    for r in rows.iter() {
        let id: ExpenseId = r.get("expense_id");
        let payer: UserId = r.get("payer");
        let amount: i64 = r.get("amount");
        let description: Option<String> = r.try_get("description").ok();
        let group_id: Option<GroupId> = r.try_get("group_id").ok();
        let timestamp_ms: i64 = r.get("timestamp_ms");
        let e = map.entry(id).or_insert(ExpenseRow {
            id,
            payer,
            participants: vec![],
            amount,
            description,
            group_id,
            timestamp_ms,
        });
        if let Ok(user_id) = r.try_get::<UserId, _>("user_id") {
            e.participants.push(user_id);
        }
    }
    map.into_values().collect()
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

fn build_user_from_row(row: SqliteRow) -> Result<UserRow> {
    let id: UserId = row.get("id");
    let username: String = row.get("username");
    let email: String = row.get("email");
    let password_hash: String = row.get("password_hash");
    let created_at: DateTime<chrono::FixedOffset> =
        DateTime::parse_from_rfc3339(row.get::<String, _>("created_at").as_str())?;
    let updated_at: DateTime<chrono::FixedOffset> =
        DateTime::parse_from_rfc3339(row.get::<String, _>("updated_at").as_str())?;
    let deleted_at: Option<DateTime<chrono::FixedOffset>> = row
        .try_get::<String, _>("deleted_at")
        .ok()
        .and_then(|s| DateTime::parse_from_rfc3339(s.as_str()).ok());

    Ok(UserRow {
        id,
        username,
        email,
        password_hash,
        created_at,
        updated_at,
        deleted_at,
    })
}
