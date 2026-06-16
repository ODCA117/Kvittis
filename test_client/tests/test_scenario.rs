use common::NewUser;
use rust_kvittis_client::UnauthClient;

const BASE_URL: &str = "http://localhost:3000";
const PASSWORD: &str = "secret_password";

fn create_new_user(username: String) -> NewUser {
    let email = username.clone() + "@gmail.com";
    NewUser {
        username,
        email,
        password: PASSWORD.to_owned(),
    }
}

// ────────────────────────────────────────────────────────────────────────────
// TC1 — ListForUser returns payer + participant expenses
// ────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn tc1_list_for_user_includes_payer_and_participant() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;

    // Unique names to avoid clashes with parallel tests
    let tc1_user_1 = create_new_user("tc1_user_1".to_owned());
    let tc1_user_2 = create_new_user("tc1_user_2".to_owned());
    let tc1_user_3 = create_new_user("tc1_user_3".to_owned());
    let a = client.register_user(tc1_user_1).await?.user;
    let b = client.register_user(tc1_user_2).await?.user;
    let c = client.register_user(tc1_user_3).await?.user;
    let auth_a = client.login_user(a.username, PASSWORD.to_owned()).await?;

    // E1: payer A, participants [A, B], no group
    let e1 = auth_a
        .create_expense(a.id, vec![a.id, b.id], 100, None, None)
        .await?;
    // E2: payer C, participants [A, C], no group
    let e2 = auth_a
        .create_expense(c.id, vec![a.id, c.id], 200, None, None)
        .await?;

    // ListForUser(A) must contain both E1 and E2
    let expenses = auth_a.list_expenses_for_user(a.id).await?;
    let ids: Vec<_> = expenses.iter().map(|e| e.id).collect();
    assert!(ids.contains(&e1.id), "E1 should appear (A is payer)");
    assert!(ids.contains(&e2.id), "E2 should appear (A is participant)");

    // Cleanup
    auth_a.delete_expense(e1.id).await?;
    auth_a.delete_expense(e2.id).await?;
    auth_a.delete_user().await?;
    let auth_b = client.login_user(b.username, PASSWORD.to_owned()).await?;
    auth_b.delete_user().await?;
    let auth_c = client.login_user(c.username, PASSWORD.to_owned()).await?;
    auth_c.delete_user().await?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// TC2 — User balances (non-group) basic split
// ────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn tc2_user_balances_basic_split() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let tc2_user_1 = create_new_user("tc2_user_1".to_owned());
    let tc2_user_2 = create_new_user("tc2_user_2".to_owned());

    let a = client.register_user(tc2_user_1).await?.user;
    let b = client.register_user(tc2_user_2).await?.user;
    let auth_a = client.login_user(a.username, PASSWORD.to_owned()).await?;

    // E1: payer A, participants [A, B], amount 100 → B owes A 50
    let e1 = auth_a
        .create_expense(a.id, vec![a.id, b.id], 100, None, None)
        .await?;

    // UserBalances(A): entry for B with amount +50
    let balances_a = auth_a.get_user_balances(a.id).await?;
    let entry_a = balances_a
        .iter()
        .find(|e| e.other == b.id)
        .expect("A should have a balance entry for B");
    assert_eq!(entry_a.amount, 50, "B owes A 50");

    // UserBalances(B): entry for A with amount -50
    let balances_b = auth_a.get_user_balances(b.id).await?;
    let entry_b = balances_b
        .iter()
        .find(|e| e.other == a.id)
        .expect("B should have a balance entry for A");
    assert_eq!(entry_b.amount, -50, "B owes A 50");

    // Cleanup
    auth_a.delete_expense(e1.id).await?;
    auth_a.delete_user().await?;
    let auth_b = client.login_user(b.username, PASSWORD.to_owned()).await?;
    auth_b.delete_user().await?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// TC3 — User balances ignore group expenses
// ────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn tc3_user_balances_ignore_group_expenses() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let tc3_user_1 = create_new_user("tc3_user_1".to_owned());
    let tc3_user_2 = create_new_user("tc3_user_2".to_owned());

    let a = client.register_user(tc3_user_1).await?.user;
    let b = client.register_user(tc3_user_2).await?.user;
    let auth_a = client.login_user(a.username, PASSWORD.to_owned()).await?;
    let g = auth_a
        .create_group("tc3_group_g")
        .await?;

    // E1: non-group expense, payer A, participants [A, B], amount 100
    let e1 = auth_a
        .create_expense(a.id, vec![a.id, b.id], 100, None, None)
        .await?;
    // E2: group expense, same split — should NOT affect UserBalances
    let e2 = auth_a
        .create_expense(a.id, vec![a.id, b.id], 100, None, Some(g.id))
        .await?;

    // UserBalances(A) vs B should only reflect E1 → net +50
    let balances = auth_a.get_user_balances(a.id).await?;
    let entry = balances
        .iter()
        .find(|e| e.other == b.id)
        .expect("A should have a balance entry for B");
    assert_eq!(
        entry.amount, 50,
        "Only E1 should count; net should be +50 not +100"
    );

    // Cleanup
    auth_a.delete_expense(e1.id).await?;
    auth_a.delete_expense(e2.id).await?;
    auth_a.delete_group(g.id).await?;
    auth_a.delete_user().await?;
    let auth_b = client.login_user(b.username, PASSWORD.to_owned()).await?;
    auth_b.delete_user().await?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// TC4 — Group overview includes all expenses (even if caller not involved)
