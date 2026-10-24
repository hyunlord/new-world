# M-UX execution decisions

- D1 (2026-10-25): The attachment overrides checkout/branch instructions: only `/Users/rexxa/work/new-world-mux-dev`, branch `v8/mux`, based on fetched `origin/v8/main` `67a6232ab41a4bf5051b11303bcac97543b111f0`. No builds/runs/edits in pass4 or play2-dev; no process signals. First commit `ee10a4ea2` preserves the supplied goal verbatim.
- D2: Execute the owner's detailed A/B/C plan without an approval handoff. All unresolved choices are recorded here; no owner questions during execution. User scope takes precedence over workflow skill approval defaults. Native agents receive bounded ownership and cannot publish or recursively delegate.
- D3: Freeze a base release probe and colony fixture summaries outside Git before simulation edits. Colony summaries already exclude timing/Git metadata: compare them with literal `cmp`, not a new normalization filter. Existing baseline identity data remains `/Users/rexxa/work/m55-data/base/identity`.
- D4: Split behavior-changing commits from behavior-preserving refactors. Before each behavior change, record expected effects and the fixture comparison. Every refactor requires baseline identity plus soak/story/neighbours/F10/M55 seeds 0–7 × 48-day byte identity with steward off. No mixed commit and no relaxed gate.
- D5: Stage A implements camera/controls/help/alerts/learning/steward as required; optional drops follow A9, A6, B5, B4 only. UI styling derives from the existing Galmuri theme and panels. UI and Rust source ownership stay separate; one coordinator runs Godot and gates.
- D6: Every stage freezes a clean commit before the full gate, and publishes only that green commit to v8/wip then v8/main by fast-forward. No report-only ungated changes reach v8/main. A report records pre-gate evidence and points to the external exact-HEAD gate receipt; final delivery reports actual receipts after completion.
- D7: Raw builds, captures, calibration rounds, measurements and receipts live in `/Users/rexxa/work/mux-data/`. Preserve failed attempts and held-out results. Calibration is seeds 0–7 only, at most six rounds; held-out 1000–1031 is never used to tune rules.
- D8: Conservative hard-stop clock starts 2026-10-25 03:00 KST; deadline 2026-10-26 11:00 KST. At the limit finish the current step, gate, publish only green work, report partial requirements explicitly. Stage targets A 10 h, B 14 h, C 4 h do not reduce required scope.

## Execution ledger

| Step | Status | Evidence / next action |
|---|---|---|
| A0 isolated checkout, original goal, safety capture | COMPLETE | 382893842 instruments; base identity + nine fixtures complete, exact coverage/invariants pass |
| A1–A4 camera, controls, minimap, data-driven help | IMPLEMENTED / QA | Main scene integrated; camera27/minimap38 PASS; controls/help scale220 PASS; final source strict check pending |
| A5/A7 alerts and learning helper | IMPLEMENTED / QA | Alerts247/learning100 PASS; ui-run8 167 PASS; scale-run4 220 PASS and screenshot review92; physical input NOT_VERIFIED |
| A8 steward, replay, save and calibration | IMPLEMENTED / QA | Immutable rounds1–4 FAIL; round3 fuel3/alive5, round4 fuel5/alive4 of8; D27 preregistered before round5, two rounds remain; candidate-a3 OFF identity +9 fixtures PASS, a4 PENDING; held-out unused |
| A6/A9 optional goals/settings and A10 rendering | IMPLEMENTED / QA | Scale-run4 220 PASS; D21 CPU/actual GPU run3 failures0; comparison6 all20 captures reviewed90 PASS, disclosed minor limits; no drops |
| A11 UI/runtime/held-out evaluation, full gate, publish/report | PENDING | UI8 actual completed house tick2880:10 roofed beds/10 assigned living people; single-world round3 source only. Final gate/held-out/active full-game FPS pending; IN_PROGRESS / NOT_PUBLISHED |
| B1 whole-tree review and prioritized cleanup plan | PENDING | Review precedes refactors |
| B2/B3 registries and data-only extension gate | PENDING | At least items/blueprints/recipes mandatory |
| B4/B5 measured behavior changes; B6 safe refactors | PENDING | Separate commits and fixtures |
| B7 full gate, publish/report | PENDING | No completion claimed |
| C1 proof-backed dead-code cleanup | PENDING | Preserve frozen baseline/research paths |
| C2 issue/history backfill and GitHub mirrors | PENDING | Initial registry started; gh authenticated as hyunlord; backfill pending |
| C3 final gate, publish, clean removal and report | PENDING | Owner launcher checkout remains untouched |

## D9 — Preserve signed terrain contrast by lifting the darkest land preparation

Expected visual change (before implementation): A10 flips water below land while retaining the existing absolute luminance gap of 0.10, including the darkest daylight multiplier 0.473. The current hill palette luminance 0.2042516 cannot meet this even against black water (upper bound 0.0966110). The prepared land palette therefore receives a 0.28 luminance floor while retaining its existing 0.38 ceiling; dark-blue water is bounded at 0.055. The conservative minimum night difference is (0.28 - 0.055) × 0.473 = 0.106425. Water contrast compression avoids negative-channel clipping. Source art remains intact. This is a separate renderer behavior change, not a simulation-preserving refactor claim; capture and signed contrast tests must verify it. Snow/drift and shore changes remain A10 requirements.

## D10 — Measurement coverage and stockpile interpretation

The stockpile target is an area of `6 + adults/2` cells, rounded up to the smallest enclosing square; interpreting it as a side length would produce 121 cells for ten adults. Steward evaluation emits opt-in `mux` measurements only when explicitly enabled so OFF fixture summaries retain their original schema. Missing first-winter or cold-snap observations cannot count as successful protection. Calibration is still restricted to six recorded rounds on seeds 0–7, with the original 32-world acceptance thresholds unchanged.

## D11 — Separate new-game preferences from loaded state

`SimReads.new_game_started` is emitted only for a successful explicit `reset_world`, not the shared `world_reset` signal also emitted by save loading. The coordinator applies the user's default steward preference at initial game creation and this explicit boundary; saved steward settings are authoritative on load. No simulation field is edited from GDScript; the preference is a recorded `cmd_set_steward` command.

## D12 — Preserve failed UI runs and isolate settings

