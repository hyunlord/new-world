#!/usr/bin/env bash
# Deep probe: correctly target the node-claude pid (NOT the perl wrapper), dump its
# full descendant tree + lsof (pipes/sockets/MCP children) at work-done and during
# any lingering window. Mirrors the exact pipeline invocation.
set -u
OUT="$(cd "$(dirname "$0")" && pwd)"
MODE="${1:-tee}"; CEIL="${2:-180}"
TAG="deep_${MODE}"; LOG="$OUT/${TAG}.log"; PROBE="$OUT/${TAG}.probe.txt"
: > "$PROBE"
say(){ echo "[$(date +%H:%M:%S)] $*" | tee -a "$PROBE"; }
export CLAUDECODE=""; unset CLAUDE_CODE_ENTRYPOINT 2>/dev/null || true

run_with_timeout(){ local s=$1; shift; perl -e '
  use POSIX ":sys_wait_h"; my $d=shift @ARGV; my $pid=fork();
  if(!defined $pid){die "fork: $!";} if($pid==0){exec @ARGV; die "exec: $!";}
  $SIG{ALRM}=sub{kill "TERM",$pid; for(1..5){last if waitpid($pid,WNOHANG)>0; sleep 1;} kill "KILL",$pid; waitpid($pid,0); exit 142;};
  alarm $d; waitpid($pid,0); alarm 0; exit($?>>8);' "$s" "$@"; }

# node-claude pid = child of the perl wrapper whose comm is node/claude (exclude perl itself)
find_claude(){
  local p
  for p in $(pgrep -f "claude --agent harness-generator" 2>/dev/null); do
    local c; c=$(ps -o comm= -p "$p" 2>/dev/null)
    case "$c" in *perl*) ;; *node*|*claude*) echo "$p"; return;; esac
  done
}
# recursive descendant lister
tree(){ local pid=$1 ind=$2; ps -o pid,ppid,stat,etime,comm -p "$pid" 2>/dev/null | tail -1 | sed "s/^/$ind/"; for k in $(pgrep -P "$pid" 2>/dev/null); do tree "$k" "  $ind"; done; }

PROMPT='Measurement probe. Use the Bash tool for these, in order, then stop:
1. cargo --version
2. echo probe > /tmp/exithang_deep.txt && cat /tmp/exithang_deep.txt
3. Reply with exactly: GENERATOR_PROBE_DONE
Do not start background processes.'

say "MODE=$MODE CEIL=$CEIL launching"
t0=$(date +%s)
( if [ "$MODE" = redir ]; then
    run_with_timeout "$CEIL" claude --agent harness-generator -p "$PROMPT" --dangerously-skip-permissions --output-format text > "$LOG" 2>&1
    echo "RC=$?" >> "$PROBE"
  else
    run_with_timeout "$CEIL" claude --agent harness-generator -p "$PROMPT" --dangerously-skip-permissions --output-format text 2>&1 | tee "$LOG" >/dev/null
    echo "PIPESTATUS=${PIPESTATUS[0]}" >> "$PROBE"
  fi
  date +%s > "$OUT/${TAG}.exit_epoch" ) &
BG=$!
DEEP_DONE=0
while kill -0 "$BG" 2>/dev/null; do
  now=$(date +%s); el=$((now-t0))
  done=$(grep -c "GENERATOR_PROBE_DONE" "$LOG" 2>/dev/null | head -1 | tr -dc '0-9'); done=${done:-0}
  CPID=$(find_claude)
  if [ "$done" -gt 0 ] && [ "$DEEP_DONE" -eq 0 ]; then
    DEEP_DONE=1
    say "WORK-DONE el=${el}s — deep snapshot. node-claude pid=${CPID:-GONE}"
    {
      echo "##### DEEP SNAPSHOT at work-done el=${el}s #####"
      echo "--- full process tree from perl wrapper ---"
      PERL=$(pgrep -f "claude --agent harness-generator" | while read -r p; do [ "$(ps -o comm= -p $p)" = perl ] && echo $p; done | head -1)
      [ -n "${PERL:-}" ] && tree "$PERL" "" || echo "(perl wrapper gone — claude already exited)"
      if [ -n "${CPID:-}" ]; then
        echo "--- node-claude pid=$CPID descendants ---"; tree "$CPID" ""
        echo "--- lsof node-claude: PIPES ---"; lsof -p "$CPID" 2>/dev/null | grep -iE "pipe|fifo" | head -30
        echo "--- lsof node-claude: SOCKETS (network/IP/unix) ---"; lsof -p "$CPID" 2>/dev/null | grep -iE "ipv4|ipv6|tcp|unix" | head -30
        echo "--- lsof node-claude: fd 0/1/2 ---"; lsof -p "$CPID" 2>/dev/null | awk 'NR==1||$4 ~ /^[012][rwu]/' | head -20
        echo "--- sample node-claude 3s ---"; sample "$CPID" 3 2>/dev/null | sed -n '/Call graph/,/Total number/p' | head -40
      fi
    } >> "$PROBE" 2>&1
  fi
  sleep 1
done
t_end=$(date +%s); EX=$(cat "$OUT/${TAG}.exit_epoch" 2>/dev/null || echo "$t_end")
say "PIPELINE RETURNED el=$((t_end-t0))s  rc-line: $(grep -E 'PIPESTATUS|RC=' "$PROBE" | tail -1)"
say "last log: $(tail -1 "$LOG" 2>/dev/null)"
rm -f /tmp/exithang_deep.txt
say "=== $MODE done ==="