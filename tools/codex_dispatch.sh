#!/bin/bash
# tools/codex_dispatch.sh
# Claude Code가 이 스크립트를 호출해서 Codex Cloud에 티켓을 보냄
#
# 사용법:
#   bash tools/codex_dispatch.sh tickets/t-010-fix-input.md
#   bash tools/codex_dispatch.sh tickets/t-010-fix-input.md t/010-fix-input

set -euo pipefail

TICKET_FILE="$1"
BRANCH="${2:-}"
ENV_ID="${CODEX_ENV_WORLDSIM:?ERROR: CODEX_ENV_WORLDSIM 환경변수를 설정하세요}"

if [ ! -f "$TICKET_FILE" ]; then
  echo "[codex-dispatch] ERROR: 파일 없음: $TICKET_FILE"
  exit 1
fi

# 브랜치명 자동 추출 (없으면 파일명에서)
if [ -z "$BRANCH" ]; then
  SLUG=$(basename "$TICKET_FILE" .md)
  BRANCH="t/${SLUG}"
fi

TICKET_CONTENT=$(cat "$TICKET_FILE")

# --- Kanban Board Integration (optional, silent fail if server is down) ---
KANBAN_URL="${KANBAN_URL:-http://localhost:8800}"
KANBAN_INSTRUCTIONS=""
if [ -n "${BATCH_ID:-}" ]; then
  TICKET_TITLE=$(basename "$TICKET_FILE" .md | sed 's/-/ /g; s/^t /t-/')
  TICKET_ID=$(curl -sf -X POST "${KANBAN_URL}/api/tickets" \
    -H "Content-Type: application/json" \
    -d "{
      \"title\": \"${TICKET_TITLE}\",
      \"batch_id\": \"${BATCH_ID}\",
      \"created_by\": \"claude_code\",
      \"dispatch_method\": \"codex\",
      \"ticket_number\": ${TICKET_NUMBER:-0},
      \"system\": \"${SYSTEM_NAME:-}\",
      \"priority\": \"${PRIORITY:-medium}\"
    }" 2>/dev/null | python3 -c "import sys,json; print(json.loads(sys.stdin.read()).get('id',''))" 2>/dev/null || true)

  if [ -n "$TICKET_ID" ]; then
    KANBAN_INSTRUCTIONS="
--- KANBAN INTEGRATION ---
Run these at the appropriate times (fail silently if server unavailable):
  Start:    source tools/kanban/scripts/kanban_helpers.sh && kanban_start \"${TICKET_ID}\" \"codex-\$(hostname -s)\"
  Progress: kanban_log \"${TICKET_ID}\" \"info\" \"description of progress\"
  Done:     kanban_done \"${TICKET_ID}\"
  Failed:   kanban_fail \"${TICKET_ID}\" \"error description\"
--- END KANBAN ---"
  fi
fi
# --- End Kanban Integration ---

echo "╔══════════════════════════════════════╗"
echo "║  Codex Cloud Dispatch                ║"
echo "╠══════════════════════════════════════╣"
echo "║  ENV:    $ENV_ID"
echo "║  Branch: $BRANCH"
echo "║  Ticket: $TICKET_FILE"
echo "╚══════════════════════════════════════╝"

codex cloud exec \
  --env "$ENV_ID" \
  "You are implementing a ticket for the WorldSim project (Godot 4.6, GDScript).

Branch to work on: ${BRANCH}

Read AGENTS.md in the repo root for project conventions.

=== TICKET START ===
${TICKET_CONTENT}
=== TICKET END ===

After implementation:
1. Commit all changes to branch '${BRANCH}'
2. Run: bash scripts/gate.sh
3. If gate fails, fix and retry until it passes
4. Create a PR to lead/main with a clear summary

${KANBAN_INSTRUCTIONS}"

echo "[codex-dispatch] ✅ Task submitted to Codex Cloud"