The first minimap harness exposed Godot's unsupported `%g` formatter and a test-oracle quantization mismatch: `Color.to_rgba32` rounds while RGBA8 `Image.set_pixel` truncates. The corrected oracle independently passes expected colors through RGBA8 and retains exact equality; run3 passes 38 checks. The first camera harness used the shared settings path and may have recorded synthetic camera learning events. Later tests inject an external settings store before scene readiness. No unknown pre-existing settings are reverted without a backup; this process incident remains in ISSUES. Camera run1's pinch/speed failures remain preserved until diagnosed.

## Calibration ledger — round 1 (pre-run registration)

Seeds 0–7, 48 days, no player-order script, steward ON. Frozen probe, exact rule data and all Rust source/data hashes are under `/Users/rexxa/work/mux-data/calibration/round1/`. This is the first of at most six calibration rounds. Candidate OFF safety capture runs from the same frozen binary. Simultaneous correctness probes are not performance measurements. Held-out seeds 1000–1031 have not been executed or inspected.

## D13 — UI evidence requires visible, unoccluded controls

Integrated run1 executed 156 checks with three failures (pinch, tick-zero camp fixture, missing steward Chronicle kind 45 in the visible feed). Two independent screenshot reviewers also found helper/orderbar overlap, alert/minimap overlap, an unbounded help overlay and selected-person occlusion. Visual verdict is REVISE, 58/100, persisted in local workflow state. Shrink retained Control sizes to content, bound alert/help scroll regions, keep the inspector beside the map dock, and reject UI-obscured map highlights. Preserve run1 images and failures; later passes must verify both geometry and actual screenshots.

## D14 — Deduplicate the same engine gesture, not similar genuine gestures

Camera diagnostic3 proves one submitted gesture object reaches Window.window_input twice in the same process frame, then produces two transformed objects. Godot 4.6's Input dispatch path matches this observation. Preserve source identity through event metadata and reject only repeated source identity within a frame; do not debounce by factor or elapsed time, which would drop genuine gestures. Regression checks must include two distinct same-frame gestures and exact pan distance. Synthetic window input is not physical-trackpad acceptance.

## Calibration round 1 — result

FAIL: stockpile day1 8/8, walled beds day10 8/8, first-winter cold deaths zero 8/8, cold-snap firewood nonzero 4/8 (required6), alive day48 4/8 (required8), exact ledger8/8. Original held-out bars remain unchanged. Round1 artifacts and binary stay immutable. Diagnose deaths and fuel windows before a preregistered round2; held-out seeds remain unused.

## D15 — Preregister calibration round 2 policy before editing

Round1's 73 deaths were combat; all beast events had zero drafted people. No starvation, dehydration or exposure deaths occurred. Seeds1/2/3 exhausted fuel after losing their workforce; seed5 retained 90 ground firewood but no stockpiled/settlement fuel. The existing simulation already prevents wolves targeting roofed people and automatically flees toward reachable roofs. Do not change wolves, auto-draft or relax thresholds.

Candidate adjustment: retain the six rule families, rule3's missing-beds trigger and one-building limit. Its first room may include existing stockpile cells plus data-defined service space as well as beds; preserve existing orders/zones/buildings and only place beds on valid empty cells. Prefer enclosing the stockpile. Rule4 prefers workshop work/output tiles near the actual usable hearth stockpile, avoiding a footprint-origin distance proxy. Expected effect: routine storage/work under a roof and less fuel-hauling pressure; larger construction risks the day10 bed bar. Survival benefit is unverified. Record exact source/data/binary hashes before round2; only seeds0–7, same48days, no player orders, no held-out tuning.

## D16 — Keep contour-width thresholds when geometry changes

Ground run1's bank failure was exactly one far-band pixel. Diagnostic2 proved local (20,51), radius32 signed distance -9.361538 lies inside the unchanged10px bank; the old radius22 distance was -12.382308. The Cartesian oracle sampled a rounded corner while asserting straight-edge distance. Select verified3×3 straight shoreline topology, retain near40..360/far0/ink0.95 bars, and add the measured rounded-corner membership assertion. Clean run3 passes all checks except the intentionally stale golden. The intended golden is refreshed only after repeatable fresh-bake equality; original1b046ce17467 and new9fdf37fec519 remain in external logs. Signed night contrast remains at least0.1112 against0.10, hue at least0.4683 against0.12.

## Intermediate OFF safety comparison

Frozen round1 binary vs immutable base: baseline identity PASS (228748 shared simulation values, zero differences;64 detailed files equal excluding git_sha), all nine colony summaries literal byte-identical. `/Users/rexxa/work/mux-data/candidate-a1-compare.log` records PASS. This does not cover the later command-lifecycle/fire-helper fixes or round2 policy; recapture the final source before its commit.

### D15 addendum — clear a valid shared-home site through ordinary orders

Before implementation: keep the original nearest stockpile unchanged. A read-only house preview may ignore only harvestable trees that will be cleared; every other placement rejection remains authoritative. An otherwise valid template containing such trees first emits ordinary `DesignateChopTiles` (or waits on existing chop marks), then places the structure on a later interval only after actual clearing. Respect non-chop player marks, vetoes, knowledge and the data camp-exclusion radius. No direct tree/terrain mutation or zone relocation. Regression must show preparation precedes structure and preserves existing zones. This may improve shelter coverage but is not an evaluation result.

## D17 — Natural rock banks participate in shore rounding

Before edit: the lower-camp river's square shoreline remains because natural blocking rock is marked crisp and excluded from water/land contour classification in both the sampled and optimized baker paths. Classify natural rock as shore land while keeping constructed walls crisp. Expected visual effect is rounded natural hill/river corners at the requested32px radius, with structure edges unchanged. Add paired natural-rock/constructed-wall regressions and rerun existing shore/contrast tests; this is an intentional renderer behavior change and may change baked golden bytes again. Snow shader changes wait for correctly ordered season captures and a visible tree/wall fixture.

## D18 — Make the existing snow accumulation mask visible

Before edit: corrected20-image comparison3 establishes seasonal canopy ordering and rounded natural-rock banks, but visible tree/wall fixtures still show gray ground with no recognizable accumulation (visual79/100 REVISE). The current snow pigment0.38 is close to prepared land0.28–0.38. Use data-defined snow color[0.78,0.82,0.86], retaining alpha≤0.45, feature≥512px and existing drift masks. Expected peak composite luminance is approximately0.520473–0.575473, not a change to the prepared-land brightness cap. This intentional winter appearance change increases dark-water separation; source art stays unchanged. Recheck all contrast/mask tests and actual day/night1×/¼× captures before acceptance.


