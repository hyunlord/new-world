## Timed batches and their start loads

Section 1: a timed batch starts only when the 1-minute load average is < 4 and no Godot / cargo / rustc / worldsim process runs; otherwise it waits (10-minute checks) and after 2 hours runs anyway, `contended`. The launcher (`tools/revitalize/perf_batch_p15.sh`) logs every check to `/Users/rexxa/work/p15-data/batches.log`. The load came from foreign processes this run did not start.

| batch / sweep | gate passed (KST) | 1-min load at start | JSON `contended` | contended in section 1's sense |
|---|---|---:|---|---|
| overhead (Part 2) | 00:14:45 (after 30 min) | 3.33 | false | no |
| Part 3 batch: natural | 00:15:26 | 2.95 | false | no |
| Part 3 batch: fed (25–2,000) | (same gate) 00:22:49 | 13.41 | false | **yes** — started inside the one-gate batch above the limit (review round 1, M1) |
| Part 3 batch: repeatability ×3 | (same gate) 00:23:55 | 9.85 | false | **yes** |
| Part 3 batch: raw (25–2,000) | (same gate) 00:23:59 | 9.85 | false | **yes** |
| Part 3 batch: map | (same gate) 00:24:58 | 7.02 | false | **yes** |
| supplementary fed 3,000–8,000 | 01:25:53 (after 60 min) | 2.09 | false | no |
| recheck fed n = 500 (M1) | 01:43:44 (after 10 min) | 2.94 | false | no |
| repeatability ×3 (M1) | 01:43:51 | 3.57 | false | no |

The recheck reproduces the contended fed n = 500 runs closely (median mean 0.2971 vs 0.3034 ms, −2.1 %; pooled p99.9 0.608 vs 0.634 ms; identical end fingerprints on all eight seeds). The gated repeats (0.3084, 0.2987, 0.2998 ms; CV 1.8 %) are the repeatability figure; the contended ones (CV 0.5 %) are kept for reference.
