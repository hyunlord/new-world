#!/usr/bin/env bash
# F1 + S0 verification (PHASE 2a). Replicates the EXACT deployed blocks
# (run_with_timeout post-F2, _exithang_watchdog/_exithang_find_claude, and the
# F1 redirect + `|| gen_rc=$?` capture) under `set -euo pipefail` like the real
# script. Two checks:
#   A) stub commands → timeout branch reachable (gen_rc=142) + fast path (gen_rc=0), log written
#   B) real light `claude --agent harness-generator -p "reply DONE"` → clean exit,
#      log non-empty, gen_rc=0, watchdog torn down, outcome.txt written
set -euo pipefail
PROJECT_ROOT="$(cd "$(dirname "$0")/../../../.." && pwd)"
OUT="$(cd "$(dirname "$0")" && pwd)"
RESULT_DIR="$OUT/verify_rd"; mkdir -p "$RESULT_DIR"
LOG="$OUT/verify_f1_s0_result.txt"; : > "$LOG"
say(){ echo "$@" | tee -a "$LOG"; }
export CLAUDECODE=""; unset CLAUDE_CODE_ENTRYPOINT 2>/dev/null || true

# --- exact post-F2 run_with_timeout ---
run_with_timeout() {
    local seconds=$1; shift
    if command -v timeout >/dev/null 2>&1; then timeout "$seconds" "$@"; else
        perl -e '
            use POSIX ":sys_wait_h"; my $deadline=shift @ARGV; my $pid=fork();
            if(not defined $pid){die "fork failed: $!";} if($pid==0){ setpgrp(0,0); exec @ARGV; die "exec failed: $!";}
            $SIG{ALRM}=sub{ kill "TERM",-$pid; for(1..5){last if waitpid($pid,WNOHANG)>0; sleep 1;} kill "KILL",-$pid; waitpid($pid,0); exit 142; };
            alarm $deadline; waitpid($pid,0); alarm 0; exit($?>>8);' "$seconds" "$@"
    fi
}
# --- exact deployed watchdog fns ---
_exithang_find_claude() { set +e; local p c; for p in $(pgrep -f 'claude --agent harness-generator' 2>/dev/null); do c=$(ps -o comm= -p "$p" 2>/dev/null); case "$c" in *perl*) ;; *node*|*claude*) echo "$p"; return 0;; esac; done; }
_exithang_watchdog() {
    set +e; local capture_dir="$1" log_file="$2" deadline="$3"; local started captured=0 seen=0 now cpid last_act ch
    started=$(date +%s)
    while :; do
        sleep 10; now=$(date +%s); cpid=$(_exithang_find_claude)
        if [[ -n "$cpid" ]]; then seen=1; else
            if [[ $seen -eq 1 ]]; then echo "fast_completion elapsed=$(( now - started ))s log_bytes=$(wc -c < "$log_file" 2>/dev/null || echo NA)" > "$capture_dir/outcome.txt"; return 0; fi
            continue; fi
        last_act=$(find "$PROJECT_ROOT/rust" -type f -newermt "@$(( now - 90 ))" 2>/dev/null | head -1)
        if [[ $captured -eq 0 && -z "$last_act" && $(( now - started )) -gt 120 ]]; then
            captured=1; echo "CAPTURE(stub-or-real) elapsed=$(( now-started ))s cpid=$cpid" > "$capture_dir/capture.txt"
        fi
    done
}

# ---------- CHECK A: stub timeout-branch reachability ----------
say "=== CHECK A1: stub that exceeds 3s deadline → expect gen_rc=142, branch reachable ==="
attempt=A1
gen_rc=0
run_with_timeout 3 bash -c 'sleep 10' > "$RESULT_DIR/generator_log_attempt${attempt}.txt" 2>&1 || gen_rc=$?
say "  gen_rc=$gen_rc  (expect 142)"
if [[ $gen_rc -eq 124 || $gen_rc -eq 142 ]]; then say "  TIMEOUT BRANCH REACHED ✓ (would write GENERATOR_TIMEOUT marker + die)"; else say "  ✗ branch NOT reached"; fi

say "=== CHECK A2: stub that prints + exits 0 → expect gen_rc=0, log non-empty ==="
attempt=A2; gen_rc=0
run_with_timeout 10 bash -c 'echo HELLO_FROM_STUB; exit 0' > "$RESULT_DIR/generator_log_attempt${attempt}.txt" 2>&1 || gen_rc=$?
say "  gen_rc=$gen_rc (expect 0); log='$(cat "$RESULT_DIR/generator_log_attempt${attempt}.txt")' bytes=$(wc -c < "$RESULT_DIR/generator_log_attempt${attempt}.txt"|tr -d ' ')"

# ---------- CHECK B: real light claude + watchdog teardown ----------
say ""
say "=== CHECK B: real light claude --agent harness-generator -p 'reply DONE' (F1 redirect + S0 watchdog) ==="
attempt=B
cap="$OUT/verify_capture_$(date +%H%M%S)"; mkdir -p "$cap"
_exithang_watchdog "$cap" "$RESULT_DIR/generator_log_attempt${attempt}.txt" 60 &
wd_pid=$!
gen_rc=0
run_with_timeout 60 \
    claude --agent harness-generator \
        -p 'Reply with exactly the word DONE and nothing else. Do not use any tools.' \
        --dangerously-skip-permissions --output-format text \
        > "$RESULT_DIR/generator_log_attempt${attempt}.txt" 2>&1 || gen_rc=$?
kill "$wd_pid" 2>/dev/null || true; wait "$wd_pid" 2>/dev/null || true
[[ -f "$cap/outcome.txt" ]] || echo "completed gen_rc=$gen_rc log_bytes=$(wc -c < "$RESULT_DIR/generator_log_attempt${attempt}.txt" 2>/dev/null || echo NA)" > "$cap/outcome.txt"
say "  gen_rc=$gen_rc (expect 0)"
say "  log bytes=$(wc -c < "$RESULT_DIR/generator_log_attempt${attempt}.txt" | tr -d ' ') last='$(tail -1 "$RESULT_DIR/generator_log_attempt${attempt}.txt" 2>/dev/null)'"
say "  watchdog outcome: $(cat "$cap/outcome.txt" 2>/dev/null)"
LEAK=$(pgrep -f _exithang_watchdog | grep -v grep || true)
say "  lingering _exithang_watchdog procs: ${LEAK:-NONE ✓}"
say ""
say "=== F1+S0 verdict ==="
say "  A1 timeout branch reachable, A2 fast path rc0+log, B real-claude rc0+nonempty-log+watchdog-torn-down+outcome → see above"
rm -rf "$RESULT_DIR"