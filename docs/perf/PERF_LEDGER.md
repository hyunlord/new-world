# Performance ledger (ADR 0016 C3)

One block of rows per pass, appended by `tools/v8/bench_ledger.py append` from a run of the standing benchmark
(`worldsim_perf bench`, `sim_test::bench`; method in M1.5 DECISIONS D10): the colony world 192 × 192 (seed
15100, `WorldGenConfig::colony_stress`, fed as `worldsim_perf quick` is) at N = 200 / 1,000 / 2,000 without orders,
N = 200 with the scaled U-script (`tools/v8/bench_script.json`), and N = 200 on 64 × 64 (for the per-tile memory
figure); 1,440 warm-up ticks, then 2,880 profiled ticks; release build; medians of 3 repeats. Times are µs per
tick (`tick_profiled`). RSS/agent is the slope of the resident size after `build_world_with` between N = 200 and
2,000; RSS/tile the difference between 192² and 64² at N = 200. The load column holds the host's 1/5/15-minute
load averages before each repeat (the Mac is shared: a contended session is said so in the note).

The gate step `bench` (`tools/v8/bench.sh`) compares a fresh run with the latest pass's rows: more than +10 % on
a scene's tick mean or p99, or on RSS/agent, fails unless the pass's CHANGELOG justifies it (`--justified`).
C3's ceilings (N = 1,000 mean ≤ 5 ms, N = 2,000 ≤ 10 ms, ≤ 4 KiB per agent, ≤ 96 B per tile) are reported PASS/MISS
and do not fail the gate in M1.5.

| date | commit | pass | N | scene | tick mean (µs) | p99 (µs) | max (µs) | RSS/agent (B) | RSS/tile (B) | load (1/5/15 min) | note |
|---|---|---|---|---|---|---|---|---|---|---|---|
