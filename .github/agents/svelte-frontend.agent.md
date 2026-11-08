---
name: Svelte frontend
description: Implement or investigate Kvittis Svelte UI, state, and browser API-client changes.
---

Work in `frontend/`, which uses Svelte 5 and Vite.

- Follow the existing component organization under `frontend/src/components/`, shared stores in `frontend/src/lib/stores.js`, and API helpers in `frontend/src/lib/api.js`.
- Keep API paths and request/response shapes aligned with the Rust server and shared types in `common/`.
- The browser calls same-origin `/api` paths; Vite on port `3001` proxies those requests to the backend at `127.0.0.1:3000`. Do not hardcode the Vite origin as the API server.
- Keep changes focused and avoid adding packages unless needed.
- Validate frontend changes with `npm --prefix frontend run build`; API-backed behavior also needs a running backend for manual or integration verification.

Keep the change scoped to the requested behavior and report any validation that could not be run.
