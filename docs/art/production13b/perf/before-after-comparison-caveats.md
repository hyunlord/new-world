# Before/after comparison caveats — offline premeasurement audit

Scope: Art Pass13 admitted historical before versus future frozen Art Pass13b after. This file does not validate an unrun after session, change any acceptance bar, normalize timings, or infer missing after values. Only this external document was written; no Godot/build/measurement or repository edits were performed.

## Exact calibration values, recomputed

| Reference | R seconds | 1.05R ceiling seconds | SHA-256 |
|---|---:|---:|---|
| Historical Art Pass13 | 2.0089961669873446 | 2.109445975336712 | `e839f658f57ad018e3fa4a5d5fbc8e57561183ab13d38212bb1e877a742a772d` |
| Fresh Art Pass13b | 2.762010041042231 | 2.9001105430943426 | `e240ebf3592bd25d70f673350dc0f990295b8a413cfbcc0d2e819153033520b4` |

Both contain15 completed runs spaced at least20seconds apart. R was independently recomputed as the median of the five fastest durations and matches exactly. Both record kernel `p13-int32-byte-sweep-v1-64`,64sweeps,262144working-set bytes, identical Python3.14.4/Clang21 interpreter and macOS27.0.1-arm64 platform strings, and checksum4216682400 for every run. Fresh/historical R is **1.3748209610494642**, approximately37.4821% higher. That is a probe-duration observation, not a frame-time correction factor or an attributed cause. No thermal/power/background-load explanation is established by this audit. Most fresh calibration runs had load≥8; calibration formation permits this, while each admitted measurement still requires load<8 at both endpoints.

The source-specific rules remain unchanged: each measurement must satisfy its own immutable reference's P≤1.05R, load<8, successful exit, valid frame evidence and matching provenance. Historical accepted sessions remain assessed against historical R and their original calibration hash. Fresh after sessions must bind the fresh reference path/hash/R. A more permissive absolute fresh probe ceiling does not retroactively invalidate historical evidence or prove equivalent machine performance across dates.

`session_validity_p13.require_valid` binds evidence to the currently selected `P13_REFERENCE` (defaulting to the old path if unset). Therefore validate old and new artifacts in their respective reference contexts; a historical sidecar rejected under fresh reference is a provenance mismatch, not new evidence that the historical session was contended. The Part13b quiet wrapper requires an explicit reference variable. Keep all original probes and rejected attempts.

## Meaning of the comparisons

- Historical-before/current-after frame ratios are **descriptive cross-session ratios**. Report raw before and after milliseconds and after/before without normalization. The higher fresh R is a material comparability limitation. Changes include accepted kernels and UI changes as well as possible environmental drift; the ratio alone cannot isolate a walker-kernel effect.
- Same-session alternating control/candidate measurements give stronger **controlled candidate-effect evidence**: release/source snapshot held fixed, fresh-reset workloads, explicit switches, matched noncandidate flags, valid endpoint admission, and three150-frame pairs. They remain sequential observations subject to drift and state evolution; no statistical significance or uncertainty interval is inferred.
- The accepted historical declump ratios0.7962943015737632 (W500 frozen) and0.9642023798670909 (W200 running) belong to those original paired experiments. Their controls are not the Part4 matrix before denominator. Candidate identity is an additional requirement, not a speed claim.
- H1≤6ms median, H2≤16.67ms p99, H3≤16.67ms median, H4≤10ms median, H5≤16.67ms median retain their literal bars. H6/H11 remain reported rather than newly assigned pass/fail. No R adjustment may rescue a failed bar. No candidate-specific speed criterion is changed here.

## Source/runtime/workload equivalence audit

**Verified in existing source:** the current `perf_frames_p13.gd` SHA-256 is `ae563b64efd939bfc9859c20e1dbc4b12d252d5ffc300614ddec55a8e2368e78`, exactly the admitted historical collector digest. Its workload constants remain:

| World | Seed | Initial tick | Population setup | Rule family |
|---|---:|---:|---|---|
| W0 |20261010|40560|natural; historical29|live game_v26|
| W1 |20261013|126994|natural; historical71|live game_v26|
| W200 |15100|1440|stress200|game_v19|
| W500 |15100|1440|stress500|game_v19|
| W-multi |19603|90840|natural; historical88|live game_v26|

