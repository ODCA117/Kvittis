use std::{collections::HashMap, path::Path};

use anyhow::{Result, anyhow};
use common::{ExpenseId, FriendRequestId, GroupId, GroupRole, UserId};
use sqlx::{
    FromRow, Row, SqlitePool,
    sqlite::{SqlitePoolOptions, SqliteRow},
};
use tracing::{debug, info, warn};

use crate::db::{ExpenseRow, FriendRequestRow, GroupMember, GroupRow, Store, UserRow};

pub struct SqliteStore {
    pool: SqlitePool,
}

impl FromRow<'_, SqliteRow> for UserRow {
    fn from_row(row: &SqliteRow) -> Result<Self, sqlx::Error> {
        let id: UserId = row.get("id");
        let username: String = row.get("username");
        let email: String = row.get("email");
        let password_hash: String = row.get("password_hash");
        let created_at: i64 = row.get("created_at");
        let updated_at: i64 = row.get("updated_at");
        let deleted_at: Option<i64> = row.try_get("deleted_at").ok();

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
        // FIXME: search for duplicate username as well!
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
            .bind(user.created_at)
            .bind(user.updated_at)
            .bind(user.deleted_at)
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
            .bind(user.created_at)
            .bind(user.updated_at)
            .bind(user.deleted_at)
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

    async fn get_user_by_id(&self, id: UserId) -> Result<Option<UserRow>> {
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

    async fn get_user_by_name(&self, name: String) -> Result<Option<UserRow>> {
        let user: Option<UserRow> = sqlx::query_as(
            r#"
                SELECT id, username, email, password_hash, created_at, updated_at, deleted_at
                FROM users
                WHERE username = $1 AND deleted_at IS NULL
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        info!(" USER: {:?}", &user);
        Ok(user)
    }

    /* Should not delete. Do soft delete instead */
    async fn delete_user(&self, id: UserId) -> Result<()> {
        let mut user = self
            .get_user_by_id(id)
            .await?
            .ok_or_else(|| anyhow!("User not found"))?;
        user.deleted_at = Some(chrono::Utc::now().timestamp_millis());

        print_sql_result(
            sqlx::query(
                r#"
                    UPDATE users
                    SET deleted_at = $1
                    WHERE id = $2
            "#,
            )
            .bind(user.deleted_at)
            .bind(user.id)
            .execute(&self.pool)
            .await,
        )?;

        info!("User deleted: {:?}", id);
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
    async fn _update_user(&self, mut user: UserRow) -> Result<UserRow> {
        user.updated_at = chrono::Utc::now().timestamp_millis();
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
            .bind(user.updated_at)
            .bind(user.id)
            .execute(&self.pool)
            .await,
        )?;

        Ok(user)
    }

    async fn get_user_friends(&self, id: UserId) -> Result<Vec<UserId>> {
        debug!("user: {:?}", id);
        let friend_rows = print_sql_result(
            sqlx::query(
                r#"
                    SELECT
                        CASE
                            WHEN user1_id = $1 THEN user2_id
                            ELSE user1_id
                        END AS friend_id
                    FROM friendships
                    WHERE user1_id = $1 OR user2_id = $1;
                "#,
            )
            .bind(id)
            .fetch_all(&self.pool)
            .await,
        )?;

        let friends = friend_rows.iter().map(|f| f.get("friend_id")).collect();
        Ok(friends)
    }

    async fn create_friend_request(&self, request: FriendRequestRow) -> Result<FriendRequestRow> {
        // dbg!(&request);
        match print_sql_result(sqlx::query(
            r#"
                INSERT INTO friend_requests (id, sender_id, receiver_id, status, created_at, updated_at)
                VALUES ($1, $2, $3, $4, $5, $6)
            "#)
            .bind(request.id)
            .bind(request.sender_id)
            .bind(request.receiver_id)
            .bind(&request.status)
            .bind(request.created_at)
            .bind(request.updated_at)
            .execute(&self.pool)
            .await,
        ) {
            Ok(_) => Ok(request),
            Err(_e) => Err(anyhow!("Failed to insert into friend_requests table")),
        }
    }

    async fn get_friend_request(&self, id: FriendRequestId) -> Result<FriendRequestRow> {
        let request: Result<FriendRequestRow> = print_sql_result(
            sqlx::query_as(
                r#"
                    SELECT
                        id,
                        sender_id,
                        receiver_id,
                        status,
                        created_at,
                        updated_at
                    FROM friend_requests
                    WHERE id = $1
            "#,
            )
            .bind(id)
            .fetch_one(&self.pool)
            .await,
        );

        request
    }

    async fn get_outgoing_requests(&self, user: UserId) -> Result<Vec<FriendRequestRow>> {
        let requests: Vec<FriendRequestRow> = print_sql_result(
            sqlx::query_as(
                r#"
                    SELECT
                        id,
                        sender_id,
                        receiver_id,
                        status,
                        created_at,
                        updated_at
                    FROM friend_requests
                    WHERE sender_id = $1 AND status = 'Pending'
                "#,
            )
            .bind(user)
            .fetch_all(&self.pool)
            .await,
        )?;

        if !requests.is_empty() {
            warn!("requests status: {:?}", &requests[0].status);
        }
        Ok(requests)
    }

    async fn get_incoming_requests(&self, user: UserId) -> Result<Vec<FriendRequestRow>> {
        let requests: Vec<FriendRequestRow> = print_sql_result(
            sqlx::query_as(
                r#"
                    SELECT
                        id,
                        sender_id,
                        receiver_id,
                        status,
                        created_at,
                        updated_at
                    FROM friend_requests
                    WHERE receiver_id = $1 AND status = 'Pending'
                "#,
            )
            .bind(user)
            .fetch_all(&self.pool)
            .await,
        )?;

        if !requests.is_empty() {
            warn!("requests status: {:?}", &requests[0].status);
        }
        Ok(requests)
    }

    async fn update_friend_request(&self, request: FriendRequestRow) -> Result<()> {
        print_sql_result(
            sqlx::query(
                r#"
                    UPDATE friend_requests
                    SET status = $1, updated_at = $2
                    WHERE id = $3;
                "#,
            )
            .bind(request.status)
            .bind(request.updated_at)
            .bind(request.id)
            .execute(&self.pool)
            .await,
        )?;

        Ok(())
    }

    async fn delete_friend_requests_from_user(&self, user_id: UserId) -> Result<()> {
        print_sql_result(
            sqlx::query(
                r#"
                    DELETE FROM friend_requests
                    WHERE sender_id = $1 OR receiver_id = $1;
                "#,
            )
            .bind(user_id)
            .execute(&self.pool)
            .await,
        )?;

        Ok(())
    }

    async fn add_friendship(&self, user1: UserId, user2: UserId) -> Result<()> {
        debug!("Add friendship: user1: {:?}, user2 {:?}", user1, user2);
        print_sql_result(
            sqlx::query(
                r#"
                    INSERT INTO friendships (user1_id, user2_id)
                    VALUES ($1, $2);
                "#,
            )
            .bind(user1)
            .bind(user2)
            .execute(&self.pool)
            .await,
        )?;

        Ok(())
    }

    async fn remove_friendship(&self, user: UserId) -> Result<()> {
        print_sql_result(
            sqlx::query(
                r#"
                    DELETE FROM friendships
                    WHERE user1_id = $1 OR user2_id = $1;
                "#,
            )
            .bind(user)
            .execute(&self.pool)
            .await,
        )?;

        Ok(())
    }

    // --- Groups ---
    async fn create_group(&self, group: GroupRow) -> Result<GroupRow> {
        debug!("group: {:?}", group);
        print_sql_result(
            sqlx::query(
                r#"
                    INSERT INTO groups (id, name)
                    VALUES ($1, $2)
                "#,
            )
            .bind(group.id)
            .bind(&group.name)
            .execute(&self.pool)
            .await,
        )?;

        Ok(group)
    }

    async fn get_group(&self, id: GroupId) -> Result<GroupRow> {
        let group: GroupRow = print_sql_result(
            sqlx::query_as(
                r#"
                    SELECT id, name
                    FROM groups
                    WHERE id = $1
                "#,
            )
            .bind(id)
            .fetch_one(&self.pool)
            .await,
        )?;

        Ok(group)
    }

    async fn get_groups(&self) -> Result<Vec<GroupRow>> {
        // Get groups where a specific user is a member
        print_sql_result(
            sqlx::query_as(
                r#"
                    SELECT * FROM groups
                "#,
            )
            .fetch_all(&self.pool)
            .await,
        )
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

    async fn add_group_member(&self, user: UserId, group: GroupId, role: GroupRole) -> Result<()> {
        print_sql_result(
            sqlx::query(
                r#"
                    INSERT INTO group_members ( group_id, user_id, role )
                    VALUES ($1, $2, $3)
                "#,
            )
            .bind(group)
            .bind(user)
            .bind(role.to_string())
            .execute(&self.pool)
            .await,
        )?;

        Ok(())
    }

    async fn get_group_members(&self, group: GroupId) -> Result<Vec<GroupMember>> {
        let users = print_sql_result(
            sqlx::query(
                r#"
                    SELECT group_id, user_id, role
                    FROM group_members
                    WHERE group_id = $1
                "#,
            )
            .bind(group)
            .fetch_all(&self.pool)
            .await,
        )?;

        let members = users
            .iter()
            .map(|r| {
                let group_id = r.get("group_id");
                let user_id = r.get("user_id");
                let role = r.get("role");
                GroupMember {
                    group_id,
                    user_id,
                    role,
                }
            })
            .collect();

        Ok(members)
    }

    // --- Expenses ---
    async fn create_expense(&self, expense: ExpenseRow) -> Result<ExpenseRow> {
        // Insert the expense in the expenses table.
        print_sql_result(
            sqlx::query(
                r#"
                    INSERT INTO expenses (id, payer_id, amount, description, group_id, created_at)
                    VALUES ($1, $2, $3, $4, $5, $6)
                "#,
            )
            .bind(expense.id)
            .bind(expense.payer)
            .bind(expense.amount)
            .bind(&expense.description)
            .bind(expense.group_id)
            .bind(expense.created_at)
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
                        e.created_at        AS created_at,
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
            let created_at: i64 = r.get("created_at");
            let user_id: UserId = r.get("user_id");
            let _username: String = r.get("username");
            let e = map.entry(id).or_insert(ExpenseRow {
                id,
                payer,
                participants: vec![],
                amount,
                description,
                group_id,
                created_at,
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
                        e.created_at        AS created_at,
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
                        e.created_at      AS created_at,
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
        let created_at: i64 = r.get("created_at");
        let e = map.entry(id).or_insert(ExpenseRow {
            id,
            payer,
            participants: vec![],
            amount,
            description,
            group_id,
            created_at,
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

fn _build_user_from_row(row: SqliteRow) -> Result<UserRow> {
    let id: UserId = row.get("id");
    let username: String = row.get("username");
    let email: String = row.get("email");
    let password_hash: String = row.get("password_hash");
    let created_at: i64 = row.get("created_at");
    let updated_at: i64 = row.get("updated_at");
    let deleted_at: Option<i64> = row.try_get("deleted_at").ok();

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
