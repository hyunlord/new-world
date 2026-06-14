#!/usr/bin/env bash
# Live measurement: run `claude --agent harness-generator` the EXACT pipeline way,
# background it, and probe the process tree / fds / syscalls during run + any hang.
# Discriminates H1 (claude exits, descendant lingers) vs H2 (claude itself won't exit).
set -u
OUT="$(cd "$(dirname "$0")" && pwd)"
MODE="${1:-tee}"            # tee | redir | devnull | light
CEIL="${2:-150}"            # manual ceiling (seconds), NOT 1800
PROMPTSEL="${3:-heavy}"     # heavy | bg  (bg = agent leaves a lingering child)
TAG="live_${MODE}"
LOG="$OUT/${TAG}.log"        # claude stdout/stderr capture
PROBE="$OUT/${TAG}.probe.txt"
: > "$PROBE"
say(){ echo "[$(date +%H:%M:%S)] $*" | tee -a "$PROBE"; }

# Mirror pipeline env (L306-307)
export CLAUDECODE=""
unset CLAUDE_CODE_ENTRYPOINT 2>/dev/null || true

run_with_timeout() {  # exact perl fallback copy
    local seconds=$1; shift
    perl -e '
        use POSIX ":sys_wait_h";
        my $deadline = shift @ARGV; my $pid = fork();
        if (not defined $pid) { die "fork failed: $!"; }
        if ($pid == 0) { exec @ARGV; die "exec failed: $!"; }
        $SIG{ALRM} = sub { kill "TERM",$pid; for(1..5){last if waitpid($pid,WNOHANG)>0; sleep 1;} kill "KILL",$pid; waitpid($pid,0); exit 142; };
        alarm $deadline; waitpid($pid,0); alarm 0; exit ($? >> 8);
    ' "$seconds" "$@"
}

PROMPT_LIGHT='Reply with exactly the word DONE and nothing else. Do not use any tools.'
PROMPT_HEAVY='You are doing a measurement probe. Do these steps with tools, then stop:
1. Use Bash to run: echo hello > /tmp/exithang_probe_file.txt
2. Use Bash to run: ls -la /tmp/exithang_probe_file.txt
3. Use Bash to run: cargo --version
4. Reply with exactly: GENERATOR_PROBE_DONE
Do not start any background processes. Keep it short.'
PROMPT_BG='You are doing a measurement probe of process cleanup. Do exactly this with the Bash tool, then stop:
1. Run this single Bash command EXACTLY (it starts a background process and returns immediately):
   nohup sleep 45 & echo "started bg child pid $!"
2. Confirm you saw the pid printed.
3. Reply with exactly: GENERATOR_PROBE_DONE
Do NOT redirect the background process output. Do NOT wait for the sleep. Run the command verbatim.'

case "$PROMPTSEL" in bg) PROMPT_HEAVY="$PROMPT_BG";; esac

launch() {
  case "$MODE" in
    light)
      run_with_timeout "$CEIL" claude -p "$PROMPT_LIGHT" --output-format text 2>&1 | tee "$LOG" >/dev/null
      echo "PIPESTATUS=${PIPESTATUS[0]}" ;;
    tee)
      run_with_timeout "$CEIL" claude --agent harness-generator -p "$PROMPT_HEAVY" \
        --dangerously-skip-permissions --output-format text 2>&1 | tee "$LOG" >/dev/null
      echo "PIPESTATUS=${PIPESTATUS[0]}" ;;
    redir)
      run_with_timeout "$CEIL" claude --agent harness-generator -p "$PROMPT_HEAVY" \
        --dangerously-skip-permissions --output-format text > "$LOG" 2>&1
      echo "RC=$?" ;;
    devnull)
      run_with_timeout "$CEIL" claude --agent harness-generator -p "$PROMPT_HEAVY" \
        --dangerously-skip-permissions --output-format text < /dev/null 2>&1 | tee "$LOG" >/dev/null
      echo "PIPESTATUS=${PIPESTATUS[0]}" ;;
  esac
}

say "MODE=$MODE CEIL=$CEIL — launching pipeline in background"
t0=$(date +%s)
( launch; echo "$(date +%s)" > "$OUT/${TAG}.exit_epoch" ) &
PIPE_BG=$!
say "pipeline bg pgid=$PIPE_BG"

