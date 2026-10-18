# Revitalize baseline — Part 0

Recorded on DGX Spark, 2026-10-10, 12:07–12:18 KST.
Base: `d409e9b941ccc868747e4eb63953eafa6831bff7` (`origin/lead/main`).
Dedicated branch: `codex/revitalize-2026-10`.
No simulation behavior changed in this Part.

| Environment | Observed |
|---|---|
| Host | aitopatom-d6bb, Linux 6.14.0-1015-nvidia, aarch64 |
| CPUs | 20 |
| Memory | 119 GiB total, 102 GiB available at start |
| Disk `/home` | 848 GiB available at start |
| Rust | rustc 1.93.1 (01f6ddf75 2026-02-11), pinned 1.93 toolchain |
| Cargo | 1.93.1 |
| Build/test parallelism | CARGO_BUILD_JOBS=10, test-threads=8 |
| Godot | 4.6.stable.official.89cea1439 Linux arm64; official ZIP SHA512 verified |
| Pillow | 12.2.0 |
| Xvfb | unavailable; passwordless sudo denied |
| Git hooks | `.git/hooks/pre-commit` absent; harness hooks not installed |

## Executed gate

| Check | Result | Wall time |
|---|---|---|
| cargo build -p sim-bridge | PASS | 30 s |
| Godot headless import | PASS | 5 s |
| cargo test --workspace -- --test-threads=8 | 1,729 passed, 0 failed | 315 s |
| cargo clippy --workspace -- -D warnings | 0 warnings | 13 s |
| cargo clippy --workspace --all-targets -- -D warnings | 0 warnings | 4 s |
| cargo test --workspace --release -- --test-threads=8 | 1,729 passed, 0 failed | 121 s (includes release build) |
| Forbidden simulation transcendental scan | 0 matches | included |
| gdscript_strict_check.sh | 14 files clean; 10 FFI calls match; PASS | 3 s |
| Godot headless 900-frame smoke | PASS; no non-audio ERROR/SCRIPT ERROR | 6 s |
| git diff --check | PASS | included |

Every Godot command ran through `timeout 600`, one process at a time.
Memory remained above the 24 GiB safety threshold. Raw command outputs are
retained locally under `.harness/evidence/revitalize/logs/p0/`.

## Comparison with the supplied x86 reference

The supplied reference says 1,730 tests; this pristine base executes 1,729
in both debug and release, across 107 test-result groups including empty
and documentation groups. No Rust source or test file was changed for this
measurement. No architecture-dependent test guard was found in sim-test.
The historical one-test difference cannot be attributed without that
planner's exact worktree/log; it is not silently added to the measured
count. All subsequent count accounting starts from **1,729**.
Clippy and strict-check counts match the supplied reference.

## Capture exception

`xvfb-run` is missing, and `sudo -n apt-get install -y xvfb ripgrep` failed
because a password is required. Following Part 0 item 5, every screenshot
capture step is skipped. No fabricated or substituted screenshots are
provided. Headless smoke proves loading/execution, not visual acceptance.

## Part 1 legacy probe

Measured 2026-10-10 on the same host using the frozen `legacy_scene()`.
Invocation: `--scene legacy --seeds 0..4 --days 10 --sample-every 120 --workers 4 --check-determinism`.
Probe build metadata is `049e8043` (Part 0 HEAD with Part 1 instrumentation
in the working tree). Four primary runs and four independent twins completed;
`DETERMINISM PASS`. All 121 sampled fingerprints also match across the four
primary seed labels. Legacy deliberately ignores the seed.

| Metric | DGX measured | Supplied planner reference |
|---|---:|---:|
| Alive, day 10 | 44 | 44 |
| Births through day 10 | 213 | 213 |
| Dehydration deaths through day 10 | 233 (exact tally) | approximately 233 |
| Starvation / combat deaths | 0 / 0 | 0 / 0 |
| Food units / water tiles at day 10 | 0 / 0 | 0 / 0 |
| Buildings / construction sites at day 10 | 3 / 0 | 3 / 0 |
| Stockpiled food at day 10 | 0 | 0 |
| Idle / agent-samples (tick 0 included, every 120 ticks) | 98.167888% | 90.6% |
| Idle-with-breach / agent-samples (same sampling) | 70.403065% | 61.6% |
| Home distance mean at day 10 (Chebyshev to formation tile) | 17.386364 | 16.2 |
| Median run wall seconds, includes twin and I/O | 13.682738 | 10.6 (different hardware/method) |
| Median primary-engine ms / agent-tick | 0.009588558 | not supplied |

Population and cumulative deaths agree with the planner. The idle shares and
home-distance measurement differ; the planner's exact sampling code and raw
observations are unavailable, so their cause is not established and no platform
or simulation discrepancy is inferred. The table states the current metric
and sampling definitions explicitly. Wall time includes validation of a second
engine and is not a like-for-like hardware performance comparison.

The immutable two-day pin uses tick **2880**, alive **45**, births **42**,
starvation **0**, dehydration **61**, combat **0**, fingerprint
`527233d10db4ed16`. The day-10 fingerprint is `b9ce9cb344a98de5`.
The pin test executes two independent fixture engines and checks both against
these literal two-day measurements. Separate staged accounting tests force
one starvation death, one dehydration death, one birth, and exercise combat
accounting without treating bootstrap spawns as births.

Raw samples: `.harness/evidence/revitalize/probe/p1_legacy/legacy_s0.jsonl`
through `legacy_s3.jsonl`. Aggregate summaries accompany the Part 1 probe
artifacts. Screenshot capture remains skipped under the Part 0 Xvfb exception;
there are no baseline screenshots or visual-acceptance claims.
