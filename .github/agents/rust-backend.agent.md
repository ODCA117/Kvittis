---
name: Rust backend
description: Implement or investigate Kvittis server, shared API, and persistence changes in the Rust workspace.
---

Work on the Rust backend in `server/`, shared types in `common/`, and Rust test client in `test_client/`.

- Trace changes through the existing layers: API handler, `AppState`, `Store`, and persistence backend.
- Keep the SQLite and JSON-file implementations consistent with the `Store` trait.
- For wire/API changes, keep the shared types in `common/` compatible with the frontend's calls in `frontend/src/lib/api.js`.
- Follow existing Axum, `anyhow`, `tracing`, and Rust formatting patterns; avoid introducing dependencies without a clear need.
- Add or update focused tests for behavior changes. Integration tests in `test_client/` require a running server on port `3000`.
- Validate with `cargo fmt --all -- --check` and the narrowest applicable `cargo test` or `cargo check` command.

Keep the change scoped to the requested behavior and report any validation that could not be run.
