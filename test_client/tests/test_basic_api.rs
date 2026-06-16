// These are more integration tests and scenario tests that are more complex.
use anyhow::anyhow;
use common::{FriendRequestAction, NewUser};
use rust_kvittis_client::UnauthClient;

const BASE_URL: &str = "http://localhost:3000";
const PASSWORD: &str = "secret_password";

// ====== Helpers =======

fn create_new_user(username: String) -> NewUser {
    let email = username.clone() + "@gmail.com";
    NewUser {
        username,
        email,
        password: PASSWORD.to_owned(),
    }
}

// ====== Basic Testing ======

#[tokio::test]
async fn test_register_user() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user = create_new_user("register_test_user".to_owned());
    let user = client.register_user(new_user.clone()).await?.user;
    assert_eq!(user.username, new_user.username);
    let auth_client = client
        .login_user(user.username, PASSWORD.to_owned())
        .await?;
    dbg!(&auth_client);
    let resp = auth_client.get_user().await?.user;
    assert_eq!(resp.username, new_user.username);

    // Cleanup - use unauth client for delete_user
    auth_client.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_user() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user = create_new_user("test_delete_user".to_owned());
    let user = client.register_user(new_user.clone()).await?.user;
    let auth_client = client
        .login_user(user.username.clone(), PASSWORD.to_owned())
        .await?;

    let resp = auth_client.get_user().await?.user;
    assert_eq!(resp.username, "test_delete_user");

    // Cleanup
    auth_client.delete_user().await?;

    // Try to log in again
    let login_result = client.login_user(user.username, PASSWORD.to_owned()).await;
    assert!(login_result.is_err());
    Ok(())
}

// Would like to have it as an admin thing, but not for everyone I guess.
// It is just a client so maybe I can implement it anyway.
#[tokio::test]
async fn test_get_users() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user1 = create_new_user("get_users_test_user1".to_owned());
    let new_user2 = create_new_user("get_users_test_user2".to_owned());
    let user1 = client.register_user(new_user1.clone()).await?.user;
    let user2 = client.register_user(new_user2.clone()).await?.user;
    let user1 = client
        .login_user(user1.username, PASSWORD.to_owned())
        .await?;
    let user2 = client
        .login_user(user2.username, PASSWORD.to_owned())
        .await?;

    // let usernames: Vec<String> = users.iter().map(|u| u.user.username.clone()).collect();
    // assert!(usernames.contains(&new_user1.username.to_string()));
    // assert!(usernames.contains(&new_user2.username.to_string()));

    // Need to delete users afterwards.
    user1.delete_user().await?;
    user2.delete_user().await?;

    Ok(())
}

#[tokio::test]
async fn test_send_friend_request() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user1 = create_new_user("send_friend_request_user1".to_owned());
    let new_user2 = create_new_user("send_friend_request_user2".to_owned());
    let user1 = client.register_user(new_user1.clone()).await?.user;
    let user2 = client.register_user(new_user2.clone()).await?.user;

    let auth_user1 = client
        .login_user(user1.username, PASSWORD.to_owned())
        .await?;
    let auth_user2 = client
        .login_user(user2.username, PASSWORD.to_owned())
        .await?;

    let friend_req = auth_user1.send_friend_request(user2.id).await?.request;

    let user1_pending_reqs = auth_user1.get_pending_friend_requests().await?;
    let user2_pending_reqs = auth_user2.get_pending_friend_requests().await?;
    assert_eq!(user1_pending_reqs.outgoing.len(), 1);
    assert_eq!(user2_pending_reqs.incoming.len(), 1);
    assert_eq!(
        user1_pending_reqs.outgoing[0].id,
        user2_pending_reqs.incoming[0].id
    );

    auth_user1
        .handle_friend_request(friend_req.id, FriendRequestAction::Cancel)
        .await?;
    auth_user1.delete_user().await?;
    auth_user2.delete_user().await?;

    Ok(())
}

