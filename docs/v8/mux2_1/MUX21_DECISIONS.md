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
p6 1c8d2979e introduced per-frame Layout.place. Its nested square traversal discards interior cells only after visiting them, and repeats searches even for unchanged rounded positions. Markers already use MultiMesh and cached textures. Hypothesis requires measured bisection/profile before a fix. Issue: https://github.com/hyunlord/new-world/issues/638 (ISS-343).

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
This is BLOCKED and incomplete, not a successful performance pass or exhaustion of the original six-hour limit (08:32:12–14:32:12UTC). Preserve the original goal, all invalid receipts, archives and branch for explicit resumption. Reacquire remote/source/process/load state and original time constraints before resuming. Full gate NOT_RUN; no push; both published refs still BASE in `remote-blocked.txt`. The report and documentation record partial work without closing ISS-343/#638.
Final documentation check initially failed with incompatible stale release crate artifacts from historical preparation. Cleaning only owned local release packages and rebuilding BASE resolved it: `blocked-docs-check-rebuilt.log` ends `docs: PASS`, exit0. Preserve the initial failed check and direct compiler log. Goal byte comparison, diff whitespace check and unchanged production/protected-source diff also pass; none substitutes for the unrun full gate.

## D10: Conflict-safe migration onto published M-CORE
The owner superseded the failed-rebase stop and authorized a new branch, v8/mux2-1b, from published M-CORE 3814828c06d1a7a211b13b54368e4e0a488cc0e3. The original v8/mux2-1 branch remains at5c41e604eaa947c91841b1f9053a185d879386ad. All five original commits were cherry-picked; conflicts were documentation-only (ISSUES, CHANGELOG, HISTORY). Published M-CORE rows are retained byte-for-byte. The M-UX2.1 regression is now ISS-343/#638, avoiding M-CORE numbering; current ledger343 rows: fixed203/open107/deferred31/wont-fix2. The original goal and blocked report remain historical evidence, not a current acceptance claim. Migration changed no product or simulation source. An intermediate resolution script failed on the host Ruby version; its accidentally staged conflict text was repaired forward before final validation, with no history rewrite. Parent owns resumed measurements, new acceptance and publication; no builds or GitHub mutations occurred during migration.

## D11: Resumed owner contract, 2026-10-28
The planner's morning note replaces BASE with published M-CORE3814828c06d1a7a211b13b54368e4e0a488cc0e3 and resets the target/hard budget to5/8h. First observed resume time2026-10-27T23:45:54Z, conservative hard stop2026-10-28T07:45:54Z; the10:50KST note does not reset this budget. Both explicit notes authorize execution without approval. The latest note authorizes docs-only cherry-pick conflict resolution, preservation of the old branch, ISS-343 renumbering, and publication from v8/mux2-1b. Any code conflict remains a stop condition. GitHub #638 title/body and explanatory comment were updated and read back (`resume-20261028/issue638-after.json`).
The old blocked report is historical until replaced by this resumed pass's measured report. Mandatory three-build comparison now uses3a51636d/0207b497/3814828c; bisection is required only if that comparison and quiet profile do not identify1c8d2979e. Repeat admission records1/5/15min load with1min<8, no other Godot window and no Cargo build. Invalid captures remain invalid and are repeated; contention is tracked in15min windows up to8, bounded by the unchanged hard stop.
Added UI-only scope: Korean inspector mind reason for working adult/hungry/fleeing people at scales1.0 and1.25; seed0 chronicle after3days with kind counts and data-owned default filter only if mind_choice appears; retain person why access; wolf-defence steward toast and drafted rings. Do not change simulation recording or protected crates.

## Resumed execution plan (supersedes the blocked ledger above)
1. COMPLETE: preserve old branch; docs-only migration to v8/mux2-1b; ISS-343 registry and GitHub reconciliation.
2. COMPLETE: docs check, debug/release bridges and imports;12valid interleaved comparisons and quiet BASE profile.
3. COMPLETE implementation: exact free-space placement and tile-target cache;244windowed assertions. Performance acceptance remains step5.
4. COMPLETE feature captures: work/hunger/flee atbothscales, filtered chronicle and real defence toast/rings. Additional final-source capture automation is being hardened; failed input/load attempts remain evidence, not passes.
5. IN_PROGRESS: full FPS bars, original GPU visibility/ring checks, natural per-frame rebake evidence; preserve every invalid/failing attempt.
6. WAITING: committed-head full gate, documentation/current captures, fast-forward explicit branch publication and issue closure with public commit; non-force worktree removal only after publication.

