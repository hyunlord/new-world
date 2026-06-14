#!/usr/bin/env bash
# Volume-stress probe: drive the harness-generator agent through MANY sequential
# tool calls over minutes (writes to /tmp only, never rust/) to reproduce the
# real generator's sustained-session conditions. If claude lingers post-work,
# sample + lsof the live hang to disambiguate H1 (descendant holds pipe) vs H2
# (claude-node itself blocked). NO rust/scripts edits. NO bypass.
set -u
OUT="$(cd "$(dirname "$0")" && pwd)"
CEIL="${1:-220}"
TAG="vol"; LOG="$OUT/${TAG}.log"; PROBE="$OUT/${TAG}.probe.txt"
: > "$PROBE"
say(){ echo "[$(date +%H:%M:%S)] $*" | tee -a "$PROBE"; }
export CLAUDECODE=""; unset CLAUDE_CODE_ENTRYPOINT 2>/dev/null || true
rm -f /tmp/volprobe_*.txt

run_with_timeout(){ local s=$1; shift; perl -e '
  use POSIX ":sys_wait_h"; my $d=shift @ARGV; my $pid=fork();
  if(!defined $pid){die "fork: $!";} if($pid==0){exec @ARGV; die "exec: $!";}
  $SIG{ALRM}=sub{kill "TERM",$pid; for(1..5){last if waitpid($pid,WNOHANG)>0; sleep 1;} kill "KILL",$pid; waitpid($pid,0); exit 142;};
  alarm $d; waitpid($pid,0); alarm 0; exit($?>>8);' "$s" "$@"; }
find_claude(){ local p c; for p in $(pgrep -f "claude --agent harness-generator" 2>/dev/null); do c=$(ps -o comm= -p "$p" 2>/dev/null); case "$c" in *perl*) ;; *node*|*claude*) echo "$p"; return;; esac; done; }
tree(){ local pid=$1 ind=$2; ps -o pid,ppid,stat,etime,comm -p "$pid" 2>/dev/null|tail -1|sed "s/^/$ind/"; for k in $(pgrep -P "$pid" 2>/dev/null); do tree "$k" "  $ind"; done; }

PROMPT='You are a measurement probe simulating a long generator session. Execute the following as MANY SEPARATE Bash tool calls — do NOT combine them into one command, run each as its own tool call so the session accumulates turns. After all of them, stop.
Run these 30 commands, each as a separate Bash tool call, in order:
echo step1 > /tmp/volprobe_1.txt ; cat /tmp/volprobe_1.txt
echo step2 > /tmp/volprobe_2.txt ; cat /tmp/volprobe_2.txt
echo step3 > /tmp/volprobe_3.txt ; cat /tmp/volprobe_3.txt
echo step4 > /tmp/volprobe_4.txt ; cat /tmp/volprobe_4.txt
echo step5 > /tmp/volprobe_5.txt ; cat /tmp/volprobe_5.txt
echo step6 > /tmp/volprobe_6.txt ; cat /tmp/volprobe_6.txt
echo step7 > /tmp/volprobe_7.txt ; cat /tmp/volprobe_7.txt
echo step8 > /tmp/volprobe_8.txt ; cat /tmp/volprobe_8.txt
echo step9 > /tmp/volprobe_9.txt ; cat /tmp/volprobe_9.txt
echo step10 > /tmp/volprobe_10.txt ; cat /tmp/volprobe_10.txt
echo step11 > /tmp/volprobe_11.txt ; ls -la /tmp/volprobe_*.txt | wc -l
echo step12 > /tmp/volprobe_12.txt ; date
echo step13 > /tmp/volprobe_13.txt ; uname -a
echo step14 > /tmp/volprobe_14.txt ; pwd
echo step15 > /tmp/volprobe_15.txt ; echo ok
echo step16 > /tmp/volprobe_16.txt ; echo ok
echo step17 > /tmp/volprobe_17.txt ; echo ok
echo step18 > /tmp/volprobe_18.txt ; echo ok
echo step19 > /tmp/volprobe_19.txt ; echo ok
echo step20 > /tmp/volprobe_20.txt ; echo ok
echo step21 > /tmp/volprobe_21.txt ; echo ok
echo step22 > /tmp/volprobe_22.txt ; echo ok
echo step23 > /tmp/volprobe_23.txt ; echo ok
echo step24 > /tmp/volprobe_24.txt ; echo ok
echo step25 > /tmp/volprobe_25.txt ; echo ok
echo step26 > /tmp/volprobe_26.txt ; echo ok
echo step27 > /tmp/volprobe_27.txt ; echo ok
echo step28 > /tmp/volprobe_28.txt ; echo ok
echo step29 > /tmp/volprobe_29.txt ; echo ok
echo step30 > /tmp/volprobe_30.txt ; echo ok
Then reply with exactly: GENERATOR_PROBE_DONE
Do NOT start background processes. Do NOT touch any file outside /tmp.'

