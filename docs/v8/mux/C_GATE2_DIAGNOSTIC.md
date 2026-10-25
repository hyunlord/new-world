# Stage C gate2 failure and bounded diagnosis

The clean final-documentation candidate `bcea734ae63f47cf2de605dba407808b9d1340de` ran the unchanged full gate from2026-10-25 22:01:41 to23:09:33 KST. Result: **35/36 PASS, overall exit1**. Only `bench` failed. Debug printed2228passed/0failed/5ignored and release2227/0/5; these are printed aggregates, not a deduplicated census. HEAD, tracked inputs, symlinks and ignored Cargo.lock did not drift. No publication occurred.

## Original failure retained

| Standing scene | Ledger p99 | Candidate p99 | Ratio | Original result |
|---|---:|---:|---:|---|
| n200 |855µs|971µs|1.136|FAIL|
| n200_side64 |508µs|570µs|1.122|FAIL|

Ledger remains M-ART2.1 (`3a51636d`); its thresholds and contents were not changed. All means and other p99 comparisons passed. All six absolute C3 ceilings passed; allocator1870.3044B/agent, RSS outer2220.9422B/agent and96B/tile. Admission load6.23 and end7.00 were below the existing8 ceiling. The later nav start8.06 is a separate observation and cannot establish the cause of this failure.

Natural mean0.2643ms×1.041, p990.5370ms×1.051, p99.90.6455ms×1.063; quick mean0.3895ms×1.053. Original fingerprints match. Identity228748/0/0, Chronicle64/19683 and normalized raw64 passed, as did the other35 steps. Gate1 natural tail×2.400 remains ISS313; this standing failure is separately ISS314 OPEN.

Raw directory: `/Users/rexxa/work/mux-data/gate-stage-c-2/`. Receipt SHA256 `f130b5294c2a287e31d14fa9d3563d0a93cc80b7ff39fb0325acf22cacac16da`. Independent review: `/Users/rexxa/work/mux-data/stage-c/gate2-failure-independent.md`. Between the two gate source manifests only nine documentation paths differ; Cargo.lock is identical. B5-bound production Rust/data/bench inputs differ from the current tree only in three integration-test files. There is no recovered hash proving the exact gate1 benchmark binary, so unchanged source is not presented as exact binary identity.

## Fixed experiment, all observations preserved

Before execution, root registered six full-six-scene `bench --repeats 1` runs in AB/BA/AB order. A is the frozen B5 binary SHA256 `234635e4f5c3d3e41fe4ab5a4a3f2c5a51be3a0729c643d675fe546558fab6d2`; B is the current gate2 binary copied before execution, SHA256 `5b01607002709214966eeb87dfd2c1edc34969393668d4d7ab7d33da5fea795a`. No builds, source edits or outcome-driven retries occurred during these runs. Seed15100, warmup1440, measured2880ticks and ordinary bench commands remain unchanged.

All six exited0. The18 paired scene comparisons have equal fingerprints, alive counts, births, deaths, founders, jobs taken and jobs done. Timing ratios are observations, not a predefined acceptance test:

| Scene | Median B/A mean | Median B/A p99 | B p99 across three runs (µs) |
|---|---:|---:|---|
| n200 |0.993741|0.989773|874 / 871 / 908|
| n1000 |1.000184|0.996501|5078 / 5171 / 5127|
| n2000 |1.000724|0.997288|11031 / 11035 / 11341|
| n200_script |1.000919|1.003138|1012 / 959 / 1051|
| n200_side64 |1.024005|1.031128|530 / 524 / 651|
| n1000_wolves |1.001784|1.002816|5014 / 4985 / 4965|

The first A load10.48→7.38 makes the first pair contended; later run boundaries range4.34–7.38. Do not discard that pair, claim a uniformly quiet session, attribute variation solely to load, or interpret near-one ratios as a proven fix. The B5 comparator is not the standing ledger's M-ART2.1 binary. This experiment neither replaces the failed gate nor revises the ledger. Tail variation, including the later651µs side64 observation, remains explicit.

Raw preregistration, binaries, logs, receipt and comparison: `/Users/rexxa/work/mux-data/stage-c/bench-diagnostic-1/`. Completion receipt SHA256 `84f562f43d4dd53c45f75f379a51c8842fd36f5acae866b630fb7cd74358231d`.

## Next verification boundary

D49 authorizes one new complete clean-head gate after recording this failure and linking ISS314. Product code, thresholds, benchmark ledger and gate configuration stay unchanged; no justification waiver. A future green gate is additional exact-head evidence and does not close ISS313/314, establish cause, or erase either original failure. If it fails, investigate before any further run. Only a full gate status0 may authorize the explicit fast-forward publication.
