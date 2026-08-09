#!/usr/bin/env bash
set -euo pipefail

# Usage: ./merge-round.sh <round> <agent-01-name> <agent-02-name> ...
# Merges agent branches sequentially, runs quality gate, cleans up.

usage() {
  echo "usage: $0 <round> <agent-name> [agent-name...]" >&2
  exit 2
}

[[ $# -ge 2 ]] || usage
ROUND=$1
shift
AGENTS=("$@")

REPO="$(git rev-parse --show-toplevel)"
cd "$REPO"

echo "=== Merging Round ${ROUND}: ${#AGENTS[@]} agents ==="

# Step 1: Manual commit any uncommitted work
for agent in "${AGENTS[@]}"; do
  WT="${REPO}/.worktrees/${ROUND}-agent-${agent}"
  if [ -d "$WT" ]; then
    cd "$WT"
    CHANGES=$(git status --short 2>/dev/null | wc -l | tr -d ' ')
    AHEAD=$(git log main..HEAD --oneline 2>/dev/null | wc -l | tr -d ' ')
    if [ "$CHANGES" -gt 0 ] && [ "$AHEAD" -eq 0 ]; then
      echo "Committing uncommitted work for agent-${agent}..."
      git add -A
      git commit --no-verify -m "feat: ${ROUND} agent-${agent}" 2>/dev/null || true
    fi
    echo "agent-${agent}: ${AHEAD} commits, ${CHANGES} uncommitted"
  fi
done

cd "$REPO"

# Step 2: Merge sequentially
MERGED=0
for agent in "${AGENTS[@]}"; do
  BRANCH="${ROUND}/agent-${agent}"
  echo ""
  echo "--- Merging ${BRANCH} ---"
  if git merge "$BRANCH" --no-edit 2>&1; then
    MERGED=$((MERGED + 1))
  else
    echo "CONFLICT in ${BRANCH} — resolve manually, then re-run."
    echo "Merged so far: ${MERGED}/${#AGENTS[@]}"
    exit 1
  fi
done

echo ""
echo "=== All ${MERGED} branches merged ==="

# Step 3: Quality Gate
echo ""
echo "=== Quality Gate ==="
cd rust

echo "1/4 cargo fmt --check"
cargo fmt --check || { echo "FAIL: cargo fmt"; exit 1; }

echo "2/4 cargo clippy --all-features"
cargo clippy --all-features -- -D warnings 2>&1 | tail -3
if [ ${PIPESTATUS[0]} -ne 0 ]; then
  echo "FAIL: clippy errors — fix before proceeding"
  exit 1
fi

echo "3/4 cargo test --lib"
TEST_OUTPUT=$(cargo test --lib 2>&1 | tail -3)
echo "$TEST_OUTPUT"

echo "4/4 LOC gate"
cd "$REPO"
bash scripts/loc-gate.sh 2>&1 | tail -3

echo ""
echo "=== Quality Gate PASSED ==="

# Step 4: Cleanup
echo ""
echo "=== Cleanup ==="
for agent in "${AGENTS[@]}"; do
  git worktree remove --force ".worktrees/${ROUND}-agent-${agent}" 2>/dev/null || true
  git branch -D "${ROUND}/agent-${agent}" 2>/dev/null || true
done
echo "Worktrees and branches cleaned."

echo ""
echo "Ready to push: SKIP_PREFLIGHT=1 git push github main && SKIP_PREFLIGHT=1 git push origin main"