## Later component evidence

UI scale-run3:220 checks/0 failures; independent screenshot verdicts92/100 and91/100, both above the registered90 threshold. Integrated ui-run5:161/161. Physical trackpad and owner visual acceptance remain unclaimed. Python suite91/91 after giving window aspect and edge-pan margins named constants; the unchanged tile-literal gate no longer misclassifies those values. A subsequent indentation error in that edit invalidated renderer capture4; check-only confirms the fix and the failed capture is retained. New captures are required.

## Calibration round 2 — frozen run

Round2 is now frozen under `/Users/rexxa/work/mux-data/calibration/round2/`: original parameters, source hashes, binary hashes, explicit registration and rule data. Seeds0–7×48days, no player orders, ON; second of at most six rounds. A fresh OFF safety capture runs from that same immutable binary. All held-out seeds remain unused. This run includes the accepted-command lifecycle correction, shared authoritative fire multiplier, and D15 shared-home/work-output policy. Performance is not measured during concurrent correctness captures.

## Calibration round 2 — result

FAIL: stockpile8/8, beds8/8, zero cold deaths8/8, cold-snap fuel3/8 (required6), alive48days3/8 (required8), exact ledger8/8. Survival/fuel regressed from round1's4/8. Preserve both rounds and diagnose the actual shelter/work/combat trajectory before any third proposal. No held-out seeds have been used.

## D19 — Preregister a terrain-aware shelter hypothesis for round 3

Round2 produced36-tile bedrooms but zero covered storage in every seed; larger empty roofs did not implement the intended logistics protection. Do not increase room size again. Interpret a log-wall template as completing an enclosure against existing natural hill boundaries already recognized by the room system, rather than replacing those boundaries or digging terrain. Use existing pure room detection to require one connected roofable enclosure containing the beds and intended storage, with a reachable hearth-facing door; emit only ordinary legal wall/door/bed/chop preparation commands and preserve all player work. No direct terrain/room mutation, new refuge behavior, wolf changes or drafting. A terrain-blocked functional fixture must actually demonstrate covered stock before any round3 run.

Add opt-in diagnostic context to ON-only probe output before that round: placement/fallback constraints and spatial roof/storage/work context; combat victim activity/needs/refuge context where available without changing simulation state. The OFF summary schema and existing exact safety net remain unchanged. Current data cannot distinguish hunting, hauling, food/water or routine exposure, so no claim that this policy will meet survival bars is made. Hunting/need/routine/wolf mechanics remain untouched. Round3 remains seeds0–7 only and is not started until the candidate is frozen.

### D19 addendum — count usable interior cells, not blocked hill cells

Before edit: the ordinary-command terrain fixture still cannot enclose storage because blocked natural interior cells consume the fixed rectangular capacity. Retain the exact bed+stock+service open-cell demand; extend candidate height only enough to supply those usable cells, bounded by the existing room_max_tiles rule. Require connected room preview, all planned beds/storage and legal reachable door. Do not raise service_cells, reserve demand or alter terrain. This compensates unavailable cells and is not evidence of calibration success.

## D20 — Preserve hue separation under unequal night RGB tint

Before edit: real GPU tests fail the unchanged hue gap0.12 for hill rock at all four night zooms. Worst observed gap0.0506 (waterH0.6182 vs rockH0.5677); luminance remains above0.10. The CPU oracle considered daytime hue but only attenuated luminance for night, missing unequal RGB hue rotation. Set both water mean_colour/map_mode_colour to[0.050,0.018,0.120], with matching fallbacks, retaining attenuation-only texture processing and source art. Expected dark indigo-blue hue is0.718954 day/0.705242 night, predicted worst hue gap0.137542 and luminance gap0.127970. These are estimates, not pass results; rerun actual GPU and visual reviews. Add a negative control for the old palette under actual night RGB tint. Original tests/thresholds and failed logs stay intact.

### D19 addendum — vary aspect ratio without increasing usable capacity

Before edit: fixed6-column interior cannot contain the existing hearth and stockpile in the diagnosed geometry even though the same34 usable cells can fit as7×5. Search bounded aspect ratios for the same bed+stock+service demand and existing room_max_tiles cap. Prefer a valid connected enclosure containing both existing hearth and stockpile when available, without moving either or adding demand. An interior hearth means the nearest legal perimeter door. Reject incidental disconnected rooms; do not claim all outdoor routines become sheltered. Keep fallback limitations explicit, deterministic candidate ordering and measured planner cost. Support-workshop placement must not consume a pending house boundary while preparation commands are completing; use ordinary legal placement choices, not direct state mutation or cancellation of player work.

## Intermediate round2 OFF safety and latest UI evidence

Frozen round2 vs immutable base: baseline identity PASS, zero differences,64 detailed files equal excluding git_sha; all nine colony summaries literal byte-identical (`candidate-a2-compare.log`). This does not cover pending D19 round3 changes. Camera run5 passes27 checks after the named-constant/indentation repair. Legacy layout run2 passes23 checks including header reservations. Integrated UI run7 passes163 checks; tick781 house sites have27 wall plans,10 bed plans and2 delivered materials, but zero completed walls/beds. The screenshot proves construction has started, not a completed house.

D20 GPU run2 passes all hill/night pairs, including quarter-scale hue gap0.1606 and luminance0.1284. Strengthened CPU winter check still FAILS at hue0.1006; unchanged threshold0.12. Diagnose the exact winter class before further palette edits.

## Initial UI cost measurement

`ui-perf-1/mux-ui-perf.json`: real frame_post_draw intervals,120 warmup+600 sampled frames per phase in a paused scene, stable world/camera/viewport. Enabled rendered mean8.3751ms and disabled8.3889ms; process mean13.3909ms vs15.0105ms is contaminated by sequential sampling/background load and is not a claimed speedup or exclusive UI cost. Minimap observed max285us, conservative frame381us, below500us. This is one synthetic scene, not full-game FPS acceptance; final bridge/source still require verification.

## D21 — Keep navy water by correcting excessive night blue tint

