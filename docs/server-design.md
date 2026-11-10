# Server redesign

**Status: In progress. Notes only; no server changes implemented.**

## Goals

- Temporary sessions: store a group, members, and expenses on the server under a session ID.
  A timeout deletes all session-related rows. Later, support restoration through a private URL.
- Complete the expense-splitting and settlement algorithm, building on the existing calculations.
- End goal: user management with accounts, friendships, groups, and persistent expense history.

## Code review: areas to improve

Source reviewed on 2026-11-08; these findings are not runtime test results.

- Login, password hashing, JWTs, and SQLite friendships exist; logout is unfinished.
- Expense and balance handlers do not require authentication.
- Group reads/searches are not membership-scoped. Adding members checks membership, not admin role.
- Friend-request handling does not verify that the caller is the sender or recipient.
- Balance calculation and greedy settlement exist, but do not guarantee the fewest transfers.
- `last_settled` is neither advanced nor used to filter expenses.
  Expense and group lists lack date limits and pagination.
- SQLite soft-deletes users/groups but hard-deletes expenses, leaving incomplete history.
- The JSON backend has unfinished membership/friendship methods.
- Temporary sessions and timeout cleanup are missing.

Sources: [handlers](../server/src/api/), [business logic](../server/src/state.rs),
[storage](../server/src/db/), and [shared contracts](../common/src/api/).
The README's older endpoint overview is historical.