#[tokio::test]
async fn test_cancel_friend_request() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user1 = create_new_user("cancel_friend_request_user1".to_owned());
    let new_user2 = create_new_user("cancel_friend_request_user2".to_owned());
    let user1 = client.register_user(new_user1.clone()).await?.user;
    let user2 = client.register_user(new_user2.clone()).await?.user;

    let auth_user1 = client
        .login_user(user1.username, PASSWORD.to_owned())
        .await?;
    let auth_user2 = client
        .login_user(user2.username, PASSWORD.to_owned())
        .await?;

    let friend_req = auth_user1.send_friend_request(user2.id).await?.request;

    let user1_pending_reqs = auth_user1.get_pending_friend_requests().await?;
    let user2_pending_reqs = auth_user2.get_pending_friend_requests().await?;
    assert_eq!(user1_pending_reqs.outgoing.len(), 1);
    assert_eq!(user2_pending_reqs.incoming.len(), 1);
    assert_eq!(
        user1_pending_reqs.outgoing[0].id,
        user2_pending_reqs.incoming[0].id
    );

    auth_user1
        .handle_friend_request(friend_req.id, FriendRequestAction::Cancel)
        .await?;
    let user1_pending_reqs = auth_user1.get_pending_friend_requests().await?;
    let user2_pending_reqs = auth_user2.get_pending_friend_requests().await?;
    assert_eq!(user1_pending_reqs.outgoing.len(), 0);
    assert_eq!(user2_pending_reqs.incoming.len(), 0);

    auth_user1.delete_user().await?;
    auth_user2.delete_user().await?;

    Ok(())
}

#[tokio::test]
async fn test_accept_friend_request() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user1 = create_new_user("accept_friend_request_user1".to_owned());
    let new_user2 = create_new_user("accept_friend_request_user2".to_owned());
    let user1 = client.register_user(new_user1.clone()).await?.user;
    let user2 = client.register_user(new_user2.clone()).await?.user;

    let auth_user1 = client
        .login_user(user1.username, PASSWORD.to_owned())
        .await?;
    let auth_user2 = client
        .login_user(user2.username, PASSWORD.to_owned())
        .await?;

    let friend_req = auth_user1.send_friend_request(user2.id).await?.request;

    let user1_pending_reqs = auth_user1.get_pending_friend_requests().await?;
    let user2_pending_reqs = auth_user2.get_pending_friend_requests().await?;
    assert_eq!(user1_pending_reqs.outgoing.len(), 1);
    assert_eq!(user2_pending_reqs.incoming.len(), 1);
    assert_eq!(
        user1_pending_reqs.outgoing[0].id,
        user2_pending_reqs.incoming[0].id
    );

    auth_user1
        .handle_friend_request(friend_req.id, FriendRequestAction::Accept)
        .await?;

    let user1_pending_reqs = auth_user1.get_pending_friend_requests().await?;
    let user2_pending_reqs = auth_user2.get_pending_friend_requests().await?;
    assert_eq!(user1_pending_reqs.outgoing.len(), 0);
    assert_eq!(user2_pending_reqs.incoming.len(), 0);

    let user1_friends = auth_user1.get_user().await?.user.friends;
    let user2_friends = auth_user2.get_user().await?.user.friends;
    assert_eq!(user1_friends.len(), 1);
    assert_eq!(user1_friends[0], user2.id);
    assert_eq!(user2_friends.len(), 1);
    assert_eq!(user2_friends[0], user1.id);

    auth_user1.delete_user().await?;
    auth_user2.delete_user().await?;

    Ok(())
}

