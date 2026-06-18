use crate::{Expense, ExpenseId, GroupId, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Response types

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateExpenseResponse {
    pub id: ExpenseId,
    pub payer: UserId,
    pub participants: Vec<UserId>,
    pub amount: i64,
    pub description: Option<String>,
    pub group_id: Option<GroupId>,
    pub created_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetExpenseResponse {
    pub id: ExpenseId,
    pub payer: UserId,
    pub participants: Vec<UserId>,
    pub amount: i64,
    pub description: Option<String>,
    pub group_id: Option<GroupId>,
    pub created_at: DateTime<Utc>,
}

// Internal structs used by the state/db layer

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateExpenseRequest {
    pub payer: UserId,
    pub participants: Vec<UserId>,
    pub amount: i64,
    pub description: Option<String>,
    pub group_id: Option<GroupId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetExpenseRequest {
    pub id: ExpenseId,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteExpenseRequest {
    pub id: ExpenseId,
}

// Request enums

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum ExpenseRequest {
    Create {
        payer: UserId,
        participants: Vec<UserId>,
        amount: i64,
        description: Option<String>,
        group_id: Option<GroupId>,
    },
    Get {
        id: ExpenseId,
    },
    Delete {
        id: ExpenseId,
    },
    ListForUser {
        user_id: UserId,
    },
    ListForGroup {
        group_id: GroupId,
    },
}

// From trait implementations

impl From<Expense> for CreateExpenseResponse {
    fn from(value: Expense) -> Self {
        CreateExpenseResponse {
            id: value.id,
            payer: value.payer,
            participants: value.participants,
            amount: value.amount,
            description: value.description,
            group_id: value.group_id,
            created_at: value.created_at,
        }
    }
}

impl From<CreateExpenseResponse> for Expense {
    fn from(value: CreateExpenseResponse) -> Self {
        Expense {
            id: value.id,
            payer: value.payer,
            participants: value.participants,
            amount: value.amount,
            description: value.description,
            group_id: value.group_id,
            created_at: value.created_at,
        }
    }
}

impl From<Expense> for GetExpenseResponse {
    fn from(value: Expense) -> Self {
        GetExpenseResponse {
            id: value.id,
            payer: value.payer,
            participants: value.participants,
            amount: value.amount,
            description: value.description,
            group_id: value.group_id,
            created_at: value.created_at,
        }
    }
}