Before edit: D20's winter forest-floor hue0.604682 cannot remain0.12 away from true blue water near2/3; raising water red sufficiently yields violet (candidate[0.08,0.018,0.12], dayH0.767974). Reject describing violet as the requested blue. Restore D9 navy water[0.015,0.05,0.12] and lower only global night_tint blue1.08→0.96, retaining red0.86, green0.94 and night value0.55. Across99 diagnosed land samples this predicts minimum night hue gap0.156578 and signed luminance0.119048; these estimates require actual CPU/GPU verification. Daylight remains unchanged. This is an intentional night-rendering change, including sprites, so compare camp/river/people/trees/walls in day and night at1×/¼× and reject if night mood/readability regresses. No simulation behavior, source art, contrast threshold or hue exception changes.

## D22 — Count alert causes by authoritative household/order identity

Before edit: independent A5 review found newborn rows are per infant rather than distinct household and ignore beds assigned to other household members. Unreachable-order rows are per worker avoidance rather than distinct active order. Correct these read-only UI aggregations using established simulation household membership and order identity; do not invent grouping that differs from the simulation. Add multi-infant/multi-worker regression fixtures while preserving distinct causes and cycling. If bridge facts lack membership or order identity, add only deterministic read fields and verify read-only behavior; do not change simulation state.

## D23 — Retain observed learning completion and complete tooltip coverage

Before edit: the chop lesson checks only currently active chop marks. A steward or player may finish chopping while the learner is still on camera, so the already observed accomplishment disappears before Next. Persist observed lesson satisfaction/provenance separately from progression; never infer achievement from unrelated action counters. Preserve skip/off behavior and user-level learning progress across saves. Add regression for true→false observation before Next and restored settings, plus registry-based input hints for helper/settings controls without shortcuts. This changes only UI training state, not simulation or save state.

### D22 unreachable evidence boundary

The simulation stores only worker+goal tile+avoidance expiry after removing the failed job. It cannot attribute a failure to a particular simultaneous bill/designation/site. Group these facts by distinct target tile and present them as unreachable work locations, preserving real target coordinates and sorted worker provenance. Do not fabricate an order ID or count all candidate co-located orders as confirmed failures. This corrects duplicate cause cycling while making the measurement scope explicit.

## D19/D21 verification before round3

D19 controlled construction fixture passes at tick3587:16/16 storage cells,10 beds and hearth under one40-cell connected roof with undug hill adjacency; every source blueprint accepted and inventory invariants pass. It levels bounded test terrain, co-locates founders and removes local obstructions, retaining outer resources; no roof/material grant. This is not natural-world evaluation. D21 CPU and actual GPU tests pass (`a-ground-rendered-3.log`); all20 comparison6 captures reviewed independently at90/100. Remaining localized rocky ledges, faint quarter-scale snow/tree drifts, slightly rectangular wall fringe and near-black quarter-scale night water are disclosed, not erased by the numeric pass.

## Calibration round3 — frozen run

Third of at most six rounds: seeds0–7×48days, ON, no player orders. `/Users/rexxa/work/mux-data/calibration/round3/` freezes source/data/binary hashes and registration before launch; bridge build5/release build4 passed. D19 planning and opt-in non-mutating diagnostics are included. Candidate-a3 OFF safety capture uses the same frozen probe. Held-out1000–1031 remains unused. Concurrent correctness jobs are not performance evidence.

D22/D23 verification after bridge build5: alerts247 checks PASS; learning100 checks PASS; bridge facts test9 PASS. Schema56getters789keys documented. These component results do not substitute final exact-HEAD gate.

## Calibration round3 — result

FAIL: stockpile8/8, beds8/8, zero cold deaths8/8, cold-snap fuel3/8(required6), alive48days5/8(required8), exactledger8/8. Survival improves from round2's3/8 but remains below the unchanged bar. Preserve immutable round3 and inspect spatial/flight/death evidence before any round4 policy. Held-out seeds remain untouched. D22alerts247/D23learning100, strict67files57FFI and realv6 migration1 pass; final gate remains pending.

## D24 — Protect the full cancelled steward footprint

Before edit: independent review found player cancellation of a non-origin workshop tile removes the full site but the steward's stored origin-only ownership misses the intersection, allowing reissue before two days. Resolve the affected full footprint before applying the player cancellation, retain state changes only after accepted commands, and veto every affected footprint tile/target for the existing two-day duration. Add a regression cancelling only a non-origin cell. No change to OFF behavior, player-command acceptance, cancellation geometry or duration; this repairs the explicit player-wins contract.

## D25 — Bound protection geometry by the ordinary command rectangle

Before edit: parent review found `lifecycle::targets` enumerates raw stockpile/hunt/quarry rectangle coordinates before ordinary command validation. With steward ON, an oversized but normally clipped rectangle could allocate unbounded targets. Reuse the existing deterministic `commands::rect_tiles` geometry (map clipping, reversed corners, outside-map rejection), exposing it only within the crate. Add small-map oversized/reversed/outside rectangle regressions; preserve ordinary acceptance and OFF simulation state. Do not introduce a second rectangle algorithm or change limits.

## Integrated UI run8 — actual completed housing evidence

Round3 bridge source:167 checks PASS, including ordinary steward construction and completed housing at tick2880. `ui-run8/steward-housing-completion-state.json` records1 roofed room,10 built roofed beds,10 living people and10 assigned bed matches; the corresponding completed-house PNG is an actual simulated result, unlike the renderer-only snow fixture. This is the main scene's single-world functional evidence, not calibration or held-out survival. D24/D25 subsequent source corrections require a final rebuild/runtime pass.

## D26 — Preregister round4: distinguish static capacity from transient clearance

Before implementation: round3 still roofs no stock in8/8. Initial-state diagnostics (no world ticks, preserved outside the repo) show the original planner selects detached houses in8/8. Ignoring only transient people and counting actually eligible bed/service cells yields shared stock+hearth geometry in4/8. Enumerating capacity-sufficient heights until the existing perimeter is legal proves additional shared40-cell layouts in seeds1/6/7 (383/72/385 valid candidates respectively). Seed4 remains blocked by fixed nodes; only hearth coverage is demonstrated. No fixed obstacle was removed, no simulation/Flee rule changed, and these are static feasibility results, not survival.

