---
name: enterprise-deploy
description: >-
  Deploy, promote, health-check, or roll back the lean-ctx enterprise suite in
  production. Use when the user says enterprise deploy, deploy leanctx-suite,
  production rollout, pull latest on server, build Docker images, docker compose
  up, health check production, restart enterprise services, or roll back a
  failed enterprise release.
---

# Deploy the enterprise suite

Treat this as a production change. Never print `.env`, tokens, provider keys, or
customer payloads.

## Gather

1. Read `INFRASTRUCTURE.md`, `Makefile`, `docker-compose.prod.yml`, and the
   release diff in `lean-ctx-enterprise`.
2. Confirm the exact commit or immutable tag, operator authorization, deployment
   window, known-good rollback commit, and verified off-host database backup.
3. Review every new migration. Require proof that the previous application is
   compatible with the expanded schema; migrations are forward-only and are not
   automatically reversed.
4. Confirm CI `check`, `test`, and `build-images` passed for the intended commit.
5. Inspect live Coolify and Docker state before changing it; live Coolify
   routing is authoritative.
6. Determine deployment mode:
   - Git checkout: pull/fetch the pinned ref on the server.
   - Rsync snapshot: deploy from a clean local checkout with documented
     `make deploy`; do not fabricate `.git` on production.

## Act

For a Git-backed server, run `scripts/deploy-production.sh` with an explicit ref
and its two safety acknowledgements. It records the old commit, pulls the ref,
builds images, starts Compose, checks containers/public endpoints, and rolls
back application code on failure.

```bash
scripts/deploy-production.sh \
  --ref <commit-or-tag> \
  --server <user@host> \
  --key <ssh-key-path> \
  --backup-confirmed \
  --migration-reviewed \
  --confirm-production
```

Use `--remote-dir` and `--compose-file` when production differs from the
documented defaults. The matching `LEANCTX_DEPLOY_*` environment variables are
supported for automation; never commit their values.

For the documented snapshot mode, use a clean local checkout at the approved
commit, record `git rev-parse HEAD`, run `make deploy`, then `make health` and
the remote Compose checks below. The rsync excludes `.env`; verify that remains
true before deploying.

Do not run `docker compose down -v`, prune images, replace `.env`, or apply
migrations unless the user separately authorizes the exact destructive action.

## Verify

1. On the host, require every expected service to be running and no healthcheck
   to report `unhealthy`:

   ```bash
   docker compose -f docker-compose.prod.yml ps
   ```

2. Require successful public responses from Suite `/health`, dashboard, admin,
   and portal. Verify the authenticated data-plane path without logging its
   bearer token.
3. Inspect recent logs for crash loops, migration errors, authorization errors,
   and provider failures; redact secrets before sharing output.
4. Exercise one release-critical read path and any changed endpoint.
5. Record deployed/previous commits, image IDs, health results, backup location,
   migration ledger entries, operator, and deployment outcome.
6. If verification fails, stop traffic promotion and let the script redeploy the
   recorded commit. If schema compatibility is unproven, escalate instead of
   rolling back blindly.
