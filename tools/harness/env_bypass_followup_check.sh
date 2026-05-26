#!/usr/bin/env bash
# ENV-BYPASS Follow-up Check (E Phase A — 2026-05-27)
#
# Per CLAUDE.md Rule 7.1 v3.2.1, every ENV-BYPASS commit MUST be
# followed within 7 days by a formal harness re-run that produces an
# APPROVE verdict, recorded as `verified-post-bypass-<commit>` in
# `.harness/audit/env_bypass.log`.
#
# This script scans the audit log and reports:
#   • Pending follow-ups (no `verified-post-bypass-*` line yet)
#   • Their age (days since the ENV-BYPASS landed)
#   • Whether the 7-day deadline is approaching or breached
#
# Exit codes:
#   0  — no pending follow-ups, OR all pending are within deadline
#   1  — one or more follow-ups are PAST the 7-day deadline
#   2  — invocation / parse error
#
# Usage:
#   bash tools/harness/env_bypass_followup_check.sh           # report all
#   bash tools/harness/env_bypass_followup_check.sh --strict  # exit 1 on breach
#
# Invoked at pipeline start (advisory) and available standalone.
set -o pipefail
# NOTE: -u (nounset) is not set because we parse pipe-separated log
# lines where the trailing `reason` field can be empty on some rows;
# `set -u` would trigger on the empty IFS-split var on macOS bash 3.2.

PROJECT_ROOT="$(git rev-parse --show-toplevel 2>/dev/null)"
if [[ -z "$PROJECT_ROOT" ]]; then
    printf '[env-bypass-followup] not in a git repository\n' >&2
    exit 2
fi

LOG="$PROJECT_ROOT/.harness/audit/env_bypass.log"
if [[ ! -f "$LOG" ]]; then
    # No log yet — nothing to follow up on.
    exit 0
fi

STRICT_MODE=0
if [[ "${1:-}" == "--strict" ]]; then
    STRICT_MODE=1
fi

# Pipeline-startup mode emits short single-line warnings; standalone
# mode emits a fuller report.
QUIET_MODE=0
if [[ "${1:-}" == "--quiet" || "${PIPELINE_BANNER_QUIET:-0}" == "1" ]]; then
    QUIET_MODE=1
fi

# Parse log entries. Each line is pipe-separated:
#   ISO8601-timestamp|feature|actor|reason
# Closure lines are:
#   ISO8601-timestamp|feature|verified-post-bypass-<commit>|note
#   ISO8601-timestamp|feature|LAND|<commit> — context
#
# A feature is "pending" if it has an ENV-BYPASS authorization entry
# (where actor != verified-post-bypass-* and actor != LAND) without a
# matching `verified-post-bypass-` entry for the SAME feature later
# in the log.
NOW_EPOCH=$(date +%s)
DEADLINE_SECS=$((7 * 24 * 60 * 60))

# Pass 1: collect authorization entries
declare -a AUTH_FEATURES=()
declare -a AUTH_TIMES=()
declare -a AUTH_REASONS=()
while IFS='|' read -r ts feature actor reason; do
    # Skip empty / malformed lines
    [[ -z "$ts" || -z "$feature" || -z "$actor" ]] && continue
    # Skip closure entries
    if [[ "$actor" == verified-post-bypass-* || "$actor" == "LAND" ]]; then
        continue
    fi
    AUTH_FEATURES+=("$feature")
    AUTH_TIMES+=("$ts")
    AUTH_REASONS+=("${reason:-}")
done < "$LOG"

# Pass 2: collect closure entries (feature names) into a newline list.
# (bash 3.2 has no associative arrays — use a sentinel-padded string
# and grep to query.)
CLOSED_LIST=""
while IFS='|' read -r ts feature actor reason; do
    [[ -z "$ts" || -z "$feature" || -z "$actor" ]] && continue
    if [[ "$actor" == verified-post-bypass-* ]]; then
        CLOSED_LIST="$CLOSED_LIST"$'\n'"$feature"
    fi
done < "$LOG"

is_closed() {
    local needle="$1"
    printf '%s\n' "$CLOSED_LIST" | grep -Fx -- "$needle" >/dev/null 2>&1
}

# Pass 3: report pending entries
PENDING_COUNT=0
BREACHED_COUNT=0
for i in "${!AUTH_FEATURES[@]}"; do
    feature="${AUTH_FEATURES[$i]}"
    ts="${AUTH_TIMES[$i]}"
    if is_closed "$feature"; then
        continue
    fi
    PENDING_COUNT=$((PENDING_COUNT + 1))
    # Parse ISO8601 timestamp → epoch
    ts_epoch=$(date -j -f "%Y-%m-%dT%H:%M:%SZ" "$ts" "+%s" 2>/dev/null \
        || date -d "$ts" "+%s" 2>/dev/null \
        || echo "$NOW_EPOCH")
    age_secs=$((NOW_EPOCH - ts_epoch))
    age_days=$((age_secs / 86400))
    remaining_secs=$((DEADLINE_SECS - age_secs))
    remaining_days=$((remaining_secs / 86400))
    status_emoji=""
    if [[ $age_secs -ge $DEADLINE_SECS ]]; then
        status_emoji="[BREACHED]"
        BREACHED_COUNT=$((BREACHED_COUNT + 1))
    elif [[ $remaining_days -le 1 ]]; then
        status_emoji="[≤24h]"
    else
        status_emoji="[OK]"
    fi
    if [[ $QUIET_MODE -eq 1 ]]; then
        # Single-line pipeline-startup warning
        printf '%s ENV-BYPASS pending: %s (age %dd, %dd remaining)\n' \
            "$status_emoji" "$feature" "$age_days" "$remaining_days" >&2
    else
        printf '%s feature=%s age=%dd remaining=%dd authorized=%s\n' \
            "$status_emoji" "$feature" "$age_days" "$remaining_days" "$ts"
    fi
done

if [[ $QUIET_MODE -eq 0 ]]; then
    printf '\nSummary: %d pending, %d breached (>7d).\n' \
        "$PENDING_COUNT" "$BREACHED_COUNT"
fi

if [[ $STRICT_MODE -eq 1 && $BREACHED_COUNT -gt 0 ]]; then
    exit 1
fi
exit 0
