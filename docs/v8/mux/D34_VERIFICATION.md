# D34 — package-scoped development profile evidence

Status: committed as p10 `78af5e245` (after D29 `91f029e9e`). Full gate and publication remain pending. ISS030 remains open; A8 held-out acceptance remains FAILED and the policy is frozen.

## Build scope and provenance

Only `rust/Cargo.toml` changes the profile: `[profile.dev.package.sim-systems]` with `opt-level = 1`. The effective compiler/link audit confirms opt-level1 for sim-systems, debug assertions ON, debuginfo2 and default overflow checks retained. The bridge links that package artifact while retaining its ordinary dev settings. This is package-scoped, not limited to steward functions; generic instantiation follows compiler rules. Release configuration is unchanged. No claim of broad safety follows from the profile audit.

External evidence root: `/Users/rexxa/work/mux-data/`. `profile1/profile-audit.md` records the actual compiler/link invocation and installed-toolchain overflow-check defaults. The 673 runtime/data source hashes were verified; the manifest profile delta is tracked separately. The audited debug library SHA256 is `d56b46470093be2a177b9204d86530750427f2073c9d4fb3cdacb2e9ba274f75`.

## Debug state comparison

`active-perf-3/mux-active-perf.json` is complete with errors=[], both ON/OFF arms ending at tick1441, with no extra tick during save. `profile1/debug-save-comparison.json` and independent literal comparisons establish equality against active-perf-2 for both complete debug saves:

- ON SHA256: `2709f96a471d5acb80cab73dc77215c4d886100beb5e594a6f24cd0536090af3`.
- OFF SHA256: `bae367468dc4b19e9cf108a8927a32968476f6a4f00b9f7d2f8d80a9b6607912`.

This proves these two tick1441 states, not all seeds, long-run replay or general determinism. The one-tick-per-frame active instrument is separate from the automatic gameplay measurement below.

## Actual automatic gameplay

`profile1/gameplay-comparison.json` retains both sequential default-seed0 debug rounds, including every startup interval. All four profile1 automatic-accumulator phases completed at tick1441, population10→10:

| Phase | Observed ticks/s | Mean FPS | Maximum frame interval (ms) |
| --- | ---: | ---: | ---: |
| 1x OFF | 11.968 | 119.103 | 447.113 |
| 1x ON | 11.949 | 118.216 | 649.902 |
| 4x OFF | 47.499 | 117.776 | 444.356 |
| 4x ON | 47.173 | 114.380 | 646.191 |

ON recurring boundaries tick360/720/1080 cost99.179/94.797/79.422ms at1x and99.173/94.169/73.061ms at4x. Tick1440 costs13.920/13.823ms. Startup maxima remain included. Prior automatic ON maxima928.410/926.945ms and prior D29 active-call spikes remain in their raw records; no failed evidence is discarded.

Large stalls remain: ISS030 is open. These sequential development runs are not quiet, interleaved release measurements. Do not claim calibrated speedup, unchanged FPS, resolved stalls, N=1000/full-capacity acceptance, or survival success from these results.

## Remaining verification

The full sim-test result1403 passed/0 failed/1 ignored across102 result summaries was **before this profile change**. It must not be presented as profile1 coverage. Profile1 unit10 and integration9 pass; the otherwise ignored historical v6 migration separately passes1 under `MUX_V6_FIXTURES` (`steward-unit.log`, `steward-integration.log`, `v6-migration.log` under profile1). Fresh OFF safety comparison PASS (`profile1/safety-compare.log`): all9 colony summaries are literal-byte identical,228748 shared values have zero differences, and64 detail files match excluding git_sha. The integration run retains1 ignored case; the explicit v6 execution is counted separately. Final exact-source gate and publication remain pending. Release build reports every simulation crate Fresh and its probe hash remains exactly `6c1f36ca40524ac2ca9ca564d43bb82700441beae73dff2fec8ff2a771ca95a0`, equal to optimization1. This binds the existing eight-seed ON equality receipt to the unchanged release executable; it does not extend the two debug-save samples to every world. Held-out A8 failure, thresholds and post-held-out policy freeze remain unchanged. See [Stage A report](STAGE_A_REPORT.md) and [D29 record](D29_VERIFICATION.md) for earlier evidence.


## Baseline measurement and separate UI uncertainty

Baseline perf completed exit0 at2026-10-25 06:55:07 KST: `baseline/identity/perf/perf_summary.json`. Natural median mean0.253830ms, median p990.511ms; quick mean0.383802ms and stress_v26 mean0.369900ms. This is baseline data, not a candidate speedup comparison or gameplay-FPS result.

AF01 remains unresolved: UI14 minimap conservative bound545us sums process maximum311us and draw maximum234us from potentially different frames. It neither proves a single-frame violation nor establishes the500us requirement. Dedicated `ui-perf-2/mux-ui-perf.json` completed exit0 after a fresh bridge build: stable=true,600 samples per phase; lifetime conservative bound345us (process272 + draw73), budget_met=true, including startup/warmup. This proves the500us bound only in its paused tick1 scene, not overall UI cost or gameplay FPS. Prior381/455us bounds remain scene-specific. Preserve all earlier bounds and samples; no implied acceptance. ISS030 remains open and A8 remains FAILED.
