use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tracing_subscriber;
use uuid::Uuid;

type UserId = Uuid;
type GroupId = Uuid;
type ExpenseId = Uuid;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct User {
    id: UserId,
    username: String,
    friends: Vec<UserId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Group {
    id: GroupId,
    name: String,
    owner_id: UserId,
    members: Vec<UserId>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Expense {
    id: ExpenseId,
    payer: UserId,
    participants: Vec<UserId>,
    amount: f64,
    description: Option<String>,
    group_id: Option<GroupId>,
    timestamp_ms: i64,
}

#[derive(Serialize, Deserialize, Default)]
struct AppStateData {
    users: HashMap<UserId, User>,
    groups: HashMap<GroupId, Group>,
    expenses: Vec<Expense>,
}

#[derive(Clone)]
struct AppState {
    data: Arc<RwLock<AppStateData>>,
    persist_path: Option<String>,
}

#[derive(Deserialize)]
struct RegisterRequest {
    username: String,
}

#[derive(Serialize)]
struct RegisterResponse {
    id: UserId,
    username: String,
}

#[derive(Deserialize)]
struct FriendRequest {
    user_id: UserId,
    friend_id: UserId,
}

#[derive(Deserialize)]
struct CreateGroupRequest {
    name: String,
    owner_id: UserId,
    members: Vec<UserId>,
}

#[derive(Serialize)]
struct CreateGroupResponse {
    id: GroupId,
    name: String,
}

#[derive(Deserialize)]
struct CreateExpenseRequest {
    payer: UserId,
    participants: Vec<UserId>,
    amount: f64,
    description: Option<String>,
    group_id: Option<GroupId>,
}

#[derive(Serialize)]
struct ExpenseResponse {
    id: ExpenseId,
}

#[derive(Serialize)]
struct BalanceEntry {
    other: UserId,
    amount: f64,
}

#[derive(Serialize)]
struct GroupBalance {
    from: UserId,
    to: UserId,
    amount: f64,
}

#[derive(Serialize)]
#[serde(untagged)]
enum ApiResponse<T> {
    Success(T),
    Error { message: String },
}

// Helper function for error responses (generic over T)
fn json_error<T>(status: StatusCode, message: &str) -> (StatusCode, Json<ApiResponse<T>>) {
    (status, Json(ApiResponse::Error {
        message: message.to_string(),
    }))
}

// Helper function for success responses
fn json_success<T>(status: StatusCode, data: T) -> (StatusCode, Json<ApiResponse<T>>) {
    (status, Json(ApiResponse::Success(data)))
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<RegisterRequest>,
) -> (StatusCode, Json<ApiResponse<RegisterResponse>>) {
    let id = Uuid::new_v4();
    let user = User {
        id,
        username: payload.username.clone(),
        friends: vec![],
    };

    state.data.write().users.insert(id, user);
    if let Some(path) = &state.persist_path {
        let _ = save_state_to_file(&state, path);
    }

    json_success(
        StatusCode::CREATED,
        RegisterResponse {
            id,
            username: payload.username,
        }
    )
}

async fn add_friend(
    State(state): State<AppState>,
    Json(payload): Json<FriendRequest>,
) -> (StatusCode, Json<ApiResponse<serde_json::Value>>) {
    let mut data = state.data.write();
    if !data.users.contains_key(&payload.user_id) {
        return json_error(StatusCode::BAD_REQUEST, "user_id not found");
    }
    if !data.users.contains_key(&payload.friend_id) {
        return json_error(StatusCode::BAD_REQUEST, "friend_id not found");
    }

    let u = data.users.get_mut(&payload.user_id).unwrap();
    if !u.friends.contains(&payload.friend_id) {
        u.friends.push(payload.friend_id);
    }
    let v = data.users.get_mut(&payload.friend_id).unwrap();
    if !v.friends.contains(&payload.user_id) {
        v.friends.push(payload.user_id);
    }

    drop(data);
    if let Some(path) = &state.persist_path {
        let _ = save_state_to_file(&state, path);
    }

    json_success(StatusCode::OK, serde_json::json!({"ok": true}))
}

async fn create_group(
    State(state): State<AppState>,
    Json(payload): Json<CreateGroupRequest>,
) -> (StatusCode, Json<ApiResponse<CreateGroupResponse>>) {
    let mut data = state.data.write();

    if !data.users.contains_key(&payload.owner_id) {
        return json_error(StatusCode::BAD_REQUEST, "owner_id not found");
    }
    for m in &payload.members {
        if !data.users.contains_key(m) {
            return json_error(StatusCode::BAD_REQUEST, "one or more members not found");
        }
    }

    let id = Uuid::new_v4();
    let group = Group {
        id,
        name: payload.name.clone(),
        owner_id: payload.owner_id,
        members: payload.members.clone(),
    };
    data.groups.insert(id, group);

    drop(data);
    if let Some(path) = &state.persist_path {
        let _ = save_state_to_file(&state, path);
    }

    json_success(
        StatusCode::CREATED, 
        CreateGroupResponse { 
            id, 
            name: payload.name 
        }
    )
}

async fn create_expense(
    State(state): State<AppState>,
    Json(payload): Json<CreateExpenseRequest>,
) -> (StatusCode, Json<ApiResponse<ExpenseResponse>>) {
    let mut data = state.data.write();

    if !data.users.contains_key(&payload.payer) {
        return json_error(StatusCode::BAD_REQUEST, "payer not found");
    }
    for p in &payload.participants {
        if !data.users.contains_key(p) {
            return json_error(StatusCode::BAD_REQUEST, "one or more participants not found");
        }
    }
    if let Some(gid) = payload.group_id {
        if !data.groups.contains_key(&gid) {
            return json_error(StatusCode::BAD_REQUEST, "group_id not found");
        }
    }
    if payload.participants.is_empty() {
        return json_error(StatusCode::BAD_REQUEST, "participants cannot be empty");
    }
    if payload.amount <= 0.0 {
        return json_error(StatusCode::BAD_REQUEST, "amount must be > 0");
    }

    let id = Uuid::new_v4();
    let expense = Expense {
        id,
        payer: payload.payer,
        participants: payload.participants.clone(),
        amount: payload.amount,
        description: payload.description.clone(),
        group_id: payload.group_id,
        timestamp_ms: chrono::Utc::now().timestamp_millis(),
    };

    data.expenses.push(expense);

    drop(data);
    if let Some(path) = &state.persist_path {
        let _ = save_state_to_file(&state, path);
    }

    json_success(StatusCode::CREATED, ExpenseResponse { id })
}

async fn get_user_balances(
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Vec<BalanceEntry>>>) {
    let data = state.data.read();
    let user_id = user_id;
    if !data.users.contains_key(&user_id) {
        return json_error(StatusCode::NOT_FOUND, "user not found");
    }

    let debts = compute_debts(&data.expenses, None);

    let mut map: HashMap<Uuid, f64> = HashMap::new();
    for (&(from, to), amt) in debts.iter() {
        if to == user_id {
            *map.entry(from).or_default() += *amt;
        }
        if from == user_id {
            *map.entry(to).or_default() -= *amt;
        }
    }

    let mut out: Vec<BalanceEntry> = map
        .into_iter()
        .map(|(other, amount)| BalanceEntry { other, amount })
        .collect();

    out.sort_by(|a, b| b.amount.partial_cmp(&a.amount).unwrap_or(std::cmp::Ordering::Equal));

    json_success(StatusCode::OK, out)
}

async fn get_group_balances(
    State(state): State<AppState>,
    Path(group_id): Path<Uuid>,
) -> (StatusCode, Json<ApiResponse<Vec<GroupBalance>>>) {
    let data = state.data.read();
    let group_id = group_id;
    if !data.groups.contains_key(&group_id) {
        return json_error(StatusCode::NOT_FOUND, "group not found");
    }

    let debts = compute_debts(&data.expenses, Some(group_id));

    let mut pairs: HashMap<(Uuid, Uuid), f64> = HashMap::new();
    for (&(from, to), amt) in debts.iter() {
        let current = pairs.entry((from, to)).or_default();
        *current += *amt;
    }

    let mut net_pairs: Vec<GroupBalance> = Vec::new();
    let mut seen: HashMap<(Uuid, Uuid), bool> = HashMap::new();
    for (&(a, b), &amt) in pairs.iter() {
        if seen.get(&(a, b)).is_some() {
            continue;
        }
        let reverse = *pairs.get(&(b, a)).unwrap_or(&0.0);
        if amt >= reverse {
            let net = amt - reverse;
            if net > 0.0 {
                net_pairs.push(GroupBalance {
                    from: a,
                    to: b,
                    amount: net,
                });
            }
        } else {
            let net = reverse - amt;
            if net > 0.0 {
                net_pairs.push(GroupBalance {
                    from: b,
                    to: a,
                    amount: net,
                });
            }
        }
        seen.insert((a, b), true);
        seen.insert((b, a), true);
    }

    json_success(StatusCode::OK, net_pairs)
}

fn compute_debts(
    expenses: &Vec<Expense>,
    filter_group: Option<GroupId>,
) -> HashMap<(Uuid, Uuid), f64> {
    // map (from, to) => amount (from owes to)
    let mut map: HashMap<(Uuid, Uuid), f64> = HashMap::new();
    for exp in expenses.iter() {
        if let Some(gid) = filter_group {
            if exp.group_id != Some(gid) {
                continue;
            }
        }
        if exp.participants.is_empty() {
            continue;
        }

        let share = exp.amount / (exp.participants.len() as f64);
        for p in exp.participants.iter() {
            if *p == exp.payer {
                continue;
            }
            *map.entry((*p, exp.payer)).or_default() += share;
        }
    }
    map
}

fn save_state_to_file(state: &AppState, path: &str) -> Result<(), String> {
    let data = state.data.read();
    let serialized = serde_json::to_string_pretty(&*data).map_err(|e| e.to_string())?;
    std::fs::write(path, serialized).map_err(|e| e.to_string())?;
    Ok(())
}

fn try_load_state(path: &str) -> Option<AppStateData> {
    match std::fs::read_to_string(path) {
        Ok(s) => serde_json::from_str::<AppStateData>(&s).ok(),
        Err(_) => None,
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // persist file path (optional)
    let persist_path = Some("kvittis_state.json".to_string());

    // attempt load
    let loaded = persist_path
        .as_ref()
        .and_then(|p| try_load_state(p))
        .unwrap_or_default();

    let state = AppState {
        data: Arc::new(RwLock::new(loaded)),
        persist_path,
    };

    let app = Router::new()
        .route("/register", post(register))
        .route("/friend", post(add_friend))
        .route("/group", post(create_group))
        .route("/expense", post(create_expense))
        .route("/balances/{user_id}", get(get_user_balances))
        .route("/group_balances/{group_id}", get(get_group_balances))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    tracing::info!("listening on {}", "0.0.0.0:3000");
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();

}
