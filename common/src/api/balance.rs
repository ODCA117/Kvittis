use crate::{GroupId, UserId};
use serde::{Deserialize, Serialize};

// Request enums

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum BalanceRequest {
    User { user_id: UserId },
    Group { group_id: GroupId },
}

// Shared types

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