say "VOLUME probe CEIL=$CEIL — launching (exact pipeline shape: run_with_timeout … | tee)"
t0=$(date +%s)
( run_with_timeout "$CEIL" claude --agent harness-generator -p "$PROMPT" \
    --dangerously-skip-permissions --output-format text 2>&1 | tee "$LOG" >/dev/null
  echo "PIPESTATUS=${PIPESTATUS[0]}" >> "$PROBE"; date +%s > "$OUT/${TAG}.exit_epoch" ) &
BG=$!
WORKDONE=""; LASTSTEP=0; SAMPLED=0
while kill -0 "$BG" 2>/dev/null; do
  now=$(date +%s); el=$((now-t0))
  steps=$(ls /tmp/volprobe_*.txt 2>/dev/null | wc -l | tr -d ' ')
  done=$(grep -c "GENERATOR_PROBE_DONE" "$LOG" 2>/dev/null | head -1 | tr -dc '0-9'); done=${done:-0}
  if [ "$steps" != "$LASTSTEP" ]; then say "progress el=${el}s files=$steps/30 logsize=$( [ -f "$LOG" ] && wc -c <"$LOG"|tr -d ' ' || echo 0 )"; LASTSTEP=$steps; fi
  # work done = all 30 files present (tool work finished); then watch for exit
  if [ -z "$WORKDONE" ] && [ "$steps" -ge 30 ]; then WORKDONE=$now; say "ALL 30 FILES WRITTEN (tool work done) el=${el}s — watching for process exit"; fi
  if [ -n "$WORKDONE" ] && [ $((now-WORKDONE)) -ge 8 ] && [ "$SAMPLED" -eq 0 ]; then
    CPID=$(find_claude)
    if [ -n "${CPID:-}" ]; then
      SAMPLED=1
      say "HANG CANDIDATE: node-claude pid=$CPID alive $((now-WORKDONE))s after tool work done → deep capture"
      {
        echo "##### VOLUME HANG CAPTURE el=${el}s, $((now-WORKDONE))s post-workdone #####"
        echo "--- tree from perl wrapper ---"; PERL=$(pgrep -f "claude --agent harness-generator"|while read -r p; do [ "$(ps -o comm= -p $p)" = perl ]&&echo $p; done|head -1); [ -n "${PERL:-}" ]&&tree "$PERL" ""||echo "(perl gone → claude exited, perl rc already returned)"
        echo "--- node-claude pid=$CPID descendants ---"; tree "$CPID" ""
        echo "--- lsof PIPES (who holds the tee pipe) ---"; lsof -p "$CPID" 2>/dev/null|grep -iE "pipe|fifo"|head -40
        echo "--- lsof SOCKETS ---"; lsof -nP -p "$CPID" 2>/dev/null|grep -iE "ipv4|ipv6|tcp|udp|unix"|head -40
        echo "--- sample node-claude 5s (BLOCKING SYSCALL) ---"; sample "$CPID" 5 2>/dev/null|sed -n '/Call graph/,/Binary Images/p'|head -60
        echo "--- any descendant holding the pipe? (lsof each child) ---"
        for c in $(pgrep -P "$CPID" 2>/dev/null); do echo "child $c:"; lsof -p "$c" 2>/dev/null|grep -iE "pipe|fifo"|head -5; done
      } >> "$PROBE" 2>&1
    fi
  fi
  sleep 2
done
t_end=$(date +%s); EX=$(cat "$OUT/${TAG}.exit_epoch" 2>/dev/null||echo "$t_end")
say "PIPELINE RETURNED el=$((t_end-t0))s  rc: $(grep PIPESTATUS "$PROBE"|tail -1)"
[ -n "$WORKDONE" ] && say "TOOL-WORK-DONE→EXIT GAP = $((EX-WORKDONE))s" || say "all-30-files never observed (agent combined calls or stalled early)"
say "last log: $(tail -2 "$LOG" 2>/dev/null|tr '\n' ' '|cut -c1-120)"
rm -f /tmp/volprobe_*.txt
say "=== volume probe done ==="