#[tokio::test]
async fn test_search_users() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user1 = create_new_user("search_users_test_user1".to_owned());
    let new_user2 = create_new_user("search_users_test_user2".to_owned());
    let user1 = client.register_user(new_user1.clone()).await?.user;
    let user2 = client.register_user(new_user2.clone()).await?.user;

    let user1 = client
        .login_user(user1.username, PASSWORD.to_owned())
        .await?;
    let user2 = client
        .login_user(user2.username, PASSWORD.to_owned())
        .await?;

    let search_results = user1.search_users("search_users_test").await?;
    dbg!(&search_results);
    let usernames: Vec<String> = search_results
        .user
        .iter()
        .map(|u| u.username.clone())
        .collect();
    assert!(usernames.contains(&new_user1.username.to_string()));
    assert!(usernames.contains(&new_user2.username.to_string()));

    let search_results = user1.search_users("user1").await?;
    dbg!(&search_results);
    let usernames: Vec<String> = search_results
        .user
        .iter()
        .map(|u| u.username.clone())
        .collect();
    assert!(usernames.contains(&new_user1.username.to_string()));
    assert!(!usernames.contains(&new_user2.username.to_string()));

    user1.delete_user().await?;
    user2.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_create_group() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user = create_new_user("create_group_owner".to_owned());
    let user = client.register_user(new_user.clone()).await?.user;
    let auth_user = client
        .login_user(user.username, PASSWORD.to_owned())
        .await?;
    let group_name = "create_test_group";
    let group = auth_user.create_group(group_name).await?;
    assert_eq!(group.name, group_name);
    let resp = auth_user.get_group(group.id).await?;
    assert_eq!(resp.name, group_name);

    // Cleanup
    auth_user.delete_group(group.id).await?;
    auth_user.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_group() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user = create_new_user("delete_group_owner".to_owned());
    let user = client.register_user(new_user.clone()).await?.user;
    let auth_user = client
        .login_user(user.username, PASSWORD.to_owned())
        .await?;
    let group_name = "delete_test_group";
    let group = auth_user.create_group(group_name).await?;

    let resp = auth_user.get_group(group.id).await?;
    assert_eq!(resp.name, group_name);

    // Cleanup
    auth_user.delete_group(group.id).await?;
    match auth_user.get_group(group.id).await {
        Ok(_) => Err(anyhow!("Group should have been deleted")),
        Err(_) => Ok(()),
    }?;
    auth_user.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_add_user_to_group() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user_owner = create_new_user("add_user_to_group_owner".to_owned());
    let new_user_member = create_new_user("add_user_to_group_member".to_owned());
    let owner = client.register_user(new_user_owner.clone()).await?.user;
    let member = client.register_user(new_user_member.clone()).await?.user;
    let auth_owner = client
        .login_user(owner.username, PASSWORD.to_owned())
        .await?;
    let group_name = "add_user_to_group_test_group";
    let group = auth_owner.create_group(group_name).await?;

    auth_owner.add_user_to_group(group.id, member.id).await?;

    let resp = auth_owner.get_group(group.id).await?;
    assert!(resp
        .members
        .iter()
        .find(|(u, _)| u.eq(&member.id))
        .is_some());

    // Cleanup
    auth_owner.delete_group(group.id).await?;
    auth_owner.delete_user().await?;

    let auth_member = client
        .login_user(member.username, PASSWORD.to_owned())
        .await?;
    auth_member.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_get_group_by_name() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user = create_new_user("get_group_by_name_owner".to_owned());
    let owner = client.register_user(new_user.clone()).await?.user;
    let auth_owner = client
        .login_user(owner.username, PASSWORD.to_owned())
        .await?;
    let group_name = "get_group_by_name_test_group";
    let group = auth_owner.create_group(group_name).await?;

    let resp = auth_owner.search_group(group_name).await?;
    let ids: Vec<_> = resp.iter().map(|g| g.id).collect();

    assert!(ids.contains(&group.id));

    let resp = auth_owner.search_group("not_found_name").await?;
    let ids: Vec<_> = resp.iter().map(|g| g.id).collect();

    assert!(!ids.contains(&group.id));

    // Cleanup
    auth_owner.delete_group(group.id).await?;
    auth_owner.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_create_expense() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user_payer = create_new_user("create_expense_payer".to_owned());
    let new_user_borrower1 = create_new_user("create_expense_borrower1".to_owned());
    let new_user_borrower2 = create_new_user("create_expense_borrower2".to_owned());
    let payer = client.register_user(new_user_payer.clone()).await?.user;
    let borrower1 = client.register_user(new_user_borrower1.clone()).await?.user;
    let borrower2 = client.register_user(new_user_borrower2.clone()).await?.user;
    let auth_payer = client
        .login_user(payer.username, PASSWORD.to_owned())
        .await?;
    let description = "Test expense".to_string();

    let expense = auth_payer
        .create_expense(
            payer.id,
            vec![payer.id, borrower1.id, borrower2.id],
            100,
            Some(description.clone()),
            None,
        )
        .await?;

    assert_eq!(expense.payer, payer.id);
    assert_eq!(description, expense.description.unwrap());
    assert!(expense.participants.len() == 3);
    assert!(expense.participants.contains(&borrower1.id));
    assert!(expense.participants.contains(&borrower2.id));
    assert!(expense.participants.contains(&payer.id));

    // Cleanup
    auth_payer.delete_expense(expense.id).await?;
    auth_payer.delete_user().await?;
    let auth_borrower1 = client
        .login_user(borrower1.username, PASSWORD.to_owned())
        .await?;
    auth_borrower1.delete_user().await?;
    let auth_borrower2 = client
        .login_user(borrower2.username, PASSWORD.to_owned())
        .await?;
    auth_borrower2.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_get_expense() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user_payer = create_new_user("get_expense_payer".to_owned());
    let new_user_borrower1 = create_new_user("get_expense_borrower1".to_owned());
    let new_user_borrower2 = create_new_user("get_expense_borrower2".to_owned());
    let payer = client.register_user(new_user_payer).await?.user;
    let borrower1 = client.register_user(new_user_borrower1).await?.user;
    let borrower2 = client.register_user(new_user_borrower2).await?.user;
    let auth_payer = client
        .login_user(payer.username, PASSWORD.to_owned())
        .await?;
    let description = "Test expense".to_string();

    let expense = auth_payer
        .create_expense(
            payer.id,
            vec![payer.id, borrower1.id, borrower2.id],
            100,
            Some(description.clone()),
            None,
        )
        .await?;

    let expense = auth_payer.get_expense(expense.id).await?;
    assert_eq!(expense.payer, payer.id);
    assert_eq!(description, expense.description.unwrap());
    assert!(expense.participants.len() == 3);
    assert!(expense.participants.contains(&borrower1.id));
    assert!(expense.participants.contains(&borrower2.id));
    assert!(expense.participants.contains(&payer.id));

    // Cleanup
    auth_payer.delete_expense(expense.id).await?;
    auth_payer.delete_user().await?;
    let auth_borrower1 = client
        .login_user(borrower1.username, PASSWORD.to_owned())
        .await?;
    auth_borrower1.delete_user().await?;
    let auth_borrower2 = client
        .login_user(borrower2.username, PASSWORD.to_owned())
        .await?;
    auth_borrower2.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_expense() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user_payer = create_new_user("delete_expense_payer".to_owned());
    let new_user_borrower1 = create_new_user("delete_expense_borrower1".to_owned());
    let new_user_borrower2 = create_new_user("delete_expense_borrower2".to_owned());
    let payer = client.register_user(new_user_payer).await?.user;
    let borrower1 = client.register_user(new_user_borrower1).await?.user;
    let borrower2 = client.register_user(new_user_borrower2).await?.user;
    let auth_payer = client
        .login_user(payer.username, PASSWORD.to_owned())
        .await?;
    let description = "Test expense".to_string();

    let expense = auth_payer
        .create_expense(
            payer.id,
            vec![payer.id, borrower1.id, borrower2.id],
            100,
            Some(description.clone()),
            None,
        )
        .await?;

    assert_eq!(expense.payer, payer.id);
    assert_eq!(description, expense.description.unwrap());
    assert!(expense.participants.len() == 3);
    assert!(expense.participants.contains(&borrower1.id));
    assert!(expense.participants.contains(&borrower2.id));
    assert!(expense.participants.contains(&payer.id));

    // Cleanup
    auth_payer.delete_expense(expense.id).await?;
    auth_payer.delete_user().await?;
    let auth_borrower1 = client
        .login_user(borrower1.username, PASSWORD.to_owned())
        .await?;
    auth_borrower1.delete_user().await?;
    let auth_borrower2 = client
        .login_user(borrower2.username, PASSWORD.to_owned())
        .await?;
    auth_borrower2.delete_user().await?;
    Ok(())
}

// ====== Expense Testing User ======

// ====== Expense Testing Group ======
