use std::time::Duration;

use common::{
    api::{ApiResponse, GroupRequest},
    GroupId, GroupRole, NewUser,
};
use reqwest::StatusCode;
use rust_kvittis_client::{AuthClient, UnauthClient};
use serde_json::{json, Value};

fn test_client() -> anyhow::Result<UnauthClient> {
    let base_url =
        std::env::var("KVITTIS_TEST_URL").unwrap_or_else(|_| "http://localhost:3000".to_owned());
    let mut client = UnauthClient::new(&base_url)?;
    client.http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;
    Ok(client)
}

async fn register_user(client: &UnauthClient) -> anyhow::Result<AuthClient> {
    let username = format!("group_visibility_{}", rand::random::<u64>());
    let user = NewUser {
        email: format!("{username}@example.invalid"),
        username,
        password: "group_visibility_password".to_owned(),
    };
    client.register_user(user.clone()).await?;
    client.login_user(user.username, user.password).await
}

async fn assert_group_not_found(client: &AuthClient, group_id: GroupId) -> anyhow::Result<()> {
    let response = client
        .http
        .post(client.url.join("/api/group")?)
        .bearer_auth(&client.token)
        .json(&GroupRequest::Get { group_id })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(
        response.json::<Value>().await?,
        json!({"message": "Group not found"})
    );
    Ok(())
}

#[tokio::test]
async fn group_lists_and_details_are_scoped_to_membership() -> anyhow::Result<()> {
    let client = test_client()?;
    let owner = register_user(&client).await?;
    let member = register_user(&client).await?;
    let owner_user = owner.get_user().await?.user;
    let member_user = member.get_user().await?.user;

    assert!(owner.list_groups().await?.is_empty());
    assert!(member.list_groups().await?.is_empty());
    let shared = owner.create_group("Shared group").await?;
    let private = owner.create_group("Private group").await?;
    assert!(member.list_groups().await?.is_empty());
    assert_group_not_found(&member, shared.id).await?;
    assert_group_not_found(&owner, GroupId::nil()).await?;

    owner.add_user_to_group(shared.id, member_user.id).await?;
    let own_groups = owner.list_groups().await?;
    assert_eq!(own_groups.len(), 2);
    assert!(own_groups.iter().any(|group| group.id == private.id));
    let member_groups = member.list_groups().await?;
    assert_eq!(member_groups.len(), 1);
    assert_eq!(member_groups[0].id, shared.id);
    assert_eq!(member_groups[0].name, shared.name);

    let details = member.get_group(shared.id).await?;
    assert_eq!(details.members.len(), 2);
    assert!(details.members.contains(&(owner_user.id, GroupRole::Admin)));
    assert!(details
        .members
        .contains(&(member_user.id, GroupRole::Member)));
    assert_eq!(member_groups[0].last_settled, details.last_settled);
    assert_eq!(member_groups[0].created_at, details.created_at);
    assert_eq!(member_groups[0].updated_at, details.updated_at);
    assert_eq!(member_groups[0].deleted_at, details.deleted_at);
    assert_group_not_found(&member, private.id).await?;

    let response = member
        .http
        .post(member.url.join("/api/group")?)
        .bearer_auth(&member.token)
        .json(&json!({"action": "list", "user_id": owner_user.id}))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    match response.json::<ApiResponse<Vec<Value>>>().await? {
        ApiResponse::Success(groups) => {
            assert_eq!(groups.len(), 1);
            assert_eq!(groups[0]["id"], json!(shared.id));
            assert!(groups[0].get("members").is_none());
        }
        ApiResponse::Error { message } => anyhow::bail!("Listing groups failed: {message}"),
    }

    let response = owner
        .http
        .post(owner.url.join("/api/group")?)
        .bearer_auth(&owner.token)
        .json(&json!({"action": "search", "query": ""}))
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);

    owner.delete_group(shared.id).await?;
    assert!(member.list_groups().await?.is_empty());
    assert_group_not_found(&member, shared.id).await?;
    assert_group_not_found(&owner, shared.id).await?;
    let remaining = owner.list_groups().await?;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, private.id);

    owner.delete_group(private.id).await?;
    owner.delete_user().await?;
    member.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn group_reads_require_authentication() -> anyhow::Result<()> {
    let client = test_client()?;
    for request in [
        GroupRequest::List,
        GroupRequest::Get {
            group_id: GroupId::nil(),
        },
    ] {
        for token in [None, Some("invalid-token")] {
            let mut request = client
                .http
                .post(client.url.join("/api/group")?)
                .json(&request);
            if let Some(token) = token {
                request = request.bearer_auth(token);
            }
            assert_eq!(request.send().await?.status(), StatusCode::UNAUTHORIZED);
        }
    }
    Ok(())
}
