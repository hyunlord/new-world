#!/bin/bash
# build.sh <tag>: on the DGX, build release worldsim_perf (plain and subprofile) from ~/worldsim-3d/perf2/repo
# into bins/<tag>/{plain,sub}; little cores 10-14, -j 5, waits while load >= 10.
set -uo pipefail
W=~/worldsim-3d/perf2; TAG=${1:?tag}; source ~/.cargo/env
waitload() { while [ $(cut -d. -f1 /proc/loadavg) -ge 10 ]; do sleep 30; done; }
cd $W/repo/rust
# Cargo.lock is gitignored: reuse the DGX lock of the content1 tree (same origin/v8/main deps).
cp $W/Cargo.lock.base Cargo.lock; sha256sum Cargo.lock
mkdir -p $W/bins/$TAG/plain $W/bins/$TAG/sub
waitload
CARGO_TARGET_DIR=$W/target taskset -c 10-14 nice cargo build --release -p sim-test --bin worldsim_perf -j 5 > $W/build-$TAG-plain.log 2>&1 || { echo BUILD_FAIL plain; tail -30 $W/build-$TAG-plain.log; exit 1; }
cp $W/target/release/worldsim_perf $W/bins/$TAG/plain/
waitload
CARGO_TARGET_DIR=$W/target-sub taskset -c 10-14 nice cargo build --release -p sim-test --features subprofile --bin worldsim_perf -j 5 > $W/build-$TAG-sub.log 2>&1 || { echo BUILD_FAIL sub; tail -30 $W/build-$TAG-sub.log; exit 1; }
cp $W/target-sub/release/worldsim_perf $W/bins/$TAG/sub/
sha256sum $W/bins/$TAG/*/worldsim_perf
echo BUILD_DONE
