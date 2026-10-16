# Stage A evaluation cohort audit

**PASS — completed v21/v20 cohort completeness and recorded provenance only.** This is not a Stage A metric verdict or a Part 8 fresh review.

Audit UTC: `2026-10-15T09:20:50.162146Z`. Read-only file inspection, hashing, and Git checks only; no probes, builds, or evaluators were started. I5 contents and status were not audited.

Raw root: `/Users/rexxa/work/p17-data/a-evaluation-observerfix`. Summary copies: `/Users/rexxa/work/new-world-pass4/docs/revitalize/probe/p17-a_eval`.

## Complete cohorts

| Scene | Unique seeds | Days per run | Trace end tick | Raw seed files | Exit | Determinism | Runtime seconds |
| --- | --- | --- | --- | --- | --- | --- | --- |
| game_v21 | 32: 17000–17031 | 120 | 172800 | 256 | 0 | PASS | 1113.886764 |
| game_v20 | 32: 17000–17031 | 120 | 172800 | 256 | 0 | PASS | 447.395394 |

Both schema-13 summaries have exactly the expected seeds, `[17000,17032)` seed metadata, and day keys 1–120. Every run has 120 entries in each of alive_daily, settlements_daily, and settlements_alive_daily. Each log independently lists the same 32 unique seeds and ends with DETERMINISM PASS; both summaries set determinism_checked=true. Determinism is the retained probe twin-check result, not a new simulation rerun.

All 64 trace metadata files carry the correct scene, seed and producer SHA, Pass 17 schema 17, and end_tick=172800 (120 × 1440). Each scene has eight expected artifacts per seed: main, agents, deaths, fissions, hours, people, routine JSONL and trace metadata JSON, plus summary.json and summary.md.

| Scene | Raw seed artifact bytes | Born-event misses | Component mismatches | Observation misses |
| --- | --- | --- | --- | --- |
| game_v21 | 1413049654 | 0 | 0 | 0 |
| game_v20 | 430364761 | 0 | 0 | 0 |

## Pre-registration and exact UTC order

- Pre-registration and launch commit: `000d3706c6463dd19cd6b1efce20ae60a7acaa9f`.
- Producer commit: `50a73474e06d3a9abbb535729e9c7e1e134954b7`.
- Git commit timestamp: `2026-10-15T08:35:22Z` (epoch 1792053322; committer offset +09:00).
- Receipt validation: `2026-10-15T08:35:38.787655Z`.
- First recorded probe launch boundary: `2026-10-15T08:35:38.982376Z` — **16.982376 seconds after the pre-registration commit**.
- game_v21: `2026-10-15T08:35:38.982376Z` through `2026-10-15T08:54:12.869140Z`.
- game_v20: `2026-10-15T08:54:12.993499Z` through `2026-10-15T09:01:40.388893Z`.

Git confirms both producer and pre-registration are ancestors of the recorded launch commit. Pre-registration equals launch. The producer-to-launch Rust diff is empty. The committed METRICS.md contains Pass 17 Stage A pre-registration and exactly one row each for R1, R2, R3-P, R3-NS, R3-RD, R4 and R5.
Committed METRICS SHA-256: `0dca789ef9431d25570a420b26142ce47767d180fd2cb28dfe789bb3f037f70b`, matching the receipt. The retained remote-head snapshot names the same preregistration commit for codex/revitalize-2026-10-wip; no new remote query was made.

The inspected runner writes its start record before invoking each command. The recorded start is a pre-exec boundary, not a kernel process-creation timestamp; under this ordering the actual command cannot precede it. The v21 completion precedes v20 launch.

## Commands and retained build

Both parsed argument vectors exactly match the retained worldsim_probe, scene, seeds 17000..17032, days 120, sample-every 120, all 24 hourly clock samples, workers 4, check-determinism, agent-trace, and separate scene output directories.
Frozen binary directory: `/Users/rexxa/work/p17-data/a-observerfix-bin`. Raw and frozen source receipts equal the producer. The actual five executable hashes match both manifests; the first four binary-check records cover before/after each completed cohort.

