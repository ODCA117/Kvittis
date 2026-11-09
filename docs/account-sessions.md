# Account sessions and logout

**Status: Server/common implementation in progress; stored-session validation is
implemented. Next: fix the SQLite migration (see [review TODOs](../SECURITY-REVIEW.md)).
Frontend integration is pending.**

## Behavior

Each login creates a separate database session. The JWT identifies that session;
the server validates both the JWT and the stored session on every authenticated request.
Logout revokes only the current login. A copied JWT then fails validation.
Other devices remain signed in. These sessions are separate from temporary-group sessions.
Store session metadata, not JWT strings or passwords.

## Shared types (`common/`)

- Add `AccountSessionId = Uuid` in `common/src/lib.rs`.
- Keep `UnauthorizedUserRequest::Login`, `LoginResponse`, and bearer-token transport unchanged.
- Keep `AuthorizedUserRequest::Logout` without a user/session ID supplied by the client.
- Add/export a typed `LogoutResponse { status: String }`, returning `"Logged out"`.
  Continue using `ApiResponse` and `{ "message": ... }` for errors.
- JWT claims remain server-internal; no shared JWT or public session-list DTO is required.

## Claims and storage (`server/`)

Extend `Claims` in `server/src/api/auth.rs`:

```rust
pub struct Claims {
    pub sub: UserId,
    pub sid: AccountSessionId,
    pub app: String,
    pub exp: usize,
}
```

Add `AccountSessionRow`: `id`, `user_id`, `created_at`, `expires_at`, `revoked_at`.
Store UTC milliseconds; JWT `exp` remains Unix seconds.
Keep the current 24-hour lifetime and derive both expiries from the same second-aligned time.

Extend `Store` with create/get session, revoke owned session, revoke all user sessions,
and bounded expired-session cleanup operations. Revocation records the first revocation time.
Add an atomic account-deletion operation that also revokes its sessions.

- SQLite: add a new migration with session primary key, user foreign key, and expiry/user indexes.
  Use transactions for account deletion/revocation; do not edit the initial migration.
- JSON: add a session map with a serde default so older files load with no sessions.
  Persist changes atomically before success; failed writes must not publish mutated state.
  Active-account checks must reject `deleted_at` rows consistently with SQLite.
- Clean expired sessions at startup and periodically in bounded batches.
  Cleanup failure is logged/retried; expiry denies access even before physical deletion.
  Cleanup deletes session metadata only, never users, groups, or expenses.

## Login, validation, and logout

1. Login: verify credentials and active account; generate a random session UUID and signed JWT.
   Persist the matching session before returning the existing `LoginResponse`.
   Signing/storage failures return an internal error, not a successful or usable login.
2. Validation: check bearer token, expected signature algorithm, `app == "Kvittis"`, and JWT expiry.
   Load `sid`; require matching `sub`, no revocation, valid session expiry, and an active user.
   Reject at `now >= expires_at`; do not rely on JWT clock tolerance to extend the session.
3. Logout: replace the placeholder in `authorized_user_handler` with revocation of `claims.sid`,
   constrained by `claims.sub`. Persist it before returning `200` and `LogoutResponse`.
   An already revoked token fails authentication with `401`, including on a repeated logout.
4. Account deletion: atomically revoke all logins with deletion. Re-registration cannot revive them.

Missing, invalid, expired, revoked, or legacy JWTs without `sid` return `401`.
Storage failures return `500`, are logged, and never bypass validation.
Replace the extractor's blanket invalid-token mapping and its current `{ "error": ... }` response.
Do not log JWTs, credentials, or password hashes; remove existing token/hash debug logging.
No refresh tokens, session-management UI, or logout-all endpoint in this first implementation.
Do not cache active sessions initially: it would delay revocation.
Requests authenticated before logout may already be running; logout does not cancel them.

Expense/balance routes lack authentication: require the session-validating extractor.
Session validity does not replace membership/ownership checks; enforce those separately.

## Tests and rollout

Test login persistence, invalid/expired/mismatched sessions, deleted users, logout, and two devices.
Run the same store tests for SQLite/JSON, including restart, cleanup, and failed-write handling.
Test HTTP `401`/`500` errors and rejection of tokens without `sid`; old logins must sign in again.
Run Rust formatting, workspace check, and common/server tests; use isolated HTTP test data.

## Frontend follow-up (pending; not implemented)

Call the existing logout API before clearing local state; clearing alone does not revoke a JWT.
Clear credentials/data after success or `401`. On network/server failure, show that server logout
is unconfirmed; local token removal must not be presented as confirmed revocation.
Handle authentication `401` responses and prompt for login again. Validate with the frontend build.