Implement actual eligible capacity accounting, bounded height alternatives near existing stock/hearth anchors under unchanged room_max_tiles, and smallest legal area within the stock/hearth coverage tier before proximity. For a preferred statically valid site blocked only by people, retain ephemeral planning reservations and wait for clearance instead of committing immediately to detached fallback. Final ordinary commands retain the real people/terrain/building/node/zone/reservation validator; never emit an invalid source plan or move a person. Keep bed+service demand, six rule families, cooldowns, stock position, reserve-day targets, wolf/Flee/draft behavior and all acceptance bars unchanged. Regression checks must prove clearance waiting, real validation on issuance, bounded footprint and no gratuitous enlargement; preserve the ordinary construction proof. Risks: waiting can delay beds beyond day10; seed4 stock coverage remains unresolved; outdoor exposure remains. Freeze exact source/binary before round4 of6, then test only seeds0–7×48days. D24/D25 correctness fixes are included and disclosed; held-out seeds remain untouched.

D24/D25 targeted validation: non-origin cancellation initially failed2/3 tests; complete footprint protection passes3/3. Bounded16-cell rectangle negative control then fails as expected before D25; final shared-clipping suite passes5/5 including u32::MAX, reversed/outside rectangles and unchanged OFF results/state. Runtime binaries must be rebuilt before subsequent acceptance. The ordinary rectangle algorithm is unchanged; only crate-local visibility is widened to reuse it.

## Planned final Steward cost measurement

After final source freezes, use existing worldsim_perf natural/colony with ordinary tick0 SetSteward scripts, seeds0–1×4days, alternating ON/OFF repeated samples and recorded background load. Report per-system max_us_incl_warmup as well as steady mean/p99/max; a360-tick planner spike can be invisible at p99. This is a timing experiment on already used calibration seeds, not another48-day tuning round or held-out evaluation. Do not infer full-game FPS acceptance from it.

Frozen round3 OFF safety comparison passes baseline identity (228748 shared values,0 differences;64 detailed files equal exceptgit_sha) and all9 literal colony summaries. It excludes subsequent D24/D25/D26 source changes; final source recapture is still required.

## Calibration round4 — frozen result

Round4 is immutable under `/Users/rexxa/work/mux-data/calibration/round4/` with source/data/binary hashes and registration (bridge build6, release build5). Fourth of six allowed rounds, seeds0–7×48days, ON, no player script; held-out seeds remain unused. FAIL: stock8/8, beds8/8, cold8/8, fuel5/8(required6), alive4/8(required8), exactledger8/8. Surviving seeds1/4/5/7,17 people. Candidate-a4 OFF capture is running from the same probe. Analyze spatial/death evidence before spending either remaining calibration round.

## Active-frame performance evidence boundary

The launcher loads macos.debug `rust/target/debug/libsim_bridge.dylib`; release probe timing cannot establish actual window responsiveness. Existing ui-perf-1 executes tick0 before warmup then keeps tick1 paused, so its PASS only covers paused UI/minimap cost. A separate controlled active-window harness will retain tick0 and every360 boundary, recording actual bridge advance cost and frame_post_draw intervals. Synthetic one-tick/frame stepping remains distinct from normal-speed gameplay FPS.

## D27 — Preregister House completion role before round5

Round4 roofs every hearth/workshop but shared rooms classify Workshop; bedroom_sleepers remains0 across all8 seeds. Existing bed selection prefers legacy shelters over non-Bedroom bedrolls. Complete an exclusively steward-built bed enclosure through the ordinary SetRoomRole command within the existing House family, with optional serde-default completion_role in rule data (configured Bedroom). Never branch on the content name or introduce a seventh family. Require accepted owned bed and boundary provenance and all finished beds owned; existing foreign overrides prevent completion. Complete even when physical bed demand is already met, retaining scope, cooldown and replay.

Player precedence is deliberately conservative: any valid raw player SetRoomRole (accepted or idempotent Nothing, including Auto while OFF) permanently suppresses this automatic role-completion lane via the existing saved command log. Unknown-room rejection does not suppress it. This avoids falsely claiming per-room ID history survives split/merge without geometry history; it may decline an unrelated later automatic bedroom, an accepted narrow limitation. No new save shape is needed.

Expected ON effects: roofed beds gain Bedroom assignment preference, Bedroom sleep/mood rules and shared-bedroom illness transmission apply; the former Workshop900-permille crafting-time benefit is lost. Physical roofs, Flee/wolves, resources, drafting, stock geometry, reserve days and thresholds remain unchanged. These are material behavior changes, not a cosmetic label. OFF identity and all9 literal fixture summaries must remain identical. Before round5, test ordinary role submission, actual bed assignment, data-only alternate role, manual/Auto precedence across save/load and topology changes, invalid-room rejection, cooldown and replay. Freeze source/binary after passing tests. Only two calibration rounds remain; held-out seeds stay unused until the policy freezes.

## D28 — Observe panel opening as an event

Before edit: final A7 audit identifies a short open-close interaction entirely between the HUD's0.5-second fact polls. The user actually opened P/T/N/R or the inspector, but learning progress misses it. Subscribe to existing panel visibility/open events and latch when visible; retain expensive simulation fact polling at2Hz. Add a regression for open then close between fact polls, preserve user-level progress and steward attribution, and avoid duplicate signal connections on rebinding. UI-only change; no simulation/save change.

Frozen round4 OFF safety capture and comparison complete: baseline identity228748 shared values with0 differences,64 detailed files equal except git_sha, and all9 colony summary files literal byte-identical. External evidence: candidate-a4-compare.log. This covers D24/D25/D26 and excludes subsequent D27/D28; final source recapture remains required.

Stage gate isolation: use PLAY_ROOT within this owned worktree at .godot/mux-play-check and external PLAY_DATA=/Users/rexxa/work/mux-data/stage-a-play-check, with CARGO_TARGET_DIR unset. The unchanged six launcher checks genuinely exercise released main, M1 and V7 in registered nested detached checkouts, never pass4/play2-dev. Remove only these inspected nested worktrees before removing the final owned parent. Full gate baseline uses this pass's frozen baseline/identity (with its own perf capture), not an unverified older pass dataset.

D27 final validation: strict ALL built boundary/door ownership correction fails the old code and passes final unit10, integration9 (real-v6 fixture explicitly ignored pending separate invocation), workspace all-target clippy. Controlled construction reaches roofed stock16/beds10 at tick3587 and all10 assigned roofed Bedroom beds at tick5027. Full-save role replay, manual roles/Auto across OFF/save/load, scope and cooldown pass. Independent source re-audit resolves the mixed-boundary finding. Source hashes/handoff are external steward-d27-* artifacts.

