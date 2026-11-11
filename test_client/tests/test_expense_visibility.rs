use std::time::Duration;

use common::{api::AuthorizedUserRequest, ExpenseId, GroupId, NewUser};
use reqwest::StatusCode;
use rust_kvittis_client::{AuthClient, UnauthClient};
use serde_json::{json, Value};

const PASSWORD: &str = "expense_visibility_password";

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
    let username = format!("expense_visibility_{}", rand::random::<u64>());
    let user = NewUser {
        email: format!("{username}@example.invalid"),
        username,
        password: PASSWORD.to_owned(),
    };
    client.register_user(user.clone()).await?;
    client.login_user(user.username, user.password).await
}

async fn assert_error(
    client: &AuthClient,
    endpoint: &str,
    request: Value,
    status: StatusCode,
    message: &str,
) -> anyhow::Result<()> {
    let response = client
        .http
        .post(client.url.join(endpoint)?)
        .bearer_auth(&client.token)
        .json(&request)
        .send()
        .await?;
    assert_eq!(response.status(), status, "Request: {request}");
    assert_eq!(response.json::<Value>().await?, json!({"message": message}));
    Ok(())
}

#[tokio::test]
async fn non_group_expenses_require_payer_or_participant_access() -> anyhow::Result<()> {
    let client = test_client()?;
    let payer = register_user(&client).await?;
    let participant = register_user(&client).await?;
    let stranger = register_user(&client).await?;
    let payer_id = payer.get_user().await?.user.id;
    let participant_id = participant.get_user().await?.user.id;

    let payer_only = payer
        .create_expense(payer_id, vec![], 100, None, None)
        .await?;
    assert!(payer
        .get_expense(payer_only.id)
        .await?
        .participants
        .is_empty());
    assert_eq!(payer.list_expenses_for_user(payer_id).await?.len(), 1);
    for action in ["get", "delete"] {
        for id in [payer_only.id, ExpenseId::nil()] {
            assert_error(
                &stranger,
                "/api/expense",
                json!({"action": action, "id": id}),
                StatusCode::NOT_FOUND,
                "Expense not found",
            )
            .await?;
        }
    }
    assert_eq!(payer.get_expense(payer_only.id).await?.id, payer_only.id);
    assert!(stranger.list_expenses_for_user(payer_id).await?.is_empty());

    assert_error(
        &stranger,
        "/api/expense",
        json!({
            "action": "create", "payer": payer_id, "participants": [participant_id],
            "amount": 100, "group_id": null
        }),
        StatusCode::FORBIDDEN,
        "Not allowed",
    )
    .await?;
    assert_eq!(payer.list_expenses_for_user(payer_id).await?.len(), 1);

    let shared = participant
        .create_expense(payer_id, vec![participant_id], 200, None, None)
        .await?;
    assert_eq!(payer.get_expense(shared.id).await?.id, shared.id);
    assert_eq!(participant.get_expense(shared.id).await?.id, shared.id);
    let visible = participant.list_expenses_for_user(payer_id).await?;
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].id, shared.id);

    participant.delete_expense(shared.id).await?;
    assert_error(
        &payer,
        "/api/expense",
        json!({"action": "get", "id": shared.id}),
        StatusCode::NOT_FOUND,
        "Expense not found",
    )
    .await?;
    payer.delete_expense(payer_only.id).await?;
    assert_error(
        &payer,
        "/api/expense",
        json!({"action": "delete", "id": payer_only.id}),
        StatusCode::NOT_FOUND,
        "Expense not found",
    )
    .await?;

    payer.delete_user().await?;
    participant.delete_user().await?;
    stranger.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn group_membership_and_expense_involvement_grant_independent_access() -> anyhow::Result<()> {
    let client = test_client()?;
    let owner = register_user(&client).await?;
    let member = register_user(&client).await?;
    let outsider = register_user(&client).await?;
    let stranger = register_user(&client).await?;
    let owner_id = owner.get_user().await?.user.id;
    let member_id = member.get_user().await?.user.id;
    let outsider_id = outsider.get_user().await?.user.id;
    let group = owner.create_group("Expense visibility").await?;
    owner.add_user_to_group(group.id, member_id).await?;

    let outsider_pays = owner
        .create_expense(outsider_id, vec![member_id], 100, None, Some(group.id))
        .await?;
    let outsider_participates = owner
        .create_expense(owner_id, vec![outsider_id], 200, None, Some(group.id))
        .await?;
    let private = owner
        .create_expense(owner_id, vec![member_id], 300, None, Some(group.id))
        .await?;
    let member_uninvolved = owner
        .create_expense(owner_id, vec![owner_id], 400, None, Some(group.id))
        .await?;

    assert_eq!(
        owner.get_expense(outsider_pays.id).await?.id,
        outsider_pays.id
    );
    assert_eq!(
        member.get_expense(member_uninvolved.id).await?.id,
        member_uninvolved.id
    );
    assert_eq!(owner.list_expenses_for_group(group.id).await?.len(), 4);
    assert_eq!(member.list_expenses_for_group(group.id).await?.len(), 4);
    assert!(stranger.list_expenses_for_group(group.id).await?.is_empty());
    let visible = outsider.list_expenses_for_group(group.id).await?;
    assert_eq!(visible.len(), 2);
    assert!(visible.iter().any(|e| e.id == outsider_pays.id));
    assert!(visible.iter().any(|e| e.id == outsider_participates.id));
    let visible_for_owner = outsider.list_expenses_for_user(owner_id).await?;
    assert_eq!(visible_for_owner.len(), 1);
    assert_eq!(visible_for_owner[0].id, outsider_participates.id);
    assert!(outsider
        .list_expenses_for_group(GroupId::nil())
        .await?
        .is_empty());

    for action in ["get", "delete"] {
        assert_error(
            &outsider,
            "/api/expense",
            json!({"action": action, "id": private.id}),
            StatusCode::NOT_FOUND,
            "Expense not found",
        )
        .await?;
    }
    for group_id in [group.id, GroupId::nil()] {
        assert_error(
            &outsider,
            "/api/expense",
            json!({
                "action": "create", "payer": outsider_id, "participants": [outsider_id],
                "amount": 100, "group_id": group_id
            }),
            StatusCode::FORBIDDEN,
            "Not allowed",
        )
        .await?;
    }
    assert_eq!(owner.list_expenses_for_group(group.id).await?.len(), 4);
    assert!(!member.get_group_balances(group.id).await?.is_empty());
    for group_id in [group.id, GroupId::nil()] {
        assert_error(
            &outsider,
            "/api/balance",
            json!({"action": "group", "group_id": group_id}),
            StatusCode::NOT_FOUND,
            "Group not found",
        )
        .await?;
    }

    member.delete_expense(member_uninvolved.id).await?;
    owner.delete_group(group.id).await?;
    assert_error(
        &member,
        "/api/expense",
        json!({"action": "get", "id": outsider_participates.id}),
        StatusCode::NOT_FOUND,
        "Expense not found",
    )
    .await?;
    assert_error(
        &owner,
        "/api/expense",
        json!({"action": "get", "id": outsider_pays.id}),
        StatusCode::NOT_FOUND,
        "Expense not found",
    )
    .await?;
    assert_eq!(
        outsider.get_expense(outsider_participates.id).await?.id,
        outsider_participates.id
    );
    assert_error(
        &owner,
        "/api/balance",
        json!({"action": "group", "group_id": group.id}),
        StatusCode::NOT_FOUND,
        "Group not found",
    )
    .await?;
    assert_error(
        &owner,
        "/api/expense",
        json!({
            "action": "create", "payer": owner_id, "participants": [owner_id],
            "amount": 100, "group_id": group.id
        }),
        StatusCode::FORBIDDEN,
        "Not allowed",
    )
    .await?;

    outsider.delete_expense(outsider_pays.id).await?;
    outsider.delete_expense(outsider_participates.id).await?;
    member.delete_expense(private.id).await?;
    owner.delete_user().await?;
    member.delete_user().await?;
    outsider.delete_user().await?;
    stranger.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn user_balances_are_only_available_to_their_owner() -> anyhow::Result<()> {
    let client = test_client()?;
    let payer = register_user(&client).await?;
    let participant = register_user(&client).await?;
    let payer_id = payer.get_user().await?.user.id;
    let participant_id = participant.get_user().await?.user.id;
    assert!(participant
        .get_user_balances(participant_id)
        .await?
        .is_empty());
    let expense = payer
        .create_expense(payer_id, vec![participant_id], 100, None, None)
        .await?;
    assert_eq!(payer.get_user_balances(payer_id).await?[0].amount, 100);
    assert_eq!(
        participant.get_user_balances(participant_id).await?[0].amount,
        -100
    );
    for user_id in [payer_id, common::UserId::nil()] {
        assert_error(
            &participant,
            "/api/balance",
            json!({"action": "user", "user_id": user_id}),
            StatusCode::FORBIDDEN,
            "Not allowed",
        )
        .await?;
    }
    payer.delete_expense(expense.id).await?;
    payer.delete_user().await?;
    participant.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn all_expense_and_balance_actions_require_a_valid_session() -> anyhow::Result<()> {
    let client = test_client()?;
    let auth = register_user(&client).await?;
    let user = auth.get_user().await?.user;
    let other_session = client
        .login_user(user.username, PASSWORD.to_owned())
        .await?;
    let response = auth
        .http
        .post(auth.url.join("/api/auth_user")?)
        .bearer_auth(&auth.token)
        .json(&AuthorizedUserRequest::Logout)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK);

    let requests = [
        (
            "/api/expense",
            json!({
                "action": "create", "payer": user.id, "participants": [],
                "amount": 100, "group_id": null
            }),
        ),
        (
            "/api/expense",
            json!({"action": "get", "id": ExpenseId::nil()}),
        ),
        (
            "/api/expense",
            json!({"action": "delete", "id": ExpenseId::nil()}),
        ),
        (
            "/api/expense",
            json!({"action": "list_for_user", "user_id": user.id}),
        ),
        (
            "/api/expense",
            json!({"action": "list_for_group", "group_id": GroupId::nil()}),
        ),
        (
            "/api/balance",
            json!({"action": "user", "user_id": user.id}),
        ),
        (
            "/api/balance",
            json!({"action": "group", "group_id": GroupId::nil()}),
        ),
    ];
    for (endpoint, payload) in requests {
        for token in [None, Some("invalid-token"), Some(auth.token.as_str())] {
            let mut request = auth.http.post(auth.url.join(endpoint)?).json(&payload);
            if let Some(token) = token {
                request = request.bearer_auth(token);
            }
            assert_eq!(request.send().await?.status(), StatusCode::UNAUTHORIZED);
        }
    }
    assert!(other_session
        .list_expenses_for_user(user.id)
        .await?
        .is_empty());
    other_session.delete_user().await?;
    Ok(())
}
