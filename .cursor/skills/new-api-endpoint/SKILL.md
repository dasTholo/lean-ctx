---
name: new-api-endpoint
description: >-
  Add or extend an HTTP API endpoint in the enterprise leanctx-suite, including
  Axum handler, routes.rs registration, dashboard proxy allowlist, tests, and API
  documentation. Use when the user says new API endpoint, add endpoint, add
  route, register Axum route, create handler, expose backend API to dashboard,
  proxy route, or update leanctx-suite API docs.
---

# Add a leanctx-suite API endpoint

## Gather

1. Work from the `lean-ctx-enterprise` root and read its current instructions.
2. Define method, canonical path, request/response schema, status codes, auth
   scope, tenant boundary, persistence behavior, and idempotency before editing.
3. Inspect the nearest handler in `bins/leanctx-suite/src/api/`, its unit tests,
   `api/mod.rs`, `routes.rs`, and
   `dashboard/src/app/api/proxy/route.ts`.
4. Reuse `SuiteState`, `routes::authorize`, and `ApiError`; do not create a
   parallel auth/error convention. Confirm whether the endpoint is intentionally
   public like `/health` and `/ready`.
5. Locate the maintained API documentation. `deploy/README.md` contains the
   current API table; update any additional OpenAPI/reference artifact found by
   repository search.

## Act

1. Create or extend `bins/leanctx-suite/src/api/<domain>.rs`:
   - Use typed Axum extractors and serializable request/response structs.
   - Validate at the boundary and return stable `ApiError` variants.
   - Authorize before accessing tenant data and derive tenant identity from the
     authenticated context, never an untrusted body field.
   - Add a `/// METHOD /path — purpose.` handler doc comment.
2. Export a new module from `api/mod.rs` only when creating a new domain file.
3. Import the module and register the exact method/path in `routes.rs`. Keep
   static routes before conflicting `/{id}` routes.
4. If the dashboard must call it, add the narrowest anchored regex to
   `ALLOWED_PATHS` in `dashboard/src/app/api/proxy/route.ts`. Never allow an
   arbitrary `/api/.*` passthrough; confirm GET/POST/PUT/DELETE forwarding and
   response handling fit the endpoint.
5. Add tests beside the handler for success, invalid input, missing/insufficient
   auth, tenant isolation, and domain failures. Add route-level coverage when a
   reusable `SuiteState` fixture exists.
6. Update the API table/reference with method, path, auth, request, response,
   errors, and one redacted example.

## Verify

Run from the enterprise root:

```bash
cargo fmt --check
cargo clippy -p leanctx-suite --all-targets --all-features -- -D warnings
cargo test -p leanctx-suite
```

Then run the dashboard checks from `$frontend-dev` when its proxy or client code
changed. Confirm the final diff contains handler, route registration, proxy
allowlist when needed, tests, and docs. Test one authorized request and one
unauthorized request without exposing credentials.
