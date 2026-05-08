// These are more integration tests and scenario tests that are more complex.
use anyhow::anyhow;
use common::NewUser;
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
    auth_client.delete_user(user.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_user() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user = create_new_user("test_delete_user".to_owned());
    let user = client.register_user(new_user.clone()).await?.user;
    let auth_client = client
        .login_user(user.username, PASSWORD.to_owned())
        .await?;

    let resp = auth_client.get_user().await?.user;
    assert_eq!(resp.username, "test_delete_user");

    // Cleanup
    auth_client.delete_user(user.id).await?;
    match auth_client.get_user().await {
        Ok(_) => Err(anyhow!("User should have been deleted")),
        Err(_) => Ok(()),
    }
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
    let auth_client = client
        .login_user(user1.username, PASSWORD.to_owned())
        .await?;

    // Should not be allowed to fetch users.
    let resp = auth_client.get_users().await;
    assert!(resp.is_err());

    // let usernames: Vec<String> = users.iter().map(|u| u.user.username.clone()).collect();
    // assert!(usernames.contains(&new_user1.username.to_string()));
    // assert!(usernames.contains(&new_user2.username.to_string()));

    // Need to delete users afterwards.
    auth_client.delete_user(user1.id).await?;
    auth_client.delete_user(user2.id).await?;

    Ok(())
}

#[tokio::test]
async fn test_search_users() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user1 = create_new_user("search_users_test_user1".to_owned());
    let new_user2 = create_new_user("search_users_test_user2".to_owned());
    let user1 = client.register_user(new_user1.clone()).await?.user;
    let user2 = client.register_user(new_user2.clone()).await?.user;
    let auth_client = client
        .login_user(user1.username, PASSWORD.to_owned())
        .await?;

    let search_results = auth_client.search_users("search_users_test").await?;
    dbg!(&search_results);
    let usernames: Vec<String> = search_results
        .user
        .iter()
        .map(|u| u.username.clone())
        .collect();
    assert!(usernames.contains(&new_user1.username.to_string()));
    assert!(usernames.contains(&new_user2.username.to_string()));

    let search_results = auth_client.search_users("user1").await?;
    dbg!(&search_results);
    let usernames: Vec<String> = search_results
        .user
        .iter()
        .map(|u| u.username.clone())
        .collect();
    assert!(usernames.contains(&new_user1.username.to_string()));
    assert!(!usernames.contains(&new_user2.username.to_string()));

    auth_client.delete_user(user1.id).await?;
    auth_client.delete_user(user2.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_create_group() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user = create_new_user("create_group_owner".to_owned());
    let user = client.register_user(new_user.clone()).await?.user;
    let auth_client = client
        .login_user(user.username, PASSWORD.to_owned())
        .await?;
    let group_name = "create_test_group";
    let group = auth_client
        .create_group(group_name, user.id, vec![user.id])
        .await?;
    assert_eq!(group.name, group_name);
    let resp = auth_client.get_group(group.id).await?;
    assert_eq!(resp.name, group_name);

    // Cleanup
    auth_client.delete_group(group.id).await?;
    auth_client.delete_user(user.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_group() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user = create_new_user("delete_group_owner".to_owned());
    let user = client.register_user(new_user.clone()).await?.user;
    let auth_client = client
        .login_user(user.username, PASSWORD.to_owned())
        .await?;
    let group_name = "delete_test_group";
    let group = auth_client
        .create_group(group_name, user.id, vec![user.id])
        .await?;

    let resp = auth_client.get_group(group.id).await?;
    assert_eq!(resp.name, group_name);

    // Cleanup
    auth_client.delete_group(group.id).await?;
    match auth_client.get_group(group.id).await {
        Ok(_) => Err(anyhow!("Group should have been deleted")),
        Err(_) => Ok(()),
    }?;
    auth_client.delete_user(user.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_add_user_to_group() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user_owner = create_new_user("add_user_to_group_owner".to_owned());
    let new_user_member = create_new_user("add_user_to_group_member".to_owned());
    let owner = client.register_user(new_user_owner.clone()).await?.user;
    let member = client.register_user(new_user_member.clone()).await?.user;
    let auth_client = client
        .login_user(owner.username, PASSWORD.to_owned())
        .await?;
    let group_name = "add_user_to_group_test_group";
    let group = auth_client
        .create_group(group_name, owner.id, vec![owner.id])
        .await?;

    auth_client.add_user_to_group(group.id, member.id).await?;

    let resp = auth_client.get_group(group.id).await?;
    assert!(resp.members.contains(&member.id));

    // Cleanup
    auth_client.delete_group(group.id).await?;
    auth_client.delete_user(owner.id).await?;
    auth_client.delete_user(member.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_get_group_by_name() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user = create_new_user("get_group_by_name_owner".to_owned());
    let owner = client.register_user(new_user.clone()).await?.user;
    let auth_client = client
        .login_user(owner.username, PASSWORD.to_owned())
        .await?;
    let group_name = "get_group_by_name_test_group";
    let group = auth_client
        .create_group(group_name, owner.id, vec![owner.id])
        .await?;

    let resp = auth_client.search_group(group_name).await?;
    let ids: Vec<_> = resp.iter().map(|g| g.id).collect();

    assert!(ids.contains(&group.id));

    let resp = auth_client.search_group("not_found_name").await?;
    let ids: Vec<_> = resp.iter().map(|g| g.id).collect();

    assert!(!ids.contains(&group.id));

    // Cleanup
    auth_client.delete_group(group.id).await?;
    auth_client.delete_user(owner.id).await?;
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
    let auth_client = client
        .login_user(payer.username, PASSWORD.to_owned())
        .await?;
    let description = "Test expense".to_string();

    let expense = auth_client
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
    auth_client.delete_expense(expense.id).await?;
    auth_client.delete_user(payer.id).await?;
    auth_client.delete_user(borrower1.id).await?;
    auth_client.delete_user(borrower2.id).await?;
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
    let auth_client = client
        .login_user(payer.username, PASSWORD.to_owned())
        .await?;
    let description = "Test expense".to_string();

    let expense = auth_client
        .create_expense(
            payer.id,
            vec![payer.id, borrower1.id, borrower2.id],
            100,
            Some(description.clone()),
            None,
        )
        .await?;

    let expense = auth_client.get_expense(expense.id).await?;
    assert_eq!(expense.payer, payer.id);
    assert_eq!(description, expense.description.unwrap());
    assert!(expense.participants.len() == 3);
    assert!(expense.participants.contains(&borrower1.id));
    assert!(expense.participants.contains(&borrower2.id));
    assert!(expense.participants.contains(&payer.id));

    // Cleanup
    auth_client.delete_expense(expense.id).await?;
    auth_client.delete_user(payer.id).await?;
    auth_client.delete_user(borrower1.id).await?;
    auth_client.delete_user(borrower2.id).await?;
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
    let auth_client = client
        .login_user(payer.username, PASSWORD.to_owned())
        .await?;
    let description = "Test expense".to_string();

    let expense = auth_client
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
    auth_client.delete_expense(expense.id).await?;
    auth_client.delete_user(payer.id).await?;
    auth_client.delete_user(borrower1.id).await?;
    auth_client.delete_user(borrower2.id).await?;
    Ok(())
}

// ====== Expense Testing User ======

// ====== Expense Testing Group ======
