
// These are more integration tests and scenario tests that are more complex.
use anyhow::anyhow;
use common::{GroupId, User, UserId, api::{GetGroupResponse, GetUserResponse}};
use rust_kvittis_client::kvittis_client::KvittisClient;

const BASE_URL: &str = "http://localhost:3000";

// ====== Basic Testing ======

#[tokio::test]
async fn test_register_user() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;
    let username = "register_test_user";
    let user = client.register_user(username).await?;
    assert_eq!(user.username, username);
    let resp = client.get_user(user.id).await?;
    assert_eq!(resp.username, username);

    // Cleanup
    client.delete_user(user.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_user() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;
    let username = "delete_test_user";
    let user = client.register_user(username).await?;

    let resp = client.get_user(user.id).await?;
    assert_eq!(resp.username, username);

    // Cleanup
    client.delete_user(user.id).await?;
    match client.get_user(user.id).await {
        Ok(_) => Err(anyhow!("User should have been deleted")),
        Err(_) => Ok(()),
    }
}

// Would like to have it as an admin thing, but not for everyone I guess.
// It is just a client so maybe I can implement it anyway.
#[tokio::test]
async fn test_get_users() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;
    let username1 = "get_users_test_user1";
    let username2 = "get_users_test_user2";
    let user1 = client.register_user(username1).await?;
    let user2 = client.register_user(username2).await?;

    let users = client.get_users().await?;
    let usernames: Vec<String> = users.iter().map(|u| u.username.clone()).collect();
    assert!(usernames.contains(&username1.to_string()));
    assert!(usernames.contains(&username2.to_string()));

    client.delete_user(user1.id).await?;
    client.delete_user(user2.id).await?;

    Ok(())
}

