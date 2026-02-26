#!/bin/bash
# Post-dispatch hook: auto-update kanban ticket after ask_codex/Task completes
# Reads JSON from stdin, reads ticket ID from temp file, updates status
# Exits 0 always — non-blocking

KANBAN_URL="${KANBAN_URL:-http://localhost:8800}"
INPUT=$(cat)

TOOL_USE_ID=$(echo "$INPUT" | jq -r '.tool_use_id // ""' 2>/dev/null)
[ -z "$TOOL_USE_ID" ] && exit 0

TEMP_FILE="/tmp/kanban-${TOOL_USE_ID}.json"
[ ! -f "$TEMP_FILE" ] && exit 0

TICKET_ID=$(jq -r '.ticket_id // ""' "$TEMP_FILE" 2>/dev/null)
if [ -z "$TICKET_ID" ]; then
  rm -f "$TEMP_FILE"
  exit 0
fi

# Determine success/failure from tool_response
SUCCESS=$(echo "$INPUT" | jq -r '
  if .tool_response.success == false then "failed"
  elif .tool_response.error then "failed"
  elif (.tool_response | tostring | test("\"error\"|\"Error\"|\"FAIL\""; "")) then "failed"
  else "done"
  end
' 2>/dev/null || echo "done")

if [ "$SUCCESS" = "done" ]; then
  curl -sf --max-time 5 -X PATCH "$KANBAN_URL/api/tickets/$TICKET_ID" \
    -H "Content-Type: application/json" \
    -d '{"status":"done"}' > /dev/null 2>&1 || true
else
  ERROR_MSG=$(echo "$INPUT" | jq -r '
    .tool_response.error //
    .tool_response.message //
    (.tool_response | tostring | .[0:500])
  ' 2>/dev/null || echo "Unknown error")
  curl -sf --max-time 5 -X PATCH "$KANBAN_URL/api/tickets/$TICKET_ID" \
    -H "Content-Type: application/json" \
    -d "$(jq -n --arg err "$ERROR_MSG" '{status:"failed", error_message: $err}')" \
    > /dev/null 2>&1 || true
fi

rm -f "$TEMP_FILE"
exit 0
