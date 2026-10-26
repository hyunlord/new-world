# M-UX2.1 decisions and execution ledger

## Contract
Owner brief is MUX21_GOAL.md; BASE 9855bd082ad88a4c49296410f206f148d98c4df9.
Dedicated worktree /Users/rexxa/work/new-world-q4-dev, branch v8/mux2-1.
Raw evidence /Users/rexxa/work/mux21-data. Six-hour limit starts at start-utc.txt.
Owner explicitly authorizes implementation, commits, issue management and gated publication without additional approval.

## Plan / acceptance ledger
1. BLOCKED: Isolated release builds and common harness prepared; valid interleaved comparisons 0/12 because shared-host endpoint load repeatedly violates <8.
2. BLOCKED: Contended BASE profile recorded; measured bisection depends on valid comparisons. Static p6 candidate is not a confirmed culprit.
3. BLOCKED: Renderer/UI fix depends on measured causality; no production change made.
4. BLOCKED: FPS bars, GPU visibility and natural rebake acceptance depend on the missing fix and valid measurement environment.
5. PARTIAL / BLOCKED: Issue, registry, Korean report and history/changelog recorded; full gate not run because required acceptance remains unavailable.
6. BLOCKED: No publication without full gate; no worktree removal before publication.

## D1: Measurement admission
Initial load was 39.75 / 48.26 / 43.08. No valid FPS claim can be made at this load.
Do not signal any other process. Prepare builds while waiting for permissible measurement conditions.

## Hypotheses (not findings)
- Quarter marker path may issue expensive geometry or excessive calls per visible person.
- Whole-map visibility may increase canopy/chunk work beyond marker costs.
- UI facts/minimap or badge layout may have per-frame work scaling with all visible people.
Each requires measured historical comparison and an ablation before assigning causality.

## D2: Historical build isolation
Historical source archives are under the owned worktree `.mux21-builds/{mart21,mux}`; no other checkout is used to build or run. Archives are immutable comparison source, not product edits. Each build gets its own copied release bridge at the Godot debug load path. Common dependency artifacts may be reused only after sequential builds; hash each copied bridge.

## D3: Static candidate, not a causal finding
p6 1c8d2979e introduced per-frame Layout.place. Its nested square traversal discards interior cells only after visiting them, and repeats searches even for unchanged rounded positions. Markers already use MultiMesh and cached textures. Hypothesis requires measured bisection/profile before a fix. Issue: https://github.com/hyunlord/new-world/issues/638 (ISS333).

## D4: Reject shared historical Cargo output
The first shared-target preparation produced byte-identical mart21/mux bridges despite extensive Rust differences, and mux's log reported a 0.57-second no-op. These copies are INVALID historical build evidence. Archive mtimes plus shared Cargo fingerprints can reuse stale local packages. Rebuild each historical revision with its own target directory; retain initial logs/hashes, and do not use their import results as compatibility evidence.

## D5: Common measurement harness
`tools/v8/mux21_fps.gd` copies BASE mart2_fps.gd's timing/camera/frame-counting procedure. Only historical bake telemetry is feature-detected and the always-on-top setting is explicit. The exact same file is copied to each historical archive. Absent historical telemetry is explicitly unavailable, not measured rebake evidence. BASE short GPU smoke completed with matching frame counters, 1920x1080 visible always-on-top window, vsync0 and LOD active. Its 1s/2s duration and contended environment disqualify it as FPS acceptance evidence.

## Progress checkpoint
- Goal commit: 1b3518941.
- BASE release build and double import succeeded; common harness parsed and completed a real windowed smoke.
- Initial historical shared-target builds rejected (D4); isolated rebuild running with four jobs, log paths `mart21-isolated-build.log`, `mux-isolated-build.log`.
- Admission attempt base/N200/quarter rejected at load98.67, recorded at `comparison/base-n200-z0.25-a1/admission.json`.
- No renderer/UI product changes, no simulation changes, no valid FPS comparisons, no bisect result, no fix/gate/publication claim.
- Next: install isolated historical dylibs by replacing destination inode; double import again; historical short smoke; interleaved valid comparisons when load<8. Preserve invalid initial outputs.

## Isolated build verification
Both historical isolated builds completed. SHA256: M-ART2.1 `72fc3e05aeffb3982eedaa1cafe03e70bfded690bd3feb0e2c3de34458f1848d`; M-UX `7c23d171771a730549f62229cd88fe0bfe5b0dfeed5983d46d029803bb36f0ba`; BASE `593a4572e87ed6f023d7bf23ba44102e05546ad44d254efc27fd3cfbc09ea149`.
Each historical build passed its second import with no script/errors and completed short windowed N10 quarter-zoom common-harness smoke. Historical bake telemetry correctly reports unavailable. All three common harness files have SHA256 `9c7ff2f1fd40b767edd99d5e1bbf332775557be917f172356255e3b4733e07ba`.
Remote v8/main and v8/wip still resolve to BASE (remote-before.txt); no push performed.

