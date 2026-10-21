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
| 2026-10-22 | 3183e77d | base M1-ART | 200 | n200 | 355.7 | 778 | 1345 | 1993 | 81.5 | 6.21 8.79 9.66; 8.30 8.95 9.66; 6.52 8.29 9.36 | M1.5 base line, measured with the M1.5 benchmark patched into 3183e77d (DECISIONS D3); session 00:32–00:35 KST 2026-10-22, 1-min load 6.2–8.3 (shared Mac; interleaved base/candidate, 3 rounds) |
| 2026-10-22 | 3183e77d | base M1-ART | 1000 | n1000 | 1790.7 | 3852 | 7646 | 1993 | 81.5 | 6.21 8.79 9.66; 8.04 8.89 9.64; 6.52 8.29 9.36 | M1.5 base line, measured with the M1.5 benchmark patched into 3183e77d (DECISIONS D3); session 00:32–00:35 KST 2026-10-22, 1-min load 6.2–8.3 (shared Mac; interleaved base/candidate, 3 rounds) |
| 2026-10-22 | 3183e77d | base M1-ART | 2000 | n2000 | 4277.0 | 8631 | 18148 | 1993 | 81.5 | 7.31 8.94 9.71; 7.79 8.82 9.61; 6.66 8.26 9.34 | M1.5 base line, measured with the M1.5 benchmark patched into 3183e77d (DECISIONS D3); session 00:32–00:35 KST 2026-10-22, 1-min load 6.2–8.3 (shared Mac; interleaved base/candidate, 3 rounds) |
| 2026-10-22 | 3183e77d | base M1-ART | 200 | n200_script | 389.5 | 834 | 1238 | 1993 | 81.5 | 7.48 8.90 9.68; 7.27 8.64 9.52; 7.46 8.34 9.34 | M1.5 base line, measured with the M1.5 benchmark patched into 3183e77d (DECISIONS D3); session 00:32–00:35 KST 2026-10-22, 1-min load 6.2–8.3 (shared Mac; interleaved base/candidate, 3 rounds) |
| 2026-10-22 | 3183e77d | base M1-ART | 200 | n200_side64 | 189.6 | 344 | 614 | 1993 | 81.5 | 7.36 8.85 9.65; 6.93 8.55 9.49; 7.46 8.34 9.34 | M1.5 base line, measured with the M1.5 benchmark patched into 3183e77d (DECISIONS D3); session 00:32–00:35 KST 2026-10-22, 1-min load 6.2–8.3 (shared Mac; interleaved base/candidate, 3 rounds) |
| 2026-10-22 | d5b5b34b | M1.5 | 200 | n200 | 357.7 | 775 | 1248 | 2148 | 82.0 | 7.36 8.85 9.65; 6.93 8.55 9.49; 7.46 8.34 9.34 | M1.5 code d5b5b34b (calendar, fertility, spatial index, save/load; the simulation of the final commit); same session as the base line; ceilings 4/4 PASS |
| 2026-10-22 | d5b5b34b | M1.5 | 1000 | n1000 | 1787.0 | 3848 | 7719 | 2148 | 82.0 | 7.36 8.85 9.65; 6.93 8.55 9.49; 7.50 8.33 9.33 | M1.5 code d5b5b34b (calendar, fertility, spatial index, save/load; the simulation of the final commit); same session as the base line; ceilings 4/4 PASS |
| 2026-10-22 | d5b5b34b | M1.5 | 2000 | n2000 | 4291.1 | 8445 | 40298 | 2148 | 82.0 | 7.82 8.90 9.66; 6.79 8.46 9.44; 7.22 8.26 9.30 | M1.5 code d5b5b34b (calendar, fertility, spatial index, save/load; the simulation of the final commit); same session as the base line; ceilings 4/4 PASS |
| 2026-10-22 | d5b5b34b | M1.5 | 200 | n200_script | 382.1 | 792 | 1632 | 2148 | 82.0 | 8.30 8.95 9.66; 6.52 8.29 9.36; 7.10 8.16 9.24 | M1.5 code d5b5b34b (calendar, fertility, spatial index, save/load; the simulation of the final commit); same session as the base line; ceilings 4/4 PASS |
| 2026-10-22 | d5b5b34b | M1.5 | 200 | n200_side64 | 185.8 | 332 | 626 | 2148 | 82.0 | 8.30 8.95 9.66; 6.52 8.29 9.36; 7.01 8.13 9.22 | M1.5 code d5b5b34b (calendar, fertility, spatial index, save/load; the simulation of the final commit); same session as the base line; ceilings 4/4 PASS |
