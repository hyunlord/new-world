#!/bin/bash
# Kanban Board API helpers for Codex agents
# Source this file in Codex prompts: source tools/kanban/scripts/kanban_helpers.sh
# Environment: KANBAN_URL (default: http://localhost:8800)

KANBAN_URL="${KANBAN_URL:-http://localhost:8800}"

kanban_start() {
  # Mark ticket as in_progress. Args: $1=ticket_id, $2=agent_name
  curl -sf -X PATCH "${KANBAN_URL}/api/tickets/$1" \
    -H "Content-Type: application/json" \
    -d "{\"status\": \"in_progress\", \"assignee\": \"$2\"}"
}

kanban_log() {
  # Send progress log. Args: $1=ticket_id, $2=level(info|warn|error), $3=message
  local MSG=$(echo "$3" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read().strip()))")
  curl -sf -X POST "${KANBAN_URL}/api/tickets/$1/logs" \
    -H "Content-Type: application/json" \
    -d "{\"level\": \"$2\", \"message\": $MSG, \"source\": \"codex\"}"
}

kanban_done() {
  # Mark ticket done with diff. Args: $1=ticket_id
  local DIFF_STAT=$(git diff HEAD~1 --stat 2>/dev/null || echo "no diff available")
  local DIFF_FULL=$(git diff HEAD~1 2>/dev/null || echo "no diff available")
  local STAT_JSON=$(echo "$DIFF_STAT" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))")
  local FULL_JSON=$(echo "$DIFF_FULL" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read()))")
  curl -sf -X PATCH "${KANBAN_URL}/api/tickets/$1" \
    -H "Content-Type: application/json" \
    -d "{\"status\": \"done\", \"diff_summary\": $STAT_JSON, \"diff_full\": $FULL_JSON}"
}

kanban_fail() {
  # Mark ticket failed. Args: $1=ticket_id, $2=error_message
  local ERR=$(echo "$2" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read().strip()))")
  curl -sf -X PATCH "${KANBAN_URL}/api/tickets/$1" \
    -H "Content-Type: application/json" \
    -d "{\"status\": \"failed\", \"error_message\": $ERR}"
}

kanban_review() {
  # Mark ticket for review. Args: $1=ticket_id
  curl -sf -X PATCH "${KANBAN_URL}/api/tickets/$1" \
    -H "Content-Type: application/json" \
    -d '{"status": "review"}'
}

kanban_claim() {
  # Claim a ticket. Args: $1=ticket_id, $2=agent_name
  curl -sf -X PATCH "${KANBAN_URL}/api/tickets/$1" \
    -H "Content-Type: application/json" \
    -d "{\"status\": \"claimed\", \"assignee\": \"$2\"}"
}

# ===== Claude Code Integration Functions =====

kanban_create_batch() {
  # Create a new batch (prompt session). Args: $1=title, $2=source_prompt (optional)
  local TITLE_JSON=$(echo "$1" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read().strip()))")
  local BODY="{\"title\": $TITLE_JSON"
  if [ -n "$2" ]; then
    local SRC_JSON=$(echo "$2" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read().strip()))")
    BODY="$BODY, \"source_prompt\": $SRC_JSON"
  fi
  BODY="$BODY}"
  curl -sf -X POST "${KANBAN_URL}/api/batches" \
    -H "Content-Type: application/json" \
    -d "$BODY" 2>/dev/null | python3 -c "import sys,json; print(json.loads(sys.stdin.read()).get('id',''))" 2>/dev/null
}

kanban_create_ticket() {
  # Create a ticket linked to a batch. Args:
  #   $1=title, $2=batch_id, $3=dispatch_method (codex|direct),
  #   $4=ticket_number, $5=system (optional), $6=priority (optional, default "medium")
  local TITLE_JSON=$(echo "$1" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read().strip()))")
  local PRIORITY="${6:-medium}"
  local BODY="{\"title\": $TITLE_JSON, \"batch_id\": \"$2\", \"created_by\": \"claude_code\", \"dispatch_method\": \"$3\", \"ticket_number\": $4"
  if [ -n "$5" ]; then
    BODY="$BODY, \"system\": \"$5\""
  fi
  BODY="$BODY, \"priority\": \"$PRIORITY\"}"
  curl -sf -X POST "${KANBAN_URL}/api/tickets" \
    -H "Content-Type: application/json" \
    -d "$BODY" 2>/dev/null | python3 -c "import sys,json; print(json.loads(sys.stdin.read()).get('id',''))" 2>/dev/null
}

kanban_direct_start() {
  # Create a DIRECT ticket and immediately set to in_progress.
  # Args: $1=title, $2=batch_id, $3=ticket_number, $4=system (optional)
  local TID=$(kanban_create_ticket "$1" "$2" "direct" "$3" "$4")
  if [ -n "$TID" ]; then
    curl -sf -X PATCH "${KANBAN_URL}/api/tickets/$TID" \
      -H "Content-Type: application/json" \
      -d '{"status": "in_progress", "assignee": "claude_code"}' >/dev/null 2>&1
  fi
  echo "$TID"
}

kanban_direct_done() {
  # Mark a DIRECT ticket as done. Args: $1=ticket_id
  curl -sf -X PATCH "${KANBAN_URL}/api/tickets/$1" \
    -H "Content-Type: application/json" \
    -d '{"status": "done"}' >/dev/null 2>&1
}

kanban_direct_fail() {
  # Mark a DIRECT ticket as failed. Args: $1=ticket_id, $2=error_message
  local ERR=$(echo "$2" | python3 -c "import sys,json; print(json.dumps(sys.stdin.read().strip()))")
  curl -sf -X PATCH "${KANBAN_URL}/api/tickets/$1" \
    -H "Content-Type: application/json" \
    -d "{\"status\": \"failed\", \"error_message\": $ERR}" >/dev/null 2>&1
}
