# Kvittis

A simple, user-friendly expense splitting app for groups and friends.
Track shared expenses, settle debts, and keep everything fair — all in one place.

---

## Current work
Updating expence handling
- [x] Make it possible to retrieve all expenses in a group
- [x] Make it possible to retrieve expenses in a group since it was settled.
- [ ] Softdeletion of groups
- [ ] expense reduce nbr transactions

## Todo:
- Implement different permission levels (admin, user, group_member)
- Group add balance summary
- Group add easy settle to reduce number of transactions to settle a group balance.
- Add email handler? (otherwise remove email)

## Workspace overview

The project is a Cargo workspace containing three crates:

| Crate | Path | Role |
|---|---|---|
| `common` | `common/` | Shared domain types and API request/response DTOs |
| `server` | `server/` | Axum HTTP server — handles all business logic and persistence |
| `test_client` | `test_client/` | Integration test library and seed binary |

> **Frontend** — planned for a future iteration. See [Client](#client) below.

---

## Architecture

```
┌─────────────┐   HTTP/JSON   ┌──────────────────────────────────────────────┐
│ test_client │ ────────────► │                   server                     │
│  (or UI)    │ ◄──────────── │                                              │
└─────────────┘               │  main.rs  ──►  api.rs  ──►  state.rs         │
                              │                               │              │
                              │                         db/mod.rs (Store)    │
                              │                        /              \      │
                              │              db_sqlite.rs          db_file.rs│
                              └──────────────────────────────────────────────┘
```

### `common/`

Defines the shared types used by both `server` and `test_client`:

- **Domain models** — `User`, `Group`, `Expense` and their ID type aliases (`UserId`, `GroupId`, `ExpenseId`), all backed by `Uuid`.
- **API DTOs** (`common/src/api.rs`) — request and response structs (e.g. `RegisterRequest`, `CreateExpenseRequest`, `GetGroupResponse`) serialized via `serde`.
- **`ApiResponse<T>`** — a `#[serde(untagged)]` enum used by every endpoint: on success it serializes exactly as `T`; on error as `{ "message": "…" }`.

### `server/`

An [Axum](https://github.com/tokio-rs/axum) HTTP server with the following module layout:

| Module | File | Responsibility |
|---|---|---|
| Entry point | `main.rs` | Parses CLI args, selects DB backend, wires Axum routes, starts listener |
| CLI | `cli.rs` | `clap`-based args: `--port`, `--ip`, `--db-type`, `--data-dir` |
| Logger | `logger.rs` | `tracing-subscriber` init; log level controlled via `RUST_LOG` |
| API handlers | `api.rs` | One async fn per route; extracts JSON/path params, calls `AppState`, returns `ApiResponse<T>` |
| Service layer | `state.rs` | `AppState` wraps the `Store` behind `Arc<RwLock<_>>`; generates UUIDs, timestamps, and enforces business rules |
| Storage trait | `db/mod.rs` | `Store` trait + row types (`UserRow`, `GroupRow`, `ExpenseRow`) |
| SQLite backend | `db/db_sqlite.rs` | `sqlx` connection pool; `sqlx::migrate!` runs `migrations/` on startup |
| File backend | `db/db_file.rs` | JSON file store; atomic write-on-mutate via tmp-file rename |
| Migrations | `migrations/` | Plain SQL schema; currently one file: `20250101_initial_schema.sql` |

#### Request flow

```
HTTP request
    │
    ▼
Axum router  (main.rs — route table)
    │
    ▼
Handler fn  (api.rs — extracts State<AppState>, Path<…>, Json<…>)
    │
    ▼
AppState method  (state.rs — builds domain objects, generates UUID/timestamp)
    │
    ▼
Store trait method  (db/mod.rs)
    │
    ├──► SqliteStore  (db/db_sqlite.rs — sqlx queries, FK-aware inserts/deletes)
    └──► FileStore    (db/db_file.rs  — BTreeMap in memory, persisted as JSON)
    │
    ▼
Result<T, anyhow::Error>
    │
    ▼
Handler maps Ok → ApiResponse::Success / Err → ApiResponse::Error
    │
    ▼
JSON response
```

#### Persistence backends

Two backends are selectable at startup via `--db-type`:

| Flag value | Backend | Notes |
|---|---|---|
| `sql` | `SqliteStore` | Recommended. Schema applied automatically via `sqlx::migrate!`. DB file at `<data-dir>/store.db`. |
| `file` | `FileStore` | Simple JSON file at `<data-dir>/store.json`. Good for quick experiments; current CLI default. |

> **Note:** SQLite (`--db-type sql`) is the intended long-term default. The CLI default will be switched to `sql` in a future change.

#### API endpoints

| Method | Path | Description |
|---|---|---|
| `POST` | `/register` | Register a new user |
| `GET` | `/user/{user_id}` | Get a user by ID |
| `DELETE` | `/user/{user_id}` | Delete a user |
| `GET` | `/users` | List all users |
| `POST` | `/search_user` | Search users by username substring |
| `POST` | `/friend` | Add a friend relationship |
| `POST` | `/create_group` | Create a group |
| `GET` | `/group/{group_id}` | Get a group by ID |
| `DELETE` | `/group/{group_id}` | Delete a group |
| `POST` | `/search_group` | Search groups by name substring |
| `POST` | `/new_group_member` | Add a member to a group |
| `POST` | `/create_expense` | Create an expense |
| `POST` | `/get_expense` | Get an expense by ID |
| `POST` | `/delete_expense` | Delete an expense |
| `GET` | `/balances/{user_id}` | *(not yet implemented)* User balance summary |
| `GET` | `/group_balances/{group_id}` | *(not yet implemented)* Group balance summary |


---

## Running the server

**SQLite (recommended):**

```bash
RUST_LOG=info cargo run -p server -- --db-type sql --data-dir ./data
```

**JSON file store:**

```bash
RUST_LOG=info cargo run -p server -- --db-type file --data-dir ./data
```

The server listens on `127.0.0.1:3000` by default.
Use `--port` and `--ip` to override.

---

## Test client

The `test_client` crate contains:

- **`rust_kvittis_client`** (`test_client/src/kvittis_client.rs`) — a `reqwest`-based async client that wraps every server endpoint.
- **Integration tests** (`test_client/tests/test_user.rs`) — `#[tokio::test]` functions that hit a running server.
- **Seed binary** (`test_client/src/bin.rs`, binary name `test_setup_client`) — registers Alice, Bob, Charlie and creates two groups for manual exploration.

> **Note:** The test client currently hardcodes `http://localhost:3000` as the server base URL.

### How to run the tests

The integration tests require a running server. Use **two terminals**:

**Terminal 1 — start the server:**

```bash
cd /path/to/Kvittis
RUST_LOG=info cargo run -p server -- --db-type sql --data-dir ./data
```

**Terminal 2 — run the tests:**

```bash
cd /path/to/Kvittis
cargo test -p test_client
```

Or from inside the `test_client` directory:

```bash
cd /path/to/Kvittis/test_client
cargo test
```

To run a single test by name:

```bash
cargo test -p test_client test_scenario_group_expense_flow
```

To seed the server with some permanent users and groups (useful for manual exploration):

```bash
cargo run -p test_client --bin test_setup_client
```

### Test coverage

| Test | What it covers |
|---|---|
| `test_register_user` | Register a user; fetch by ID; delete |
| `test_delete_user` | Register; delete; confirm 404-style error on fetch |
| `test_get_users` | Register two users; assert both appear in list; delete |
| `test_search_users` | Register two users; search by prefix; verify inclusion/exclusion |
| `test_create_group` | Register owner; create group; fetch by ID |
| `test_delete_group` | Create group; delete; confirm error on fetch |
| `test_add_user_to_group` | Create group with owner; add a second member; verify membership |
| `test_get_group_by_name` | Create group; search by name; verify included / excluded |
| `test_create_expense` | Create 3 users; create expense; verify fields |
| `test_get_expense` | Create expense; fetch by ID; verify fields |
| `test_delete_expense` | Create expense; delete; verify gone |
| `test_scenario_group_expense_flow` | **End-to-end scenario:** register 3 users → create group → add members → verify membership → create group expense → fetch and verify expense → full cleanup |

### Test client TODO

- [ ] Add `BASE_URL` env-var override so tests can target non-default ports
- [ ] Friend-related scenarios (needs a delete-friend endpoint)
- [ ] Balance verification scenarios (blocked on server implementation)
- [ ] Create `TestScenarios.md` with a written description of each scenario

---

## Client

The frontend client is planned for a future iteration.

### TODO

- [ ] Define architecture
- [ ] Minimized UI
- [ ] Login / authentication
- [ ] Create / Edit / Remove expenses
- [ ] View and list expenses
- [ ] Friends list and add friend
- [ ] Search users