Build7/release6 completed; frozen round5 source and binaries include D27 and are preregistered before seeds0–7×48days ON launch. This is round5 of6; held-out remains unused. Candidate-a5 OFF capture is running from the same frozen probe.

D28 runtime: ui-run9 finishes190/190 checks, including23 between-poll visibility regressions; no script/parse errors. All6 current HUD sheet frames are from that actual run. Actual completed house remains tick2880. The12-frame renderer sheet preserves comparison6 and labels its4 renderer-only fixture images explicitly.

Active-window debug measurement1 completes both1441-tick arms with valid provenance and no harness errors. ON advance mean3762.853us/median1660us/p992904us/max1048501us; OFF mean1830.508us/median1782us/p992739us/max11158us. ON tick0/360/720/1080/1440 costs1048501/515069/890300/458772/17526us. Frame post-draw means11.035/8.683ms and maxima1506.759/460.422ms; OFF tick0 includes substantial non-advance startup/render overhead. This identifies repeated ON planner spikes, not exclusive-system cost or normal-speed FPS acceptance. Keep original measurements; optimize only with exact command/result equivalence and the complete OFF safety net, in a separate behavior-preserving commit after the feature commit.

## Calibration round5 — frozen result

FAIL: stockpile8/8, beds8/8, zero cold deaths8/8, fuel3/8(required6), alive4/8(required8), ledger8/8. Bedroom role and observed bedroom sleeping now work, but this does not establish survival. Preserve all prior rounds and this failure. Held-out1000–1031 is still unused. No sixth round is authorized merely to guess a policy; first require a mechanism supported by existing evidence within ordinary commands and unchanged OFF behavior.

## D29 — Written behavior-preserving planner optimization plan

The active-frame evidence establishes repeated360-tick planner stalls, not just a startup frame. After committing the behavior-changing feature separately, first change only House candidate sorting from sort_by_key to sort_by_cached_key with the exact existing tuple. Both sorts are stable; x/y/width and area identify each nonzero-size candidate including height. The pure key reads no RNG or mutable state. Caching keys once per candidate removes repeated stock scans without caching worlds or commands across ticks/rules. Do not change candidate enumeration, tie order, scope, cooldown, validity, or policy.

Verification before its own commit: targeted existing planning/provenance/replay tests; exact ON summary/command/save equivalence against frozen round5 (a mechanical equality replay, not a sixth policy calibration); baseline identity and9 literal OFF fixture summaries; rebuild actual debug bridge and repeat active-window timing with the same1441-tick ON/OFF harness and recorded background load. Preserve old measurements even if improved. If residual cost remains, diagnose it and write a separate bounded follow-up before editing; do not blindly add persistent planner caches. Full stage gate still required on the final clean HEAD.

## D30 — Freeze policy after five rounds; retain unmet survival/fuel bars

Round5 diagnosis proves Bedroom use (max bedroom_sleepers10 in every seed,11 in seed5), but64 beast deaths remain versus69 in round4; survivors24 versus17, living colonies still4/8. All19 Sleep→flight entries begin unroofed3–5 tiles from reachable roof. Logs do not contain assigned bed/seek target/path at entry, so this is not proof of faulty bedroom assignment or doorway navigation. Seed5 has actual192 fuel-zero ticks despite14 survivors; other fuel failures follow extinction. Illness/work loss and fewer beast events confound aggregate improvement. No supported ordinary-command change currently addresses all survival failures without violating OFF identity or the no-auto-draft contract.

Freeze D27 policy at the round5 source/binary manifest rather than spend round6 on an unsupported guess. Five of six maximum calibration rounds used; no further policy tuning after held-out inspection. Execute the prescribed held-out1000–1031 ON48days once and paired original-baseline/candidate OFF no-order runs. Keep every unchanged threshold and mark any failure explicitly; this decision is not a waiver or an A8 acceptance claim. Stage technical gate and evaluation bars are reported separately. Behavior-preserving D29 performance work may proceed only with exact ON/OFF equivalence; it must not exploit held-out results.

## D31 — Isolate fixture preferences and make historical OFF state explicit

Before edit:24 legacy/current main-scene loaders exist.18 have no Store injection;6 inject before entering the tree but still initially read the owner cfg during instantiation. Renderer agreement resets a new game while the probe defaults OFF, so the new production steward default can invalidate the comparison. Historical art fixtures have the same unspoken default dependency.