## D12: Quiet baseline confirms the marker path
All12 interleaved comparisons are valid; one initial MART21 N40 capture failed load and is preserved separately. Source/binary/harness hashes, frame/window/duration checks, census ranges and1-second process-overlap observations are retained in resume-20261028/comparison-summary.json.
| Founders | M-ART2.1 FPS | M-UX FPS | M-CORE BASE FPS |
|---:|---:|---:|---:|
| 10 | 453.120 | 338.924 | 317.107 |
| 40 | 490.763 | 374.202 | 267.864 |
| 200 | 339.207 | 286.806 | 34.948 |
| 1000 | 153.656 | 139.582 | 0.350 |
N10 live census reached11; these are initial-founder scenes, not fixed-census claims. Quiet N200 profile load5.72/5.07/6.28→6.11/5.51/6.33,60.004403s/2106samples: markers24.570ms, reconciliation0.316ms, ground0.181ms, cleanup0.139ms, snapshot/params0.101ms. Other measured UI costs <0.04ms. Product marker sources are unchanged from9855bd082 to3814828c; p6 1c8d2979e introduced Layout.place per frame. Under the owner's updated criterion, no further bisection is needed before fixing this measured path.
Baseline geometry reference test35cases/177assertions passes before optimization. Initial candidate used perimeter-only search, conservative viewport bounds and monotone within-pass reuse; cache only placement inputs, not colors/glyphs/selection state. No simulation changes. Existing MultiMesh and ring textures remain. The initial search candidate was superseded by D13.

## D13: Exact free-space search and UI capture findings
The first perimeter/cache diagnostic remained far below the N1000 bar. It is retained as an unsuccessful candidate, not acceptance. Replace occupied-cell scanning with maximal free rectangles and solve the original ring/row/column priority over each feasible lattice interval (ADR0048). The frozen35-case geometry oracle passes; real macOS rendering passes238 geometry/cache/presentation assertions. Dummy rendering cannot validate MultiMesh per-instance getters, so the integration arm explicitly requires a real window. Remove only the now-unused production occupancy helpers; the frozen oracle keeps the original code.

Inspector work/hunger/flee reason text fits at both requested Korean scales in the first complete capture set. A composed tool-quality word split was corrected with the existing KoreanText helper. Seed0 at4320ticks has raw kinds1:10,12:1,23:1,26:1,45:60,47:38; mind choices occupy38 of the default last60 rows. Filter their owner-feed visibility through assets/ui/chronicle_feed.json before retention, leaving personal records and Why untouched (ADR0049).

The natural defence capture exposed a UI notification bug: the defence chronicle row advances without the ordinary steward action sequence, so the HUD's sequence-equality shortcut suppressed its toast. Remove that shortcut while retaining cursor deduplication and load/rollback baselines. A focused22-check test passes; natural screenshot acceptance remains separate. JSON preparation manifests also need exact integral-field normalization before comparing live chronicle dictionaries.

At02:45UTC the next performance diagnostic was rejected with load11.32/9.36/8.23; its owned process was stopped before measurement. Begin a15-minute contention window; no result may be reported from that attempt.

## D14: Owner-authorized UI successor gate adapter
The published M-CORE gate pins an earlier tool inventory/BASE and has a pass-specific side64 adjudication. Configuration alone cannot validate this UI successor. The owner explicitly approved adapting the gate in this thread. Preserve the accepted historical failures and provenance, require unchanged simulation source, and perform a fresh published381-to-candidate comparison without extending M-CORE's exception. ADR0050 and the adapter record the exact contract. This approval does not waive any M-UX2.1 FPS, capture, visibility or publication bar.

## D15: Stable placement anchors, current presentation
The MaxRects moving-crowd diagnostic ended at load10.38 and is invalid as an acceptance result. It still motivated eliminating interpolation-driven cache misses. ADR0051 uses simulation tile targets only for placement; actual interpolated person positions remain leader anchors, and current canvas transforms remain authoritative for picking. Integer lattice bounds avoid unnecessary vector boundary trimming. The windowed oracle now passes244 assertions, including fixed-target interpolation and target-change invalidation.

Native pointer motion caused a capture-only Why click to receive down/up without pressed and lose hover. Preserve those failed attempts, add signal/hover/focus evidence, and warp the actual pointer to the injected click location before dispatching input. This does not invoke the button signal or bypass the real UI input path. A headless profiling experiment awaited frame_post_draw, which dummy rendering does not emit; it was stopped without a timing result.

