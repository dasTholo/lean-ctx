---
name: frontend-dev
description: >-
  Develop, build, or verify the lean-ctx enterprise dashboard, admin, or portal
  frontend. Use when the user says frontend dev, dashboard change, admin UI,
  customer portal, Next.js page, React component, npm ci, package-lock, frontend
  build check, TypeScript type check, tsc, ESLint, npm lint, or verify a frontend.
---

# Develop enterprise frontends

## Gather

1. Work from the `lean-ctx-enterprise` root and read `FRONTENDS.md` plus the
   target app's `package.json`, lockfile, TypeScript config, and local patterns.
2. Choose only active apps in scope: `dashboard`, `admin`, or `portal`.
   `apps/console` is planned but not production unless the user names it.
3. Inspect current `git status`, environment-variable requirements, API client,
   loading/error states, and neighboring components. Never expose `.env` values.
4. Determine browser support and accessibility expectations for the changed UI.

## Act

1. Install deterministically from the committed lockfile with `npm ci`; never
   replace it with `npm install` or silently regenerate the lockfile.
2. Match existing Next.js 14, React 18, Tailwind, component, and API-client
   conventions. Keep server/client boundaries explicit.
3. Implement loading, empty, error, and permission-denied states for data-driven
   UI. Preserve keyboard access, labels, focus behavior, contrast, and responsive
   layout.
4. Use `scripts/check-frontends.sh <enterprise-root> <app...>` to run each app's
   type check, lint, and production build. Use `--dry-run` to preview commands.
5. If an API call is new, use `$new-api-endpoint` to verify the Suite route and
   dashboard proxy allowlist.

## Verify

- Require `npm ci`, `npx tsc --noEmit`, `npm run lint`, and `npm run build` to
  succeed for every changed app.
- Exercise the affected screen at desktop and narrow viewport sizes; check the
  browser console and failed network requests.
- Verify loading, empty, error, and success states with real contracts, not mock
  production data.
- Confirm no `.env`, build output, `node_modules`, credentials, or unrelated
  lockfile changes entered the diff.
- Report each checked app and the exact failing stage if the gate is not green.
