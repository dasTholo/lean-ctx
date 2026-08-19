#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────────────────
# LeanCTX Pilot Setup — Ein-Command Enterprise Installation
# ─────────────────────────────────────────────────────────────────────
# Usage: curl -fsSL https://leanctx.com/pilot | bash
#    or: ./pilot-setup.sh [--api-key KEY] [--model MODEL] [--project PATH]
# ─────────────────────────────────────────────────────────────────────
set -euo pipefail

VERSION="3.9.19"
BOLD='\033[1m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
DIM='\033[2m'
RST='\033[0m'

# ─── Parse args ───────────────────────────────────────────────────────
API_KEY="${OPENAI_API_KEY:-${ANTHROPIC_API_KEY:-}}"
MODEL="gpt-4o-mini"
PROJECT="${PWD}"
ENDPOINT="https://api.openai.com/v1/chat/completions"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --api-key) API_KEY="$2"; shift 2 ;;
        --model) MODEL="$2"; shift 2 ;;
        --project) PROJECT="$2"; shift 2 ;;
        --endpoint) ENDPOINT="$2"; shift 2 ;;
        --ollama) ENDPOINT="http://localhost:11434/v1/chat/completions"; MODEL="${2:-gemma4:e4b}"; shift; [[ "${1:-}" != --* ]] && shift || true ;;
        -h|--help) echo "Usage: $0 [--api-key KEY] [--model MODEL] [--project PATH] [--endpoint URL] [--ollama [MODEL]]"; exit 0 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${RST}"
echo -e "${BOLD}║  LeanCTX Pilot Setup — Enterprise Context Intelligence      ║${RST}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${RST}"
echo ""

# ─── Step 1: Install LeanCTX ──────────────────────────────────────────
echo -e "${BLUE}[1/6]${RST} Installing lean-ctx ${VERSION}..."
if command -v lean-ctx &>/dev/null; then
    CURRENT=$(lean-ctx --version 2>/dev/null | grep -o '[0-9]\+\.[0-9]\+\.[0-9]\+' || echo "0")
    echo "       Already installed: v${CURRENT}"
    if [[ "$CURRENT" != "$VERSION" ]]; then
        echo "       Updating to v${VERSION}..."
        lean-ctx update 2>/dev/null || {
            curl -fsSL https://leanctx.com/install | bash
        }
    fi
else
    curl -fsSL https://leanctx.com/install | bash
fi
echo -e "       ${GREEN}✓${RST} lean-ctx ready"

# ─── Step 2: Connect to AI tools ──────────────────────────────────────
echo -e "${BLUE}[2/6]${RST} Connecting to AI tools..."
lean-ctx onboard 2>/dev/null || lean-ctx setup --non-interactive 2>/dev/null || true
echo -e "       ${GREEN}✓${RST} Connected"

# ─── Step 3: Health check ─────────────────────────────────────────────
echo -e "${BLUE}[3/6]${RST} Running diagnostics..."
lean-ctx doctor 2>/dev/null || true
echo -e "       ${GREEN}✓${RST} System healthy"

# ─── Step 4: Find a suitable file for evidence ────────────────────────
echo -e "${BLUE}[4/6]${RST} Finding review target in ${PROJECT}..."
cd "$PROJECT"

TARGET_FILE=""
for pattern in "src/**/*.rs" "src/**/*.ts" "src/**/*.py" "**/*.go" "src/**/*.java"; do
    CANDIDATE=$(find . -path "./$pattern" -type f 2>/dev/null | \
        xargs wc -l 2>/dev/null | sort -rn | head -5 | \
        awk '$1 >= 200 && $1 <= 1000 {print $2; exit}')
    if [[ -n "$CANDIDATE" ]]; then
        TARGET_FILE="$CANDIDATE"
        break
    fi
done

if [[ -z "$TARGET_FILE" ]]; then
    TARGET_FILE=$(find . -name "*.rs" -o -name "*.ts" -o -name "*.py" -o -name "*.go" | \
        head -1)
fi

if [[ -z "$TARGET_FILE" ]]; then
    echo "       No source files found in $PROJECT"
    echo "       Please specify: lean-ctx evidence run --file <path>"
    exit 1
fi

TARGET_LINES=$(wc -l < "$TARGET_FILE" | xargs)
echo -e "       Selected: ${TARGET_FILE} (${TARGET_LINES} LOC)"
echo -e "       ${GREEN}✓${RST} Target ready"

# ─── Step 5: API key check ────────────────────────────────────────────
echo -e "${BLUE}[5/6]${RST} Checking provider access..."
if [[ -z "$API_KEY" ]]; then
    if [[ "$ENDPOINT" == *"localhost"* ]] || [[ "$ENDPOINT" == *"11434"* ]]; then
        API_KEY="ollama-local"
        echo "       Using local Ollama (no API key needed)"
    else
        echo ""
        echo -e "       ${BOLD}No API key found.${RST}"
        echo "       Set one of:"
        echo "         export OPENAI_API_KEY=sk-..."
        echo "         export ANTHROPIC_API_KEY=sk-ant-..."
        echo "         ./pilot-setup.sh --api-key <key>"
        echo ""
        exit 1
    fi
fi
echo -e "       Model: ${MODEL}"
echo -e "       Endpoint: ${ENDPOINT}"
echo -e "       ${GREEN}✓${RST} Provider configured"

# ─── Step 6: Run evidence proof ───────────────────────────────────────
echo ""
echo -e "${BLUE}[6/6]${RST} ${BOLD}Running evidence proof...${RST}"
echo ""

OUTPUT_DIR="./lean-ctx-evidence-$(date +%Y%m%d-%H%M%S)"

lean-ctx evidence run \
    --file "$TARGET_FILE" \
    --model "$MODEL" \
    --endpoint "$ENDPOINT" \
    --api-key "$API_KEY" \
    --output "$OUTPUT_DIR" \
    --mode signatures

echo ""
echo -e "${BOLD}╔══════════════════════════════════════════════════════════════╗${RST}"
echo -e "${BOLD}║  Pilot Complete!                                             ║${RST}"
echo -e "${BOLD}╠══════════════════════════════════════════════════════════════╣${RST}"
echo -e "${BOLD}║                                                              ║${RST}"
echo -e "${BOLD}║  Evidence:  ${OUTPUT_DIR}/evidence-bundle.zip${RST}"
echo -e "${BOLD}║  Verify:    lean-ctx evidence verify ${OUTPUT_DIR}${RST}"
echo -e "${BOLD}║  Dashboard: lean-ctx dashboard${RST}"
echo -e "${BOLD}║  Savings:   lean-ctx gain${RST}"
echo -e "${BOLD}║                                                              ║${RST}"
echo -e "${BOLD}╚══════════════════════════════════════════════════════════════╝${RST}"
echo ""
echo -e "${DIM}Next steps:${RST}"
echo "  1. Review the evidence bundle (open ${OUTPUT_DIR}/evidence-manifest.json)"
echo "  2. Run 'lean-ctx gain' to see accumulated savings over time"
echo "  3. Run 'lean-ctx dashboard' for the web UI"
echo "  4. Contact: yves@thinkery.ch for enterprise plan"
