# Balance/Debt Test Cases (Overview)

All amounts are in **minor units** (`i64`): **cents/öre**.

## Conventions

- User balance entries (`BalanceEntry`):
  - `amount > 0`: `other` owes the requested user.
  - `amount < 0`: requested user owes `other`.
- Expense split:
  - `participants` share the cost equally.
  - Debt edges: each `participant != payer` owes their share to the payer.
  - Integer split is deterministic:
    - `base = amount / N`, `rem = amount % N`
    - sort participants by `UserId` ascending
    - first `rem` participants get `base + 1`

- Group settlement transfers (`GroupBalance`):
  - Each transfer is `{ from, to, amount }` where `from` pays `to`.
  - Deterministic settlement (for stable tests):
    - compute member net = `paid - owed_share`
    - sort debtors by `UserId`, creditors by `UserId`
    - greedy match: debtor pays current creditor up to min(need, available)

## Parallel test isolation requirements

These tests run in parallel.

- Each test case must use **unique** usernames and group names (do not reuse `"A"`, `"B"`, `"G"` as literal names across tests).
- Each test must create **all** of its own data (users, groups, expenses) and must clean up (delete expenses/groups/users) before returning.
- Prefer embedding a per-test unique suffix in names (e.g. based on a random string or timestamp) so that duplicate-name failures cannot occur.

## Stage expectations

- Stage 2: tests compile and run but **fail** because server returns “not implemented”.
- Stage 3: tests pass.

## Test cases

### TC1 — User expense listing includes payer + participant

Setup:
- Create unique users `A`, `B`, `C`.
- Create expenses:
  - `E1`: payer `A`, participants `[A, B]`, amount `100`, `group_id = null`
  - `E2`: payer `C`, participants `[A, C]`, amount `200`, `group_id = null`

Expectations:
- `ListForUser(A)` returns both `E1` and `E2`.

### TC2 — User balances (non-group) basic split

Setup:
- Create unique users `A`, `B`.
- Create expense `E1`: payer `A`, participants `[A, B]`, amount `100`, `group_id = null`.

Expectations:
- `UserBalances(A)` contains entry for `B` with `amount = +50`.
- `UserBalances(B)` contains entry for `A` with `amount = -50`.

### TC3 — User balances ignore group expenses

Setup:
- Create unique users `A`, `B`.
- Create unique group `G` with members `[A, B]`.
- Create expenses:
  - `E1`: payer `A`, participants `[A, B]`, amount `100`, `group_id = null`
  - `E2`: payer `A`, participants `[A, B]`, amount `100`, `group_id = G`

Expectations:
- `UserBalances(A)` only accounts for `E1` (so net vs `B` is `+50`, not `+100`).

### TC4 — Group overview includes all expenses (even if caller not involved)

Setup:
- Create unique users `A`, `B`, `C`.
- Create unique group `G` with members `[A, B, C]`.
- Create expenses in `G`:
  - `E1`: payer `A`, participants `[A, B]`, amount `300`
  - `E2`: payer `B`, participants `[B, C]`, amount `300`

Expectations:
- Group net positions:
  - `A = +150`, `B = 0`, `C = -150`
- Deterministic settlement transfers:
  - single transfer: `C -> A` of `150`

### TC5 — Deterministic remainder split

Setup:
- Create unique users `A`, `B`, `C`.
- Create expense: payer `A`, participants `[A, B, C]`, amount `100`, `group_id = null`.

Expectations:
- Shares sum to `100` with deterministic +1 remainder assignment based on sorted participant ids.
- Resulting balances are stable across runs.
