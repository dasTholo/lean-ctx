#!/usr/bin/env bash
set -euo pipefail

# Usage: ./launch-round.sh <round> <agent-01-name> <agent-02-name> ...
# Example: ./launch-round.sh r14 01-kernel-activate 02-compose-wire 03-tests 04-docs

usage() {
  echo "usage: $0 <round> <agent-name> [agent-name...]" >&2
  exit 2
}

[[ $# -ge 2 ]] || usage
ROUND=$1
shift
AGENTS=("$@")

REPO="$(git rev-parse --show-toplevel)"
GOALS_DIR="/tmp/codex-goals-${ROUND}"
PREAMBLE="${GOALS_DIR}/preamble.md"

if [ ! -f "$PREAMBLE" ]; then
  echo "ERROR: ${PREAMBLE} not found. Create preamble first."
  exit 1
fi

echo "=== Round ${ROUND}: ${#AGENTS[@]} agents ==="

# Ensure .worktrees is gitignored
grep -q "^\.worktrees/" "${REPO}/.gitignore" 2>/dev/null || {
  echo ".worktrees/" >> "${REPO}/.gitignore"
  echo "Added .worktrees/ to .gitignore"
}

for agent in "${AGENTS[@]}"; do
  BRANCH="${ROUND}/agent-${agent}"
  WT="${REPO}/.worktrees/${ROUND}-agent-${agent}"
  GOAL="${GOALS_DIR}/agent-${agent}.md"
  COMBINED="/tmp/codex-${ROUND}-combined-${agent}.md"

  if [ ! -f "$GOAL" ]; then
    echo "SKIP ${agent}: ${GOAL} not found"
    continue
  fi

  # Create worktree
  if [ ! -d "$WT" ]; then
    git branch "$BRANCH" main 2>/dev/null || true
    git worktree add "$WT" "$BRANCH" 2>/dev/null
    echo "Created worktree: ${WT}"
  fi

  # Install fmt-only pre-commit hook (NO clippy — too slow)
  # Worktree .git is a file pointing to the real gitdir
  GITDIR="$(cd "$WT" && git rev-parse --git-dir)"
  mkdir -p "${GITDIR}/hooks" 2>/dev/null || true
  cat > "${GITDIR}/hooks/pre-commit" << 'HOOK'
#!/bin/bash
cd rust 2>/dev/null && cargo fmt --check 2>/dev/null || { echo "cargo fmt failed"; exit 1; }
HOOK
  chmod +x "${GITDIR}/hooks/pre-commit"

  # Combine preamble + goal
  cat "$PREAMBLE" > "$COMBINED"
  printf "\n---\n\n" >> "$COMBINED"
  cat "$GOAL" >> "$COMBINED"

  echo "Launching agent-${agent}..."
  osascript -e "
    tell application \"Terminal\"
      activate
      do script \"cd '${WT}' && cat '${COMBINED}' | codex exec -s workspace-write 2>&1 | tee /tmp/codex-${ROUND}-agent-${agent}.log\"
    end tell
  "
  sleep 2
done

echo ""
echo "=== ${#AGENTS[@]} agents launched ==="
echo "Monitor: watch -n30 'for a in ${AGENTS[*]}; do echo \$a: \$(cd ${REPO}/.worktrees/${ROUND}-agent-\$a && git status --short | wc -l) changes; done'"
