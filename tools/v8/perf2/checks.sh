#!/bin/bash
# checks.sh: fmt, clippy (plain and subprofile) and the touched tests for the PERF2-0 harness on the DGX.
W=~/worldsim-3d/perf2; source ~/.cargo/env; cd $W/repo/rust; cp $W/Cargo.lock.base Cargo.lock
waitload() { while [ $(cut -d. -f1 /proc/loadavg) -ge 10 ]; do sleep 30; done; }
{
waitload
cargo fmt --all -- --check; echo "fmt exit $?"
CARGO_TARGET_DIR=$W/target-dev taskset -c 10-14 nice cargo clippy -j 5 -p sim-core -p sim-systems -p sim-test --all-targets -- -D warnings 2>&1 | tail -15; echo "clippy plain exit ${PIPESTATUS[0]}"
CARGO_TARGET_DIR=$W/target-dev taskset -c 10-14 nice cargo clippy -j 5 -p sim-test --features subprofile --all-targets -- -D warnings 2>&1 | tail -15; echo "clippy sub exit ${PIPESTATUS[0]}"
CARGO_TARGET_DIR=$W/target-dev taskset -c 10-14 nice cargo test -j 5 -p sim-core --lib nav 2>&1 | grep -E "^test result|FAILED|panicked" ; echo "test sim-core nav exit ${PIPESTATUS[0]}"
CARGO_TARGET_DIR=$W/target-dev taskset -c 10-14 nice cargo test -j 5 -p sim-test --lib -- perf2 alloc perf:: 2>&1 | grep -E "^test result|FAILED|panicked" ; echo "test sim-test exit ${PIPESTATUS[0]}"
CARGO_TARGET_DIR=$W/target-dev taskset -c 10-14 nice cargo test -j 5 -p sim-test --bin worldsim_perf 2>&1 | grep -E "^test result|FAILED" ; echo "test worldsim_perf bin exit ${PIPESTATUS[0]}"
echo CHECKS_DONE
} > $W/checks.log 2>&1