## D16: Complete FPS bars, measured natural rebake miss
At runtime/tool HEAD b0401ca2e all seven required release FPS bars pass with the common windowed30s+60s procedure. Quarter N10/40/200/1000:377.513/381.968/246.246/115.303; half N100/200:179.936/123.757; full N200:244.102. N10 observed10–11 people, not an invariant census. N40 attempt1 is INVALID at endpoint load9.81; preserve it, wait15minutes, and use only admitted attempt2. Every result retains1/5/15 loads and source/binary/window/process evidence.

Natural quarter gameplay from seed0/day3 records17 rebakes in16 measured frames, zero fresh bakes, at load2.81→2.70. Sum complete total_usec by(reset_epoch,frame): maximum10.493ms, so the4ms aim is NOT_MET. This is a measured limitation, not a reason to erase events or substitute warmup maxima. Evidence: external resume-20261028/acceptance-summary.json and comparison/candidate-n0-z0.25-a1.

Final a5 captures pass reason/Why fit at both requested settings; independent16-image reviews retain the wolf-warning/header overlap as ISS-346/#657. Quarter real-defence replay additionally passes12 actual-click/selected+drafted/moved-selection checks and captures the toast with two red circular marks. The original core mind UI test completes with0failures and both real hunger/threat transitions observed. These checks are not a substitute for the pending full46-step gate or owner playtest.

## D17: Seal real repository entries without following skill symlinks
The first pre-gate seal failed before any gate step: the historical regular-file inventory was incorrectly reused for the complete repository, which already contains seven committed skill symlinks. Preserve that historical validator unchanged. The successor now binds each full-tree Git mode plus blob object ID, including symlink text without dereferencing it; declared delta fields say before_entry/after_entry. Runtime live scopes still reject symlinks. A temporary Git repository test proves dangling links can remain unchanged, executable-bit-only changes are detected, and changed out-of-scope links are rejected. All23 adapter tests and actual BASE/HEAD inventories pass; pinned historical references validate. The failed sealed-1 attempt remains preserved; new config is sealed only after this correction is committed.

Natural rebake miss is tracked separately as ISS-347/#658; no season-transition cause is inferred from this capture. Quarter defence visual review confirms both drafted circles and selection transfer, while retaining the warning/roster inspector overlap.


## D18: Stop on the red full gate; retain completed measurements without publication
The clean full-gate source is6ed4e19966ec368812441c4f12a74fca22ae251b. Steps1–15 exit0: debug2595 and release2594 passed,8ignored each; identity228748values and64chronicles/19683entries match. Step12 legacy perf exit0 lacks per-repeat admission evidence and is not valid fresh acceptance. Step16 python_unittest exits1 after405tests/errors1: test_core_shadow_freeze.FreezeTests.test_gate_reports_only_core_ratio_misses_and_rejects_other_failures extracts performance_gate and raises NameError(root) at line105. This is within adapter ISS-345/#656, not a separate issue.

The runner had started step17 before the red result was observed. Under the user's stop-on-red rule, terminate only the owned PGID27094, preserve terminal actual/wrapper -15 and hard_deadline_reached=false. No owned process remains. Step17 is interrupted;18–46 are NOT_RUN, including fresh step25 comparison and final GPU recheck. Do not fix code, rerun tests/gate or publish after this failure. The final documentation commit is report-only and NOT_FULL_GATED; the failed source remains6ed. Raw evidence: external resume-20261028/full-gate-1, full-gate-terminal.json and red-gate-stop.json.

Retain seven valid FPS bars and final a5/quarter-defence captures. The final-measurement-audit confirms measured runtime b040 equals gate runtime6ed; it does not turn the red gate green. Natural rebake10.493ms misses4ms and remains ISS-347/#658. All five owned issues343–347 remain OPEN, no closures. Remote v8/main and v8/wip remain3814828c06d1a7a211b13b54368e4e0a488cc0e3. Keep the unpublished worktree and original evidence; no force removal. This closes the current authorized execution attempt as BLOCKED/RED/NOT_PUBLISHED rather than seeking permission to bypass the rule.

Independent diagnosis assigns this failure to this pass's adapter integration, not a pre-existing broken test. The frozen test AST-extracts the pure performance_gate into an empty namespace and clears only CORE_P7_RUN; inherited CORE_UI_CONFIG activates the added branch, where root/output exist only in the real PYEVAL caller. A later separately authorized repair should move UI dispatch to the actual callsite, preserve the legacy function/frozen test, and add successor dispatch coverage. Do not implement that repair during this stopped attempt. Failure comment: https://github.com/hyunlord/new-world/issues/656#issuecomment-6462768982 .
