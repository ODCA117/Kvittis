use std::time::Duration;

use common::{
    api::{
        auth::LogoutResponse, ApiResponse, AuthorizedUserRequest, LoginResponse,
        UnauthorizedUserRequest,
    },
    NewUser,
};
use reqwest::StatusCode;
use rust_kvittis_client::{AuthClient, UnauthClient};

const PASSWORD: &str = "session_test_password";

async fn register_test_user() -> anyhow::Result<(UnauthClient, NewUser)> {
    let base_url =
        std::env::var("KVITTIS_TEST_URL").unwrap_or_else(|_| "http://localhost:3000".to_owned());
    let mut client = UnauthClient::new(&base_url)?;
    client.http = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .build()?;

    let username = format!("session_test_{}", rand::random::<u64>());
    let user = NewUser {
        email: format!("{username}@example.invalid"),
        username,
        password: PASSWORD.to_owned(),
    };
    client.register_user(user.clone()).await?;
    Ok((client, user))
}

async fn logout(client: &AuthClient) -> anyhow::Result<()> {
    let response = client
        .http
        .post(client.url.join("/api/auth_user")?)
        .bearer_auth(&client.token)
        .json(&AuthorizedUserRequest::Logout)
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::OK, "Logout should succeed");
    match response.json::<ApiResponse<LogoutResponse>>().await? {
        ApiResponse::Success(_) => Ok(()),
        ApiResponse::Error { message } => anyhow::bail!("Logout failed: {message}"),
    }
}

async fn assert_session_rejected(
    client: &AuthClient,
    request: AuthorizedUserRequest,
) -> anyhow::Result<()> {
    let response = client
        .http
        .post(client.url.join("/api/auth_user")?)
        .bearer_auth(&client.token)
        .json(&request)
        .send()
        .await?;

    assert!(
        response.status() == StatusCode::UNAUTHORIZED,
        "Expected authentication rejection, got {}",
        response.status()
    );
    Ok(())
}

#[tokio::test]
async fn logout_revokes_copied_token_but_not_other_logins() -> anyhow::Result<()> {
    let (client, user) = register_test_user().await?;
    let first = client
        .login_user(user.username.clone(), user.password.clone())
        .await?;
    let copied = first.clone();
    let second = client
        .login_user(user.username.clone(), user.password.clone())
        .await?;

    let first_user = first.get_user().await?.user;
    assert_eq!(first_user.username, user.username);
    assert_eq!(second.get_user().await?.user.id, first_user.id);

    logout(&first).await?;
    assert_session_rejected(&copied, AuthorizedUserRequest::Get).await?;
    assert_session_rejected(&copied, AuthorizedUserRequest::Logout).await?;
    assert_eq!(
        second.get_user().await?.user.id,
        first_user.id,
        "Logging out one session must not revoke another"
    );

    let new_login = client.login_user(user.username, user.password).await?;
    assert_eq!(new_login.get_user().await?.user.id, first_user.id);
    assert_session_rejected(&copied, AuthorizedUserRequest::Get).await?;

    new_login.delete_user().await?;
    assert_session_rejected(&second, AuthorizedUserRequest::Get).await?;
    Ok(())
}

#[tokio::test]
async fn wrong_password_does_not_log_in() -> anyhow::Result<()> {
    let (client, user) = register_test_user().await?;
    let response = client
        .http
        .post(client.url.join("/api/unauth_user")?)
        .json(&UnauthorizedUserRequest::Login {
            username: user.username.clone(),
            password: "incorrect_password".to_owned(),
        })
        .send()
        .await?;
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    assert!(matches!(
        response.json::<ApiResponse<LoginResponse>>().await?,
        ApiResponse::Error { .. }
    ));

    let valid_login = client
        .login_user(user.username.clone(), user.password)
        .await?;
    assert_eq!(valid_login.get_user().await?.user.username, user.username);
    valid_login.delete_user().await?;
    Ok(())
}