Add a default-path-only WORLDSIM_TEST_SETTINGS_DIR opt-in to SettingsStore, with per-process files; explicit constructor paths take precedence and ordinary game defaults remain unchanged. Gate creates and exports an external settings directory. Standalone test commands also set it explicitly. Regressions cover explicit-path precedence, isolated persistence and no owner cfg write. Do not force steward OFF globally: smoke/new MUX UI keep production ON. Renderer agreement plus historical ground/people/trees fixture resets explicitly submit and verify ordinary SetSteward OFF before advancing (including trees' second colony reset). Do not alter goldens or acceptance comparisons to hide unintended automatic orders. Capture the original mismatched-condition negative control using isolated preferences before correcting the fixture, then rerun the actual agreement and affected art tests. This changes fixture setup and opt-in test persistence only, not normal game or simulation policy.

D31 negative control reproduced a real condition mismatch with isolated preferences: renderer agreement rejects designate_chop_tiles at tick360 while the OFF probe log expects acceptance. After explicit ordinary OFF setup, agreement passes10people/10appearances at tick900; historical ground/people/trees headless fixtures all finish with0failures, and actual GPU ground run4 also finishes0failures without changing goldens. Learning/settings115checks pass, including15 environment/path/owner-file-preservation regressions. Normal game default remains ON.

D29 semantic equality now passes: complete8-seed×48-day ON summaries (also exactly matching the original round5 summary), all8 concrete command logs, and the full69120-tick seed5 save are literal byte-identical before/after. Full OFF candidate-opt1 capture has completed; comparison and quiet active-window timing remain required before this refactor commit. Frozen D27 candidate-a5 OFF comparison also passes baseline228748values/0differences,64details exceptgit_sha, all9literal summaries. Held-out OFF original/candidate32-seed summaries are literal byte-identical (`heldout/off-identity.txt`).

D29 OFF comparison is now PASS (`candidate-opt1-compare.log`): all9 colony summaries literal-identical,228748 shared baseline values with0differences and64 detailed files equal except git_sha. The actual-window repeat is queued after the complete sim-test process exits; no concurrent test load is accepted as quiet timing. Both repository-root stable and rust-directory pinned toolchains identify as rustc/cargo1.93.1, so the earlier manifest-path invocation has no observed version mismatch. The new normal-accumulator gameplay harness passes Godot check-only; runtime remains pending and parser success is not FPS evidence.

## D32 — Make the winter preparation goal use the whole winter requirement

Before edit: independent Stage A audit finds `learning_facts.gd` compares settlement wood with `winter_wood_need`, whose bridge contract is daily winter consumption. The alert correctly multiplies daily consumption by winter_days and uses usable story.firewood. A6 can therefore claim winter readiness after only one day's fuel. Use the same authoritative usable supply and complete winter need as the alert; preserve zero/unknown requirement semantics explicitly. Add a negative regression for one daily ration before a multi-day winter and a positive exact-total boundary. UI facts/goal interpretation only: no simulation policy, resource amounts, or held-out tuning. Also audit per-command toast delivery separately before choosing a correction; no compressed last-label count may be presented as one toast per action.

D32 regression evidence: running the new126-check suite against the preserved HEAD learning facts fails4 checks (exit1), including the actual from_snapshots false-positive and exact full-winter threshold. The corrected production facts pass126/126 (exit0), with no script/parse errors. External `d32-negative.log` and `a-learning-test-8.log` retain both outcomes. Full stage gate remains pending.

## D33 — Deliver every new Steward notification and refresh live control translations

Before edit: Stage A audit AC-02 proves the HUD polls only last_label, discarding intermediate accepted actions when multiple commands occur between polls. Each action already has an authoritative kind45 Chronicle entry with monotonic sequence. Use the existing incremental chronicle API with an independent cursor, queue and per-action display identity; never consume the Chronicle panel cursor or overwrite all notifications in one frame. Bound per-poll reads, not total pending action retention. New-game/load baselines must prevent replay of historical or prior-world notifications, and direct player-order feedback remains immediate. UI-only observer change, no new bridge API or simulation policy.

AC-03 also finds map/minimap labels and P/T/N tooltips stale after the existing live locale event. Refresh from the shared catalogue on that event. Add real instantiated-scene regressions for multi-action notification ordering/duplicates/reset and locale refresh, plus strengthen existing alert-jump tests to verify actual available destinations. No new animal inspector requirement or physical-input acceptance is inferred. Preserve all A8 held-out failures.

D32 scope correction before follow-up edit: bridge story.firewood is global usable supply; per-settlement winter_wood_need is per-hearth daily consumption. The first repair and existing A5 both compare that global supply separately with each local requirement, incorrectly using a maximum if multiple settlements exist. Although default colony rules disable fission, no bridge invariant guarantees one settlement. Sum positive daily needs before multiplying by the winter duration for goal readiness; A5 adapter supplies that same aggregate daily requirement alongside global supply for each settlement cause. Preserve existing alert conditions and camera targets. Add two-settlement one-unit-short/exact-total regressions; this is UI accounting only, no simulation or policy change. Retain the first126-pass result as insufficient coverage, not final closure.

D32 final targeted validation: learning128/128 and alerts254/254 pass without script/parse errors. The old adapter with the strengthened254-check suite fails3 checks (exit1), including insufficient aggregate supply and both settlement jump causes; `d32-alert-negative.log` preserves it. The earlier single-settlement negative remains4failures. `a-learning-test-9.log` and `a-alerts-test-3.log` are the final scoped receipts; final integrated window/gate remain pending.

D33 destination audit confirms a real tile/animal route defect: CausalPanel.display_history updates content but does not show its panel, including the empty-history path. The HUD must explicitly show that existing destination before displaying causal history. Regressions assert actual panel visibility and requested tile rather than only emitted alert-target kind. No new selection type or inspector is introduced.

D33 runtime UI10 passes249 checks with real waiting migrant/trader/room/animal targets and actual reset/load coverage. Separate visual review is88/100 REVISE: HUD/help/lesson captures show only the Steward toast prefix. This remains a failed visual round. Root's isolated mutation controls disable queue delivery, live locale refresh and causal-panel showing: focused193-check run fails18 (no script/parse errors), demonstrating the new assertions reject those regressions. Original source is untouched by these controls. Fix action-label observation from the current passed state and add explicit nonempty localized action coverage before another actual-window capture; a functional249 PASS does not erase the88 visual failure.

D33 UI11 passes253 checks but fresh eight-image review is89/100 REVISE. The empty action is fixed and CausalPanel is visible; English evidence exposes legacy OrderBar labels/hints not refreshing (its locale callback refreshes only tooltips) and a clipped newborn alert title hiding its meaning/count. Extend the same locale observer fix to existing order-bar/build/gather labels and current hint, without changing commands. Shorten the English newborn alert title while retaining the full household explanation in its tooltip. Recheck rendered fit and live-language roundtrip; preserve UI10/11 failed visual scores.

D33 label-only mutation control restores the stale global-label dependency in an isolated HUD copy. The focused real-window first-action run fails exactly the new action-content assertion (6checks/1failure, exit1), without script/parse errors (`d33-label-negative.log`). This separates text correctness from sequence/queue cardinality. Production files are unchanged by the negative control.

D33 UI12 preserves a real rendered-fit failure after the first shortening: `! No bed for newborn · 1` needs316px in a308px button, so the count is clipped. Parent inspected the actual English capture; order-bar translation is corrected, but this frame remains89 REVISE. Keep the pixel-fit assertion and available width unchanged; use the shorter equivalent title `Newborn needs bed`, retaining the full household tooltip, then rerun. No threshold/layout waiver.

D33 UI13 still fails the same unchanged fit assertion (268checks/1failure):312px versus308px. The parent inspected the actual clipped count, retained89 REVISE, then measured candidate titles with the production24px font before editing (`d33-title-measure.log`). `Newborn: no bed` measures274px including icon/separator/count, so it replaces the title with34px margin. Full household tooltip, layout and test remain unchanged; actual-window confirmation still required.

D33 UI14 final functional run passes268/268, no script/parse errors. The new title including icon/count measures274px inside308px; full household tooltip remains intact. UI12/13 failures remain retained. Live replacement receipts, per-action text/queue, actual destinations and English/Korean controls are covered. Final eight-image review and scale rerun are separate pending evidence at this entry.

D29 measurement instrumentation now also saves each completed ON/OFF debug world after all1441 timed samples, recording unchanged pre/post-save tick and SHA256. The timed loop is byte-identical to the prior harness; save cost is excluded. These saves allow a later compiler-profile experiment to compare actual debug world bytes rather than relying solely on release probes. No profile change has been selected or applied; active-perf-2 still waits for the full sim-test process to finish.

D33 final independent source audit finds no actionable defect (`d33-code-review.md`). UI14 eight-image visual review92/100 PASS; scale5 nine-image review91/100 PASS with220/220 functional checks. The current HUD contact sheet contains all8 UI14 captures. Retain effective legacy scaling caps, one whole visible alert row at1.5 and hidden-panel limits; no physical-input, owner-acceptance, survival or full-game performance claim. GDScript strict5 passes67files/57FFI calls. The current failed visual/instrument rounds remain preserved.

D29 committed as91f029e9e after full sim-test exit0 (1403passed,0failed,1ignored), exact ON/OFF safety and active-perf-2 exit0. Actual unoptimized debug ON boundaries0/360/720/1080/1440 still cost500592/238384/320699/182202/16250us. See D29_VERIFICATION.md and external d29-active-comparison.json; sequential timings do not establish a controlled speedup. Runtime/data source freeze673hashes and both post-measurement tick1441 save hashes verified. Normal automatic-cadence run1 is now running, no outcome claimed yet.

## D34 — Test a package-only development optimization without disabling checks

Before edit: D29 leaves repeated182–501ms planning calls in the actual debug bridge. The owner launcher uses this debug library, so release-probe speed does not resolve the observed play path. Select the smallest compiler-profile experiment: `[profile.dev.package.sim-systems] opt-level = 1` in the workspace manifest. Preserve debug assertions, integer overflow checks, debug information, ordinary bridge path, all source algorithms/data and the frozen D27 policy. Do not copy a release dylib into the debug path or change global optimization, LTO or dependencies. Installed Cargo1.93 documentation and caveats are recorded externally in dev-profile-options.md. Test profile inherits dev; final full debug tests must run under the resulting effective configuration too.

Expected effect: less debug planning work while all simulation state remains identical. This is a build-configuration change, separate from D29 and from behavioral policy commits. Freeze the current debug library, build log, source hashes and both1441-tick ON/OFF saves before the edit. Wait for normal-cadence run1 to finish before changing any runtime/build source. Capture verbose new rustc flags and verify optimization1 with debug assertions/overflow checks retained. Rebuild the ordinary bridge, repeat identical active ON/OFF measurements and compare the two full saves literal-byte-for-byte. Repeat the normal automatic1x/4x gameplay measurement with full source freeze and recorded host load. Report all raw timings, including startup and remaining stalls; do not reinterpret debug diagnostics as release C3 or an admitted quiet speedup comparison.

Before its own commit: targeted Steward command/replay/save tests; actual debug save equality; unchanged release ON/OFF candidate source and safety receipt verification (profile.dev does not alter release flags), plus fresh original-baseline identity and nine literal OFF fixture comparisons. Full final gate remains mandatory. If checks or save equality fail, retain the failed experiment and fix forward; no threshold relaxation, algorithm/policy change or held-out tuning. If timing remains poor, report that result before choosing any separately registered follow-up. A8 acceptance stays FAILED.

D34 verification completed: effective sim-systems rustc flags opt-level1/debuginfo2/debug-assertions=on; the installed compiler's overflow-check default follows debug assertions and stays enabled. Independent audit confirms the linked ordinary debug library hash. Unit10, integration9 and separately invoked real-v6 migration1 pass. Actual ON/OFF tick1441 full saves are literal-identical to the prior debug build. Release build reports Fresh, and worldsim_probe remains exactly6c1f36ca40524ac2ca9ca564d43bb82700441beae73dff2fec8ff2a771ca95a0, binding prior8-seed ON equality to the unchanged executable. Fresh profile1/safety comparison passes228748baseline values with0differences,64details exceptgit_sha and9literal colony summaries; all fixture invariants and baseline determinism pass.

D34 automatic gameplay2 completes all4phases: ON1x11.949TPS/118.216meanFPS/max649.902ms, ON4x47.173TPS/114.380meanFPS/max646.191ms. Startup is retained; recurring ON planning frames remain73–99ms. Source673hashes match the preregistered runtime freeze. ISS-030 remains open: no unchanged-FPS, calibrated speedup, full-capacity or latency-resolution claim. The existing full1403-test run predates this profile; the final full gate must cover it. External profile1/{profile-audit.md,debug-save-comparison.json,gameplay-comparison.json,safety-compare.log} binds these conclusions. The minimap's separate UI14 conservative545us upper bound remains an evidence gap pending the registered final dedicated check; this compiler experiment does not resolve that UI budget by inference.

## D35 — Repair stale test expectations and a missing inventory row without changing runtime behavior

The first complete35-step gate on clean6b0f572e4 ended2026-10-25 08:03:24 KST with status1. Both workspace test profiles report2059passed/1failed/1ignored: the same `sim-core::chronicle::tests::discriminants_are_stable` still expects45 kinds/names after M-UX appended StewardAction at45. The inventory closure separately reports only DESIGN.md uncovered among2221tracked paths. All other32steps exit0, including the new deterministic2-world×24-day Steward fixture. Preserve the failed run at external `gate-stage-a-1/`; the earlier1403 sim-test-only run never covered the core unit target.

Keep every historical discriminant assertion and the enumeration/unique-name checks. Update both totals to46 and explicitly assert StewardAction code45 and name `steward_action`; change no production code, save representation or policy. Add precisely one DESIGN.md KEEP rule and the identical generated row, retaining historical inventory rows. Reject wholesale inventory regeneration: its V8-0 fallback would misclassify live later UI files and rewrite unrelated historical decisions. Verify the new row exactly agrees with its rule and closure has zero uncovered paths.

This is a test/documentation-tool repair, not a gameplay refactor or another Steward calibration round. Targeted debug/release checks and a new clean committed full gate are required before publication. First-run performance and all passing steps remain evidence for that exact earlier HEAD; they do not transfer a green gate to this change. ISS031/032 track the omissions; A8 FAILED and ISS030 remain unchanged.

D35 targeted verification: the formerly failing exact Chronicle test passes once in debug and once in release (both0failures); fmt passes. Inventory closure now reports2221tracked paths/1944rows/0uncovered, and an independent rule-to-row equality assertion passes. These scoped results do not replace the required fresh full gate.
