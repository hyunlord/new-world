#!/usr/bin/env bash
# Mechanism isolation test — proves how `run_with_timeout CMD | tee LOG` behaves
# when CMD spawns a backgrounded grandchild that inherits fd 1/2 (stdout/stderr).
# No claude API used. Pure shell mechanics, mirroring harness_pipeline.sh L260-292 + L991.
set -u
OUT="$(cd "$(dirname "$0")" && pwd)"
LOG="$OUT/mech_result.txt"
: > "$LOG"
say(){ echo "$@" | tee -a "$LOG"; }

# Exact copy of harness_pipeline.sh run_with_timeout() (perl fallback branch only,
# since this host has no GNU timeout — verified).
run_with_timeout() {
    local seconds=$1; shift
    perl -e '
        use POSIX ":sys_wait_h";
        my $deadline = shift @ARGV;
        my $pid = fork();
        if (not defined $pid) { die "fork failed: $!"; }
        if ($pid == 0) { exec @ARGV; die "exec failed: $!"; }
        $SIG{ALRM} = sub {
            kill "TERM", $pid;
            for (1..5) { last if waitpid($pid, WNOHANG) > 0; sleep 1; }
            kill "KILL", $pid;
            waitpid($pid, 0);
            exit 142;
        };
        alarm $deadline;
        waitpid($pid, 0);
        alarm 0;
        exit ($? >> 8);
    ' "$seconds" "$@"
}

# A "worker" that does its work, writes its result file, prints output, EXITS —
# but first spawns a backgrounded grandchild that OUTLIVES it and inherits fd 1/2.
# This is the H1 shape: worker (=claude) finishes & exits; grandchild (=lingering
# MCP server / detached bg process) keeps the stdout/stderr pipe write-end open.
cat > "$OUT/worker.sh" <<'WORKER'
#!/usr/bin/env bash
# grandchild: sleeps holding inherited fd 1/2 (NO redirection — mirrors a bg
# process launched without </dev/null >/dev/null 2>&1)
sleep 30 &
echo "WORKER: work done, result written, exiting now (grandchild pid=$!)"
# worker exits immediately; grandchild keeps running
exit 0
WORKER
chmod +x "$OUT/worker.sh"

say "=== M3a: WITH '| tee' (current pipeline shape) ==="
say "Expect: worker exits in <1s, but tee blocks until grandchild dies (~30s)."
t0=$(date +%s)
run_with_timeout 120 "$OUT/worker.sh" 2>&1 | tee "$OUT/mech_tee.log" >/dev/null
rc=${PIPESTATUS[0]}
t1=$(date +%s)
say "M3a elapsed=$((t1-t0))s  rc=$rc  (if elapsed≈30 → tee waited for grandchild = H1 CONFIRMED)"
say ""

say "=== M3b: WITH '> file 2>&1' (no tee, redirect to file) ==="
say "Expect: same — a file redirect's write-end is ALSO inherited by grandchild,"
say "        so even '>' can block? Actually NO: '>' opens the file in the parent"
say "        shell; the grandchild inherits that fd but the SHELL does not wait on it."
t0=$(date +%s)
run_with_timeout 120 "$OUT/worker.sh" > "$OUT/mech_redir.log" 2>&1
rc=$?
t1=$(date +%s)
say "M3b elapsed=$((t1-t0))s  rc=$rc  (if elapsed<2 → redirect does NOT block = tee is the amplifier)"
say ""

say "=== M4: perl wrapper kills only \$pid, not the group (grandchild check) ==="
say "Launch worker under a SHORT 3s ceiling so ALRM fires; grandchild sleeps 30s."
cat > "$OUT/worker_slow.sh" <<'WS'
#!/usr/bin/env bash
setsid sleep 40 >/dev/null 2>&1 &   # detached grandchild in its OWN session
echo "WORKER_SLOW: spawned detached grandchild pid=$! ; now sleeping 40s myself"
sleep 40
WS
chmod +x "$OUT/worker_slow.sh"
t0=$(date +%s)
run_with_timeout 3 "$OUT/worker_slow.sh" > "$OUT/mech_kill.log" 2>&1
rc=$?
t1=$(date +%s)
say "M4 elapsed=$((t1-t0))s  rc=$rc  (rc=142 → perl SIGKILL fired at deadline)"
gc=$(pgrep -fl "sleep 40" | grep -v grep || true)
say "M4 surviving 'sleep 40' grandchildren after wrapper returned: ${gc:-NONE}"
pkill -f "sleep 40" 2>/dev/null || true
say ""
say "=== done ==="
