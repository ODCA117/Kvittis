# Account session review

Reviewed: 2026-11-10

**Work in progress: fix the TODOs below before considering this feature ready.**
See [the implementation plan](docs/account-sessions.md) for the broader feature scope.

## Completed

- [x] **Enforce stored-session validity during JWT validation.** SQLite session
  lookup rejects missing rows; validation rejects revoked, expired (including
  the expiry boundary), or subject-mismatched sessions and missing/deleted users.
  The expected app claim is checked. Storage failures do not bypass validation.
  Confirmed by source review; login/logout HTTP regression tests now also pass.
- [x] **Fix the SQLite migration syntax.** The initial migration runs successfully
  in an isolated fresh SQLite database. Keeping the session table in this file
  is accepted while undeployed; stores will be reset when the migration changes.
- [x] **Return the declared logout response.** The handler now serializes the
  shared `LogoutResponse` object through `json_success`.
- [x] **Run formatting and validation for the current changes.** Formatting,
  workspace compilation, and server Clippy checks pass. The existing
  common/server test command passes but runs zero tests.
- [x] **Add login/logout HTTP tests.** `test_client/tests/test_account_sessions.rs`
  covers successful login, wrong-password rejection, typed logout responses,
  copied-token rejection after logout, repeated logout rejection, independent
  logins, logging in again without reviving an old token, and rejection after
  account deletion. Verified against an isolated SQLite server.

The original finding that logout revocation was ignored is addressed.
`db_file.rs` is inactive and excluded from implementation and review.

## Next task: bound cleanup batches

- [ ] **Bound cleanup batches.** `AppState::cleanup_account_sessions` fetches every
  expired session and deletes them individually while holding the application
  write lock. Limit each batch to avoid blocking requests for a large backlog.

## Remaining tasks

- [ ] **Add remaining session tests.** Cover persistence across restart,
  missing/expired/mismatched sessions, and cleanup with SQLite.
- [ ] **Repeat validation after the remaining fixes.** Include focused tests and
  a running-server login/logout check.

## Deferred FIXME

- [ ] **Distinguish authentication and storage errors in HTTP responses.**
  The extractor currently maps all validation errors to `InvalidToken` and
  returns `400`. Return `401` for authentication failures and `500` for storage
  failures. Deferred by request; not a blocker for the completed validation task.

## Validation

- `cargo check --workspace`: passed.
- `cargo test -p common -p server`: passed, but ran zero tests.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy -p server`: passed without warnings.
- `npm --prefix frontend run build` (2026-11-08): passed with existing
  accessibility warnings.
- Initial migration executed in isolated in-memory SQLite: passed after the
  comma fixes.
- `cargo test -p test_client --test test_account_sessions`: two tests passed
  against an isolated SQLite server. Requires a running SQLite server at
  `http://localhost:3000`; set `KVITTIS_TEST_URL` to use another address.

The fixed lock deadlock and millisecond conversions were confirmed by inspection.
