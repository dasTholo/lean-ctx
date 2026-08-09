#!/usr/bin/env bash
set -euo pipefail

dry_run=0
if [[ ${1:-} == --dry-run ]]; then
  dry_run=1
  shift
fi

crate_dir=${1:-.}
manifest="$crate_dir/Cargo.toml"
[[ -f "$manifest" ]] || { echo "Cargo.toml not found: $manifest" >&2; exit 2; }

run() {
  if [[ $dry_run == 1 ]]; then
    printf '+ '
    printf '%q ' "$@"
    printf '\n'
  else
    "$@"
  fi
}

echo "== cargo fmt =="
run cargo fmt --manifest-path "$manifest" --check

echo "== cargo clippy =="
run cargo clippy --manifest-path "$manifest" --all-targets --all-features -- -D warnings

echo "== cargo test =="
run cargo test --manifest-path "$manifest" --all-features

echo "== cargo doc =="
if [[ $dry_run == 1 ]]; then
  echo "+ RUSTDOCFLAGS='-D warnings' cargo doc --manifest-path $manifest --all-features --no-deps"
else
  RUSTDOCFLAGS="${RUSTDOCFLAGS:-} -D warnings" \
    cargo doc --manifest-path "$manifest" --all-features --no-deps
fi

echo "== TODO/FIXME inventory =="
if [[ $dry_run == 1 ]]; then
  echo "+ rg -n --hidden --glob '!target/**' --glob '!.git/**' '\\b(TODO|FIXME)\\b' $crate_dir"
  todo_output=""
else
  todo_output=$(rg -n --hidden \
    --glob '!target/**' --glob '!.git/**' \
    '\b(TODO|FIXME)\b' "$crate_dir" || true)
fi
if [[ -n "$todo_output" ]]; then
  printf '%s\n' "$todo_output"
  if [[ ${FAIL_ON_TODOS:-0} == 1 ]]; then
    echo "TODO/FIXME markers are blockers under FAIL_ON_TODOS=1" >&2
    exit 1
  fi
else
  echo "none"
fi

if [[ $dry_run == 1 ]]; then
  echo "crate quality gate dry run complete"
else
  echo "crate quality gate passed"
fi
