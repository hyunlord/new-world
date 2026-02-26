#!/bin/bash
# Session end hook: finalize batches, clean up orphaned temp files
# Exits 0 always

KANBAN_URL="${KANBAN_URL:-http://localhost:8800}"

# Clean up any orphaned temp files from this session
rm -f /tmp/kanban-*.json 2>/dev/null

# Check server reachability
if ! curl -sf --max-time 3 "$KANBAN_URL/api/stats" > /dev/null 2>&1; then
  exit 0
fi

# Touch all active batches to trigger recalculation
ACTIVE_BATCHES=$(curl -sf --max-time 5 "$KANBAN_URL/api/batches?status=active&limit=10" \
  2>/dev/null | jq -r '.batches[].id' 2>/dev/null)

for BID in $ACTIVE_BATCHES; do
  curl -sf --max-time 3 -X PATCH "$KANBAN_URL/api/batches/$BID" \
    -H "Content-Type: application/json" \
    -d '{}' > /dev/null 2>&1 || true
done

exit 0