Rust `ffi/world_node.rs` retains live-reset game_v26 and stress-reset game_v19. This verifies source intent, not an unrun binary or future world population. After outputs still must establish actual initial tick/population/alive settlements, framing/zoom, and subsequent population evolution; matching seed alone is insufficient.

Matrix controls are three fresh-reset300-frame blocks per condition,30warmup each,1920×1080,zoom4,frozen speed0 and running2×,camera process frozen,vsync disabled, running max_fps60/frozen0. Process+render CPU excludes setup; pacing/setup/GPU-status fields remain separate. Tick-carrying subsets and population-loss exclusion remain required. Historical W500 running500→392/392/404 is excluded, never an eligible500-person baseline. W-multi88→87 remains recorded rather than called fixed-population evidence.

Both required running section worlds retain W20015100/1440/stress200 and W-multi19603/90840/live,300counted+30discarded,zoom4,1920×1080,speed2. The section validator checks complete/aligned callbacks, matching exclusive fields, sum=total, zero remainder, explicit UI/layer/callback conditions and restoration. Generated instrumentation preserves ordinary statements and hashes source/output/clock; added timing overhead remains part of these separate diagnostic spans. Do not add section medians, nested getters, or ablation deltas together.

**Provenance coverage:** the matrix wrapper pins checkout HEAD, release/copied-debug dylib equality and SHA, collector SHA, and checks them before/after sessions. It does not independently pin every renderer/helper/asset file. Section runner compares `provenance()` before/after: HEAD, release/copy hashes,project.godot and all .gd/.tscn/.gdshader files under scripts/scenes/shaders, plus generated renderer/clock hashes. That manifest does not itself cover every font/texture/RON/resource. Runtime Godot/version and actual machine must be checked from after artifacts against historical Godot4.6-stable official89cea143 and recorded machine; no new runtime invocation was performed here.

**Leader's stated launch control (planned, not yet measured by this audit):** preserve the existing `run_running_identity_p13b.sh --snapshot` full source/HEAD/build/candidate-environment snapshot before/after each Part4 group, retain generated-timer manifests, and explicitly export `P13B_WALKER_KERNEL=1`, `P12_DECLUMP_KERNEL=1`, `P13_WRITE_SKIP=0`. Current renderer source defaults match walkerON/declumpON/writeSkipOFF. The inspected snapshot-only branch hashes all files under scripts/scenes/shaders/data/assets/localization, Rust .rs/.ron/Cargo files excluding target, project.godot and extension descriptors; it records copied-debug dylib SHA, HEAD and all three candidate environment variables. Its --snapshot branch exits before launching Godot, so the identity runner’s later --fixed-fps option does not apply to this use. Matrix/section runners still separately verify debug/release equality. Matrix/section raw frames do not contain actual per-frame candidate getter stamps. Thus final-build switch evidence rests on frozen source plus recorded launch environment/snapshots; do not describe it as per-frame switch verification. Alternating candidate collector does assert actual switch state during its blocks and records `switch_state_after`, which is a separate stronger control.

## Required metric coverage before measurement

| Requirement | Existing capture/summary coverage | Remaining work or unavailable field |
|---|---|---|
| Matrix median/p90/p99/max | Raw frame samples and full per-block/pooled summaries for primary,process,render,setup,pacing,ticks; nearest rank | Report both all-frame and tick-carrying results; no missing raw timing field identified |
| Frames,ticks,population | Counted/discarded rows, ticks_run, fresh block start/end states, population flags | Counted tick totals and endpoint tables are derived from retained raw rows; don't use wall duration as ticks |
| Both running exclusive timer sets | Raw exclusive_usec,total_usec and published median/p90/p99/max | Section summary lacks primary-frame/getter/tick/pop aggregates; compute from raw300 counted rows as before-reference did |
| Getter call/time cost | Per-frame ffi_calls and ffi_call_usec including zero-call frames | Report mean calls/frame and mean/median ms/frame; getters are nested, not additional exclusive cost |
| Accepted kernel ratios | Alternating summary control/candidate percentiles, counts, pairs and pooled ratio; raw state/ticks | Retain each candidate's identity/provenance and frozen noncandidate configuration; never infer an unmeasured pair |
| GPU | gpu_status=unavailable, original numeric0 may occur | Unavailable, not measured zero; not substitute CPU/pacing |
| FoundingGroups separate callback | Explicit unavailable_separate_callback; null time | Unavailable, not zero; parent timer does not cover it |
| H7–H10 visual/readability/input facts | Separate image/identity/reader/input evidence | Matrix/timers cannot generate or replace these results |
| W500 eligible running before | No eligible historical blocks | No valid before ratio for eligible500-running result; preserve excluded raw numbers separately |

