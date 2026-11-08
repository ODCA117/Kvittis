# Account session review

Reviewed: 2026-11-08

**Work in progress: fix the TODOs below before considering this feature ready.**
This branch preserves the reviewed implementation; it does not fix the findings.
See [the implementation plan](docs/account-sessions.md) for the broader feature scope.

## Security finding

| # | Severity | File | Lines | Vulnerability | Confidence |
|---|----------|------|-------|---------------|------------|
| 1 | HIGH | server/src/state.rs | 84-97, 156-163 | Logout revokes the stored session, but JWT validation does not check session state; copied tokens remain usable | 9/10 |

`validate_jwt` verifies the JWT without loading its session. Neither revocation
nor deletion of the session row currently invalidates the token. Account deletion
also is not checked during validation.

## TODO: fix next

- [ ] **Enforce session validity on every authenticated request.** Add a store
  lookup by session ID; reject missing, revoked, expired, or subject-mismatched
  sessions and inactive accounts. Check the expected app claim. Return `401` for
  authentication failures and `500` for storage failures without bypassing checks.
- [ ] **Fix the SQLite migration.**
  `server/migrations/20250101_initial_schema.sql:21-29` lacks a comma before
  `FOREIGN KEY` and has a trailing comma before `)`. Restore the original
  migration and create a new session migration, so already-migrated databases
  do not fail checksum validation. Validate both fresh and existing databases.
- [ ] **Implement file-backed session storage.**
  `server/src/db/db_file.rs:152-165` has `todo!()` in all four session operations.
  Login panics and the cleanup task panics at its first tick. Add persistent
  session state with backward-compatible loading; report write errors rather
  than publishing changes that were not persisted.
- [ ] **Bound cleanup batches.** `server/src/state.rs:595-604` fetches every
  expired session and deletes them individually while holding the application
  write lock. Limit each batch to avoid blocking requests for a large backlog.
- [ ] **Return the declared logout response.**
  `server/src/api/users.rs:67-70` returns a JSON string instead of the new
  `LogoutResponse` object in `common/src/api/auth.rs`.
- [ ] **Add focused session tests.** Cover login persistence, logout rejecting a
  copied token, independent device sessions, missing/expired/mismatched sessions,
  deleted users, cleanup, and both persistence backends.
- [ ] **Run formatting and validation after fixes.** The formatting check
  currently fails on two expressions in `server/src/state.rs`.

## Validation at review time

- `cargo check --workspace`: passed.
- `cargo test -p common -p server`: passed, but ran zero tests.
- `npm --prefix frontend run build`: passed with existing accessibility warnings.
- `cargo fmt --all -- --check`: failed on two expressions in `state.rs`.
- Initial migration executed in isolated in-memory SQLite: failed with
  `near "FOREIGN": syntax error`.

No end-to-end login/logout or running-server integration test was performed.
The fixed lock deadlock and millisecond conversions were confirmed by inspection.
