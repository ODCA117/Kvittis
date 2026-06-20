// These are more integration tests and scenario tests that are more complex.
use anyhow::anyhow;
use common::{FriendRequestAction, GroupRole, NewUser};
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

    let group = auth_user.get_group(group.id).await?;
    assert!(group.members.contains(&(user.id, GroupRole::Admin)));

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

// ====== Balance API Tests ======

#[tokio::test]
async fn test_list_expenses_for_user() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user_payer = create_new_user("test_expenses_payer".to_owned());
    let new_user_participant1 = create_new_user("test_expenses_participant1".to_owned());
    let new_user_participant2 = create_new_user("test_expenses_participant2".to_owned());

    let payer = client.register_user(new_user_payer.clone()).await?.user;
    let participant1 = client
        .register_user(new_user_participant1.clone())
        .await?
        .user;
    let participant2 = client
        .register_user(new_user_participant2.clone())
        .await?
        .user;

    let auth_payer = client
        .login_user(payer.username, PASSWORD.to_owned())
        .await?;

    // Create expense where payer pays for all participants
    let expense1 = auth_payer
        .create_expense(
            payer.id,
            vec![payer.id, participant1.id, participant2.id],
            300, // 300 cents = 3.00
            Some("Dinner".to_string()),
            None,
        )
        .await?;

    // Create another expense
    let expense2 = auth_payer
        .create_expense(
            payer.id,
            vec![payer.id, participant1.id],
            200, // 200 cents = 2.00
            Some("Coffee".to_string()),
            None,
        )
        .await?;

    // List expenses for the payer
    let user_expenses = auth_payer.list_expenses_for_user(payer.id).await?;
    assert_eq!(user_expenses.len(), 2);
    let expense_ids: Vec<_> = user_expenses.iter().map(|e| e.id).collect();
    assert!(expense_ids.contains(&expense1.id));
    assert!(expense_ids.contains(&expense2.id));

    // List expenses for participant1
    let participant1_expenses = auth_payer.list_expenses_for_user(participant1.id).await?;
    assert_eq!(participant1_expenses.len(), 2);

    // List expenses for participant2 (should only have 1 expense)
    let participant2_expenses = auth_payer.list_expenses_for_user(participant2.id).await?;
    assert_eq!(participant2_expenses.len(), 1);
    assert_eq!(participant2_expenses[0].id, expense1.id);

    // Cleanup
    auth_payer.delete_expense(expense1.id).await?;
    auth_payer.delete_expense(expense2.id).await?;

    let auth_participant1 = client
        .login_user(participant1.username, PASSWORD.to_owned())
        .await?;
    auth_participant1.delete_user().await?;

    let auth_participant2 = client
        .login_user(participant2.username, PASSWORD.to_owned())
        .await?;
    auth_participant2.delete_user().await?;

    auth_payer.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_list_expenses_for_group() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user_owner = create_new_user("test_group_expenses_owner".to_owned());
    let new_user_member1 = create_new_user("test_group_expenses_member1".to_owned());
    let new_user_member2 = create_new_user("test_group_expenses_member2".to_owned());
    let new_user_non_member = create_new_user("test_group_expenses_non_member".to_owned());

    let owner = client.register_user(new_user_owner.clone()).await?.user;
    let member1 = client.register_user(new_user_member1.clone()).await?.user;
    let member2 = client.register_user(new_user_member2.clone()).await?.user;
    let non_member = client
        .register_user(new_user_non_member.clone())
        .await?
        .user;

    let auth_owner = client
        .login_user(owner.username, PASSWORD.to_owned())
        .await?;

    // Create a group
    let group = auth_owner.create_group("test_group_expenses").await?;

    // Add members to the group
    auth_owner.add_user_to_group(group.id, member1.id).await?;
    auth_owner.add_user_to_group(group.id, member2.id).await?;

    // Create expense in the group where owner pays for all members
    let expense1 = auth_owner
        .create_expense(
            owner.id,
            vec![owner.id, member1.id, member2.id],
            300,
            Some("Group dinner".to_string()),
            Some(group.id),
        )
        .await?;

    // Create another expense in the group
    let expense2 = auth_owner
        .create_expense(
            member1.id,
            vec![owner.id, member1.id, member2.id],
            200,
            Some("Group coffee".to_string()),
            Some(group.id),
        )
        .await?;

    // Create an expense NOT in the group
    let expense3 = auth_owner
        .create_expense(
            owner.id,
            vec![owner.id, non_member.id],
            100,
            Some("Private lunch".to_string()),
            None,
        )
        .await?;

    // List expenses for the group - should return 2 expenses
    let group_expenses = auth_owner.list_expenses_for_group(group.id).await?;
    assert_eq!(group_expenses.len(), 2);
    let group_expense_ids: Vec<_> = group_expenses.iter().map(|e| e.id).collect();
    assert!(group_expense_ids.contains(&expense1.id));
    assert!(group_expense_ids.contains(&expense2.id));
    assert!(!group_expense_ids.contains(&expense3.id));

    // Verify that group_id is set correctly
    for expense in group_expenses {
        assert_eq!(expense.group_id, Some(group.id));
    }

    // Cleanup
    auth_owner.delete_expense(expense1.id).await?;
    auth_owner.delete_expense(expense2.id).await?;
    auth_owner.delete_expense(expense3.id).await?;
    auth_owner.delete_group(group.id).await?;

    let auth_member1 = client
        .login_user(member1.username, PASSWORD.to_owned())
        .await?;
    auth_member1.delete_user().await?;

    let auth_member2 = client
        .login_user(member2.username, PASSWORD.to_owned())
        .await?;
    auth_member2.delete_user().await?;

    let auth_non_member = client
        .login_user(non_member.username, PASSWORD.to_owned())
        .await?;
    auth_non_member.delete_user().await?;

    auth_owner.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_user_balances_basic() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user_payer = create_new_user("test_balances_payer".to_owned());
    let new_user_owes = create_new_user("test_balances_owes".to_owned());

    let payer = client.register_user(new_user_payer.clone()).await?.user;
    let owes = client.register_user(new_user_owes.clone()).await?.user;

    let auth_payer = client
        .login_user(payer.username, PASSWORD.to_owned())
        .await?;

    // Payer pays 300 cents for both users (150 each)
    // payer: 300, owes: 0 -> after split: payer +150, owes -150
    let _expense = auth_payer
        .create_expense(
            payer.id,
            vec![payer.id, owes.id],
            300,
            Some("Dinner".to_string()),
            None,
        )
        .await?;

    // Get balances for the payer
    let balances = auth_payer.get_user_balances(payer.id).await?;
    dbg!(&balances);

    // Payer should have one balance entry for the other user
    assert_eq!(balances.len(), 1);
    assert_eq!(balances[0].other, owes.id);
    // Payer paid 300 for 2 people, so owes owes 150
    assert_eq!(balances[0].amount, 150);

    // Get balances for the owes user
    let auth_owes = client
        .login_user(owes.username, PASSWORD.to_owned())
        .await?;
    let balances_owes = auth_owes.get_user_balances(owes.id).await?;
    dbg!(&balances_owes);

    // Owes should have one balance entry for the payer (negative)
    assert_eq!(balances_owes.len(), 1);
    assert_eq!(balances_owes[0].other, payer.id);
    // From owes perspective, they owe payer 150 (negative amount)
    assert_eq!(balances_owes[0].amount, -150);

    // Cleanup
    auth_payer.delete_user().await?;
    auth_owes.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_user_balances_multiple_expenses() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user_payer = create_new_user("test_balances_multi_payer".to_owned());
    let new_user_friend1 = create_new_user("test_balances_multi_friend1".to_owned());
    let new_user_friend2 = create_new_user("test_balances_multi_friend2".to_owned());

    let payer = client.register_user(new_user_payer.clone()).await?.user;
    let friend1 = client.register_user(new_user_friend1.clone()).await?.user;
    let friend2 = client.register_user(new_user_friend2.clone()).await?.user;

    let auth_payer = client
        .login_user(payer.username, PASSWORD.to_owned())
        .await?;

    // Expense 1: Payer pays 300 for all three (100 each)
    let _expense1 = auth_payer
        .create_expense(
            payer.id,
            vec![payer.id, friend1.id, friend2.id],
            300,
            Some("Dinner".to_string()),
            None,
        )
        .await?;

    // Expense 2: Friend1 pays 200 for all three
    let auth_friend1 = client
        .login_user(friend1.username, PASSWORD.to_owned())
        .await?;
    let _expense2 = auth_friend1
        .create_expense(
            friend1.id,
            vec![payer.id, friend1.id, friend2.id],
            200,
            Some("Coffee".to_string()),
            None,
        )
        .await?;

    // Get balances for the payer
    let balances = auth_payer.get_user_balances(payer.id).await?;
    dbg!(&balances);

    // Payer should have balance entries for both friends
    // From expense1: friend1 owes payer 100, friend2 owes payer 100
    // From expense2: payer owes friend1 (200/3 = 66 each, but integer division: 66, 67, 67)
    // Net: friend1: 100 - 67 = 33 (owes payer 33)
    //      friend2: 100 - 67 = 33 (owes payer 33)

    // Check that we have entries for both friends
    let balance_map: std::collections::HashMap<_, _> =
        balances.iter().map(|b| (b.other, b.amount)).collect();

    assert!(balance_map.contains_key(&friend1.id));
    assert!(balance_map.contains_key(&friend2.id));

    // Verify net amounts (300/3 = 100 from expense1, 200/3 = 66 or 67 from expense2)
    // Payer: +100 from friend1 (expense1) -67 to friend1 (expense2) = +33 from friend1
    // Payer: +100 from friend2 (expense1) -67 to friend2 (expense2) = +33 from friend2
    let friend1_balance = balance_map.get(&friend1.id).unwrap();
    let friend2_balance = balance_map.get(&friend2.id).unwrap();

    // Both should be positive (friends owe payer)
    assert!(*friend1_balance > 0);
    assert!(*friend2_balance > 0);

    // Cleanup
    auth_payer.delete_user().await?;
    auth_friend1.delete_user().await?;
    let auth_friend2 = client
        .login_user(friend2.username, PASSWORD.to_owned())
        .await?;
    auth_friend2.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_group_balances_basic() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user_owner = create_new_user("test_group_balances_owner".to_owned());
    let new_user_member1 = create_new_user("test_group_balances_member1".to_owned());
    let new_user_member2 = create_new_user("test_group_balances_member2".to_owned());

    let owner = client.register_user(new_user_owner.clone()).await?.user;
    let member1 = client.register_user(new_user_member1.clone()).await?.user;
    let member2 = client.register_user(new_user_member2.clone()).await?.user;

    let auth_owner = client
        .login_user(owner.username, PASSWORD.to_owned())
        .await?;

    // Create a group
    let group = auth_owner.create_group("test_group_balances").await?;

    // Add members to the group
    auth_owner.add_user_to_group(group.id, member1.id).await?;
    auth_owner.add_user_to_group(group.id, member2.id).await?;

    // Owner pays 300 for all three members
    let _expense1 = auth_owner
        .create_expense(
            owner.id,
            vec![owner.id, member1.id, member2.id],
            300,
            Some("Group expense 1".to_string()),
            Some(group.id),
        )
        .await?;

    // Get group balances
    let group_balances = auth_owner.get_group_balances(group.id).await?;
    dbg!(&group_balances);

    // Should have 2 balance entries (member1 -> owner, member2 -> owner)
    assert_eq!(group_balances.len(), 2);

    // Each member should owe owner 100
    for balance in group_balances {
        assert_eq!(balance.amount, 100);
        assert!(balance.from != owner.id);
        assert_eq!(balance.to, owner.id);
    }

    // Cleanup
    auth_owner.delete_group(group.id).await?;

    let auth_member1 = client
        .login_user(member1.username, PASSWORD.to_owned())
        .await?;
    auth_member1.delete_user().await?;

    let auth_member2 = client
        .login_user(member2.username, PASSWORD.to_owned())
        .await?;
    auth_member2.delete_user().await?;

    auth_owner.delete_user().await?;
    Ok(())
}

