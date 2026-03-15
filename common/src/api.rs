use crate::{Expense, ExpenseId, Group, GroupId, NewUser, User, UserId};
use serde::{Deserialize, Serialize};

// ── Response types ────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct RegisterResponse {
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LoginResponse {
    pub user: User,
    pub token: String,
    pub token_type: TokenType,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetUserResponse {
    pub user: User,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGroupResponse {
    pub id: GroupId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetGroupResponse {
    pub id: GroupId,
    pub name: String,
    pub owner_id: UserId,
    pub members: Vec<UserId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateExpenseResponse {
    pub id: ExpenseId,
    pub payer: UserId,
    pub participants: Vec<UserId>,
    pub amount: i64,
    pub description: Option<String>,
    pub group_id: Option<GroupId>,
    pub timestamp_ms: i64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetExpenseResponse {
    pub id: ExpenseId,
    pub payer: UserId,
    pub participants: Vec<UserId>,
    pub amount: i64,
    pub description: Option<String>,
    pub group_id: Option<GroupId>,
    pub timestamp_ms: i64,
}

// ── Internal structs used by the state/db layer ───────────────────────────────

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGroupRequest {
    pub name: String,
    pub owner_id: UserId,
    pub members: Vec<UserId>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewGroupMemberRequest {
    pub group_id: GroupId,
    pub new_member: UserId,
}

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

// ── Request enums (one per resource, action-based) ───────────────────────────

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum UnauthorizedUserRequest {
    Register { user: NewUser },
    Login { username: String, password: String },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum AuthorizedUserRequest {
    Get { user_id: UserId },
    Delete { user_id: UserId },
    List,
    Search { query: String },
    AddFriend { user_id: UserId, friend_id: UserId },
    Logout,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum GroupRequest {
    Create {
        name: String,
        owner_id: UserId,
        members: Vec<UserId>,
    },
    Get {
        group_id: GroupId,
    },
    Delete {
        group_id: GroupId,
    },
    Search {
        query: String,
    },
    AddMember {
        group_id: GroupId,
        new_member: UserId,
    },
}

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

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum BalanceRequest {
    User { user_id: UserId },
    Group { group_id: GroupId },
}

// ── Shared types ──────────────────────────────────────────────────────────────

/// Amount is in minor units (cents/öre). Positive means `other` owes the
/// requesting user; negative means the requesting user owes `other`.
#[derive(Serialize, Deserialize, Debug)]
pub struct BalanceEntry {
    pub other: UserId,
    pub amount: i64,
}

/// Amount is in minor units (cents/öre). `from` owes `to` this amount.
#[derive(Serialize, Deserialize, Debug)]
pub struct GroupBalance {
    pub from: UserId,
    pub to: UserId,
    pub amount: i64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum ApiResponse<T: Serialize> {
    Success(T),
    Error { message: String },
}

impl From<User> for GetUserResponse {
    fn from(value: User) -> Self {
        GetUserResponse { user: value }
    }
}

impl From<GetUserResponse> for User {
    fn from(value: GetUserResponse) -> Self {
        value.user
    }
}

impl From<User> for RegisterResponse {
    fn from(value: User) -> Self {
        RegisterResponse { user: value }
    }
}

impl From<RegisterResponse> for User {
    fn from(value: RegisterResponse) -> Self {
        value.user
    }
}

impl From<GetGroupResponse> for Group {
    fn from(value: GetGroupResponse) -> Self {
        Group {
            id: value.id,
            name: value.name,
            owner_id: value.owner_id,
            members: value.members,
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
            timestamp_ms: value.timestamp_ms,
        }
    }
}

impl From<Expense> for CreateExpenseResponse {
    fn from(value: Expense) -> Self {
        CreateExpenseResponse {
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

impl From<Expense> for GetExpenseResponse {
    fn from(value: Expense) -> Self {
        GetExpenseResponse {
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

#[derive(Debug, Serialize, Deserialize)]
pub enum TokenType {
    Bearer,
}
