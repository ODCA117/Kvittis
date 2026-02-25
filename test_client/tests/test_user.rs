
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

// ────────────────────────────────────────────────────────────────────────────
// TC1 — ListForUser returns payer + participant expenses
// ────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn tc1_list_for_user_includes_payer_and_participant() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;

    // Unique names to avoid clashes with parallel tests
    let a = client.register_user("tc1_user_a").await?;
    let b = client.register_user("tc1_user_b").await?;
    let c = client.register_user("tc1_user_c").await?;

    // E1: payer A, participants [A, B], no group
    let e1 = client.create_expense(a.id, vec![a.id, b.id], 100, None, None).await?;
    // E2: payer C, participants [A, C], no group
    let e2 = client.create_expense(c.id, vec![a.id, c.id], 200, None, None).await?;

    // ListForUser(A) must contain both E1 and E2
    let expenses = client.list_expenses_for_user(a.id).await?;
    let ids: Vec<_> = expenses.iter().map(|e| e.id).collect();
    assert!(ids.contains(&e1.id), "E1 should appear (A is payer)");
    assert!(ids.contains(&e2.id), "E2 should appear (A is participant)");

    // Cleanup
    client.delete_expense(e1.id).await?;
    client.delete_expense(e2.id).await?;
    client.delete_user(a.id).await?;
    client.delete_user(b.id).await?;
    client.delete_user(c.id).await?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// TC2 — User balances (non-group) basic split
// ────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn tc2_user_balances_basic_split() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;

    let a = client.register_user("tc2_user_a").await?;
    let b = client.register_user("tc2_user_b").await?;

    // E1: payer A, participants [A, B], amount 100 → B owes A 50
    let e1 = client.create_expense(a.id, vec![a.id, b.id], 100, None, None).await?;

    // UserBalances(A): entry for B with amount +50
    let balances_a = client.get_user_balances(a.id).await?;
    let entry_a = balances_a.iter().find(|e| e.other == b.id)
        .expect("A should have a balance entry for B");
    assert_eq!(entry_a.amount, 50, "B owes A 50");

    // UserBalances(B): entry for A with amount -50
    let balances_b = client.get_user_balances(b.id).await?;
    let entry_b = balances_b.iter().find(|e| e.other == a.id)
        .expect("B should have a balance entry for A");
    assert_eq!(entry_b.amount, -50, "B owes A 50");

    // Cleanup
    client.delete_expense(e1.id).await?;
    client.delete_user(a.id).await?;
    client.delete_user(b.id).await?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// TC3 — User balances ignore group expenses
// ────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn tc3_user_balances_ignore_group_expenses() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;

    let a = client.register_user("tc3_user_a").await?;
    let b = client.register_user("tc3_user_b").await?;
    let g = client.create_group("tc3_group_g", a.id, vec![a.id, b.id]).await?;

    // E1: non-group expense, payer A, participants [A, B], amount 100
    let e1 = client.create_expense(a.id, vec![a.id, b.id], 100, None, None).await?;
    // E2: group expense, same split — should NOT affect UserBalances
    let e2 = client.create_expense(a.id, vec![a.id, b.id], 100, None, Some(g.id)).await?;

    // UserBalances(A) vs B should only reflect E1 → net +50
    let balances = client.get_user_balances(a.id).await?;
    let entry = balances.iter().find(|e| e.other == b.id)
        .expect("A should have a balance entry for B");
    assert_eq!(entry.amount, 50, "Only E1 should count; net should be +50 not +100");

    // Cleanup
    client.delete_expense(e1.id).await?;
    client.delete_expense(e2.id).await?;
    client.delete_group(g.id).await?;
    client.delete_user(a.id).await?;
    client.delete_user(b.id).await?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// TC4 — Group overview includes all expenses (even if caller not involved)
// ────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn tc4_group_overview_includes_all_expenses() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;

    let a = client.register_user("tc4_user_a").await?;
    let b = client.register_user("tc4_user_b").await?;
    let c = client.register_user("tc4_user_c").await?;
    let g = client.create_group("tc4_group_g", a.id, vec![a.id, b.id, c.id]).await?;

    // E1: payer A, participants [A, B], amount 300 → A net +150, B net -150
    let e1 = client.create_expense(a.id, vec![a.id, b.id], 300, None, Some(g.id)).await?;
    // E2: payer B, participants [B, C], amount 300 → B net +150, C net -150
    // Combined: A=+150, B=0, C=-150
    let e2 = client.create_expense(b.id, vec![b.id, c.id], 300, None, Some(g.id)).await?;

    // GroupBalance should contain a transfer: C -> A of 150
    let transfers = client.get_group_balances(g.id).await?;
    assert_eq!(transfers.len(), 1, "Should have exactly one settlement transfer");
    let t = &transfers[0];
    assert_eq!(t.from, c.id, "C is the debtor");
    assert_eq!(t.to, a.id, "A is the creditor");
    assert_eq!(t.amount, 150, "C pays A 150");

    // Cleanup
    client.delete_expense(e1.id).await?;
    client.delete_expense(e2.id).await?;
    client.delete_group(g.id).await?;
    client.delete_user(a.id).await?;
    client.delete_user(b.id).await?;
    client.delete_user(c.id).await?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// TC5 — Deterministic remainder split
// ────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn tc5_deterministic_remainder_split() -> anyhow::Result<()> {
    let client = KvittisClient::new(BASE_URL)?;

    let a = client.register_user("tc5_user_a").await?;
    let b = client.register_user("tc5_user_b").await?;
    let c = client.register_user("tc5_user_c").await?;

    // amount=100, 3 participants → base=33, rem=1
    // Sorted by UserId, first participant gets 34, others get 33. Sum = 34+33+33 = 100.
    let e1 = client.create_expense(
        a.id,
        vec![a.id, b.id, c.id],
        100,
        None,
        None,
    ).await?;

    // Fetch balances twice and verify they are identical (deterministic)
    let balances_first = client.get_user_balances(a.id).await?;
    let balances_second = client.get_user_balances(a.id).await?;

    // Shares for each non-payer must be 33 or 34 (sum to 100 - payer's own share)
    // A paid 100 and gets back shares from B and C; total B+C owe A = 66 or 67.
    // The net for A vs B and A vs C combined must equal 100 - share(A).
    let net_a: i64 = balances_first.iter().map(|e| e.amount).sum();
    let net_a_again: i64 = balances_second.iter().map(|e| e.amount).sum();
    assert_eq!(net_a, net_a_again, "Balances must be deterministic across calls");

    // The total B+C owe A is amount - share(A): either 66 or 67 depending on sort order.
    // In any case it must be exactly (100 - floor(100/3)) or (100 - ceil(100/3)).
    assert!(
        net_a == 66 || net_a == 67,
        "A's net balance should be 66 or 67 (100 minus A's share), got {net_a}"
    );

    // Cleanup
    client.delete_expense(e1.id).await?;
    client.delete_user(a.id).await?;
    client.delete_user(b.id).await?;
    client.delete_user(c.id).await?;
    Ok(())
}