# Probe loop
LAST_LOGSIZE=-1; WORKDONE_EPOCH=""; STABLE=0
while kill -0 "$PIPE_BG" 2>/dev/null; do
  now=$(date +%s); el=$((now-t0))
  sz=$( [ -f "$LOG" ] && wc -c < "$LOG" | tr -d ' ' || echo 0 )
  # detect "work done" = log mentions DONE marker OR log size stable for 3 probes
  grepdone=$(grep -c "GENERATOR_PROBE_DONE\|^DONE$\|DONE" "$LOG" 2>/dev/null | head -1 | tr -dc '0-9'); grepdone=${grepdone:-0}
  if [ "$sz" = "$LAST_LOGSIZE" ]; then STABLE=$((STABLE+1)); else STABLE=0; fi
  LAST_LOGSIZE=$sz
  if [ -z "$WORKDONE_EPOCH" ] && { [ "$grepdone" -gt 0 ] || [ "$STABLE" -ge 3 ]; }; then
    WORKDONE_EPOCH=$now
    say "WORK-DONE detected at el=${el}s (logsize=$sz, donemark=$grepdone) — process still in pipeline? probing tree…"
  fi
  # snapshot process tree every ~6s, and intensively once work appears done
  if [ $((el % 6)) -eq 0 ] || [ -n "$WORKDONE_EPOCH" ]; then
    {
      echo "===== el=${el}s logsize=$sz stable=$STABLE donemark=$grepdone ====="
      echo "-- perl wrappers --"; pgrep -fl "perl -e" | grep -v grep || echo none
      echo "-- claude (this probe's) procs --"; ps -eo pid,ppid,stat,etime,command | grep -E "claude (-p|--agent harness-generator)" | grep -v grep | grep -v live_probe || echo none
      # find the node claude pid for harness-generator
      CPID=$(pgrep -f "claude --agent harness-generator" | head -1)
      if [ -n "${CPID:-}" ]; then
        echo "-- descendants of claude pid=$CPID --"
        pgrep -P "$CPID" | while read -r c; do ps -o pid,ppid,stat,command -p "$c" | tail -1; pgrep -P "$c" | while read -r g; do ps -o pid,ppid,stat,command -p "$g" | tail -1; done; done
        echo "-- lsof: who holds the pipe / claude fds (first 25) --"
        lsof -p "$CPID" 2>/dev/null | grep -E "PIPE|FIFO|1[uw]|2[uw]" | head -25 || true
      else
        echo "(no claude --agent harness-generator process alive)"
      fi
      echo "-- any tee for this tag --"; pgrep -fl "tee $LOG" | grep -v grep || echo "none (tee gone or never)"
    } >> "$PROBE" 2>&1
  fi
  # once work looks done, capture a sample of claude to see blocking syscall
  if [ -n "$WORKDONE_EPOCH" ] && [ $((now-WORKDONE_EPOCH)) -ge 4 ]; then
    CPID=$(pgrep -f "claude --agent harness-generator" | head -1)
    if [ -n "${CPID:-}" ] && [ ! -f "$OUT/${TAG}.sample.txt" ]; then
      say "claude pid=$CPID still alive ${now}-${WORKDONE_EPOCH}=$((now-WORKDONE_EPOCH))s after work done → sampling syscalls"
      sample "$CPID" 3 -file "$OUT/${TAG}.sample.txt" 2>/dev/null || say "sample failed"
    fi
  fi
  sleep 2
done

t_end=$(date +%s)
EXIT_EPOCH=$(cat "$OUT/${TAG}.exit_epoch" 2>/dev/null || echo "$t_end")
say "PIPELINE RETURNED at el=$((t_end-t0))s"
if [ -n "$WORKDONE_EPOCH" ]; then
  say "COMPLETION→EXIT GAP = $((EXIT_EPOCH-WORKDONE_EPOCH))s  (workdone el=$((WORKDONE_EPOCH-t0))s, exit el=$((EXIT_EPOCH-t0))s)"
else
  say "work-done never separately detected (process exited promptly or before first stable window)"
fi
say "last log line: $(tail -1 "$LOG" 2>/dev/null)"
rm -f /tmp/exithang_probe_file.txt
say "=== $MODE done ==="
