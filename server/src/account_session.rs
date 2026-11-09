use crate::db::AccountSessionRow;
use chrono::{DateTime, Utc};
use common::UserId;
use uuid::Uuid;

pub type AccountSessionId = Uuid;

#[derive(Debug, Clone)]
pub struct AccountSession {
    id: AccountSessionId,
    user_id: UserId,
    created_at: DateTime<Utc>,
    expires_at: DateTime<Utc>,
    revoked_at: Option<DateTime<Utc>>,
}

impl AccountSession {
    pub fn new(user_id: UserId, created_at: DateTime<Utc>, expires_at: DateTime<Utc>) -> Self {
        Self {
            id: Uuid::new_v4(),
            user_id,
            created_at,
            expires_at,
            revoked_at: None,
        }
    }

    pub fn get_id(&self) -> AccountSessionId {
        self.id
    }

    pub fn is_revoked(&self) -> bool {
        self.revoked_at.is_some()
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at <= Utc::now()
    }

    pub fn is_valid(&self, user_id: Uuid) -> bool {
        if self.is_revoked() || self.is_expired() || self.user_id != user_id {
            false
        } else {
            true
        }
    }
}

impl From<AccountSession> for AccountSessionRow {
    fn from(value: AccountSession) -> Self {
        AccountSessionRow::new(
            value.id,
            value.user_id,
            value.created_at.timestamp_millis(),
            value.expires_at.timestamp_millis(),
            value.revoked_at.map(|dt| dt.timestamp_millis()),
        )
    }
}

impl From<AccountSessionRow> for AccountSession {
    fn from(value: AccountSessionRow) -> Self {
        AccountSession {
            id: value.id,
            user_id: value.user_id,
            created_at: DateTime::from_timestamp_millis(value.created_at)
                .unwrap_or(DateTime::<Utc>::MAX_UTC),
            expires_at: DateTime::from_timestamp_millis(value.expires_at)
                .unwrap_or(DateTime::<Utc>::MAX_UTC),
            revoked_at: value
                .revoked_at
                .map(|ms| DateTime::from_timestamp_millis(ms).unwrap_or(DateTime::<Utc>::MAX_UTC)),
        }
    }
}
