# Copilot instructions for Kvittis

## Project layout

- This repository is a Cargo workspace: `server/` is the Axum API and persistence layer, `common/` holds shared domain and API types, and `test_client/` contains the Rust HTTP client and integration tests.
- `frontend/` is a Svelte 5 app built and served by Vite.
- Keep the API contract consistent across `common/src/api/`, `server/src/api/`, and `frontend/src/lib/api.js`.
- SQLite is the active `Store` backend. `server/src/db/db_file.rs` is currently inactive; exclude it from implementation and review unless explicitly requested.
- The server defaults to `127.0.0.1:3000`; Vite serves the frontend on port `3001` and proxies `/api` requests to the server on port `3000`.

## Change guidance

- Read nearby code and follow its existing naming, error handling, and formatting patterns before making changes.
- Keep changes focused. Avoid unrelated refactors and new dependencies unless the task needs them.
- For API changes, update the shared request/response types, server handlers, frontend API calls, and relevant tests as needed.
- For persistence changes, update the `Store` trait and SQLite backend, plus relevant tests.
- Never add credentials or real secrets. `.env.example` is a placeholder; use local environment configuration for secrets.
- Preserve the current behavior of callers and the active database backend unless the task explicitly changes it.

## Collaboration

- Help me understand the code and problem; guide with explanations, directions, and small examples rather than giving a complete answer.
- Help me explore designs, learn relevant techniques, and get a second opinion. Keep implementation steps few and easy to follow.
- Before implementing a feature, check that implementation is what I want. Boilerplate or an explicit implementation request can be handled directly.
- For complex work, give a brief overview, then detail only the current step and, when useful, the next one.
- Keep responses concise and focused on the important points.

## Build and test

- Format Rust: `cargo fmt --all -- --check`
- Check Rust crates: `cargo check --workspace`
- Run shared/server Rust tests: `cargo test -p common -p server`
- Build the frontend: `npm --prefix frontend run build`
- The `test_client` integration tests call a running server at `http://localhost:3000`; start the server before running those tests.
- When a change crosses the API boundary, validate both the relevant Rust code/tests and the frontend build.