// ────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn tc4_group_overview_includes_all_expenses() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;

    let tc4_user_1 = create_new_user("tc4_user_1".to_owned());
    let tc4_user_2 = create_new_user("tc4_user_2".to_owned());
    let tc4_user_3 = create_new_user("tc4_user_3".to_owned());

    let a = client.register_user(tc4_user_1).await?.user;
    let b = client.register_user(tc4_user_2).await?.user;
    let c = client.register_user(tc4_user_3).await?.user;
    let auth_a = client.login_user(a.username, PASSWORD.to_owned()).await?;
    let g = auth_a
        .create_group("tc4_group_g")
        .await?;

    // E1: payer A, participants [A, B], amount 300 → A net +150, B net -150
    let e1 = auth_a
        .create_expense(a.id, vec![a.id, b.id], 300, None, Some(g.id))
        .await?;
    // E2: payer B, participants [B, C], amount 300 → B net +150, C net -150
    // Combined: A=+150, B=0, C=-150
    let e2 = auth_a
        .create_expense(b.id, vec![b.id, c.id], 300, None, Some(g.id))
        .await?;

    // GroupBalance should contain a transfer: C -> A of 150
    let transfers = auth_a.get_group_balances(g.id).await?;
    assert_eq!(
        transfers.len(),
        1,
        "Should have exactly one settlement transfer"
    );
    let t = &transfers[0];
    assert_eq!(t.from, c.id, "C is the debtor");
    assert_eq!(t.to, a.id, "A is the creditor");
    assert_eq!(t.amount, 150, "C pays A 150");

    // Cleanup
    auth_a.delete_expense(e1.id).await?;
    auth_a.delete_expense(e2.id).await?;
    auth_a.delete_group(g.id).await?;
    auth_a.delete_user().await?;
    let auth_b = client.login_user(b.username, PASSWORD.to_owned()).await?;
    auth_b.delete_user().await?;
    let auth_c = client.login_user(c.username, PASSWORD.to_owned()).await?;
    auth_c.delete_user().await?;
    Ok(())
}

// ────────────────────────────────────────────────────────────────────────────
// TC5 — Deterministic remainder split
// ────────────────────────────────────────────────────────────────────────────
#[tokio::test]
async fn tc5_deterministic_remainder_split() -> anyhow::Result<()> {
    let client = UnauthClient::new(BASE_URL)?;
    let tc5_user_1 = create_new_user("tc5_user_1".to_owned());
    let tc5_user_2 = create_new_user("tc5_user_2".to_owned());
    let tc5_user_3 = create_new_user("tc5_user_3".to_owned());

    let a = client.register_user(tc5_user_1).await?.user;
    let b = client.register_user(tc5_user_2).await?.user;
    let c = client.register_user(tc5_user_3).await?.user;
    let auth_a = client.login_user(a.username, PASSWORD.to_owned()).await?;

    // amount=100, 3 participants → base=33, rem=1
    // Sorted by UserId, first participant gets 34, others get 33. Sum = 34+33+33 = 100.
    let e1 = auth_a
        .create_expense(a.id, vec![a.id, b.id, c.id], 100, None, None)
        .await?;

    // Fetch balances twice and verify they are identical (deterministic)
    let balances_first = auth_a.get_user_balances(a.id).await?;
    let balances_second = auth_a.get_user_balances(a.id).await?;

    // Shares for each non-payer must be 33 or 34 (sum to 100 - payer's own share)
    // A paid 100 and gets back shares from B and C; total B+C owe A = 66 or 67.
    // The net for A vs B and A vs C combined must equal 100 - share(A).
    let net_a: i64 = balances_first.iter().map(|e| e.amount).sum();
    let net_a_again: i64 = balances_second.iter().map(|e| e.amount).sum();
    assert_eq!(
        net_a, net_a_again,
        "Balances must be deterministic across calls"
    );

    // The total B+C owe A is amount - share(A): either 66 or 67 depending on sort order.
    // In any case it must be exactly (100 - floor(100/3)) or (100 - ceil(100/3)).
    assert!(
        net_a == 66 || net_a == 67,
        "A's net balance should be 66 or 67 (100 minus A's share), got {net_a}"
    );

    // Cleanup
    auth_a.delete_expense(e1.id).await?;
    auth_a.delete_user().await?;
    let auth_b = client.login_user(b.username, PASSWORD.to_owned()).await?;
    auth_b.delete_user().await?;
    let auth_c = client.login_user(c.username, PASSWORD.to_owned()).await?;
    auth_c.delete_user().await?;
    Ok(())
}