No missing required **raw CPU/tick/population** field was identified in the existing Part4 matrix and section collectors. The identified omissions are summary aggregation, broader source/launch-control evidence, actual per-frame switch stamps, and explicitly unavailable GPU/separate-callback data. This is static preparation only: no after session or actual runtime equivalence has been accepted by this file.

## Source hashes at audit time

- `docs/art/production13b/perf/before-reference.json` — SHA-256 `9baa9440709d47a083b6ba39d43f4184639898c15bce995dab80ef87294ad322`
- `docs/art/production13/perf/calibration.json` — SHA-256 `e839f658f57ad018e3fa4a5d5fbc8e57561183ab13d38212bb1e877a742a772d`
- `docs/art/production13b/perf/calibration.json` — SHA-256 `e240ebf3592bd25d70f673350dc0f990295b8a413cfbcc0d2e819153033520b4`
- `tools/art_pipeline/run_perf_matrix_p13b.sh` — SHA-256 `47daddc8f4cd7d1b6c1725df8f660b9ba629144089e228e9ac03751da65db654`
- `tools/art_pipeline/run_agent_sections_p13b.py` — SHA-256 `f74b5a23cc7bb577cd05f24a84a18be23e91dc6ed1e6a5b33ee40fb3126ec0cc`
- `tools/art_pipeline/session_validity_p13.py` — SHA-256 `039b46ec8db6d0bc46248417cda182fda8aeeb17b9ddc6b1dd092315e38bc9ce`
- `tools/art_pipeline/cpu_probe_p13.py` — SHA-256 `ca5f708a9cfff630e5d670fb84a2ca9c423943b83035ff88ba2958c9cf98bcdf`
- `tools/art_pipeline/summarize_perf_p13.py` — SHA-256 `0169f22d586d38c64c7b2db8d1de0a9a9c861ee141beecac8f93d31b6348f904`
- `scripts/test/art/perf_frames_p13.gd` — SHA-256 `ae563b64efd939bfc9859c20e1dbc4b12d252d5ffc300614ddec55a8e2368e78`
- `scripts/test/art/profile_agent_sections_p13.gd` — SHA-256 `a2a850af3d3ae7432dfe6f8aa014df047b72d67b30818a6d29910b1b1810b1a7`
- `tools/art_pipeline/run_alternating_p13.py` — SHA-256 `e29f42baeeff934f19d49422ba3572a623c36395162951d25b5037f9ffeb30a1`
- `tools/art_pipeline/run_alternating_p13b.py` — SHA-256 `378d66cde8bba1e9294b2c67d85e3a435e3784516cd2d88a3f31c7f93a3c990e`
- `scripts/test/art/perf_alternating_frames_p13b.gd` — SHA-256 `6ee7648ebda175977616c133b408bc36cdc95c3423cd287598fb59220de67d9b`
- `scripts/ui/agent_renderer.gd` — SHA-256 `218fef2ac93dbfb2cc78e0b7d3389790b108ca2b7f47c0c9789f8d7785b3f5fc`
- `rust/crates/sim-bridge/src/ffi/world_node.rs` — SHA-256 `14e5124a716a73c292a288c0bb9b212cf4b30a3ecf715e622c3ba6c3f02b83b7`
- `tools/art_pipeline/run_running_identity_p13b.sh` — SHA-256 `14580986f35cf9139ee9d020b87c51b4216347dad1cd8fc21d8d8e6e0683fa1d`