## BASE diagnostic attribution (contended, not acceptance)
`base-profile-contended/profile.json`: 2054 rendered samples over60.009742s at N200 quarter zoom. Load48.19 before /28.77 after. Marker refresh25.053ms/frame; person reconciliation0.333ms; ground process0.203ms; agent cleanup/placement0.142ms; parameter/snapshot0.109ms. Nested marker/agent-total spans overlap and must not be summed. Ground chunk bakes0 in this window; draw calls335.73, objects1384.15, Performance process32.55ms (monitor sampled mean). Buildings0.040ms, HUD0.037ms, minimap0.032ms, alerts0.009ms. This supports inspecting marker layout; it does not establish an uncontended budget or a guilty commit.

## Bisection preparation
Prepare p9,p4,p6,p5 midpoint/adjacent snapshots inside the owned worktree. p5→p6 Rust trees are byte-identical; p9→BASE runtime Rust/Cargo trees are identical (only sim-test test differs), so p6 uses p5 bridge and p9 uses BASE bridge with recorded tree proof. No causal conclusion until actual measurements.

## Instrumentation validation
Both mux21_fps.gd and mux21_profile.gd pass the repository strict parser/warnings check; all59 detected FFI calls match exports (`instrumentation-strict.log`). Profile harness completed the real N200 quarter-zoom diagnostic window with no script errors. This validates instrumentation execution only, not final FPS bars.

## D6: Preserve live stress-world population semantics
The first full historical N10 attempt was INVALID: load6.80→13.05. It also ended with11 people, despite `--stress 10`. Its original receipt remains unchanged. `reset_world_stress` and `build_stress_engine` define N as the number of initial founders, not a fixed census; the baseline simulation continues to evolve during the prescribed window. Do not freeze births, change rules, alter fixtures, or silently label11 observed people as exactly10.
The common harness now records setup population and measured minimum/maximum in addition to the original ending census. Future receipts require setup=N and measured minimum>=N (no under-populated shortcut), and separately record `population_exact`; the report must show actual ranges. This preserves the same running-world procedure and all FPS/load thresholds. An exact-population claim is forbidden when that flag is false. All historical copies receive the identical revised harness before any valid comparison. Additional reads use the existing per-frame SimReads cache.
The first invalid capture is preserved at `comparison/mart21-n10-z0.25-a1`; no retrospective reclassification.

## D7: Provenance validation before first valid capture
External measurement runner hashes shaders, Rust source/data, tools and Cargo.lock as well as renderer/assets/scene inputs and the mapped bridge. It checks the raw bake-file hash/path, FPS ratio, frame-counter agreement and sample-window agreement. No thresholds were lowered. Capture supervisor69368 remains in the30-minute invalid-session cooldown before retrying the first row.

## D8: Independent measurement review and corrections
Read-only review found that sampling population immediately after process_frame could populate SimReads before WorldSim node processing. Corrected to use the existing frame_post_draw callback; no extra pre-process agent-snapshot reads remain. The earlier short population smoke is retained as superseded instrumentation evidence, not acceptance.
The reviewer confirms founders semantics but warns that population cost need not be monotonic: an observed10–11 capture does not prove exact10 performance. Keep this limitation and exactness flag in the final table, rather than presenting it as an exact-N pass.
Source provenance now compares working files against a private Git index read from the claimed revision and rejects unexpected runtime inputs (only the two named MUX21 instrumentation scripts are allowed additions). The mapped bridge must equal the separately built historical artifact. This prevents stable-but-wrong source/binary captures and prevents calling a future modified candidate BASE.
The generic Art13 calibration protocol is superseded by the explicit owner-specific method. Its30-minute cooldown/6-attempt/3-hour limits are therefore removed together; endpoint load<8, duration and all FPS thresholds remain. Own supervisor51167/session69368 was confirmed and terminated with exit143 solely to apply this protocol correction while waiting; no foreign process was signaled. Every prior invalid attempt remains.
Natural rebake acceptance must aggregate measured rebake events by rendered frame; the inherited per-chunk summary is insufficient for the4ms/frame goal. This remains pending and is not relabeled PASS.

## D9: Terminal blocked audit
The same external load blocker recurred across at least three consecutive goal turns. The first complete historical capture was invalid (6.80 before, 13.05 after); subsequent admission waits repeatedly failed, including 232.77 at09:21UTC and 57.73 at09:25UTC. Final recorded load was39.25 (`load-blocked.txt`). Valid mandatory comparisons remain0/12. No safe independent task can establish the missing causal comparison or acceptance while that condition persists.
Own waiting supervisor PID7953/session69103 was verified and stopped with exit143 during blocked finalization; no active GPU capture was interrupted and no foreign process was signaled. Earlier checkpoints describing active supervisors are historical, superseded by this entry. No measurement supervisor remains active.
This is BLOCKED and incomplete, not a successful performance pass or exhaustion of the original six-hour limit (08:32:12–14:32:12UTC). Preserve the original goal, all invalid receipts, archives and branch for explicit resumption. Reacquire remote/source/process/load state and original time constraints before resuming. Full gate NOT_RUN; no push; both published refs still BASE in `remote-blocked.txt`. The report and documentation record partial work without closing ISS333/#638.
Final documentation check initially failed with incompatible stale release crate artifacts from historical preparation. Cleaning only owned local release packages and rebuilding BASE resolved it: `blocked-docs-check-rebuilt.log` ends `docs: PASS`, exit0. Preserve the initial failed check and direct compiler log. Goal byte comparison, diff whitespace check and unchanged production/protected-source diff also pass; none substitutes for the unrun full gate.
