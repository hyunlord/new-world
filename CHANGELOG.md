# Changelog

All notable changes per pass. Format: [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
Older history (pre-V7, V7, revitalize passes 1–21, art passes 1–13b, world-model
WM-1–6) is summarised in [docs/HISTORY.md](docs/HISTORY.md); the full record is at
tags `v7-final-sim` and `v7-final-art`.

## M-CORE performance repair — 2026-10-27 (in progress, not published)

- Planner P-1–P-6 resume repair after the original p8 failure: ordinary red gates require diagnosis, correction and full re-run. The original failures below remain historical evidence. Required performance is standing step25 without justification plus C3 time ceilings; the mind/old0.5 ratio remains a target. Timing requires each interleaved repeat to start at1-minute load<8, with all three loads retained.
- Preserving commit p12 `1ca297b82` passes ten literal colony summary hashes, the exact D26 canonical outcome/why comparison and baseline identity. Valid candidate8 timing still fails the standing bar, invoking the separately validated planner P-3d fallback (D30).
- The scheduled candidate retains validated active ownership between agent-phased eight-tick full evaluations, with immediate incumbent/critical/finish guards and real effects every tick. It preserves initiating causal IDs without fabricated scores/witnesses; policy157nodes and scores stay frozen. Compact guard caching, batched lifecycle writes and metadata/history reuse reduce redundant work.
- Candidate14 preserves the scheduled calibration summaries while caching exact guard paths, leaving scratch untouched on exact hits, and acknowledging verified untouched primary snapshots. Core83/runtime-save168 tests, strict clippy and default release pass; final paired session passes N1000/N2000 and C3 but fails required n200_side64 mean/p99; publication remains blocked. Alias/invited/replay refresh fallbacks remain; the cache is derived and cold after load.
- Exploratory candidate2 shadow passes OFF2031192/2041927 and ON1684600/1692967, with independently reviewed categories, control identity,16live guard replays and13primitive adapter cases. Core53/runtime118 tests and strict clippy pass. These exploratory captures remain historical. Final committed826a7ecd round2 shadow and fresh held-out1264–1295 outcomes/why now pass, with exact frozen-source and independent review receipts. The final33-pair session fails required side64 mean1.348105/p991.314991 versus BASE; full45 gate stopped22:41:15KST for the22:43 hard stop: INCOMPLETE/wrapper143, steps1–24 exit0 (step12 timings INVALID), step25 interrupted,26–45 NOT_RUN. No publication or stall waiver is claimed.
- P-5 report-only diagnosis records all48chronicle kinds and explicit unknown ancestry; story-depth instrumentation follow-up ISS341/#647 remains OPEN. See CORE_STORY_DIAGNOSTICS and CORE_DECISIONS.

## M-CORE — 2026-10-26–27 (final FAIL, not published)

- Final p8 gate `44e0a0266c085d409c16c77f4a2a0181b8a2a750` completes42/45,exit1 in7012s (Oct27 04:02:02–05:58:54 KST). Failures:25 bench,41 native stall,45 core evaluation. Both stall arms complete;1×max91.652ms/two>50ms fails,½×max49.522ms/zero>50ms passes. Isolation42/data-only43/shadow44 pass. Goal-directed stop/report: no tuning, waiver, retry or push; worktree/evidence retained and locally fixed issues remain remotely OPEN (D24).

- Final performance is valid FAIL:33 accepted pairs/66 members, invalid0, admitted load4.1201171875–7.06201171875. N1000 tick2.082979→19.927196ms (×9.566682), mind/decision×13.150984; N2000 tick38.421873ms. Twelve standing1.1 regressions and both C3 time ceilings fail; the two CORE report-only bars also miss. After-build memory C3 passes; synthetic full-ring9060.066667B/person is a separate report-only miss, not natural occupancy. No waiver: publication BLOCKED, ISS339/#645 OPEN, full45-step gate pending (D23, CORE_PERF).

### Added
- Owner contract at `docs/v8/core/CORE_GOAL.md`; isolated branch from published M-UX2 `9855bd082ad88a4c49296410f206f148d98c4df9`.
- Measured decision/seam inventory: 91 semantic rows, 247 observer keys, 11 complete groups / 66 worlds, explicit zero counts and admitted inclusive costs in `docs/v8/core/CORE_INVENTORY.md`; evidence and sequencing decisions in `CORE_DECISIONS.md`.
- Approved depth roadmap preserved verbatim from Appendix A in `docs/v8/DEPTH_ROADMAP.md`.
- Off-by-default `perf_arms` observations with 247 runtime keys, primary/twin isolation, zero-hit rows and strict complete-fixture aggregation. Inclusive costs remain diagnostic, not quiet performance evidence.
- `worldsim_probe story-yield` schema 2 measures verified cause edges and distinct observed kind paths per calendar year, retaining missing-ancestor diagnostics and partial-year exposure. Twelve focused tests and a two-world 25-day ordinary-probe fingerprint control pass.
- Separate CORE outcome scorer for matched BASE/new-brain worlds and exposed-only cold-snap arithmetic; ten tests and synthetic CLI controls pass. Receipts bind both scorer and validator files. Held-out seeds have not been run.

### Changed
- P6 production switch committed locally as `fbc9a9babc9bb5df1d06ca1e655901e480521f66`, frozen policy unchanged. Committed p7 `be12050c951dfb025f3440d34b657eec0663b817` isolates the legacy selector in `runtime/baseline/decision`, separately removes dead colony gates/job seam and retires the live old-shadow runner; external freeze-4/p5 source and binaries remain. Two-file pure-move hashes and six preserved-body fuel regressions on the real mind pass. Final systems309, default/feature focused19+2+6, perf observer4, isolation source/registration4, formatting and strict Clippy pass; independent scoped review approves, and post-isolation capture now passes: 228,748 baseline values/zero differences, 64 chronicle files/19,683 entries equal excluding git_sha, nine OFF and ON summaries equal to the switch capture; ON determinism PASS (D20). P7 is committed locally. Optional `sim-baseline` crate dropped to avoid an additional shared-helper/registrar split; neutral mechanics remain shared (D20).
- Planner correction (2026-10-26 22:00 KST), recorded in CORE_DECISIONS D10: timing admission is 1-minute load <8 before every repeat with all three load averages recorded; comparison binaries must be interleaved in one session. Earlier contended arm timings remain INVALID. Three high-load windows 15 minutes apart permit non-timing work before final measurement.
- ROADMAP schedules M-CORE and M6–M11; superseded M6-finalization/M-PERF rows remain as history. VISION carries the approved v1.0 definition and retains the previous definition.
- The initial default and feature measurement-tool binaries match BASE on 228,748 baseline values, 64 chronicle files, raw samples and all nine OFF fixtures; both ON two-seed 24-day controls match, as does the feature eight-seed 48-day control. The corrected final-tool binary also passes the full baseline/OFF and ON 24-day comparisons. Mind modules and authoring records are implemented; final round-4 shadow-stage acceptance is 99.913210% OFF / 99.805452% ON, zero unexplained categories and zero residual port defects. Earlier round-1 FAIL and round-2 not-accepted results remain in CORE_SHADOW. The admitted BASE design share is s=0.699002982 at N1000; the total-tick ratio bar is 0.700498509 and mind/old-decision bar is 0.5, neither an achieved improvement. Production registration is switched in the working tree and its captures are complete; the switch is committed as p6; post-isolation identity also passes. Final paired performance, held-outs, native UI, story comparison, full gate and publication remain pending.


- Production colony registration and decoded-save routing now use the mind while baseline retains the old selector (CORE_DECISIONS D19). Switch validation passes 310 systems, 11 command and 7 full-save tests plus formatting/Clippy. The completed capture chain exits 0: baseline compares 228,748 values with zero differences, 64 chronicle files / 19,683 entries equal, and equal raw digests; all nine OFF fixtures have exact coverage with zero violations; the ON two-seed ×24-day probe reports DETERMINISM PASS. Evidence: `/Users/rexxa/work/core-data/switch-capture-1.log`, `switch-safety-1/`, `switch-safety-1.log`, `switch-steward-1/`, `switch-steward-1.log`, and `switch-dev/`. Colony changes after the switch are not claimed byte-identical to BASE. These captures are not held-out, timing or full-gate acceptance; p6 is committed as `fbc9a9babc9bb5df1d06ca1e655901e480521f66`; publication remains pending.

- Narrow external BASE Cargo.lock provenance validation retains immutable manifest/pin and original ancestor/equal-Rust-tree rules; all 15 validator fixtures pass after preserving initial RED/path-alias failure. No BASE rebuild, measurement or performance acceptance follows (`core-data/final-perf-plan.md`, D20).

- Final p7 features[] build and held-out/story capture chain complete (all seven exits0). All outcome bars pass, retaining seed1238 wolf deaths BASE0→mind3 and distinct cold exposure denominators17/19. Why coverage469721/469721 and strict critical12159/12159 passes. Story is reference-only MEASURED:64world-years each, entries7676/19722, zero deep chains, missing ancestry121/125; no story-quality or numerical acceptance claim (D22).
- Native UI02 REVISE is retained; native03 now passes independent pixel/integrity review of14 real PNGs after Why-locale and MindChoice47 repairs. Hunger/threat transitions, unchanged input hashes and EN/KO critical feed verified; one candidate per popup and2–3line summary wrap limit the claim. Chronicle RED10, intermediate fixture failure1 and finalGREEN12 retained; strict5files/FFI60 pass. ISS337/338 locally fixed, GitHub643/644 OPEN until publication. Final performance, full gate and publication remain pending.

### Fixed (working tree, not published)
- Routine cooldown fallback and invitation-preview boundaries (ISS-333), and Idle+Job presence versus ownership/release predicates (ISS-334), with preserved RED/GREEN evidence and final round-4 shadow audit. The narrow selected-node/AgentId gather-quarrel RNG correction (ISS-335) retains baseline RNG and port effects; measured colony exposure remains zero. Fixes are in local unpublished p5 `0ef68a1d2b6eb946e150963071b2e623fa58d4b0`; GitHub #639/#640/#641 are registered and OPEN pending publication. Broader ISS-048/#524 debt and historical ISS-132 are unchanged.
- Completed full-runtime calibration candidate on seeds 0–7 ×48 days: six outcome bars 8/8, cold-snap fuel 5/5 exposed (3 unexposed), determinism PASS, reason coverage 135,368/135,368 and critical choices 2,984/2,984 with zero failures. Policy frozen before held-outs; these are calibration results, not final acceptance or publication.

## M-UX2 — 2026-10-26 (p18 published under owner decisions)

- Added: owner playtest contract preserved first at `f2103ad47`, isolated `v8/mux2` worktree, strict visibility/stall and defence/flee regressions. Initial defence regression fails3/3; after repairing one missing-health fixture, all5 flee cases fail on actual behavior. Original failed runs remain recorded. No acceptance or publication claim yet.
- Records: scope, frozen dependency input and unchanged acceptance bars are in `docs/v8/mux2/MUX2_DECISIONS.md`. Prior M-UX failures remain historical evidence.

### Added
- M-UX2 Korean progress report, per-commit file inventory and twelve-panel evidence sheet; failed acceptance and unpublished remote state are explicit.
- Independent default-on steward defence, fast approach response, player-intent protection and deterministic physical-weapon assignment. Save11 freezes v7–10 layouts; Chronicle46 carries actual batch count. Focused defence17 plus2,058 exhaustive matching cases, fingerprint1, actual v10 migration1 covering4saves and legacy-wire1 pass; full gate and natural-world acceptance remain pending.
- Opt-in immutable M-UX2 observation, exact transient fatal-animal provenance, physical-wall housing checks and paired ON/OFF scorer. Observer4 and scorer14 focused tests pass. Both32-world arms must match frozen provenance; only ON thresholds govern acceptance. No held-out results yet.
- Optional native/script frame profiling, including system/prune/snapshot-freeze timing; simulation never reads it. The original debug baseline fails the stall bar at both zooms; no improvement claim yet.

### Changed
- Published p18 [`b19f91b15f8ad1b4f6b140b17d0d7960e895a4cd`](https://github.com/hyunlord/new-world/commit/b19f91b15f8ad1b4f6b140b17d0d7960e895a4cd) to both v8/wip and v8/main under owner D-A..D-C. Original failures remain historical; p16 owns the41/41 full gate. p19 records publication/registry receipts only; its documentation checks are attributed separately to the root verification run. Non-forced worktree removal failed because only untracked .omo/ and graft/ remained.
- Owner D-A..D-C corrects cold-snap acceptance to exposed worlds: frozen ON20/20 PASS alongside original20/32 FAIL, without rerun or policy change. The1000-person half-scale35.98FPS target moves to M11 (ISS332 open); all four fresh measurements are valid (N100/half164.882764487796; N200/half119.291035278699,full225.75111993954,quarter35.1016343917384), and publication requires N100/half≥60FPS. Retain rebake7.239ms, prior stall failures and ISS030 OPEN. Technical gate remains p16 41/41; no new gate/publication claim. Earlier blocked entries retain their historical contract context.
- Final technical gate on p16 `b530eb26d43ef99bb426ad3e454b294875dc230b` passes41/41,status0 at15:53:08KST, with fresh final-step stall capture and5,891 unchanged input hashes (D24). Both zooms pass the original bar; ISS331 is scoped process/fixed. Previous gate/capture failures remain. Held-out fuel20/32<24/32 and valid FPS10=35.982133928869<60 keep final acceptance BLOCKED/NOT_PUBLISHED. Subsequent p17 records are documentation-only and are not a new full-gate run.
- p15 gate ends40/41,exit1: step23 rebuilds the debug bridge after stall3 capture, and step41 correctly rejects its changed hash (D23,ISS331 OPEN). Default stall capture now occurs after all builds before strict verification; explicit receipts remain verify-only. Four order/failure regressions move RED4→GREEN4; full rerun is pending. Stall3 PASS remains limited to its captured binary; FPS10 and held-out failures still block publication.
- On p15, independently audited stall3 passes both arms:1×max45.363ms/bins1,2,2,2,1 and half49.829ms/bins1,0,2,0,1, with zero>50ms. Startup3.828/4.329s and all earlier failures remain retained; half has0.171ms headroom, with no causal cache-removal claim (D22). Valid FPS10 still FAILS at35.982133928869; natural rebake NOT_MEASURED. p15 full gate fails40/41 on binary binding, p14 remains40/41 exit1, and FPS/held-out failures block publication.
- Stall2 completes on p14 but remains FAIL:1×45.759ms/>50=0/bins1,1,0,0,2; half108.817ms/>50=5/bins5,0,3,2,0. Reject tree caching for no demonstrated getter benefit, not a proved causal regression (D21). After the p14 full gate (40/41 steps, exit1 solely failed stall receipt), restore all six candidate paths exactly to p13; retain capture fixes and original oracles. Candidate-specific probes are removed with the candidate, not to turn a failed test green. Post-removal Python191, fmt/clippy/debug bridge and docs/schema pass; full gate and fresh normal-debug stalls remain pending. No acceptance is claimed.
- Historical p14 candidate (later rejected by D21): retain the view-side tree conversion candidate with native 2/2 and actual Godot 22/22 frozen-dictionary checks; keep failed controls (D19). Reject/revert owner classification reuse despite 115 functional checks: valid FPS8=36.926600998528 shows no observed improvement over FPS5=37.2035. Its production/oracle/gate changes are removed and tested files remain external. Existing owner99/extent75 and all original oracles plus the actual tree probe remain; post-revert Python73 (16.741s) and all original oracles plus the tree probe pass; valid FPS9 also FAILS at37.0299790343991 (2,222 matching rendered frames/60.005435s,load7.175→6.294,topmost/visible/focus true,debug restored). The single foreground pair does not establish statistical significance. Tree stall acceptance and full gate have not run; held-out fuel20/32 remains binding FAIL.
- Fix capture rendering observability and window continuity (ISS330, D20): primary post-draw counts, Engine comparison, boundary window evidence and windowed/topmost capture retain all durations and thresholds. Focused Python73 passes in16.750s; valid FPS8 maintains2,216 matching rendered frames but FAILS60FPS at36.926600998528. Preserve FPS7 zero-draw failure; this is a capture-process fix, not performance acceptance.
- Align the live bridge API regression with the current `bridge_api_mux2.txt` inventory already used by gate step 13 (ISS329, D18). Only its docstring and expected-list path change; exact export equality, historical inventories and removed-API checks remain intact. Retained baseline: 204 tests / 1 failure in 30.393s; rerun: 204 tests OK in 28.598s, exit 0. No production API or policy changes; full gate remains NOT RUN.
- Gate evidence accepts the genuine format10 scene manifest with strict identity checks and preserves failure exceptions across lock context managers (D17). Original stall failures remain failures.
- Renderer/tooling p9 candidate: bounded label/extent placement, geometry caches, actual lit warmup and append-only bake evidence. Why42 and immutable text1625nodes/22PNGs pass scoped checks; independent UI review now passes22images. DDA/cell-reuse experiment is reverted/rejected despite222focused checks: valid FPS5=37.203518943 shows no observed benefit over FPS4=37.9236362595418. Failed runs remain external; no DDA fixture/reference ships. Post-revert37Python tests and all pre-experiment oracles pass; final5minute stalls and full gate remain pending. Frozen64world held-out is VALID/FAIL (ON fuel20/32<24); publication stays blocked. Separate engine_ui_facts6→7test-only correction passes9tests; original full preflight failure retained (D16).
- Native p8: frozen House/Bills/input/navigation optimization comparison preserves228,748 baseline values (0differences),64Chronicle files/19,683entries (except git_sha), and all9colony summaries. Independent dirty-nav counter controls correct an eager-cache discrepancy; focused heap/navigation references pass. Optional phase telemetry is diagnostic only. Current-policy migration of9genuine format6–9 files passes; historical-future0PASS/4FAIL remains separately attributed, not waived. Native workspace checks are in progress; final runtime/full-gate/held-out acceptance remains pending (D15a).
- Causal explanations open only through explicit Why/Alt actions, fit a compact anchored panel and use an empty-result toast. Korean text wraps by words with visible alert counts; saved defence settings and causal event categories are localized. Windowed input30/text1652 and independent two-scale visual checks pass, with prior failures retained.
- Half-scale keeps route-I figures; quarter markers have bright light/household/dark rings. Coincident figures gain non-obscuring counts and inspector cycling. Candidate5 passes eight immutable-input GPU scenes and independent visual review; marker21/coincident17 and selected/drafted21 checks pass. Current-policy scenes and 1,000-person FPS remain pending.
- Housing now requires a constructed perimeter and actual door; natural terrain cannot satisfy the physical-house target. Built-space job eligibility reuses the existing actionable sleep threshold and real bed availability, preventing exhausted workers from reserving work they immediately cancel. Housing/steward14+10, sleep6 and fuel11 focused tests pass. Calibration2 reaches8/8 physical houses and protects all4 exposed cold-snap worlds; unexposed4 remain failures in the fixed denominator, so the calibration aggregate remains FAIL.
- Avoid repeated house reservation search after workshops exist and cache deterministic workshop sort keys. Frozen before/after binaries preserve all228,748 baseline values,64 Chronicle files and all9 steward-OFF colony summaries;16 controlled reference scenarios also match. This does not establish the final frame-time bar.
- Flight chooses reachable safe roofs using actual routes and speeds, rechecks every30ticks, and uses a defended hearth only as fallback. Pending bites cannot kill a person after roof entry. Flee/wildlife24 focused tests pass.
- Fuel hauling precedes competing work at the worker's same Haul priority while preserving higher priorities, Haul=0 and existing jobs. Historical seed5 reproduced90loose fuel with reachable capacity and repeated Hunt/Chop selection; expanded regressions change4fail/7pass to11pass.
- These are intentional colony-policy changes: expect different orders, survival, fuel stocks and colony fingerprints, including OFF colonies affected by flight/hauling. Baseline identity and broader comparisons remain pending; do not call them behavior-preserving.

### Removed
- Rejected M-UX2 tree conversion cache and its candidate-only probes/checker tests. Six paths restore p13 exactly; original tree schema, renderer checks and all mandatory acceptance bars remain. Candidate source/evidence is preserved in p14 and external artifacts.

## M-UX + REVIEW — 2026-10-25 (in progress)

- Removed (C candidate, verification ongoing): eight unused wood-frame helpers and the127-line legacy ArtSpriteLibrary plus UID. Live daylight lookup, all15 world_0 UI bindings, assets and V7/baseline remain. See D46/C_CLEANUP_VERIFICATION.md.
- Stage B gate2 passed36/36 at5e20c65a1d52ba2a96a6e524614d3a7679f31a44 and both wip/main were verified after fast-forward publication; gate1 failure remains recorded.

- Changed (B gate repair, test-only): preserve the first full gate failure (31/36, status1 at60f337169), align five stale fixtures with the current catalog/API and explicit bootstrap. Three focused Rust tests, Python133, rebuilt Locale runtime/strict and workspace clippy pass; product behavior sources remain unchanged. A clean full rerun is required before publication (D45, ISS305/306/308–310).

- Changed (B5, focused Rust, windowed path and recapture verified; full stage gate pending): versioned flora selects and persists actual tree wood material; felling and aligned bridge/tree/stump/canopy reads share it. Format10 freezes explicit format9 migration and retains historical tree oak. D44/ADR0044 define intentional material and old-save visual differences; ordinary chop/haul exposes the original stump and actual pine/oak walls in six windowed captures; no blind species or owner approval claim.

- Changed (B4 H, focused and recapture verified): hunters keep stockpile priority and otherwise carry meat through the ordinary haul path to legal live-hearth ground. Both pickup and travel validate actual item specs and destination conditions; phase1 controls4/expected failures3 become23 final boundary/save/replay passes, with33 existing regression passes. Original baseline identity is unchanged; nine OFF recaptures have two literal-equal summaries and seven retained intended-delta sets, zero invariant violations. Calibration8×48 is VALID with1665 crafts; all107 natural haul terminals remain UNRESOLVED, not successful-delivery proof. Full B gate remains pending; the frozen four-arm heldout later passed the registered F4a/F9a and controlled arithmetic bars, without establishing uniform natural improvement (ADR0043/D42; B4_HELDOUT_VERIFICATION.md).

- Changed (B4 T, ordinary recapture verified): authored optional trader policy uses fresh adult/recent practitioner counts, rare-era precedence and weights30/10/1 while keeping saved NeighbourRule layout and historical purchase semantics. Public boundary tests15 and independent observer8 pass; existing neighbours16 pass. Original identity228748/0 and nine valid OFF recaptures pass: three summaries identical, six changed with preserved trade/economy diffs and zero invariant violations. Commit-linked BPT calibration8×48 is VALID with1644 exact primary crafts, controlled2304cold rows/32paid finishes; acceptance remains NOT_CLAIMED (ADR0042/D41).

- Changed (B4 P, unit verified): explicit migrant answers and actual arrivals now check current all-Agent capacity for the complete wave, without reserving slots or partial admission. Ten boundary cases and existing neighbours/FFI/save regressions pass; saved fields and generation-only capped semantics stay intact. Generic capacity-refusal wording remains ISS291. See ADR0041/D40 and `B4_P_VERIFICATION.md`; original identity228748/0, nine literal OFF summaries and BP calibration8×48 pass. No capacity event occurs in those calibration worlds, so no natural improvement is claimed. Full Stage B gate/publication remain pending.

- Added (B4 O, unit verified): opt-in immutable probe observations, controlled apparel/craft fixtures, strict independent arithmetic scorer and frozen evaluation preregistration (ADR0040/D39). Eight calibration worlds complete48days; all1773 primary craft events reconcile, controlled2304cold rows/32paid craft finishes validate. Ordinary summaries, command logs and saves match B6 with observation OFF/ON at ticks100/4000. Final original identity/nine OFF/no-player-orders ON comparisons pass, as do final48-day observation OFF/ON and post-craft save equality. Held-out acceptance is NOT_CLAIMED; first ON input mismatch and corrected rerun remain recorded.

- Changed (B6, unit verified): resource snapshot collection and basic Food/Water/Sleep consumption now live in cohesive child modules behind the original facades. Focused tests, rebuilt renderer agreement and fresh original identity/nine literal OFF/frozen Stage A ON comparisons pass; full Stage B gate remains pending. No performance improvement claimed.

- Stage B core checkpoint `e90b7bb86` is committed with original identity, nine literal OFF summaries and frozen Stage A ON summary/eight logs identical; core372/save14 and historical continuation3 pass. It is not published. Stage A p13 gate3 completed35/35 and was published to both targets; its held-out failures remain unchanged.
- Added (Stage B, under verification): pressure/animal registries, actual identity through effects/hunting/history/presentation, format9 with frozen6/7/8 readers, and seven-family selected-pack simulation/UI fixtures. Sparse novel illness provenance is cleaned on recovery/death. Gate step36 `data_only_ext` passes its focused simulation/headless run (39 checks); separate windowed pack40, bundled UI284, isolated M5 UI43 and scale220 checks pass. Current core378 and strict clippy pass; original identity, nine literal OFF summaries and frozen Stage A ON summary/eight command logs pass; full Stage B gate remains pending. Intermediate codec assertion, fixture failures and the shared-settings isolation incident ISS283 remain recorded.
### Added
- M-EXT core bridge exports `bootstrap_catalog`, `create` and `get_content_catalog`; snapshot schema9 introduced full-width core content columns and `wall_is_door`; ecology schema10 adds pressure/animal metadata. The gate uses the pass-specific103-export `docs/v8/bridge_api_mux.txt`; historical API lists are retained.
- Frozen staged owner contract (`docs/v8/mux/MUX_GOAL.md`, `ee10a4ea2`), execution decisions and initial issue registry. Stage A was published at ca9f531ca; B/C publication remains pending and failed A8 acceptance bars remain failed.
- Historical early Stage A component evidence: shared controls/alerts/lessons registries, visible camera dock, bulk read-only UI facts, and recorded steward commands with v6-save migration. Latest component checks:67 strict scripts/57 FFI calls,254 alert checks and128 learning checks. These are component results, not stage acceptance.
- Exact baseline and nine-fixture capture/comparison tool, plus separate opt-in steward evaluation and gate fixture.
### Changed
- Stage A gate2 completed2026-10-25 09:07:08 KST, exit0, on clean p12 f38d488f43156145a44898671e77b26b3ee6c496:35/35 steps exit0; workspace debug/release each2060 passed/0 failed/1 ignored. Receipt: `/Users/rexxa/work/mux-data/gate-stage-a-2/gate.txt`. ISS031/032 are fixed by p12 plus gate2; public28 named defects are fixed narrowly, while003/006/009/030 remain open. Physical input remains NOT_VERIFIED, A8 acceptance FAILED and measured latency unresolved. As of this document preparation, p13 final-report HEAD gate3 and publication remain pending; no future gate or owner acceptance is claimed.
- Stage A gate1 at6b0f572e4 finished2026-10-25 08:03:24 KST with status1:32/35 steps exit0. Workspace debug/release each report2059 passed/1 failed/1 ignored; both fail `chronicle::tests::discriminants_are_stable` (stale45 count, actual46). Inventory closure reports only DESIGN.md uncovered among2221 paths/1943 rows. All other steps, including steward2-seed×24-day determinism/ledger, pass. At gate1, ISS031/032 remained OPEN pending a fresh full gate; minimal count/code/name and single inventory KEEP-rule/row repairs do not establish a full-gate pass. Raw evidence: `/Users/rexxa/work/mux-data/gate-stage-a-1/{gate.txt,test_debug.log,test_release.log,inventory_closure.log}`.
- Renderer candidate adds dark water, natural-rock shore rounding and visible soft snow/drifts (MUX D9/D17/D18). D21 navy/night-tint correction passes CPU and actual GPU contrast plus a fresh20-image review at90/100; faint quarter-scale drifts and localized rocky ledges remain disclosed.
- UI scale run5 passes220 runtime assertions with nine-image visual review91/100; integrated UI run14 passes268 with eight-image visual review92/100. The current HUD sheet contains eight UI14 frames. An actual completed10-bed roof at tick2880 remains distinct from survival acceptance. Controlled Rust D27 assignment at tick5027 is separate evidence.
- D32 (`f1532878b`) compares global usable firewood with summed settlement daily requirements over the whole winter in both goals and alerts. D33 (`c20e1fdee`) repairs notification replacement/queue handling, causal-panel visibility and English/Korean command refresh; strict run5 passes67 scripts/57 FFI calls and scoped code review finds no actionable defect.
- Visual failures remain recorded: UI10 88 REVISE (empty action toast), UI11 89 REVISE (stale Korean commands/clipped English alert), UI12/13 each268 checks with1 fit failure and89 REVISE (316/308 then312/308 px). UI14 fits274/308 px without relaxing thresholds. D33 negative controls preserve18 failures/193 checks and1 label failure/6 checks.
- Calibration rounds1–5 remain FAIL. D30 freezes policy after five of at most six rounds; completed held-out ON scores stock32/32, beds30/32(required32), zero-cold-deaths31/32(required28), fuel21/32(required24), survival22/32(required32), ledger32/32. A8 acceptance is FAILED; no policy tuning follows held-out inspection and no waiver is implied.
- D29 cached sorting passes exact8-seed ON summaries/commands and full seed5 save equivalence, plus candidate-opt1 OFF identity228748 values/zero differences and9 literal fixture summaries. Candidate-a5 and paired held-out OFF also pass. D29 p9 is committed at91f029e9e. Active debug-window run2 exits0: ON/OFF advance means2422.547/1634.375us, ON max500592us. Earlier1.05-second spikes remain recorded; large stalls persist. These sequential diagnostics establish no calibrated speedup or gameplay-FPS acceptance. Both automatic gameplay rounds are now complete; see the separately scoped D34 evidence.
- D34 profile1 is committed as p10 78af5e245: only sim-systems dev opt-level1, with debug assertions/default overflow checks retained.673 source hashes verified and both tick1441 debug saves byte-identical. Four automatic gameplay phases complete: ON1x/4x11.949/47.173TPS, mean118.216/114.380FPS, max649.902/646.191ms including startup; OFF mean119.103/117.776FPS, max447.113/444.356ms. Recurring ON boundaries73–99ms remain; ISS030 stays open. No unchanged-FPS/calibrated-speedup/resolved-stall claim. Fresh scoped unit10/integration9 with1 ignored and explicit v6 migration1 pass; OFF safety passes9 literal summaries,228748 values/zero differences and64 details (git_sha excluded). Baseline perf exits0 at06:55:07 KST; dedicated ui-perf-2 exits0, stable=true,600 samples/phase: lifetime bound345us (272+73), startup/warmup retained, proving only the paused tick1 scene. AF01 UI14 bound545us uncertainty and prior381/455us measurements remain recorded; no overall-budget/FPS claim. Full gate and publication pending (`docs/v8/mux/D34_VERIFICATION.md`).
- D31 test-settings isolation is committed at d0c7f2b04; explicit paths and production ON defaults are preserved. Historical renderer fixtures explicitly select OFF through ordinary commands; corrected agreement and GPU ground run4 pass without golden changes. Pre-D34-profile full sim-test exits0 with1403 passed/0 failed/1 ignored across102 result summaries; the ignored case retains separate v6 evidence. Full gate, stage acceptance and publication remain pending.

- Changed (V8-PLAY2, 2026-10-25): protect launcher worktrees and embedded-data builds; add preparation-only mode and current Korean play/key guide (`docs/v8/play2/PLAY2_REPORT.md`).
## M-ART2.1 — 2026-10-24

The in-game look finished before M-EXT (`docs/v8/m_art2_1/MART21_GOAL.md`, record `MART21_DECISIONS.md`, report
`MART21_REPORT.md`): textured water with rounded shores, ART-TREES' summer-green species with seasons, the ¼× map with
baked canopy, household rings on the LOD markers, and D19 (`a1c20220`) brought through a gate. No simulation change
(identity byte-identical); one bridge read (three snapshot columns). Gate: 34 of 34 at `3a51636d` (bench ×0.983–×0.992,
alloc/agent +0 B); FPS 1× 290.2, ½× 190.2, ¼× 242.8, 1,000 people ½× 149.1 and ¼× 137.5.

### Added
- Rounded shores and hill edges (`scripts/world/art/ground_baker.gd`): the water outline is the closed-form signed
  distance field of the water tiles opened and closed by a disc of `water.shore_radius_px` (22) — straight shores on
  the tile edge, quarter circles at steps, diagonal water kept joined — cached as 32 × 32 quadrant masks per
  (quadrant, 2 × 2 pattern, noise phase) and applied with native blits; coverage `water.contour_soft_px` (6) dithered,
  the 10-px bank and 2-px ink following the contour; undug hill tops round against open ground (`hill.edge_radius_px`
  16) except where a face hangs (D2).
- Water lift (`ground_layer.gd` `water_lift`, `shaders/ground_chunk.gdshader` `water_lift`): per chunk an RGB offset on
  the unchanged 05 water texture so its mean clears every land ground touching water by 0.10 luminance at full night
  and 0.12 turns of hue, at every zoom's map tint, recomputed per tone (D3).
- ART-TREES import (`tools/v8/art_import.json` `source_commit` `f182769f`, `flora` section; `assets/v8art/flora/`): the
  species catalogue and habit defaults with schemas, 21 frames, 21 canopy masks, the canopy-mass texture and the
  data-only fixture `tests/test_alder.json`; `art_import.py --verify` checks catalogue paths and bound species (D4).
- `scripts/world/art/flora_resolver.gd`: the port of `resolve_flora.py`; parity on 352 cases
  (`tools/v8/flora_parity_fixture.json`, expected from the art line's own resolver) in gate step `resolver_parity`.
- `shaders/tree_season.gdshader`: per-pixel CIELAB season shift on the canopy mask, bare winter frames for deciduous
  trees, the snow cap band, a first-day-of-season blend with a per-pixel dissolve between frames.
- ¼× canopy bake (`ground_baker.gd` `_canopy_pass`, `ground_layer.gd` `_prepare_canopies`): at texture factor 4 tree
  tiles carry the season-tinted canopy mass, darkened at the edge; the tree layer hides their sprites
  (`trees.quarter_canopy`, `canopy_radius_px`, `canopy_edge_px`, `canopy_edge_value`).
- Household rings on the LOD markers (`agent_layer.gd` `household_keys`, `lod_markers.gd` ring colour per instance,
  `lod.household_ring_rgb` six hues) from three new read-only `get_agent_snapshot` columns `mothers`, `fathers`,
  `partners` (`sim-bridge` `collect_kin_columns`; snapshot_schema.md) (D5).
- Tests: `mart2_ground_test.gd` §7–8 (L-shaped river radius, straight edge, bank follows contour, hill corners, water
  texture std, contrast by day / winter / drought, fords, the step instrument along water edges);
  `tools/v8/mart21_trees_test.gd` (four seasons per species, bare winter, the data-only species, ¼× canopy and
  hidden sprites, households; windowed canopy saturation, autumn records, the five ¼× map classes' ΔE) in gate step
  `mart2_render`; `engine_appearance_ffi.rs` `kin_columns_line_up_with_the_snapshot_and_the_kin_component`;
  `tools/v8/mart21_shots.gd` (the compare set and the seasons strip from saves).

### Changed
- Trees resolve (species, size, season, variant) through the flora catalogue; bindings `trees.species` names species
  ids by weight (still a keyed hash: M-EXT replaces it with the snapshot's species column); stumps are the felled
  species' stump; berry bushes use the ART-TREES bush frames with their species' season tint.
- `render_params.json`: `water` gains `shore_radius_px`, `contour_soft_px`, `texture_lift`, `map_mode_colour`
  (replacing `colour`; `texture_std` removed); new `hill` and `trees` sections; `lod.household_ring_rgb`.
- The M-ART2 rendered tree check draws through `TreeLayer.make_plant` in summer; bare-land samples skip tree tiles.
- Review fixes (MART21_DECISIONS D7): the quadrant-mask cache key (canopy and shore masks collided; new
  `mask_cache_isolation` test), household keys kept by AgentId and cleared on reset, a dissolve keeps each frame's
  own season, the ¼× canopy tinted by exact per-pixel Lab, sprites hidden only where a canopy was baked, schema.py's
  remaining rules in the flora resolver, season names by code from bindings `trees.season_by_code`, no tint halo
  around ¼× forests; new tests `shore_pixels_quarter_circles`, `canopy_edges_all_sides`, `season_state`.
- `docs/images/mart2_compare_sheet.png` now shows old · M-ART2 · M-ART2.1 for six scenes; `docs/images/mart21_seasons.png`
  added; `docs/images/mart21_contact_sheet.png` replaces `mart2_contact_sheet.png`.

### Removed
- D19's flat water recolour (`tone.gd` `water_colour`, 18 lines; `water.colour`, `water.texture_std`).
- The TRIAL2b tree, stump and bush sprites (`assets/v8art/structures/{oak,birch,pine}_{large,small}.png`, `stump.png`,
  `berry_full.png`, `berry_bare.png`; at `v8/art-polish2` `c8ed8608`) and `tree_layer.gd` `tree_id` / `_tree_cfg`.
- The old straight bank and ink masks (`ground_baker.gd` `_ink_pair`, the water-side `_side_mask` path).

## M-ART2 — 2026-10-24

### D19 pre-gate terrain review
- Changed: ground-only ENV1 maxima to mean luminance 0.38 and mean saturation 0.30; separate blue-water contrast
  and per-terrain map tint controls in `render_params.json`. Standing-tree source files remain unchanged.
- Added: real-GPU tree saturation and all-zoom water/class separation checks in `mart2_ground_test.gd -- --rendered`,
  including a deliberately capped-tree negative control; ADR 0035 and the D19 evidence record.
- Prior gate/FPS numbers below describe `32cd61a7`; they do not certify the D19 renderer revision.

The adopted art in the real game (`docs/v8/m_art2/MART2_GOAL.md`, record `docs/v8/m_art2/MART2_DECISIONS.md`, report
`docs/v8/m_art2/MART2_REPORT.md`): 64-px tiles in one projection, route I people with face variants, the material
system for walls/floors/doors, ground/water/edges/hill faces by code, LOD markers, day/night/winter by tone, every
renderer tunable in `assets/v8art/render_params.json`. No simulation change (identity byte-identical).

### Added
- Part 0: ADR 0033 (the extensibility contract: kinds are data; the renderer names no content and resolves through
  `assets/v8art/manifest.json`, `bindings.json` and the material resolver) and ADR 0034 (renderer data flow:
  snapshot → resolution → chunks and sprites → tone); the extensibility checklist in `worldsim-architecture` and
  "the renderer resolves through manifests" in `worldsim-godot`; AGENTS.md's first screen names the contract.
- Part 1: `tools/v8/art_import.py` + `art_import.json` copy the adopted art from `v8/art-polish2` at `c8ed8608` by git
  object reads (bytes unchanged, idempotent, `--check`, `--verify`) into `assets/v8art/` (664 files, 13.9 MB):
  route I frames (36 looks, 340 frames), POLISH2 portraits (34 looks × 6 variants, 128 px), the material system JSON,
  schemas and textures, ground and water textures, the snow overlay, trees and structures with anchors, items,
  wolf and deer sheets, face variant and transform records, top colours; `manifest.json` (id, role, path, source,
  SHA-256, size, anchor/footprint, the 36 logical looks with their two fallbacks) and `LICENSE.md`; hand-written
  `bindings.json` (sim codes → art ids, one place to add content) and `render_params.json` (every tunable);
  `assets/v8art/README.md` and `ADDING_ART.md`.
- Part 2 (bridge reads; no simulation change, identity byte-identical): `get_agent_snapshot` columns `sexes`
  (0 male, also without `Sex`; 1 female), `look_ids` (index into the 36 logical looks, nested age × sex × top × hair)
  and `variants` (face variant 0–5; elders 3 or 4) from the pure collector `sim_bridge::ffi::collect_appearance`
  (`ffi/appearance.rs`). Hair style and face variant are pure functions of the AgentId (not saved).
- `rust/crates/sim-bridge/data/appearance.ron` (schema 1, read strictly with unknown fields rejected; a malformed file
  sends look 0 / variant 0): the look axes, the life stage and top mappings, the hair and variant hash constants and
  the elder variants.
- `get_space` columns `wall_materials` / `floor_materials` (the `MaterialId` u32 bit pattern as i32, `-1` none),
  `wall_styles` / `floor_styles` (the building blueprint's name in PascalCase: `LogWall`, `StoneWall`, `Door`,
  `PackedFloor`; `DugFloor` for a dug hill tile) and `wall_categories` / `floor_categories` (the material's registry
  category: `Stone`, `Wood`, `Animal`, `Mineral`, `Plant`, `Mod`; `""` unknown) — MART2_DECISIONS D6.
- `ffi/material_category.rs` and `sim-bridge/build.rs`: the bridge embeds every `sim-core/data/material/**/*.ron` file
  so a category resolves without the disk (the live `material_registry` is asked first; it is empty in today's worlds).
- `worldsim_probe --ascii … --people-json` writes `appearance` (`agent_id`, `x`, `y`, `sex`, `look`, `variant` per
  person) from the same collectors (C2 agreement extended to appearance).
- Tests `engine_appearance_ffi.rs` (pure function across save/load and a twin, spread of hair and variants, elders and
  children, the people JSON = the bridge, baseline) and two `engine_space_ffi.rs` tests (columns = `TileGrid`,
  categories from the material files on disk, no material hashes to `0xFFFFFFFF`, baseline empty).
- Part 3 (`scripts/world/art/`): `art_data.gd` (manifest, bindings and params lookups; F12 reload in debug builds with
  the toast "렌더 설정 다시 읽음"), `material_resolver.gd` (a port of the art line's `resolve.py` with its loader
  checks: style exact → blueprint → category default → document default; tint material → category → style),
  `ground_baker.gd` (16×16-tile chunks of 1,024² px baked on the CPU with native `Image` ops: runs of one ground per
  blit, 12-px noise-dithered blends, 10-px dithered banks into transparent water, displaced 2-px ink), `wall_baker.gd`
  (walls.py option (b): tops along the run, faces only where the south is open, ends at face ends, hill faces),
  `tone.gd` (ENV1 ground cap, winter desaturation and snow, drought, Lab shift, wall-top tone);
  `scripts/world/colony/ground_layer.gd` (lazy LRU chunk cache, reduced-resolution textures below 1×, incremental
  space rebake, doors as standing sprites, ford stones) and `shaders/ground_chunk.gdshader` (water under the chunk
  alpha with drift).
- Part 4: people as feet-anchored nodes in the y-sorted `ColonyView/World` (route I frame plan, E/W by flip, anchors
  from the manifest); `shaders/person_variant.gdshader` (the art line's Lab shift per masked pixel, elders' grey hair,
  ink protected, selection/drafted outline); `scripts/world/colony/lod_markers.gd` (below 1×: MultiMesh discs in the
  top colour with an ink ring, activity glyphs at ½×); fixed skin/hair masks (`tools/v8/art_masks.py`, 340 RGBA masks
  in `assets/v8art/people/masks/`, `skin_regions.json`); trees, structures, workshops, hearth states, items by count
  bucket, carcasses, resource nodes, zones, designation marks, site ghosts (`scripts/world/art/site_ghost.gd`),
  animals, visitors and rooms drawn from the bindings.
- Night by tone: one `CanvasModulate` (value × 0.55 with the blue shift) and an additive `PointLight2D` per lit
  hearth restoring value within 5 tiles; people away from a fire get a pale 1-px halo at night
  (`people.night_halo_*`, MART2_DECISIONS D17).
- Part 5 (UI): the inspector's 128-px portrait of the person's look and face variant and their look in words
  (`LOOK_TOP_*`, `LOOK_HAIR_*`, `UI_INSPECT_LOOK_FMT`); the HUD zoom indicator (`UI_ZOOM`); 32-px item icons from the
  bindings (`scripts/ui/item_icons.gd`) in the trade dialog and on workshop recipe buttons; order-bar icons and build
  previews/site ghosts at 64 px.
- Tests and gate steps: `tools/v8/mart2_ground_test.gd` (bake determinism and recorded hash, dithered edges that fail
  with blending off, tone caps, banks, a dig rebakes one chunk, data-only wall by fixture), `mart2_people_test.gd`
  (appearance = bridge, anchors in four facings, LOD switch, params reload, deterministic tint),
  `mart2_ui_test.gd` (windowed, 42 checks), `test_art_import.py`, `test_art_masks.py`,
  `test_mart2_extensibility.py` (no content literal and no 16-px tile size in the renderer, with planted failures),
  `resolver_parity.py/.gd` (39 cases equal to the art line's `resolve.py`); gate steps 31 `art_manifest`,
  32 `resolver_parity`, 33 `extensibility_grep`, 34 `mart2_render`, and appearance agreement inside step 23.
- Tooling: `capture.gd --mart2-set` (`mart2_shots.gd`, 14 owner shots, each also saving its world),
  `mart2_old_shots.gd` (the base renderer on the same saves), `mart2_fps.gd/.sh` (FPS, process and draw times,
  texture memory; release library at the debug path, replaced not overwritten); `docs/images/mart2_contact_sheet.png`
  and `docs/images/mart2_compare_sheet.png`, `docs/v8/m_art2/screens/SCREENS.md`.

### Changed
- `docs/v8/snapshot_schema.md` schema version 7. The bridge `#[func]` list is unchanged (97 items,
  `docs/v8/bridge_api_m5_5.txt`).
- `View.TILE` 16 → 64 with the base point (tile_left + 32, tile_top + 43); `scenes/main.tscn`: `GroundLayer` replaces
  `TerrainLayer`, a y-sorted `ColonyView/World` holds walls, doors, trees, structures, animals and people.
- Camera zoom ladder ¼×, ½×, 1×, 2× (default 1×) from the params, was 1–4× integer at 16 px.
- `renderer_agreement` compares looks and variants by AgentId besides tiles (settles on `activity`).
- Review fixes (MART2_DECISIONS D18): an F12 reload or a season/drought change re-applies the built space before
  rebaking (walls no longer vanish, hill caps follow the tone); a space change rebakes all eight neighbour chunks; hill
  tops darker by `ground.value_by_ground`; dug floors drawn as packed earth; drafted outline 2 px.
- Performance (MART2_DECISIONS D4 update; windowed, vsync off, release library): seed 0 day 20 at 1× 328.6 FPS, ½× 238.2,
  ¼× 120.65; the 1,000-person world at ½× 186.8, ¼× 173.5; texture memory ≤ 265 MB; chunk rebake max 3.7 ms. The
  simulation tick is unchanged (no Rust behaviour change; identity byte-identical).
- The M1–M5 UI tests and `smoke.gd` use `View.TILE`.

### Removed
- `scripts/world/colony/terrain_layer.gd` (the 16-px terrain drawing).
- The M1 tile sheet path: `tools/v8/gen_tiles.py`, `tools/v8/test_gen_tiles.py`, `assets/v8/tiles.png`,
  `assets/v8/tiles.json`, `economy_text.gd` `ITEM_SPRITES`, and `porter_presentation.gd`'s 16-px pack icon and route
  drawing (nothing called them).

## M5.5 — 2026-10-24

Neighbours and the M5 corrections (`docs/v8/m5_5/M55_GOAL.md`): two neighbours off the map send migrant waves who bring
their knowledge and seasonal caravans whose goods are clues to technologies the colony lacks and who buy its surplus
blades; the undrafted flee from wolves, pressures no longer stack, the hunter carries the meat home, a wrap takes one
hide and a knife four uses, mood and quarrels are tuned to be felt and every quarrel's cause is a record that can be
checked; ADR 0016 is amended for the memory instrument. Report: `docs/v8/m5_5/M55_REPORT.md`.

### Added
- ADR 0031 (neighbours and migrants), ADR 0032 (traders, barter, eureka); amendments to ADR 0016 (memory instrument),
  0027 (no stacked pressure), 0028 (flight, the hunter's haul), 0029 (one hide a wrap, four uses a knife), 0030 (mood
  tuned, causes as records).
- `ColonyRule::M5_5` (`WorldRules::colony` uses it): `StoryRule::M5_5`, `WildlifeRule::M5_5`, `MoodRule::M5_5`,
  `NeighbourRule::M5_5`.
- Systems: `FleeSystem` (123, every 3), `NeighbourSystem` (174, daily), `VisitorSystem` (175, every 3, offset 1).
- Neighbours (`ColonyState::neighbours`): two per world from the seed (`STREAM_NEIGHBOURS`); migrant waves on days
  8–12 and 20–26 and a seasonal roll after (cap 40 people), accepted or declined by the player; migrants join with
  reserved ids, their neighbour's knowledge (source `Migrant`, floored as apprenticed) and a wrap half the time.
- Caravans one a season (`STREAM_TRADE` goods, never in a major or before day 6), barter at 80 % / 120 % of the value
  table `sim-core/data/colony/values.ron`, `JobKind::Trade` hauling owed items to the caravan, the ledger terms
  `traded_in` / `traded_out`, attitude +3 a trade and +1 per 10 of a gift; eureka: an artifact no colonist can make
  triples the discovery draw of its producing nodes and meets their stockpile conditions.
- Commands `AcceptMigrants`, `DeclineMigrants`, `Trade` (reasons `NoVisitor`, `Unbalanced`), the probe and test order
  `DebugGive`; chronicle kinds 38–44 (`MigrantsArrived`, `MigrantsJoined`, `MigrantsDeclined`, `TraderArrived`,
  `Traded`, `TraderLeft`, `Eureka`); the chronicle's need samples (`NeedSample`, `CauseRef::Sample`).
- Bridge (`ffi/neighbours.rs`): `get_neighbours`, `get_wave`, `get_caravan`, `cmd_accept_migrants`,
  `cmd_decline_migrants`, `cmd_trade`; the agent snapshot's `fleeing`; `get_agent_m5`'s `fleeing`;
  `get_year_summary`'s `migrants_joined`, `trades`, `eurekas` (`docs/v8/bridge_api_m5_5.txt`, schema 6).
- Benchmark: the RSS outer bound (5 build-only repeats; ceilings `rss_per_agent_le_5120`, `rss_per_tile_le_128`).
- Gate step 30 `neighbours` (two worlds × 48 days: waves accepted, a trade, ledger exact).
- Probe: the `m55` block, `--neighbours on|off`; `tools/v8/m55_u_script.json`, `m55_u_nodraft_script.json`;
  `docs/v8/m5_5/m55_score.py`.
- UI: the migrant dialog (pauses the clock at 1×), the trade dialog with the balance bar, visitors drawn on the map,
  the N neighbours panel, the visitor banner, the tech map's rumours and eureka star, "!" over fleeing people and the
  inspector's flight line, chronicle lines for kinds 38–44 and the year line; 96 locale keys
  (`docs/v8/m5_5/M55_LOCALE.md`); `tools/v8/m55_ui_test.gd` (35 checks); `capture.gd --m55-set`.
- `MoodRule::cause_records` (M5.5 true): the need-sample causes; the M5 rule keeps M5's references.

### Changed
- The storyteller holds an illness back during or 2 days after a major and a major while 2 or more are sick.
- Undrafted people near a hunting wolf flee to a roofed room or the hearth they reach first (the decision seam skips
  them; `decision/system.rs` +0) and strike back at half damage; wolves run two tiles a step when they chase.
- A kill turns the hunt into a haul of the meat to the nearest stockpile cell; a carcass's leftovers are hauled first.
- `make_hide_wrap` takes 1 cured hide (was 2); a knife lasts 4 uses (was 8).
- Mood (calibrated on seeds 0–7, DECISIONS D15): pressure weight −80 (was −40), sick −160 (was −80), breakdown at 105
  (was 100); quarrel base 1 ‰ (was 2), relation divisor 200 (was 100), mood divisor 40 (was 25).
- A quarrel's cause is a concrete record (a need sample, a death, a pressure, an earlier quarrel), checked by
  `mood::check_cause`.
- Save format 6.
- Content lives in data (owner directive 2026-10-24, M5.5 DECISIONS D28): `sim-core/data/colony/neighbours.ron` (the
  neighbour rule, name keys, the goods table with `lacked` / `staple` tags, the guaranteed node, the migrants' top),
  `story.ron`, `wildlife.ron`, `mood.ron` (the M5.5 tunables), `values.ron` schema 2 (`made_by`, `clue_nodes`,
  `tags`); `ColonyRule::m5_5()` replaces the `M5_5` constants; `get_neighbours`' `name_keys` are locale key strings
  and `get_wave` adds `tops`. The files are read strictly (schema version, unknown fields; `rules::data`), every name
  in them must resolve (a test), and a broken file leaves its feature off. Behaviour unchanged (held-out evaluation
  compared equal, five runs × 32 worlds). M-EXT debt listed in M5.5 DECISIONS D29.
- Review fixes (M5.5 DECISIONS D25): items given to a caravan stay owed until a hauler picks them up and cannot be
  given twice; owed entries are matched by kind; eureka acts only under the neighbour rule and is counted only when
  the clue decided the discovery; a wave accepted on its way joins at the hearth; visitors with no meeting tile still
  wait and leave; `mood::check_cause` requires a record that fits the cause.

### Removed
- `docs/images/m5_contact_sheet.png` — replaced by `docs/images/m55_contact_sheet.png`.

## M5 — 2026-10-23

Pressure and story (`docs/v8/m5/M5_GOAL.md`): a storyteller sends cold snaps, droughts, wolf packs and illness at
times it chooses; people hunt deer, draft spears against wolves, scrape hides into wraps that keep the least
cold-tolerant warm first, and have moods that sour into quarrels and breakdowns, each with a "→ 왜?" chain in the
chronicle. Evaluation: `docs/v8/m5/M5_EVALUATION.md`; report: `docs/v8/m5/M5_REPORT.md`.

### Added
- ADR 0027 (storyteller and seasonal pressure), ADR 0028 (animals, hunting, drafting), ADR 0029 (hides, apparel,
  knife wear), ADR 0030 (mood, quarrels, chronicle causes).
- Rules `StoryRule::M5`, `WildlifeRule::M5`, `MoodRule::M5` in `ColonyRule::M5` (`WorldRules::colony` uses it).
- Systems: `DraftSystem` (121), `StorytellerSystem` (170, daily), `AnimalSystem` (171, every 3), `MoodSystem` (172,
  every 60 at minute 7), `QuarrelSystem` (173, every 30 ticks in five slices of initiators, one every 6; the nearest
  8 partners within 2 tiles, D18); person–animal fights in `ColonyState::fights`, resolved by
  `CombatSystem`.
- Pressure effects where the behaviour lives: cold snap (burn ×2, exposure ×2 outside a room, roofed sleepers warm only
  by a lit hearth, outdoor work ×1,300 ‰), drought (forage ×400 ‰, thirst ×1,200 ‰), illness (`Sick`, keyed spread in a
  bedroom, immunity for the rest of the pressure), beasts (a pack of 2–4 wolves at the map edge).
- Deer as a herd near the camp, hunting (`WorkType::Hunt`, `JobKind::Hunt`, the `Hunting` skill, gated by `tracking`),
  hunt marks and zones; meat to the larder, spoiling after 6 days; spears (`haft_spear`, 30 attacks); the knife (a
  flint blade on the drying rack's shelf, 8 uses; butchering with a held blade); `scrape_hide`, `make_hide_wrap`,
  `make_fur_cloak`; `Apparel` and the hourly dressing pass (`JobKind::Equip`, the "모두 가죽 옷" order); a worn top
  spares cold hours at its ‰ (keyed, `STREAM_APPAREL`).
- Items Meat, CuredHide, HideWrap, FurCloak, Spear, WolfHide (codes 11–16); components `Apparel`, `Sick`, `Drafted`,
  `Mood`; the chronicle's cause side table and kinds 29–37; streams `STREAM_STORY`, `STREAM_WILDLIFE`,
  `STREAM_QUARREL`, `STREAM_APPAREL`.
- Commands `SetDrafted`, `DraftMove`, `DraftAttack`, `DraftAttackPerson`, `DesignateHunt`, `AddHuntZone`,
  `CancelHunt`, `SetDressOrder` (reject reason `NotDrafted`).
- Bridge (`ffi/story.rs`): `get_story`, `get_animals`, `get_agent_m5`, `get_drafted`, `get_chronicle_why`,
  `get_year_summary`, `get_teach_suggestion` (the M4 owner decision: the teach dialog's suggested learner), the
  orders `cmd_draft`, `cmd_undraft_all`, `cmd_draft_move`, `cmd_draft_attack`, `cmd_draft_attack_person`,
  `cmd_designate_hunt`, `cmd_add_hunt_zone`, `cmd_cancel_hunt`, `cmd_set_dress_order`; the agent snapshot's `tops`,
  `mood_bands`, `drafted`, `sick`; the chronicle rows' `index`, `link`, `link_entry`, `link_code`.
- UI (Part 5): see `docs/v8/m5/M5_LOCALE.md` and the report — D draft, the drafted list, banners, the M mood overlay,
  the inspector's mood and apparel lines, animals drawn, F7-3 / F7-4 hunt orders, the chronicle's "→ 왜?" chains and
  the 이야기 tab; `tools/v8/m5_ui_test.gd`; `capture.gd --m5-set`.
- Measurement: the probe's `m5` block (`probe/m5.rs`) and `--story on|off|no-beasts`; `tools/v8/m5_u_script.json`,
  `tools/v8/m5_u_nodraft_script.json`; `docs/v8/m5/m5_score.py` (F1–F10, `--story` for the gate fixture); the
  allocator byte counter (`sim_test::alloc::Counting`, `worldsim_perf` only) and the bench's `m5_us_per_tick`.
- Tests: `colony_story.rs`, `colony_wildlife.rs`, `colony_hides.rs`, `colony_mood.rs`, `engine_story_ffi.rs`.
- Gate step 29 `story` (two worlds × 48 days with the storyteller: F1's schedule, ledger and reservations exact).
- ARCHITECTURE §12 "Story, animals and mood"; skills: architecture rules and procedures N (pressure kind), O (animal),
  P (wearable), Q (mood modifiers), testing procedure I (pressure and control runs), lessons in all three.

### Changed
- Work order: craft, build, **hunt**, quarry, mine, chop, split, haul. Save `FORMAT_VERSION` 5. Gate: 29 steps; step 13
  checks `docs/v8/bridge_api_m5.txt`; the soak runs `--story no-beasts`.
- The ledger rule's second fix (Part 0, D3): `bench_ledger.py check --pass` never compares a pass with itself;
  `append` needs a passing gate log or `--re-measure`; the ledger gains the `alloc/agent (B)` column; M3 and M4 were
  re-measured in one quiet interleaved session ("base M3", "M4 re-measure": M5's comparison base).
- Art record (Part 0, Part 7; ADR 0018): ART-PROD1's route I accepted 13/18 adults; ART-PROD2 ended not adopted (S1
  26/36); ART-PROD3 adopted route I at 34/36 under the owner's waiver. ROADMAP's next step is M-ART2.
- Code review fixes (D17): a tool pick-up leaves another worker's reserved stack and takes only its kinds; illness
  immunity for the rest of a pressure; a missed attack keeps the hunt job; a person's blow uses the weapon held at
  the hit; a drafted person holding a non-weapon arms; the pack's map edge is found once; knife wear resets with the
  blade.

- Benchmark against "M4 re-measure" (`docs/perf/PERF_LEDGER.md`; the quiet interleaved session of `53be405d`,
  15:39–15:43, 1-minute load 5.8–7.6, `$DATA/bench5`): tick mean ×0.997–×1.082 and p99 ×0.972–×1.065 on n200, n1000,
  n2000 and n200_script; M5 systems 11 / 47 / 87 µs per tick at N = 200 / 1,000 / 2,000 (budget 120 at N = 1,000);
  alloc/agent 1,871 → 1,871 B. **Justified: n200_side64 is above +10 % (mean 241.1 → 266.5 µs ×1.105, p99 490 → 540 µs
  ×1.102) because 200 people on 64² are 9× denser than on 192², so the 2-tile quarrel neighbourhood costs more only
  there.** That scene's p99 tick (540 µs, round 3) by system: AgentDecisionSystem 208.5, AgentMovementSystem 89.7,
  JobAssignSystem 79.7, QuarrelSystem 74.9, SettlementSystem 23.6, StockpileConsumeSystem 19.0, MemorySystem 13.3,
  AnimalSystem 7.5 µs; the mean of its 29 tail ticks (at or above p99): AgentMovementSystem 256.0,
  AgentDecisionSystem 199.7, JobAssignSystem 70.3, QuarrelSystem 55.2, SettlementSystem 24.3, MemorySystem 15.2,
  AnimalSystem 7.4 µs (rounds 1 and 2: QuarrelSystem 0 in the p99 tick itself, 86 and 59 µs in the tail mean). The
  gate's bench step takes this quote for that scene only (`BENCH_JUSTIFIED`, `BENCH_JUSTIFIED_SCENES=n200_side64`;
  `bench_ledger.py check --justified-scenes`, so it cannot excuse any other scene).

- The C3 memory ceilings (`bytes_per_agent_le_4096`, `bytes_per_tile_le_96`) judge the allocator counter's slopes
  when the run has them, RSS otherwise (D19: RSS per tile read 83–118 B for one binary across sessions).

### Removed
- `docs/images/m4_contact_sheet.png` — replaced by `docs/images/m5_contact_sheet.png`.

## M4 — 2026-10-23

Knowledge and technology (`docs/v8/m4/M4_GOAL.md`): skills are owned by people and die with them; knowledge spreads by talk at the evening fire, by watching and by teaching, fades when unused and is sometimes discovered; a 24-node stone-age map; the settlement's era is observed from what its people practise; ground stone and pottery can be discovered and made. Plus the M3 follow-ups: the benchmark ledger rule, river fords, tool wear. **The colony's behaviour changes**; the `baseline` world is bit-identical (identity PASS at every Part). Evaluation `docs/v8/m4/M4_EVALUATION.md` — held-out seeds 700–731: F1–F7, F9, F11 PASS, F8 (skill growth) FAIL (the economy saturates, D20), F10 8/8 after the gating fix (D22; 7/8 before), F12 partial (knowledge systems 3.3 µs per tick at N = 1,000, C3 ceilings, identity and replay PASS; the +10 % rule fails on the N = 200 scenes, justified below, on n1000 p99 and on RSS per agent). Report `docs/v8/m4/M4_REPORT.md`. Gate 28/28 green at `941410fc` (`v8/main`).

### Added
- ADR 0024 (knowledge data, component and gating), ADR 0025 (channels, forgetting, discovery), ADR 0026 (the technology aggregate and the era).
- Knowledge data `rust/crates/sim-core/data/colony/knowledge.ron` (24 nodes: lane, era, threshold, prerequisites, difficulty, discovery conditions, gates, marker weight, seeding) and `sim_core::colony::knowledge` (`defs()`, `gates()`, `Knowledge` with 6-byte entries and the cached practitioner bitset, `TechAggregate`, `TeachOrder`, `QUAL_CAP`, `era_label`); `KnowledgeRule` (`rules/knowledge.rs`), `ColonyRule::M4`, `EconomyRule::M4`; `STREAM_KNOWLEDGE`, `STREAM_CLAY`.
- Founders seeded by the table at worldgen (prereq-aware); gating of every recipe, blueprint and designation (bills, sites and marks without a practitioner wait; the universal work list is DECISIONS D11); the quality cap from proficiency; use stamps and +2 ‰ per gated job; `SkillKind::Pottery`.
- Systems: `OralTransmissionSystem` (160, daily at 20:30), `ObservationSystem` (161), `KnowledgeDecaySystem` (162), `DiscoverySystem` (163), `TechAggregateSystem` (164); apprenticeship as paired `Teach`/`Learn` jobs (`runtime/jobs/teach.rs`); chronicle kinds `ToolBroke` 24, `KnowledgeLearned` 25, `KnowledgeDiscovered` 26, `KnowledgeAtRisk` 27, `KnowledgeLost` 28.
- Chains: clay banks (`NodeKind::ClayBank`), `Designation::ClayPit`, `JobKind::DigClay`; items Clay, RawPot, ClayPot, GroundAxe; recipes `knap_fine_blade`, `grind_axe`, `form_pot`, `fire_pot`; blueprints `grinding_stone`, `pottery_bench` and the hearth's `hearth_fire` workshop; ground axes (120 chops, faster).
- Commands `DesignateClay`, `OrderTeach`, `CancelTeach`, `DebugKill` (probe and tests); reject reasons `Unreachable`, `NoPractitioner`, `TooYoung`.
- Bridge (`ffi/knowledge.rs`): `cmd_order_teach`, `cmd_cancel_teach`, `cmd_designate_clay`, `get_tech`, `get_tech_node`, `get_agent_knowledge`, `get_teach_orders`, `get_knowledge_waits`; `get_space`'s `ford_xs`, `ford_ys` (`docs/v8/bridge_api_m4.txt`, 75 items; snapshot schema version 4).
- UI: the HUD era line and the ⚠ 기술 위험 badge with the at-risk line and the loss toast; the T tech map; the K teach dialog; the inspector's 지식 tab; the pottery column; the build menu's pottery bench and grinding stone; F7 then 2 clay pits; clay banks and fords drawn; chronicle and "기다리는 중" lines; 147 locale keys (`docs/v8/m4/M4_LOCALE.md`); `tools/v8/m4_ui_test.gd`.
- M3 follow-ups (Part 0): fords (the tile grid's `ford` column, cost 30, on the hearth's shortest way to the largest cut-off forest within 40 tiles; D5) and the "닿을 수 없음: 강 건너" refusal; tool wear (stone axes 40 chops, then a stone; `EconomyRule::tools_break`); the ledger rule in `tools/v8/bench_ledger.py` (contended rows are never a base) and the quiet M2/M3 re-measure.
- Measurement: the probe's `m4` block (`probe/knowledge.rs`), resolvers `knows:`, `single_holder`, `clay_banks_near`, `holders_of`; `tools/v8/m4_u_script.json`, `tools/v8/m4_f10_script.json`; `docs/v8/m4/m4_score.py` (with `--f10`); the bench's `knowledge_us_per_tick` and `job_assign_us_per_tick`.
- Tests: `colony_knowledge.rs` (15), knowledge unit tests in sim-core and sim-systems, `engine_*` collector tests for the knowledge reads.
- Gate step 28 `knowledge` (the knowledge tests and the F10 loss fixture, 8/8); the soak asserts 0 `KnowledgeLost` in 20 days.
- ARCHITECTURE §11 "Knowledge and technology"; skills: architecture rules and procedures L (knowledge node) and M (gate), testing procedure H (knowledge fixtures, `debug_kill`) and the 28-step table, lessons in architecture and pitfalls.

### Changed
- `WorldRules::colony` uses `ColonyRule::M4`. Save `FORMAT_VERSION` 4 (`Knowledge` registered). Gate: 28 steps; step 13 checks `docs/v8/bridge_api_m4.txt`.
- The V7 decision system's site snapshot leaves out knowledge-gated colony sites, so passers-by no longer build them (D22; `decision/system.rs` +4 −1 in M4).
- Colony founders spawn with an empty `Knowledge` and seeding fills a flat table and boxes once (RSS per agent; D23).
- Code review fixes (D24): an apprenticeship sends its partners only when both may craft and reach the meeting tile (a learner with Craft priority 0 kept the teacher walking to lessons forever), and not when one is hungry, thirsty, fighting or under the unreachable cooldown; a lesson's wait counts from arrival; the evening telling no longer spends a listener's lesson on a node held at 350 ‰ or more; a load rebuilds the practitioner bitset from the entries; the M2 wear branch consumes the tool's own kind; the knowledge systems' intervals are tested against the rule.
- The soak script is `tools/v8/soak_script.json` (was `m3_soak_script.json`), with `Maintain 2` stone axes after the firewood bill (D6).
- Benchmark (`docs/perf/PERF_LEDGER.md` blocks "base M3" and "M4", one quiet interleaved session, load 4.3–7.1): knowledge systems 0.7 / 3.3 / 7.4 µs per tick at N = 200 / 1,000 / 2,000; `JobAssignSystem` +4.2 µs at N = 1,000; tick mean ×1.246 (n200), ×1.061 (n1000), ×1.059 (n2000), ×1.196 (n200_script), ×1.071 (n200_side64); C3 ceilings 4/4 PASS. Justification (bench): the N = 200 scenes cost +20–25 % because Part 0's fords make the bench world's forest across the river reachable and the V7 forage selectors run more; a no-ford build reads ×0.981 of M3, and knowledge adds under 1 µs at N = 200 (M4 DECISIONS D23). Not within +10 % and not explained by the fords: n1000 p99 ×1.106, and RSS per agent (the ledger's metric ×1.09 over five rounds, ×1.14 in the session; noisy at this scale, D23) — over the memory plan.
- M3's ledger rows from the contended session are relabelled "contended, not a base"; the quiet re-measure is the M3 pass row (M2 → M3 tick mean ×0.873 / ×1.053 / ×1.024 / ×0.889 / ×1.156).

### Removed
- `docs/images/m3_contact_sheet.png` — replaced by `docs/images/m4_contact_sheet.png`.

## M3 — 2026-10-22

Space (`docs/v8/m3/M3_GOAL.md`): pathfinding, walls, doors, floors, bedrolls, digging into hills, and rooms whose role changes how people sleep, store and work. **The colony's behaviour changes** (people walk paths round walls and hills; undug hill is impassable); the `baseline` world is bit-identical (identity PASS at every Part). Evaluation `docs/v8/m3/M3_EVALUATION.md` — of record (seeds 400–431 at `71c896b6`): F2, F3, F8 PASS; F1, F4, F5, F7, F9 FAIL; after the D17 fixes, a second held-out set (seeds 600–631 at `3d73712d`): every bar PASS except F7 (skill growth; the script's knapping work ends in the first days). Report `docs/v8/m3/M3_REPORT.md`.

### Added
- ADR 0021 (pathfinding: cost map, regions, A* with a memo), ADR 0022 (walls, doors, floors, bedrolls, digging), ADR 0023 (rooms and roles).
- `sim_core::nav`: `CostMap` (0 impassable; 10 plain, 12 door, diagonal 14/10), lazy regions per `nav.version`, integer A* (octile heuristic, ties `(f, h, y, x)`, no corner cutting, goals on impassable tiles reached from a neighbour), a 4,096-entry `(from, to)` memo; `NavState` in `SimResources.nav` (paths and version saved, the rest rebuilt). `runtime::nav::{tile_cost, build, ensure, refresh}`.
- Colony movement on paths (`runtime/agent/movement.rs`): seeks and haul-home follow stored paths, idle and play steps refuse impassable tiles, decision targets out of reach move to the nearest reachable tile of their kind; `CancelReason::Unreachable` with a one-day cooldown per person and target; the job assignment offers only targets reachable from the worker's region.
- Built space (`ColonyRule::space`, `SpaceRule`): `BlueprintKind` LogWall (log 2), StoneWall (stone 3), Door (log 1), PackedFloor, Bedroll (log 1) as 1×1 sites; commands `PlaceLine` (Bresenham), `PlaceRect { hollow }`, `DesignateDig`, `DesignateDeconstruct`, `SetRoomRole`; per-tile refusals (`OutOfMap`, `Water`, `Hill`, `Built`, `Blocked`, `Person`, `Edge`); `JobKind` Clear (items off a site), Deconstruct (half the materials back), Dig (`WorkType::Mine`, `SkillKind::Mining`, 150 ticks, 3 stone, boulder removed); `TileGrid.door`, `TileGrid.dug`.
- Rooms (`sim_core::rooms`, `RoomSystem` 152): 4-connected detection bounded by walls, doors, undug hill, water and mountain; 2–120 tiles, off the edge, made by the colony; stable ids (≥ 50 % overlap); roles Workshop > Hall > Storehouse > Bedroom > Unassigned with overrides; chronicle `RoomFormed`.
- Room effects: bedrolls are beds (a roofed bedroom first, never an unreachable bed); no night cold under a roof; rest ×800 ‰ in a roofed bedroom; hauls prefer and restock a roofed storehouse; `Maintain` bills count covered stock when a storehouse exists; a roofed workshop room crafts at 900 ‰ ticks; the evening idle stays in the hall.
- Bridge (`ffi/space.rs`): `cmd_place_line`, `cmd_place_rect`, `cmd_designate_dig`, `cmd_designate_deconstruct`, `cmd_set_room_role`, `get_place_preview`, `get_space`, `get_rooms`, `get_room_detail`; `get_colony_versions` `rooms_version`; `get_colony_overlay` `item_covered`; `get_agent_economy` `bed`, `bed_room` (`docs/v8/bridge_api_m3.txt`, 67 items; snapshot schema version 3).
- M2 follow-ups (Part 0): skill tie-break by rounds of (priority level, work type) with the most skilled first (D3); quality words by band (`QUALITY_BAND_0..4`, `UI_QUALITY_FMT`); the economy soak fixture `tools/v8/soak.sh` + `tools/v8/m3_soak_script.json` (gate step 26).
- Measurement: `worldsim_perf nav-bench` (gate step 27, report only), the probe's `m3` block (`probe/space.rs`), script entries with `"every_days"` and `"after": "pocket_dug"`, resolvers for houses, storehouses and hill pockets, `tools/v8/m3_u_script.json`, `docs/v8/m3/m3_score.py`.
- Tests: `colony_nav.rs`, `colony_space.rs`, `colony_rooms.rs`, `engine_space_ffi.rs`, `colony_world.rs::colony_camp_lies_on_open_ground`, the skill tie-break test in `colony_skills.rs`, nav and rooms unit tests.
- Tooling: the colony probe's `--save-at TICK --save-out FILE` (one seed; a save the game can load), `tools/v8/capture.gd --m3-set --load FILE` (the M3 screen set), contact sheet `docs/images/m3_contact_sheet.png` (`docs/v8/m3/contact_sheet_m3.json`, 10 shots of seed 606).
- ARCHITECTURE §10 "Space, paths and rooms"; skills: architecture rules for paths and built space, procedures J (blueprint kind) and K (room role); testing procedure G (the soak) and the 27-step table; lessons in architecture, testing and pitfalls.

### Changed
- Colony worldgen moves founders off impassable ground (D9) and opens the fewest hill tiles out of a closed camp basin (D17, `colony.tallies.camp_pass_tiles`); the nav map exists from tick 0.
- The job layer gives no new job to someone at its hunger or thirst seek threshold (D16; with built space only).
- Stockpiles, items and spills never land on impassable tiles or walls; quarry stone drops at the worker; puts are region-aware (D10).
- Save `FORMAT_VERSION` 3. Gate: 27 steps (`soak`, `nav_bench`); step 13 checks `docs/v8/bridge_api_m3.txt`.
- `decision/system.rs` is unchanged in M3 (0 lines).
- Benchmark (`docs/perf/PERF_LEDGER.md` blocks "base M2" and "M3", one interleaved but CONTENDED session, load 19–25): pathfinding and region checks cost 24 % on the 64² N = 200 scene and 5 % at N = 1,000 and 2,000 on purpose (movement 23 → 96 µs, `JobAssignSystem` 64 → 123 µs per tick at N = 1,000); C3 ceilings 4/4 PASS.
- Code review fixes (D19): a load rebuilds the nav map before any command; a spent path that has not arrived is planned again; a split room's id stays with the larger part; no building over a finished bedroll; an off-map `PlaceLine` is refused; dig refreshes the map before spilling; room detection stops collecting regions that cannot be rooms.
- The priority table's levels follow `work_types` order (`ffi::levels_in_work_order`; mining's column showed chop's level).
- README's "지금 화면" points at the M3 sheet.

### Removed
- `docs/images/m1art_contact_sheet.png` — replaced by `docs/images/m3_contact_sheet.png` (the M1-ART sheet stays in history at `v8/main` before M3).

## M2 — 2026-10-22

Things and making (`docs/v8/m2/M2_GOAL.md`): stone and flint as items, workshops, bills, work priorities, skills and quality, stone axes as tools, and firewood through the job layer — the first piece taken out of the decision tick. **The colony's behaviour changes**; the `baseline` world is bit-identical (identity PASS at every Part). Evaluation `docs/v8/m2/M2_EVALUATION.md` (seeds 300–331: E1, E3, E5, E6, E9, E10 PASS; E2, E4, E7, E8 FAIL), report `docs/v8/m2/M2_REPORT.md`. Benchmark ledger `docs/perf/PERF_LEDGER.md` blocks "base M1.5" and "M2" (N = 1,000: 1.71 ms; ceilings 4/4 PASS; the decision system 82.7 % → 80.0 % of the tick). Gate 25/25 green at `c0af2eb7` (`v8/main`).

### Added
- ADR 0018 (art direction: illustrated 2D top-down at 64-px tiles, concept 05 ink-and-tone, cut-out people, generated environment); VISION §6 rewritten for it; ROADMAP rows ART-TRIAL, ART-CONCEPT, ART-TRIAL2, M-ART2, M-PERF; HISTORY and `docs/archive/INDEX.md` list the art branches.
- ADR 0019 (items, bills and the economic loop), ADR 0020 (skills and quality tables).
- Items with material and quality: `ItemKind` Log, Firewood, Stone, Flint, FlintBlade, StoneAxe, Hide; `ItemSpec { kind, material, quality }`; stacks merge per (kind, material, quality / 10); per-kind stack maxima and carry loads.
- The colony's material table (`sim_core::colony::materials`): granite, limestone, flint, chert and oak embedded from `data/material/**` and integerized once into `MaterialStats` (‰); `material::parse_ron_str`.
- Resource nodes (`ResourceNode`, `NodeKind`): boulders on 25 % of hills and 2 % of plains, flint outcrops 1 per 40 river-bank tiles (`STREAM_BOULDER`, `STREAM_OUTCROP`); `Designation::Quarry`, `Command::DesignateQuarry`, `JobKind::Quarry` (batches of 4).
- Per-kind item ledger (`jobs::invariants::ledger_of`: created = ground + carried + held + delivered + shelved + consumed) with craft transforms (`craft_start`/`craft_end`, `crafted`, `craft_inputs`) and `burned`.
- Workshops (`BlueprintKind` KnappingSpot 1×1 Stone 4, WoodBench 2×1 Log 6 + Stone 2, DryingRack 2×1 Log 4; `MaterialNeeds` with two lines; `Workshop` with work tile, shelf and bills), recipes (`data/colony/recipes.ron`: knap blade, haft axe, split firewood), bills (`Make(n)`/`Maintain(n)`, suspend, order), `BillSystem` (123, hourly), `JobKind::Supply` and `JobKind::Craft`; commands `AddBill`, `RemoveBill`, `SetBillMode`, `MoveBill`, `SetBillSuspended` (logged and replayed).
- Work priorities (`WorkPriorities`, `Command::SetWorkPriority`; 0 never … 4; default 3, elders' quarrying 0); `WorkType` Craft > Build > Quarry > Chop > Split > Haul; assignment in passes by priority level.
- Skills (`Skills`, `SkillKind` knap, woodwork, chop, quarry, build), `XP_TABLE`, `SPEED_PERMILLE`, `QUALITY_BONUS_TABLE` (`STREAM_QUALITY`), material and temperament terms; stone axes as `Tool` (picked up within 24 tiles, `AXE_PERMILLE`, worn out after 200 chops into a stone).
- Firewood through jobs: `JobKind::Gather` (deadwood → firewood items, `WorkType::Split`), `HearthFeedSystem` (146, hourly: tonight's burn from firewood items into the settlement's wood), the wood bench's auto firewood bill.
- Bridge (`ffi/economy.rs`): `cmd_designate_quarry`, `cmd_add_bill`, `cmd_remove_bill`, `cmd_set_bill_mode`, `cmd_move_bill`, `cmd_set_bill_suspended`, `cmd_set_work_priority`, `get_workshops`, `get_work_priorities`, `get_recipes`, `get_agent_economy`; `get_colony_overlay` gains `designation_kinds`, `node_xs/ys/kinds/left`, `item_qualities` (`docs/v8/bridge_api_m2.txt`, 58 items; snapshot schema version 2).
- Tests: `colony_items.rs`, `colony_workshops.rs`, `colony_skills.rs`, `colony_firewood.rs`, `engine_economy_ffi.rs`; the M2 command log replays byte-identically (`probe::colony` test).
- Probe: the `m2` block (timeline, firewood by day, priority agents, skills), `nearest_nodes`, `blueprint_spot`, script entries that wait for a workshop (`"after"`); `tools/v8/m2_u_script.json`; `docs/v8/m2/m2_score.py`.
- ARCHITECTURE §9 "Items and jobs"; skills: architecture procedures G–I (item kind, recipe, work type), the per-kind ledger pattern, lessons.

### Changed
- The decision system's firewood arms are off in the M2 colony (`jobs::hearth::decision_firewood`; `decision/system.rs` +2 −1); the baseline keeps them.
- A job holder at work on a craft yields only to a critical need (calibration, DECISIONS D13).
- Save `FORMAT_VERSION` 2.
- Gate step `bench`: a C3 ceiling miss now fails (`tools/v8/bench_ledger.py check`), no justification excuses it; `tools/v8/bench.sh` waits up to 45 minutes for a 1-minute load below 8 and labels the run quiet or CONTENDED (DECISIONS D23).
- Gate step 13 and `tools/v8/test_bridge_api.py` read `docs/v8/bridge_api_m2.txt`.
- The colony rule is `ColonyRule::M2` (`ColonyRule::economy`).
- Pause moved from P to Space (P opens the priority table); F3 opens a build menu; README updated.
- Code-review fixes (DECISIONS D21): a player's firewood bill takes over the auto-bill's craft; cancelled sites return their materials with their material (`NeedLine::material`); the decision's food "loaded" check reads the firewood rule itself again; workshops and priorities are refused without the economy (`NotYet`); unfinished workshops' work tiles are protected (`ColonyState::site_work_tiles`); resource nodes block blueprints.
- A finished workshop is not a V7 building: no `building_registry` entry and no `BuildingPlaced` (it read as a hearth; D22).
- The firewood shortfall counts only firewood a hearth can use (D19: stray loads had stopped gathering and three evaluation colonies froze in year 3).

## M1.5 — 2026-10-22

The colony's own calendar and demographics, and the simulation contract (ADR 0016) that every later pass is held to (`docs/v8/m1_5/M1_5_GOAL.md`). **The colony's behaviour changes** (calendar, ages, births); the `baseline` world is bit-identical (identity PASS). The colony fixtures are re-captured at each behaviour-changing commit (`/Users/rexxa/work/m1_5-data/{p1,p4,p6}/fixtures/`). Evaluation `docs/v8/m1_5/M1_5_EVALUATION.md`, report `docs/v8/m1_5/M1_5_REPORT.md`; benchmark ledger `docs/perf/PERF_LEDGER.md` (N = 1,000: 1.79 ms; ceilings 4/4 PASS). Gate 25/25 green at `cdd4a775` (`v8/main`).

### Added
- ADR 0016 (the simulation contract C1–C5 and its gate steps), ADR 0017 (calendar and demographics); `AGENTS.md` points at ADR 0016.
- `CalendarRule` (`WorldRules.calendar`): baseline 7-day seasons (unchanged), colony 6-day seasons and a 24-day year; every derived length is a method.
- `BirthRule::Fertility` and `FertilitySystem` (priority 150, daily at 12:00): conception 33 ‰ a day for fertile (16–44), paired, fed women; `Pregnant { conceived_tick, due_tick, father }`, 18 days of gestation, 48 days after a birth; births through the settlement's birth path (`SimResources.due_births`). `RuntimeSystem::tick_offset` (default 0).
- Colony ages in years: elders at 50, lifespans 62 ± 8 years clamped to 40–85 (`AgingRule.lifespan_bounds_ticks`), founders 16–40, childhood until 15, infants until 3 (`LifeStageRule.infant_ticks`, `Child::is_infant`).
- Bridge reads: `get_time_snapshot` `year_index`, `season_index`, `day_of_year`, `days_per_year`; `get_world_info` `ticks_per_year`; `get_agent_snapshot` columns `ages_years`, `infants`, `pregnant_days`.
- UI: the top bar reads `1년 차 · 봄 1일째`; the inspector shows ages in years (`INSPECTOR_AGE_FMT` "%d세"), the infant stage (`INSPECTOR_STAGE_INFANT`) and a pregnancy (`UI_INSPECT_PREGNANT_FMT`); deaths read "{age}세에 세상을 떠남"; `UI_TIME_YEAR_FMT`.
- Measurement: `worldsim_perf bench` / `bench-one` (`sim_test::bench`, `WorldGenConfig::colony_stress`, `tools/v8/bench_script.json`), `tools/v8/bench_ledger.py` (+ tests); the colony probe's `--chronicle-out`, `churn.json` (the decision seam counts job releases by need, kind, stage and need value in `ColonyState::release_counts`, measurement only) and a `demography` block (births per woman-year, population per year, old-age deaths, coming of age); colony runs up to 480 days.
- Spatial index `sim_core::spatial` (`ChunkIndex`, 16 × 16 chunks, ring queries, `nearest_where` with the job layer's `(distance, y, x)` order); `SimResources.spatial`; `JobAssignSystem` finds every candidate through it (colony fixtures byte-identical before and after).
- `docs/perf/PERF_LEDGER.md`, `tools/v8/bench.sh` (the gate step `bench`), `worldsim_perf bench-summarise`.
- Text renderer `worldsim_probe --ascii` (`sim_test::probe::ascii`, from the bridge's collectors), `docs/v8/snapshot_schema.md` (31 getters, schema version 1) with `check_docs_helpers.py schema` (check 10), `tools/v8/renderer_agreement.sh` + `.gd` (text renderer = Godot agent layer, seed 100, tick 900), `engine_ascii.rs`, `test_schema_docs.py`.
- Job churn remedies (`ColonyRule::M1_5`: `yield_margin_permille` 150, `finish_leg_tiles` 12; `jobs::seam::keeps_job`): a job yields only to an urgent hunger, thirst or fatigue and never to foraging, firewood or social; a near load is delivered first. M1 fixtures: finished / taken 0.21 → 0.98.
- Review fixes: a mother does not conceive on her due day, and the postpartum wait counts calendar days (48); the probe reports conceptions per at-risk woman-day; `nearest_where` visits only non-empty chunks by distance, `JobAssignSystem` skips searches found empty earlier in the tick (`JobAssignSystem` 18 → 6 µs per tick in the scripted N = 200 benchmark scene); the agents' index is built on demand (`SpatialIndex::agents_at`; `SpatialIndexSystem` removed).
- Save and load (C1): `sim_systems::save` (bincode, `FORMAT_VERSION` 1, sorted hash collections, exact hecs rebuild with the allocator's free list, system state via `RuntimeSystem::save_state`/`load_state`); bridge `save_world`, `load_world` (`docs/v8/bridge_api_m1_5.txt`, 47 items); F5 save / F9 load with toasts (`UI_SAVE_DONE`, `UI_LOAD_DONE`, `UI_SAVE_FAILED`, `UI_LOAD_FAILED`); `tools/v8/save_roundtrip.sh`, `tools/v8/save_load_check.gd`, `save_load.rs`.
- Code-review fixes (DECISIONS D18): a job holder that may not yield skips the decision for the tick (`jobs::skips_decision`, `JobHolders`; no more undone choices with per-tick side effects); a critical need releases even a near load; a due birth falls back to the mother's current settlement or waits; save test through a pregnancy and a queued birth; the engine clock is asserted equal to the calendar.
- Gate steps 22 `schema_docs`, 23 `renderer_agreement`, 24 `save_roundtrip`, 25 `bench` (`tools/v8/gate.sh`); AGENTS.md and the skills (`worldsim-architecture`: the contract rules, save registry; `worldsim-testing`: benchmark, ledger, renderer agreement, the 25-step table; lessons in all three); ARCHITECTURE §8 "Contract and benchmark"; `tools/v8/save_load_check.gd --shots`.
- Tests: `colony_demography.rs` (`calendar_derivations`, `ages_in_years`, `fertility_rate_matches_rule`, `no_births_without_partner_or_food`, `pregnancy_to_birth_household`).

### Changed
- The cancel order moved from F5 to F6 (F5 is save).
- Season readers take the day count from the calendar (`WorldRules::season_days`); the year of a tick is the calendar's year.
- Colony: `FirewoodRule.winter_prep_days` = one colony winter (calibration, DECISIONS D9); births by fertility, not by the provisioned cooldown (10 days no longer double the colony: 0 births in 10 days, 11–15 people after 3 years on the calibration seeds).
- The baseline rules dumps gain the off-valued fields `infant_ticks: None`, `lifespan_bounds_ticks: None` and the baseline calendar.

## M1-ART — 2026-10-21

The colony view drawn like the mock-up: real ground, a canopy that reads as forest, buildings and log piles you can read, people you can find (`docs/v8/m1art/ART_GOAL.md`, report `docs/v8/m1art/ART_REPORT.md`). No simulation change (`rust/` untouched; identity, colony determinism and replay pass).

### Added
- `tools/v8/gen_tiles.py` → `assets/v8/tiles.png` + `tiles.json`: one code-painted sheet (ground families, forest floor, worn earth with dither edges, hill, rock tops and cliffs, deep/shallow water with 15 bank masks and 4 inner corners, stockpile floor parts, 3 species × 3 sizes of 24×24 crowns and their shadow, stump, log piles, hut with walls/bedrolls/hearth, blueprint ghosts, lean-to, two hearth frames, layered people with stand/stride/sleep frames and halo, carried logs, tools, axe chop glyph, hover frame, order-bar icons, people palettes); `tools/v8/test_gen_tiles.py` (23 tests).
- Terrain: value-noise grass clusters on the world seed, forest floor around trees, an eroded camp clearing, the water autotile (`terrain_layer.gd`).
- Canopy chunks with y-ordered overlapping crowns, shadows (not at 1×) and stumps for a game day (`tree_layer.gd`); ground-level overlay pass for stockpile floors, log piles and blueprint ghosts with delivered piles; backed blueprint label and a 3-px bar (`overlay_layer.gd`); two-frame hearth fire, lean-to shelter (`building_layer.gd`).
- People: outline and halo layers, walking frames, west flip, logs across the shoulders, sleepers on the hut's bedrolls, warm light near a fire at night (`agent_layer.gd`).
- Order bar icons, gold active mode, macOS hint `UI_ORDER_HINT_IDLE_MAC` (en/ko); `tools/v8/capture.gd --m1-set` dumps on-screen positions per shot and adds `m1_10_morning_2x.png`; `tools/v8/m1_gui_test.gd --no-vsync`; `tools/v8/contact_sheet.py --columns` and blank cells.
- `docs/images/m1art_contact_sheet.png`; ADR 0015; `docs/v8/m1art/` (goal, checklist, decisions, report).

### Changed
- `chronicle_panel.gd` shows HutBuilt (kind 22); `UI_CAUSAL_PANEL_PLACEHOLDER` lost its dev marker "(γ-2-β)".
- `daylight_controller.gd`: the hearth glow is centred on the hearth (it sat half a tile off); the night curve is shared (`tint_at`, `night_factor_at`).
- `worldsim-godot` (craft rules, procedures 11–12, lessons), `worldsim-pitfalls` (lessons), `rendering-m1.md`, ARCHITECTURE §7, inventory rules.

### Removed
- `tools/v8/gen_temp_tiles.py` (540 lines) and `test_gen_temp_tiles.py` (renamed and rewritten as `gen_tiles.py` / `test_gen_tiles.py`); `assets/v8/temp/` (the placeholder sheet); `docs/images/m1_contact_sheet.png` (replaced).

## V8-PLAY — 2026-10-21

Screenshots of the current game for the owner and a one-command way to play any version (`docs/v8/play/PLAY_GOAL.md`, report `docs/v8/play/PLAY_REPORT.md`). No simulation or game code changed.

### Added
- `tools/play.sh [version] [--seed N] [--speed 1-4] [--no-build] [--headless-check]`: resolves Godot, plays `main`/`wip`/`m1`/`v8-0`/`v7`/any ref from its own worktree under `/Users/rexxa/work/new-world-play/`, builds and imports only when stale, logs to `/Users/rexxa/work/play-data/logs/`.
- `tools/play/WorldSim.command`, `tools/play/WorldSim-v7.command`: double-click launchers (Desktop symlinks).
- `tools/v8/test_play.sh` and gate step 21 `play_check` (exit codes 3/2/4; headless check of main, m1, v7).
- `tools/v8/capture.gd --m1-set` (the owner shot set P1–P9) and `tools/v8/contact_sheet.py` (+ `test_contact_sheet.py`).
- `docs/images/m1_contact_sheet.png` and five M1 shots; `docs/v8/play/` (goal, decisions, screen report, contact-sheet manifest, report).
- Skill lessons (`worldsim-godot`: chronicle `NOTABLE` gap, the editor binary loads the debug library, scan-then-reset captures; `worldsim-pitfalls`: bash 3.2 and Hangul after `$var`).

### Changed
- README "빌드와 실행": the desktop launcher first, then `tools/play.sh`, the manual commands last; a "지금 화면" line.
- `tools/v8/check_docs.sh`: requires a `docs/images/*_contact_sheet.png` linked from README.
- End-of-pass checklist (AGENTS.md, `worldsim-docs-upkeep`): item 7, the play check and the contact sheet.
- `worldsim-godot` procedure 4 (capture and contact sheet) and new procedure 10 (play a version); the gate has 21 steps.

## M1 — 2026-10-20

The first colony the player commands (`docs/v8/m1/M1_GOAL.md`, report `docs/v8/m1/M1_REPORT.md`). The game now runs the **colony** world (192 × 192, 10 founders); the frozen **baseline** world stays the V7 game and still passes identity (ADR 0014). Orders are recorded commands (ADR 0012); people carry them out as jobs that give way to every survival need (ADR 0013).

### Added
- Simulation (all gated by `rules.colony`, `None` in baseline):
  - `sim-core/src/colony/`: items (`ItemKind::Log`, `ItemStack`, `Carry`, the `ItemLedger`), trees, chop designations, stockpile zones, hut blueprints (`ColonySite`, `MaterialNeeds`), reservations, `Command` / `CommandResult` / `CommandRecord`, `Job` / `JobKind` / `JobStage`; `ColonyRule` and `WorldRules::colony`; `SimResources::colony`.
  - `sim-systems/src/commands.rs` (`apply_command`, `submit`) and `runtime/jobs/`: `JobAssignSystem` (124), `JobExecuteSystem` (127), the decision seam, cancel and release, merge and spill, invariants.
  - `BuildingKind::Hut`, `TargetKind::Job`, chronicle kind `HutBuilt` (22).
  - Colony worldgen (`WorldGenConfig::colony`, `worldgen/colony.rs`): area-scaled bushes and deadwood, 18 wild regions asked, about 10,000 trees, the founder adult check. New keyed streams `STREAM_TREE_LOGS`, `STREAM_COLONY_BUSH`, `STREAM_COLONY_WOOD`, `STREAM_COLONY_FOUNDER_FIX`.
- Bridge methods: `cmd_designate_chop`, `cmd_designate_chop_tiles`, `cmd_cancel_area`, `cmd_add_stockpile`, `cmd_remove_zone_area`, `cmd_place_blueprint`, `can_place_blueprint`, `get_trees`, `get_colony_overlay`, `get_colony_versions`. The agent snapshot gains the `jobs`, `carry_kinds` and `carry_counts` columns. API list `docs/v8/bridge_api_m1.txt`.
- GDScript:
  - the colony view `scripts/world/colony/`: terrain, trees, buildings, order overlay, people, order tool;
  - `scripts/ui/hud/order_bar.gd`;
  - `Locale.trf`;
  - 24 localization keys plus `BUILDING_HUT`.
- Tooling:
  - `worldsim_probe --scene colony [--commands FILE] [--commands-out DIR]`, `worldsim_perf natural --scene colony [--commands FILE]`, `tools/v8/u_script.json`;
  - `tools/v8/verify_v8_0.gd`, `gen_temp_tiles.py` → `assets/v8/temp/tiles.{png,json}`, `m1_gui_test.gd`, `test_locale_trf.gd`, `compare_chronicles.py`;
  - gate steps 17–20 (`m1_tests`, `colony_determinism`, `replay`, `uid_check`).
- Tests: `colony_commands.rs`, `colony_jobs.rs`, `colony_world.rs`, `engine_colony_ffi.rs`, probe and unit tests; `test_gen_temp_tiles.py`, `test_compare_chronicles.py`.
- Docs: ADRs 0012–0014; `docs/v8/m1/` (DECISIONS, base numbers, V8-0 screen check, evaluation, report); the target image `docs/images/v8_target_view_b.png`.

### Changed
- The live game: `build_game_engine` builds the colony world. The bridge feeds 12 ticks per second at 1× (a day in 120 s), at most 8 per frame. Keys 1–4 set 1×–4× (12–48 ticks/s), F1–F5 choose the order mode (ADR 0004 gains the tick definition).
- `decision/system.rs`: +26 −1 lines (the job seam; the memory bias-only Construction arm is off in the colony, D20).
- `ConstructionSystem`: colony sites wait for their logs; finished huts are registered through `register_dwelling`, the path planned shelters use. `BedAssignmentSystem` uses per-dwelling capacity (huts sleep 4). `despawn_agent` drops a worker's load.
- `camp::regions` checks "within `region_water_max` of water" with a precomputed grid (same predicate).
- Identity compares chronicle files with `compare_chronicles.py`: the kinds vocabulary may grow at its end.
- `.uid` files are committed (check 8 of `check_docs.sh`). `.gitignore` ignores evidence images under `docs/` except `docs/images/`.
- Baseline dumps gained `colony: None` (rules and worldgen).
- Part 6 review fixes:
  - in the colony, `ConstructionSystem` frees a `Seeking { ConstructionSite }` agent whose site is gone (cancelled or completed); before, it froze until it died;
  - a finished builder whose state already moved on ends its job with `jobs::cancel::finish_in_place`;
  - `put_items` debug-asserts that no item is left over.
- Part 6 view fixes:
  - the top bar counts huts as shelters and blueprints as sites, adds the log count, and renames 식량 → 비축 식량 and 열매 → 야생 열매;
  - the carried log is a light bundle above the head, tunics are brighter, the child's eyes sit above a skin row, elder hair is white;
  - blueprint progress bars sit under a "통나무 d/n · 작업 p%" label, and order toasts name what was ordered;
  - the clock shows 저녁 from 17:00 to 21:00;
  - `Home`/`H` centres the camera on the camp;
  - deadwood is quieter.

### Removed
- V7 renderers and helpers (21 scripts: terrain, nature, building, agent and world renderers and their helpers), `scripts/world/cold_audit.json`, `shaders/settlement_outline.gdshader`.
- Bridge methods `art_declump`, `art_walker_offsets` (`ffi/art_kernels.rs`).
- The influence overlay cycled with Space (part of the deleted `world_renderer.gd`).

## V8-0 — 2026-10-20

The clean slate for the V8 settlement sim. The simulation is unchanged
(32 worlds × 120 days: probe summaries, raw samples and chronicles identical
to `v7-final-art`); the only intended behaviour changes are that the god-hand
panel and the V7 overlays are gone from the game and the intervention methods
are gone from the bridge.

### Added
- `WorldRules::baseline(seed)` and `WorldGenConfig::baseline(seed)` (`rust/crates/sim-core/src/rules/baseline.rs`, `rust/crates/sim-systems/src/worldgen/mod.rs`): the live game's rules as one flat literal equal to V7's `game_v26`; `sim_test::baseline_engine(seed)`; probe scene `baseline` (alias `game_v26`).
- Tests: `sim-core/tests/baseline_matches_v7_final.rs` and `sim-systems/tests/baseline_worldgen_matches_v7_final.rs` (the baseline against the V7 dumps, seeds 0, 1, 7, 42); `sim-test/tests/determinism.rs` baseline tick-for-tick test; `tools/v8/test_bridge_api.py`, `tools/v8/test_gd_reach.py`.
- Tooling `tools/v8/`: `gate.sh` (16 steps), `identity.sh` + `compare_summaries.py` (32 worlds × 120 days), `smoke.sh`/`smoke.gd`, `capture.gd`, `perf.sh` + `perf_summary.py`, `bridge_api.py`, `gd_reach.py`, `inventory.py`, `locale_keys.py`, `locale_prune.py`, `system_table.py`, `check_docs.sh`, `worldsim_systems` binary (the live system list).
- Docs: `README.md`, `docs/VISION.md`, `docs/ROADMAP.md`, `docs/HISTORY.md`, `docs/ARCHITECTURE.md` (generated system table), ADRs `docs/decisions/0001`–`0011`, `docs/archive/INDEX.md`, this file; `AGENTS.md` rewritten (78 lines), `CLAUDE.md` = `@AGENTS.md` + 8 lines.
- Skills `.claude/skills/`: `worldsim-architecture`, `worldsim-godot`, `worldsim-testing`, `worldsim-pitfalls`, `worldsim-docs-upkeep` (with sourced lesson tables), plus `systematic-debugging` and `verification-before-completion` moved from `.agents/skills/`; `.agents/skills/<name>` are symlinks to them.

### Changed
- The `game_v8 … game_v26` rule chain and its scenes became the one baseline (`rules.rs` → `rules/`; `rules/mod.rs` 1,821 → 819 lines); the perf `quick` world runs under the baseline rules (`quick_stress_baseline_s15100_n500_side64_fed.json`, same fingerprint as V7 `stress --rules game_v26`).
- Seven dead rule fields removed (`capacity_weighted`, two `winter_prep`, `rain_wet_burn`, `reset_on_split`, `avoid_gather`, `exchange_first`; DECISIONS D25).
- Names: pass and version suffixes removed from Rust and GDScript identifiers and files (probe `pass*` modules renamed by content; `draw_v18` → `draw`; `*_p10…_p13b.gd` renamed; D24).
- Structure: `sim-bridge/src/ffi/world_node.rs` (4,364 lines) split into `node.rs`, `engine_build.rs`, `snapshot.rs`, `reads_agent.rs`, `reads_settlement.rs`, `causal.rs`; `sim-systems/.../decision/agent_decision.rs` (4,337) into `system.rs`, `targets.rs`, `needs.rs`, `forage.rs`, `memory_bias.rs`, `wood.rs`; GDScript laid out as `scripts/core`, `scripts/world`, `scripts/ui/hud`, `scripts/ui/panels`.
- Tests: 108 `sim-test/tests/harness_*.rs` files became 68 domain files (`agents_*`, `settlement_*`, `family_*`, `relations_*`, `chronicle_*`, `engine_*`, `determinism.rs`); tests of version deltas and frozen worlds deleted, the rest ported to the baseline.
- Localization source switched from Fluent to the category JSONs (`localization/<lang>/<category>.json`), pruned to the 156 keys the kept UI uses.
- Every `src/**/*.rs` has a `//!` header (systems: priority, interval, reads, writes, events); every `.gd` a purpose + bridge-methods comment block.
- `.claude/settings.json` committed as the neutral `{}` (the V7 harness hooks pointed at deleted scripts).

### Removed
- The god hand: GDScript panel and scene, bridge methods `apply_intervention` and `get_intervention_state`, `sim-engine/src/intervention.rs`, `sim-systems/src/runtime/intervention.rs` and intervention-only fields (ADR 0010).
- V7 overlays and panels outside the kept UI (activity badges and tooltips, village activity summary, settlement relation lines, seek/death/need-bar/activity-trail overlays, restless meter, founding-group renderer, feed ledger, status panel, zoom LOD controller): `scripts/ui` 53 → 8 files (plus 31 moved to `scripts/core`, `scripts/world`, `scripts/ui/hud`); `scripts/test` 222 files (33,947 lines) deleted.
- `docs/`: 17,280 files (2.36 GiB of pass records, evidence and boards) → 27 files (the new docs and the V8-0 records); the index of what was deleted and where it lives is `docs/archive/INDEX.md`.
- `tools/`: 895 → 25 files (V7 harness, revitalize, art pipeline, world-model tooling).
- `sim-test`: 202 → 110 files (world-model recorder, ablation scenes, pass scaffolding, version-delta tests); `sim-bridge` test fixtures of 135 MB (art kernels).
- Assets: 888 → 198 files (15.0 → 12.9 MiB); shaders 11 → 3; scenes 3 → 1; localization 47 → 8 files and 5,287 → 156 keys (the Fluent sources and the key-owner policy).
- Whole tree: 20,299 files / 2,533 MiB → 714 files / 18.5 MiB (at the gate commit `672fffef`). Everything removed is at tag `v7-final-art`.

### MUX Stage C records (2026-10-25)
- Added complete311-row issue history and25 linked lessons; historical gate/report and art adoption boundaries remain distinct.
- Changed seven named issue statuses after published B proof;105 unresolved rows await GitHub mirrors. C cleanup safety passed at93a99ce16.
- Added verified GitHub mirrors for105 OPEN rows (#513–#617), protected existing#59, and ISS312/lesson26 for safe read-after-write recovery. Registry now312:175fixed/105open/30deferred/2won't fix.
- Added C full gate1 proof (36/36 at8ea4baa49), preserving natural tail-latency increase as ISS313 OPEN; registry313 with106 OPEN.
- Added GitHub#618 for ISS313; all106 OPEN mappings verified without altering the original105 issue bodies or existing#59.

- Recorded C gate2 FAIL35/36 atbcea734a (standing n200/side64 p99), fixed-schedule six-run diagnosis, ISS314 OPEN and lesson27. No product/threshold/ledger change; final clean documentation head requires a new full gate. Registry314:175fixed/107open/30deferred/2won't fix.

- Added GitHub#619 for ISS314; all107 OPEN mappings and unchanged original106/#59 verified after preserving the initial list delay. Final gate3 remains a separate publication prerequisite.

Owner-decision measurement checkpoint: valid N100/half164.882764487796 and N200/half119.291035278699; valid N200/full225.75111993954 and quarter35.1016343917384 complete the four measurements; quarter is report-only. Thirteen scoped registry rows are approved fixed;332total197fixed103open30deferred2wontfix. No GitHub closure or publication yet.