#[tokio::test]
async fn test_search_users() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;
    let username1 = "search_users_test_user1";
    let username2 = "search_users_test_user2";
    let user1 = client.register_user(username1).await?;
    let user2 = client.register_user(username2).await?;
    
    let search_results = client.search_users("search_users_test").await?;
    dbg!(&search_results);
    let usernames: Vec<String> = search_results.iter().map(|u| u.username.clone()).collect();
    assert!(usernames.contains(&username1.to_string()));
    assert!(usernames.contains(&username2.to_string()));

    let search_results = client.search_users("user1").await?;
    dbg!(&search_results);
    let usernames: Vec<String> = search_results.iter().map(|u| u.username.clone()).collect();
    assert!(usernames.contains(&username1.to_string()));
    assert!(!usernames.contains(&username2.to_string()));

    client.delete_user(user1.id).await?;
    client.delete_user(user2.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_create_group() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;
    let user = client.register_user("create_group_owner").await?;
    let group_name = "create_test_group";
    let group = client.create_group(group_name, user.id, vec![user.id]).await?;
    assert_eq!(group.name, group_name);
    let resp = client.get_group(group.id).await?;
    assert_eq!(resp.name, group_name);

    // Cleanup
    client.delete_group(group.id).await?;
    client.delete_user(user.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_group() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;
    let user = client.register_user("delete_group_owner").await?;
    let group_name = "delete_test_group";
    let group = client.create_group(group_name, user.id, vec![user.id]).await?;

    let resp = client.get_group(group.id).await?;
    assert_eq!(resp.name, group_name);

    // Cleanup
    client.delete_group(group.id).await?;
    match client.get_group(group.id).await {
        Ok(_) => Err(anyhow!("Group should have been deleted")),
        Err(_) => Ok(()),
    }?;
    client.delete_user(user.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_add_user_to_group() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;
    let owner = client.register_user("add_user_to_group_owner").await?;
    let member = client.register_user("add_user_to_group_member").await?;
    let group_name = "add_user_to_group_test_group";
    let group = client.create_group(group_name, owner.id, vec![owner.id]).await?;

    client.add_user_to_group(group.id, member.id).await?;

    let resp = client.get_group(group.id).await?;
    assert!(resp.members.contains(&member.id));

    // Cleanup
    client.delete_group(group.id).await?;
    client.delete_user(owner.id).await?;
    client.delete_user(member.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_get_group_by_name() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;
    let owner = client.register_user("get_group_by_name_owner").await?;
    let group_name = "get_group_by_name_test_group";
    let group = client.create_group(group_name, owner.id, vec![owner.id]).await?;

    let resp = client.search_group(group_name).await?;
    let ids: Vec<GroupId> = resp.iter().map(|g| g.id).collect();

    assert!(ids.contains(&group.id));

    let resp = client.search_group("not_found_name").await?;
    let ids: Vec<GroupId> = resp.iter().map(|g| g.id).collect();

    assert!(!ids.contains(&group.id));

    // Cleanup
    client.delete_group(group.id).await?;
    client.delete_user(owner.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_create_expense() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;
    let payer = client.register_user("create_expense_payer").await?;
    let borrower1 = client.register_user("create_expense_borrower1").await?;
    let borrower2 = client.register_user("create_expense_borrower2").await?;
    let description = "Test expense".to_string();

    let expense = client.create_expense(
        payer.id,
        vec![payer.id, borrower1.id, borrower2.id],
        100,
        Some(description.clone()),
        None,
    ).await?;

    assert_eq!(expense.payer, payer.id);
    assert_eq!(description, expense.description.unwrap());
    assert!(expense.participants.len() == 3);
    assert!(expense.participants.contains(&borrower1.id));
    assert!(expense.participants.contains(&borrower2.id));
    assert!(expense.participants.contains(&payer.id));

    // Cleanup
    client.delete_expense(expense.id).await?;
    client.delete_user(payer.id).await?;
    client.delete_user(borrower1.id).await?;
    client.delete_user(borrower2.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_get_expense() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;
    let payer = client.register_user("get_expense_payer").await?;
    let borrower1 = client.register_user("get_expense_borrower1").await?;
    let borrower2 = client.register_user("get_expense_borrower2").await?;
    let description = "Test expense".to_string();

    let expense = client.create_expense(
        payer.id,
        vec![payer.id, borrower1.id, borrower2.id],
        100,
        Some(description.clone()),
        None,
    ).await?;

    let expense = client.get_expense(expense.id).await?;
    assert_eq!(expense.payer, payer.id);
    assert_eq!(description, expense.description.unwrap());
    assert!(expense.participants.len() == 3);
    assert!(expense.participants.contains(&borrower1.id));
    assert!(expense.participants.contains(&borrower2.id));
    assert!(expense.participants.contains(&payer.id));

    // Cleanup
    client.delete_expense(expense.id).await?;
    client.delete_user(payer.id).await?;
    client.delete_user(borrower1.id).await?;
    client.delete_user(borrower2.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_delete_expense() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;
    let payer = client.register_user("delete_expense_payer").await?;
    let borrower1 = client.register_user("delete_expense_borrower1").await?;
    let borrower2 = client.register_user("delete_expense_borrower2").await?;
    let description = "Test expense".to_string();

    let expense = client.create_expense(
        payer.id,
        vec![payer.id, borrower1.id, borrower2.id],
        100,
        Some(description.clone()),
        None,
    ).await?;

    assert_eq!(expense.payer, payer.id);
    assert_eq!(description, expense.description.unwrap());
    assert!(expense.participants.len() == 3);
    assert!(expense.participants.contains(&borrower1.id));
    assert!(expense.participants.contains(&borrower2.id));
    assert!(expense.participants.contains(&payer.id));

    // Cleanup
    client.delete_expense(expense.id).await?;
    client.delete_user(payer.id).await?;
    client.delete_user(borrower1.id).await?;
    client.delete_user(borrower2.id).await?;
    Ok(())
}


// ====== Expense Testing User ======

// ====== Expense Testing Group ======

