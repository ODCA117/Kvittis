// Group-related API handlers

use axum::{Json, extract::State, http::StatusCode};
use common::api::{ApiResponse, CreateGroupResponse, GetGroupResponse, GroupRequest};
use tracing::debug;

use super::auth::{json_error, json_success};
use crate::api::Claims;
use crate::state::AppState;

// ── Group handler ─────────────────────────────────────────────────────────────

// NOTE: Need to be authorized user
pub async fn group_handler(
    State(state): State<AppState>,
    claims: Claims,
    Json(payload): Json<GroupRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    match payload {
        GroupRequest::Create { name } => {
            debug!("Create group: name={:?}", name);
            let req = common::api::CreateGroupRequest { name };
            match state.create_group(claims.sub, req).await {
                Ok(g) => {
                    let resp = CreateGroupResponse {
                        id: g.id,
                        name: g.name,
                    };
                    json_success(StatusCode::CREATED, serde_json::to_value(resp).unwrap())
                }
                Err(e) => json_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create group: {}", e).as_str(),
                ),
            }
        }

        GroupRequest::Get { group_id } => {
            // FIXME: Only get groups user is member of
            debug!("Get group: {:?}", group_id);
            match state.get_group(group_id).await {
                Ok(Some(g)) => {
                    let resp = GetGroupResponse {
                        id: g.id,
                        name: g.name,
                        last_settled: g.last_settled,
                        members: g.members,
                    };
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Ok(None) => json_error(StatusCode::NOT_FOUND, "Group not found"),
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to get group"),
            }
        }

        GroupRequest::Delete { group_id } => {
            // FIXME: Only delete groups user is member of
            debug!("Delete group: {:?}", group_id);
            match state.delete_group(claims.sub, group_id).await {
                Ok(_) => json_success(
                    StatusCode::OK,
                    serde_json::json!({"status": "Group deleted"}),
                ),
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to delete group"),
            }
        }

        GroupRequest::Search { query } => {
            // FIXME: Remove
            debug!("Search groups: {:?}", query);
            match state.search_groups(&query).await {
                Ok(groups) => {
                    let resp: Vec<GetGroupResponse> = groups
                        .into_iter()
                        .map(|g| GetGroupResponse {
                            id: g.id,
                            name: g.name,
                            last_settled: g.last_settled,
                            members: g.members,
                        })
                        .collect();
                    json_success(StatusCode::OK, serde_json::to_value(resp).unwrap())
                }
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to search groups"),
            }
        }

        GroupRequest::AddMember {
            group_id,
            new_member,
            role,
        } => {
            // NOTE: Only add if admin of group
            debug!(
                "Add group member: group={:?} member={:?}",
                group_id, new_member
            );
            let req = common::api::NewGroupMemberRequest {
                group_id,
                requester: claims.sub,
                new_member,
                role,
            };
            match state.new_group_member(req).await {
                Ok(_) => json_success(
                    StatusCode::OK,
                    serde_json::json!({"status": "member added"}),
                ),
                Err(_) => json_error(StatusCode::INTERNAL_SERVER_ERROR, "Failed to add member"),
            }
        }

        GroupRequest::UpdateMember {
            group_id: _,
            member: _,
            role: _,
        } => json_error(StatusCode::NOT_IMPLEMENTED, "Not implemented"),

        GroupRequest::RemoveMember {
            group_id: _,
            member: _,
        } => json_error(StatusCode::NOT_IMPLEMENTED, "Not implemented"),
    }
}
