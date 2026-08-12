#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
RESULTS_FILE="${1:-$ROOT/scripts/benchmark/results/token_reduction.tsv}"

[[ -f "$RESULTS_FILE" ]] || { echo "Missing results: $RESULTS_FILE" >&2; exit 1; }

awk -F '\t' '
  BEGIN {
    print "| Task | Raw Tokens | Compressed Tokens | Reduction % |"
    print "|---|---:|---:|---:|"
  }
  NR == 1 { next }
  {
    reduction = ($4 == 0 ? 0 : (1 - ($5 / $4)) * 100)
    printf "| %s | %d | %d | %.1f%% |\n", $1, $4, $5, reduction
    raw += $4; compressed += $5; reductions += reduction; count++
  }
  END {
    printf "| **Average (%d tasks)** | **%.0f** | **%.0f** | **%.1f%%** |\n", \
      count, raw / count, compressed / count, reductions / count
  }
' "$RESULTS_FILE"
