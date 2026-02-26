#!/bin/bash
# Pre-dispatch hook: auto-create kanban ticket before ask_codex/Task
# Reads JSON from stdin, creates ticket, saves ID for post-dispatch
# Exits 0 always — kanban failure must not block dispatch

KANBAN_URL="${KANBAN_URL:-http://localhost:8800}"
INPUT=$(cat)

# Extract fields from Claude Code hook input
TOOL_NAME=$(echo "$INPUT" | jq -r '.tool_name // ""' 2>/dev/null)
TOOL_USE_ID=$(echo "$INPUT" | jq -r '.tool_use_id // ""' 2>/dev/null)

[ -z "$TOOL_USE_ID" ] && exit 0

# Extract prompt content (structure depends on tool)
# ask_codex: tool_input.prompt or tool_input.message
# Task: tool_input.description or tool_input.prompt
PROMPT=$(echo "$INPUT" | jq -r '
  .tool_input.prompt //
  .tool_input.message //
  .tool_input.description //
  (.tool_input | tostring)
' 2>/dev/null | head -c 50000)

# Generate title from first line of prompt (truncate to 100 chars)
TITLE=$(echo "$PROMPT" | head -1 | cut -c1-100)
[ -z "$TITLE" ] && TITLE="${TOOL_NAME} dispatch"

# Check if kanban server is reachable
if ! curl -sf --max-time 3 "$KANBAN_URL/api/stats" > /dev/null 2>&1; then
  exit 0
fi

# Get or create active batch
BATCH_ID=$(curl -sf --max-time 5 "$KANBAN_URL/api/batches/active" 2>/dev/null | jq -r '.id // ""' 2>/dev/null)
if [ -z "$BATCH_ID" ]; then
  BATCH_ID=$(curl -sf --max-time 5 -X POST "$KANBAN_URL/api/batches" \
    -H "Content-Type: application/json" \
    -d "$(jq -n --arg t "Auto-batch $(date +%Y-%m-%d_%H:%M)" '{title: $t}')" \
    2>/dev/null | jq -r '.id // ""' 2>/dev/null)
fi

[ -z "$BATCH_ID" ] && exit 0

# Create ticket with body
TICKET_ID=$(curl -sf --max-time 5 -X POST "$KANBAN_URL/api/tickets" \
  -H "Content-Type: application/json" \
  -d "$(jq -n \
    --arg title "$TITLE" \
    --arg batch_id "$BATCH_ID" \
    --arg body "$PROMPT" \
    --arg tool "$TOOL_NAME" \
    '{
      title: $title,
      batch_id: $batch_id,
      body: $body,
      dispatch_method: "codex",
      created_by: "hook",
      system: $tool,
      status: "todo"
    }')" \
  2>/dev/null | jq -r '.id // ""' 2>/dev/null)

[ -z "$TICKET_ID" ] && exit 0

# Immediately mark in_progress
curl -sf --max-time 3 -X PATCH "$KANBAN_URL/api/tickets/$TICKET_ID" \
  -H "Content-Type: application/json" \
  -d '{"status":"in_progress","assignee":"codex"}' > /dev/null 2>&1 || true

# Save ticket+batch IDs for post-dispatch hook
jq -n --arg tid "$TICKET_ID" --arg bid "$BATCH_ID" \
  '{ticket_id: $tid, batch_id: $bid}' \
  > "/tmp/kanban-${TOOL_USE_ID}.json" 2>/dev/null || true

exit 0
