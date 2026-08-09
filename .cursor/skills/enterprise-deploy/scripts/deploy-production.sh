#!/usr/bin/env bash
set -euo pipefail

server="${LEANCTX_DEPLOY_SERVER:-}"
key_file="${LEANCTX_DEPLOY_KEY:-}"
remote_dir="${LEANCTX_DEPLOY_REMOTE_DIR:-/home/administrator/lean-ctx-enterprise}"
compose_file="${LEANCTX_DEPLOY_COMPOSE_FILE:-docker-compose.prod.yml}"
release_ref=""
backup_confirmed=0
migration_reviewed=0
confirmed=0
auto_rollback=1

usage() {
  local exit_code=${1:-2}
  cat >&2 <<'USAGE'
usage: deploy-production.sh --ref <commit-or-tag> --backup-confirmed \
  --migration-reviewed --confirm-production [--server user@host] \
  [--key path] [--remote-dir path] [--compose-file path] \
  [--no-auto-rollback]

Server and key are required through options or LEANCTX_DEPLOY_SERVER and
LEANCTX_DEPLOY_KEY. LEANCTX_DEPLOY_REMOTE_DIR and LEANCTX_DEPLOY_COMPOSE_FILE
may override the remaining defaults.
USAGE
  exit "$exit_code"
}

while (($#)); do
  case "$1" in
    --ref) [[ $# -ge 2 && -n ${2:-} ]] || usage; release_ref=$2; shift 2 ;;
    --server) [[ $# -ge 2 && -n ${2:-} ]] || usage; server=$2; shift 2 ;;
    --key) [[ $# -ge 2 && -n ${2:-} ]] || usage; key_file=$2; shift 2 ;;
    --remote-dir) [[ $# -ge 2 && -n ${2:-} ]] || usage; remote_dir=$2; shift 2 ;;
    --compose-file) [[ $# -ge 2 && -n ${2:-} ]] || usage; compose_file=$2; shift 2 ;;
    --backup-confirmed) backup_confirmed=1; shift ;;
    --migration-reviewed) migration_reviewed=1; shift ;;
    --confirm-production) confirmed=1; shift ;;
    --no-auto-rollback) auto_rollback=0; shift ;;
    -h|--help) usage 0 ;;
    *) usage ;;
  esac
done

[[ -n "$release_ref" ]] || usage
[[ -n "$server" ]] || { echo "missing --server or LEANCTX_DEPLOY_SERVER" >&2; exit 2; }
[[ -n "$key_file" ]] || { echo "missing --key or LEANCTX_DEPLOY_KEY" >&2; exit 2; }
[[ "$release_ref" =~ ^[A-Za-z0-9._/-]+$ ]] || { echo "invalid ref" >&2; exit 2; }
[[ "$remote_dir" =~ ^/[A-Za-z0-9._/-]+$ ]] || { echo "invalid remote directory" >&2; exit 2; }
[[ "$compose_file" =~ ^[A-Za-z0-9._/-]+$ ]] || { echo "invalid compose file" >&2; exit 2; }
[[ $backup_confirmed == 1 && $migration_reviewed == 1 && $confirmed == 1 ]] || {
  echo "refusing production deploy without backup, migration, and production confirmations" >&2
  exit 2
}
[[ -r "$key_file" ]] || { echo "SSH key not readable: $key_file" >&2; exit 2; }

ssh_base=(ssh -i "$key_file" -o BatchMode=yes "$server")

previous_ref=$("${ssh_base[@]}" bash -s -- "$remote_dir" <<'REMOTE'
set -euo pipefail
cd "$1"
test -d .git || { echo "production directory is an rsync snapshot; use documented make deploy" >&2; exit 3; }
test -z "$(git status --porcelain)" || { echo "production checkout is dirty" >&2; exit 3; }
git rev-parse HEAD
REMOTE
)
printf 'rollback commit: %s\n' "$previous_ref"

deploy_remote() {
  local target_ref=$1
  "${ssh_base[@]}" bash -s -- "$remote_dir" "$compose_file" "$target_ref" <<'REMOTE'
set -euo pipefail
cd "$1"
compose=$2
target=$3
git fetch origin --tags --prune
if [[ "$target" == "main" ]]; then
  git checkout main
  git pull --ff-only origin main
else
  git checkout --detach "$target"
fi
git rev-parse HEAD
docker compose -f "$compose" build --parallel
docker compose -f "$compose" up -d
REMOTE
}

check_remote() {
  "${ssh_base[@]}" bash -s -- "$remote_dir" "$compose_file" <<'REMOTE'
set -euo pipefail
cd "$1"
compose=$2
failed=0
while IFS= read -r service; do
  container=$(docker compose -f "$compose" ps -q "$service")
  if [[ -z "$container" ]]; then
    echo "missing container: $service" >&2
    failed=1
    continue
  fi
  state=$(docker inspect -f '{{.State.Status}}' "$container")
  health=$(docker inspect -f '{{if .State.Health}}{{.State.Health.Status}}{{else}}none{{end}}' "$container")
  printf '%s state=%s health=%s\n' "$service" "$state" "$health"
  [[ "$state" == running && "$health" != unhealthy ]] || failed=1
done < <(docker compose -f "$compose" config --services)
exit "$failed"
REMOTE
}

check_public() {
  local endpoints=(
    "https://enterprise.leanctx.com/health"
    "https://enterprise-dashboard.leanctx.com/"
    "https://enterprise-admin.leanctx.com/"
    "https://cloud.leanctx.com/"
  )
  local endpoint code attempt
  for endpoint in "${endpoints[@]}"; do
    code=000
    for attempt in {1..12}; do
      code=$(curl -Lso /dev/null -w '%{http_code}' --connect-timeout 5 --max-time 15 "$endpoint" || true)
      [[ "$code" =~ ^[23] ]] && break
      sleep 5
    done
    printf '%s -> %s\n' "$endpoint" "$code"
    [[ "$code" =~ ^[23] ]] || return 1
  done
}

if deploy_remote "$release_ref" && check_remote && check_public; then
  echo "production deployment verified"
  exit 0
fi

echo "deployment verification failed" >&2
if [[ $auto_rollback != 1 ]]; then
  echo "automatic rollback disabled; previous commit: $previous_ref" >&2
  exit 1
fi

echo "rolling back application to $previous_ref" >&2
deploy_remote "$previous_ref"
check_remote
check_public
echo "rollback verified" >&2
exit 1
