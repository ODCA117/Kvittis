use crate::{Group, GroupId, GroupRole, UserId};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Response types

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGroupResponse {
    pub id: GroupId,
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetGroupResponse {
    pub id: GroupId,
    pub name: String,
    pub members: Vec<(UserId, GroupRole)>,
    pub last_settled: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

// Internal structs used by the state/db layer

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateGroupRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct NewGroupMemberRequest {
    pub group_id: GroupId,
    pub requester: UserId,
    pub new_member: UserId,
    pub role: GroupRole,
}

// Request enums

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum GroupRequest {
    Create {
        name: String,
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
        role: GroupRole,
    },
    UpdateMember {
        group_id: GroupId,
        member: UserId,
        role: GroupRole,
    },
    RemoveMember {
        group_id: GroupId,
        member: UserId,
    },
}

// From trait implementations

impl From<GetGroupResponse> for Group {
    fn from(value: GetGroupResponse) -> Self {
        Group {
            id: value.id,
            name: value.name,
            members: value.members,
            last_settled: value.last_settled,
            created_at: value.created_at,
            updated_at: value.updated_at,
            deleted_at: value.deleted_at,
        }
    }
}