#[tokio::test]
async fn test_group_balances_with_multiple_expenses() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let new_user_payer1 = create_new_user("test_group_multi_payer1".to_owned());
    let new_user_payer2 = create_new_user("test_group_multi_payer2".to_owned());
    let new_user_member = create_new_user("test_group_multi_member".to_owned());

    let payer1 = client.register_user(new_user_payer1.clone()).await?.user;
    let payer2 = client.register_user(new_user_payer2.clone()).await?.user;
    let member = client.register_user(new_user_member.clone()).await?.user;

    let auth_payer1 = client
        .login_user(payer1.username, PASSWORD.to_owned())
        .await?;

    // Create a group
    let group = auth_payer1
        .create_group("test_group_multi_balances")
        .await?;

    // Add members to the group
    auth_payer1.add_user_to_group(group.id, payer2.id).await?;
    auth_payer1.add_user_to_group(group.id, member.id).await?;

    // Payer1 pays 300 for all three
    let _expense1 = auth_payer1
        .create_expense(
            payer1.id,
            vec![payer1.id, payer2.id, member.id],
            300,
            Some("Group expense 1".to_string()),
            Some(group.id),
        )
        .await?;

    // Payer2 pays 200 for all three
    let auth_payer2 = client
        .login_user(payer2.username, PASSWORD.to_owned())
        .await?;
    let _expense2 = auth_payer2
        .create_expense(
            payer2.id,
            vec![payer1.id, payer2.id, member.id],
            200,
            Some("Group expense 2".to_string()),
            Some(group.id),
        )
        .await?;

    // Get group balances - should show net settlement
    let group_balances = auth_payer1.get_group_balances(group.id).await?;
    dbg!(&group_balances);

    // With two expenses:
    // Expense1: payer1 paid 300, each owes 100 -> payer1: +300, payer2: -100, member: -100
    // Expense2: payer2 paid 200, each owes ~67 -> payer2: +200, payer1: -67, member: -67
    // Net: payer1: +300-67=+233, payer2: -100+200-67=+33, member: -100-67=-167
    // Settlement: member owes payer1 167, payer2 owes payer1 33
    // Or: member -> payer1: 167, payer2 -> payer1: 33

    // The balance should show the minimal settlement transfers
    // There should be transfers that settle all debts
    assert!(!group_balances.is_empty());

    // Verify total sum of amounts equals zero (all debts cancel out)
    let total: i64 = group_balances.iter().map(|b| b.amount).sum();
    assert_eq!(total, 0);

    // Cleanup
    auth_payer1.delete_group(group.id).await?;
    auth_payer1.delete_user().await?;
    auth_payer2.delete_user().await?;
    let auth_member = client
        .login_user(member.username, PASSWORD.to_owned())
        .await?;
    auth_member.delete_user().await?;
    Ok(())
}

// ====== Expense Testing User ======

// ====== Expense Testing Group ======
