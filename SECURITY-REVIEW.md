# Account session review

Reviewed: 2026-11-09

**Work in progress: fix the TODOs below before considering this feature ready.**
See [the implementation plan](docs/account-sessions.md) for the broader feature scope.

## Completed

- [x] **Enforce stored-session validity during JWT validation.** SQLite session
  lookup rejects missing rows; validation rejects revoked, expired (including
  the expiry boundary), or subject-mismatched sessions and missing/deleted users.
  The expected app claim is checked. Storage failures do not bypass validation.
  Confirmed by source review; end-to-end authentication tests remain pending.

The original finding that logout revocation was ignored is addressed.
`db_file.rs` is inactive and excluded from implementation and review.

## Next task: fix the SQLite migration

- [ ] **Fix the SQLite migration.**
  `server/migrations/20250101_initial_schema.sql:21-29` lacks a comma before
  `FOREIGN KEY` and has a trailing comma before `)`. Restore the original
  migration and create a new session migration, so already-migrated databases
  do not fail checksum validation. Validate both fresh and existing databases.

## Remaining tasks

- [ ] **Bound cleanup batches.** `AppState::cleanup_account_sessions` fetches every
  expired session and deletes them individually while holding the application
  write lock. Limit each batch to avoid blocking requests for a large backlog.
- [ ] **Return the declared logout response.**
  `server/src/api/users.rs:67-70` returns a JSON string instead of the new
  `LogoutResponse` object in `common/src/api/auth.rs`.
- [ ] **Add focused session tests.** Cover login persistence, logout rejecting a
  copied token, independent device sessions, missing/expired/mismatched sessions,
  deleted users, and cleanup with SQLite.
- [ ] **Run formatting and validation after fixes.** The formatting check
  currently fails in the changed Rust files.

## Deferred FIXME

- [ ] **Distinguish authentication and storage errors in HTTP responses.**
  The extractor currently maps all validation errors to `InvalidToken` and
  returns `400`. Return `401` for authentication failures and `500` for storage
  failures. Deferred by request; not a blocker for the completed validation task.

## Validation

- `cargo check --workspace`: passed.
- `cargo test -p common -p server`: passed, but ran zero tests.
- `npm --prefix frontend run build`: passed with existing accessibility warnings.
- `cargo fmt --all -- --check`: failed; formatting remains pending.
- Initial migration executed in isolated in-memory SQLite: failed with
  `near "FOREIGN": syntax error`.

No end-to-end login/logout or running-server integration test was performed.
The fixed lock deadlock and millisecond conversions were confirmed by inspection.