| Binary | SHA-256 |
| --- | --- |
| worldsim_probe | `7a3808ee397b254841ef811f50798332c777be2abb184a89dc906dfa1cafa0f4` |
| worldsim_wm | `596dfc81409e7ce6d0008a97e49dfa87088789a5afea05cb763620e66ce3de21` |
| worldsim_perf | `2e36152b1c8961e66a2fbfe0a3c200cc89a4225f561e60e3b99e030e0793ac38` |
| v19_pins | `ae879eed1dc80bfbb47851f4ed244fa095e43f57d8906dd9625513ebb0b0a905` |
| v20_pins | `0f4728c23924ba5e412eb3bdf90d62021bb07d285acc0fbe79c22a03a436ac5d` |

## Artifact hashes

| Artifact | SHA-256 |
| --- | --- |
| raw/preregistration.json | `a35f42b696f7242c6f1baee316741cc6663a3dc1b7a5c523360d95245b4e23d5` |
| raw/source_commit.txt | `5bd1824aabc2b952baac42b54b11ab5582ecceaff10ee49df52e22e43e241baa` |
| raw/binary_directory.txt | `387c6c1c97b62b36bfb3ae3f4d576c54ec9491236cc15470d45737f188276f32` |
| raw/binaries.sha256 | `a41a7866f6ebef94ca2618e0ec695974f864b0a2440069dcbe13159476f91e70` |
| raw/preregistration_remote_heads.txt | `92d1ac22b76e32630d2d175860276bb361d5f96776bd5c416e15ef25ac5212e5` |
| game_v21: summary.json | `4770decf4b6f9c89ae429343660703adb2437601c644771f1b4e28ec23de6cbc` |
| game_v21: process log | `ce21ed687e34127ed236b0d2ed5db41d559035fcba6e4bc6fc1814d92657886a` |
| game_v21: exit receipt | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| game_v21: 32 metadata hash records | `405c52bfa3a04675117d95f4d2d16a6a9c1cf4ccf8056820f841b81f808f9f38` |
| game_v20: summary.json | `cdbd194b0c8ffa9676784c45bbd48c9ec64dcc9b40c6c9ae7bbd78fa017fa512` |
| game_v20: process log | `b75e1f810f8fb1d57a813117ae0de1a4a00596c216cb43b9577dff3165502fd3` |
| game_v20: exit receipt | `9a271f2a916b0b6ee6cecb2426f0b3206ef074578be55d9bc94f6f3fe3ab86aa` |
| game_v20: 32 metadata hash records | `bf4f9b03a9bb49c69f981b74d2376e1ffe2902c049665ad5b127a95835c9526b` |

Both raw summary hashes match their canonical repository copies. The preregistration receipt matches both repository copies. Metadata aggregate hashes cover the UTF-8 concatenation of `SHA256  BASENAME\n` for all 32 metadata files in ascending seed order.
The completed-cohort RUNLOG prefix ends at v20 completion: 322 bytes, SHA-256 `d4c14e3a1bf726814c5d826cb63514c42d41288dff7096e1b20c60ff44bccb54`. The first two command lines: 850 bytes, SHA-256 `b0def09b487ae9ad24b144318fa4f1704dc5a7c9937a4c6a84f515a3bc7923a2`. Prefix hashes exclude subsequent runner activity.

| Receipt-bound upstream evidence | Existing verdict | SHA-256 |
| --- | --- | --- |
| identity: `/Users/rexxa/work/new-world-pass4/docs/revitalize/verification/p17-2_a_identity_observerfix/identity.json` | PASS | `692f27584155b429f2e105f6cd54d3d600c2a2333b47ce7793085dda3b59a2c6` |
| performance: `/Users/rexxa/work/new-world-pass4/docs/revitalize/perf/p17-a/perf.json` | FAIL | `4c88573122101832e3c526d8e99d2476bc4d52f3b908d9ab7cf7d04117f8150f` |

Both evidence files still match receipt hashes and bind this producer. The performance final-binary hash matches the retained worldsim_perf. Existing verdicts remain identity PASS and performance FAIL; this audit did not rerun or reclassify either evaluation.

## Scope limits

This audit verifies the completed summaries, process receipts, all 64 trace metadata files, file inventory, committed pre-registration and frozen executable provenance. It does not hash every large JSONL trace or recompute R1–R5. It makes no claim about I5, subsequent metric evaluation, or the final Stage A verdict. The runner was left untouched.
