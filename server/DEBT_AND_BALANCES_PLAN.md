# Debt & Balance Calculation Plan

## Goals (from request)

1. A user can get all the expenses it is related to (either as payer or as participant).
2. A user can get the balances for expenses **not included in a group** (i.e. `group_id = NULL`).
3. A user can get an overview of the balance in a group it is a member of — **all** group expenses should be included in the overview even if the user is not part of a particular expense.

## Current state (repo)

- DB schema (SQLite):
  - `expenses(id, payer_id, amount, description, group_id, timestamp_ms)`
  - `expense_participants(expense_id, user_id)`
  - `groups`, `group_members`
- API shape: POST-only “action enums” exist in `common/src/api.rs`.
  - `ExpenseRequest`: `create`, `get`, `delete`
  - `BalanceRequest`: `user { user_id }`, `group { group_id }`
  - `balance_handler` in `server/src/api.rs` is currently `json_not_implemented()`.
- Store API (`server/src/db/mod.rs`) supports create/get/delete expense but no “list expenses” queries.

## Definitions & rules

### Amount units

- `Expense.amount` is stored as `i64` and denotes **minor units**: **cents/öre** (not dollars/SEK).
- Avoid floats for any balance/debt calculations; keep everything in `i64` minor-units end-to-end.

Stage 1 below updates all API balance types to use `i64` (removing `f64`).

### Expense share / who owes whom

For an expense:

- Inputs: `payer_id`, `participants[]`, `amount`.
- Interpretation: `participants` are the people who share the cost.
  - If payer is included in `participants`, they share equally among all participants.
  - If payer is not included, they do not share (participants owe the payer the full amount split) the cost. However the cost is shared equally among the participants.

Debt edges produced by one expense:

- Let `N = participants.len()` (must be `> 0`).
- Split `amount` into `N` integer shares that sum to `amount`.
- Each participant `p != payer` owes `share(p)` to the payer.
- `payer` owing themselves is ignored.

### Deterministic integer split

When `amount` is not divisible by `N`:

- Compute `base = amount / N`, `rem = amount % N`.
- Sort `participants` deterministically (e.g. by `UserId` bytes / string).
- First `rem` participants get `base + 1`, rest get `base`.

This ensures:

- Shares sum exactly to `amount`.
- Results are stable across runs.

### What “balances” mean

Define a consistent sign convention:

- For **user balances** (non-group expenses): return per-counterparty net balance.
  - `amount > 0` means **other user owes the requested user**.
  - `amount < 0` means **requested user owes the other user**.

For **group balance overview**:

- Compute net position per member:
  - `net(member) = paid_by(member) - owed_share(member)`.
  - `net > 0`: creditor; `net < 0`: debtor.
- Optionally compute settlement transfers (`from -> to`) by matching debtors to creditors.

## API additions / changes

### 1) List expenses related to a user

Add to `common/src/api.rs`:

- `ExpenseRequest::ListForUser { user_id: UserId }`

Response:

- `Vec<GetExpenseResponse>`

Behavior:

- Return all expenses where `payer_id = user_id` OR `user_id` is in `expense_participants`.
- Include both group and non-group expenses (caller can filter client-side by `group_id`).

### 2) List all expenses in a group (needed for goal #3)

Add to `common/src/api.rs`:

- `ExpenseRequest::ListForGroup { group_id: GroupId }`

Response:

- `Vec<GetExpenseResponse>`

Behavior:

- Return all expenses where `expenses.group_id = group_id`.

Membership check:

- There is no auth layer yet. If we still want to enforce “member-only” at API level, we need a way to know the caller.
- Minimal stopgap (optional): add `requester_id: UserId` to the request and verify it exists in `group_members`.

### 3) Non-group user balances

Implement `BalanceRequest::User { user_id }` in `server/src/api.rs`.

Response:

- `Vec<BalanceEntry>` where each entry is `{ other: UserId, amount: <net> }`.

Rule:

- Only consider expenses with `group_id IS NULL`.
- Only include expenses where the user is payer or participant.

Notes:

- If we keep `BalanceEntry.amount: f64` for now, compute in `i64` and convert at the end (e.g. cents to float) — but this is discouraged.

### 4) Group balance overview

Implement `BalanceRequest::Group { group_id }` in `server/src/api.rs`.

Response options (pick one):

A) Settlement transfers only (matches existing `GroupBalance` type):
- `Vec<GroupBalance>` where each is `{ from: UserId, to: UserId, amount }`.

B) Net positions (often nicer for UI) + transfers:
- Introduce a new response wrapper type, e.g.:
  - `GroupBalanceOverview { net: Vec<MemberNet>, transfers: Vec<GroupBalance> }`

Rule:

- Consider **all** expenses in the group (not only those involving the requesting user).
- Membership gating (optional until auth exists): require `requester_id` in request and verify membership.

## Store/DB work

### New `Store` trait methods

Extend `server/src/db/mod.rs`:

- `list_expenses_for_user(user_id) -> Result<Vec<ExpenseRow>>`
- `list_expenses_for_group(group_id) -> Result<Vec<ExpenseRow>>`
- (optional) `is_group_member(group_id, user_id) -> Result<bool>`

### SQLite queries

#### List expenses for user

- Select expenses where:
  - `e.payer_id = $user_id` OR
  - `EXISTS (SELECT 1 FROM expense_participants ep WHERE ep.expense_id = e.id AND ep.user_id = $user_id)`
