#!/usr/bin/env bash
set -euo pipefail

dry_run=0
if [[ ${1:-} == --dry-run ]]; then
  dry_run=1
  shift
fi

enterprise_root=${1:-}
[[ -n "$enterprise_root" ]] || {
  echo "usage: $0 [--dry-run] <enterprise-root> <dashboard|admin|portal>..." >&2
  exit 2
}
shift
(($#)) || set -- dashboard admin portal

run() {
  if [[ $dry_run == 1 ]]; then
    printf '+ '
    printf '%q ' "$@"
    printf '\n'
  else
    "$@"
  fi
}

for app in "$@"; do
  case "$app" in
    dashboard|admin|portal) ;;
    *) echo "unsupported production frontend: $app" >&2; exit 2 ;;
  esac
  app_dir="$enterprise_root/$app"
  [[ -f "$app_dir/package.json" && -f "$app_dir/package-lock.json" ]] || {
    echo "missing package.json or package-lock.json in $app_dir" >&2
    exit 2
  }

  echo "== $app: npm ci =="
  run npm --prefix "$app_dir" ci
  echo "== $app: typecheck =="
  run npm --prefix "$app_dir" exec -- tsc --noEmit
  echo "== $app: lint =="
  run npm --prefix "$app_dir" run lint
  echo "== $app: build =="
  run npm --prefix "$app_dir" run build
done

if [[ $dry_run == 1 ]]; then
  echo "frontend quality gate dry run complete"
else
  echo "frontend quality gate passed"
fi
