#!/usr/bin/env bash
# F2 verification (PHASE 2a): prove process-group kill reaps a SIGTERM-ignoring
# grandchild that the OLD single-pid kill left alive. Runs BOTH the pre-F2 and
# post-F2 perl handlers against an identical worker, under a 3s deadline.
set -u
OUT="$(cd "$(dirname "$0")" && pwd)"
LOG="$OUT/mech_f2_result.txt"; : > "$LOG"
say(){ echo "$@" | tee -a "$LOG"; }

# Worker: ignores SIGTERM, spawns a SIGTERM-ignoring grandchild (SAME process
# group — no setsid), both sleep 40s. Mirrors a claude child whose descendant
# ignores the polite signal and outlives the deadline. PER-PHASE marker passed
# as $1 so the two phases never cross-contaminate.
cat > "$OUT/f2_worker.sh" <<'WORKER'
#!/usr/bin/env bash
M="$1"
trap '' TERM
( trap '' TERM; exec -a "${M}_grandchild" sleep 40 ) &
echo "worker: grandchild pid=$! (marker ${M}_grandchild)"
exec -a "${M}_worker" sleep 40
WORKER
chmod +x "$OUT/f2_worker.sh"
MARK_PRE="f2pre_$$"; MARK_POST="f2post_$$"

# --- PRE-F2 handler: single-pid kill (the OLD code) ---
run_pre_f2() { local s=$1; shift; perl -e '
  use POSIX ":sys_wait_h"; my $d=shift @ARGV; my $pid=fork();
  if(!defined $pid){die "fork: $!";} if($pid==0){exec @ARGV; die "exec: $!";}
  $SIG{ALRM}=sub{ kill "TERM",$pid; for(1..5){last if waitpid($pid,WNOHANG)>0; sleep 1;} kill "KILL",$pid; waitpid($pid,0); exit 142; };
  alarm $d; waitpid($pid,0); alarm 0; exit($?>>8);' "$s" "$@"; }

# --- POST-F2 handler: process-group kill (the NEW code, copied from harness_pipeline.sh) ---
run_post_f2() { local s=$1; shift; perl -e '
  use POSIX ":sys_wait_h"; my $d=shift @ARGV; my $pid=fork();
  if(!defined $pid){die "fork: $!";} if($pid==0){ setpgrp(0,0); exec @ARGV; die "exec: $!";}
  $SIG{ALRM}=sub{ kill "TERM",-$pid; for(1..5){last if waitpid($pid,WNOHANG)>0; sleep 1;} kill "KILL",-$pid; waitpid($pid,0); exit 142; };
  alarm $d; waitpid($pid,0); alarm 0; exit($?>>8);' "$s" "$@"; }

survivors(){ pgrep -fl "$1" | grep -v grep; }

say "=== PRE-F2 (single-pid kill) ==="
run_pre_f2 3 "$OUT/f2_worker.sh" "$MARK_PRE" > "$OUT/f2_pre.log" 2>&1; say "rc=$? (142=deadline fired)"
sleep 2
PRE=$(survivors "${MARK_PRE}_"); say "survivors after PRE-F2 deadline:"; echo "${PRE:-NONE}" | tee -a "$LOG"
pkill -9 -f "${MARK_PRE}_" 2>/dev/null; sleep 1

say ""
say "=== POST-F2 (process-group kill) ==="
run_post_f2 3 "$OUT/f2_worker.sh" "$MARK_POST" > "$OUT/f2_post.log" 2>&1; say "rc=$? (142=deadline fired)"
sleep 2
POST=$(survivors "${MARK_POST}_"); say "survivors after POST-F2 deadline:"; echo "${POST:-NONE}" | tee -a "$LOG"
pkill -9 -f "${MARK_POST}_" 2>/dev/null; sleep 1

say ""
PRE_N=$(echo "$PRE" | grep -c . ); [ -z "$PRE" ] && PRE_N=0
POST_N=$(echo "$POST" | grep -c . ); [ -z "$POST" ] && POST_N=0
say "VERDICT: pre-F2 survivors=$PRE_N (expect >=1: grandchild leaks)  post-F2 survivors=$POST_N (expect 0: group reaped)"
if [ "$PRE_N" -ge 1 ] && [ "$POST_N" -eq 0 ]; then say "F2 PASS — group-kill reaps the grandchild the single-pid kill leaked"; else say "F2 CHECK — review numbers above"; fi
say ""
say "NOTE: a grandchild that calls setsid() to start its OWN session escapes the"
say "group and would still survive (PHASE-1 M4 used setsid). Real claude descendants"
say "do not setsid, so F2 covers them. setsid-escape remains a known edge case."