- Join participants (like `get_expense`) and rebuild `ExpenseRow { participants: Vec<UserId> }` by grouping rows.

#### List expenses for group

- Select expenses where `e.group_id = $group_id`.
- Join participants similarly.

#### Membership check (optional)

- `SELECT EXISTS (SELECT 1 FROM group_members WHERE group_id = $1 AND user_id = $2)`

## Server/state work

Add methods to `server/src/state.rs`:

- `list_expenses_for_user(user_id) -> Result<Vec<Expense>>`
- `list_expenses_for_group(group_id) -> Result<Vec<Expense>>`
- `get_user_non_group_balances(user_id) -> Result<Vec<BalanceEntry>>`
- `get_group_balance_overview(group_id, requester_id?) -> Result<...>`

Then wire them into:

- `server/src/api.rs`:
  - `expense_handler`: handle `ListForUser` and `ListForGroup`
  - `balance_handler`: implement `User` and `Group`

## Validation rules (server-side)

Add lightweight validations at expense creation time (or document as assumptions):

- `participants` must be non-empty.
- No duplicate participant IDs.
- `amount > 0`.
- (If `group_id` is set) participants should be members of the group.

## Test plan (using existing `test_client/` integration style)

Parallelism note:

- Integration tests run in parallel, so each test must use **unique usernames and group names** and must create + clean up its own users/groups/expenses.

Add tests that assert correctness and stability:

1. **ListForUser returns payer and participant expenses**
   - Create expenses where user is payer only, participant only, both.
   - Verify all appear.

2. **User balances ignore group expenses**
   - Create one non-group expense and one group expense involving same users.
   - Verify only non-group affects `BalanceRequest::User`.

3. **Group overview includes all expenses**
   - Create group with 3 users.
   - Create an expense between users 1+2 only.
   - Request group overview as user 3.
   - Verify the expense is included in computations.

4. **Remainder split determinism**
   - Use an amount not divisible by N (e.g. 100 with 3 participants).
   - Verify the same user gets the “+1” remainder consistently.

## Implementation order

## Stage 1 — Make all amounts `i64` (minor units: cents/öre)

Goal: remove floats from the API and make it explicit that all amounts are integer minor units.

Planned changes:

- `common/src/api.rs`
  - Change `BalanceEntry.amount: f64` -> `i64`.
  - Change `GroupBalance.amount: f64` -> `i64`.
  - (Optional but recommended) rename fields/docs to make units explicit (e.g. add doc comments stating “minor units”).

Acceptance criteria:

- Workspace compiles with no remaining `f64` in balance-related types.
- Any client code compiling against the API types is updated accordingly.

## Stage 2 — Add tests + stub handlers (compile & run, return not implemented)

Goal: land the test scaffolding and endpoint shapes first, while server returns a clear “not implemented” error.

Important: in this stage the new tests should **compile** but are **not expected to pass** yet — they should fail at runtime because the server returns “not implemented”. This verifies the API types/contracts are correct before implementing logic.

Planned changes:

- `common/src/api.rs`
  - Add `ExpenseRequest::ListForUser { user_id: UserId }`.
  - Add `ExpenseRequest::ListForGroup { group_id: GroupId }`.

- `test_client/src/kvittis_client.rs`
  - Add client methods for:
    - list expenses for user
    - list expenses for group
    - get user balances
    - get group balances

- `server/src/api.rs`
  - Extend `expense_handler` to match the new list variants and return `json_not_implemented()`.
  - Keep `balance_handler` returning `json_not_implemented()` (but ensure response types match Stage 1’s `i64`).

- `test_client/tests/`
  - Add/extend tests that:
    - call the new APIs
    - assert the server responds with the expected “not implemented” error
  - Keep the test data creation/cleanup pattern consistent with the existing integration tests.

Acceptance criteria:

- Tests compile and run.
- New tests fail (expected) due to “not implemented”, while existing tests remain unaffected.

## Stage 3 — Implement the API (Store + state + handlers)

Goal: implement the real list + balance logic.

Planned changes:

1. DB/store layer
   - Extend `server/src/db/mod.rs` (`Store` trait):
     - `list_expenses_for_user(user_id) -> Result<Vec<ExpenseRow>>`
     - `list_expenses_for_group(group_id) -> Result<Vec<ExpenseRow>>`
     - (optional) `is_group_member(group_id, user_id) -> Result<bool>`
   - Implement in `server/src/db/db_sqlite.rs` using join+grouping (same pattern as `get_expense`).

2. State layer
   - Add to `server/src/state.rs`:
     - `list_expenses_for_user(user_id) -> Result<Vec<Expense>>`
     - `list_expenses_for_group(group_id) -> Result<Vec<Expense>>`
     - `get_user_non_group_balances(user_id) -> Result<Vec<BalanceEntry>>`
     - `get_group_balance_overview(group_id, requester_id?) -> Result<...>`

3. HTTP handlers
   - `server/src/api.rs`:
     - Implement the expense list actions.
     - Implement `BalanceRequest::User`:
       - consider only `group_id IS NULL`
       - compute net per counterparty using deterministic integer split
     - Implement `BalanceRequest::Group`:
       - include **all** group expenses
       - compute member net positions and (if desired) settlement transfers

4. Tests
   - Update the Stage 2 tests to assert correct data instead of “not implemented”.
   - Add deterministic remainder split test cases (non-divisible amounts).

Acceptance criteria:

- All tests pass with real responses.
- Balances are stable (deterministic split) and expressed in `i64` minor units.
