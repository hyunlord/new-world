# Revitalize decisions

Run started 2026-10-10 12:07 KST (03:07 UTC); nine-hour boundary 21:07 KST.
Authoritative specification: owner attachment, copied outside the clone to
`/home/hyunlord/work/revitalize-goal.txt`.

- Dedicated clone `/home/hyunlord/work/new-world-revitalize`, branch
  `codex/revitalize-2026-10`, base `d409e9b941ccc868747e4eb63953eafa6831bff7`.
- The owner explicitly overrides stale preflight/reset, main-branch push,
  interactive harness, and planning approval instructions. Never install git
  hooks or write harness audit/baseline/reviews or OMX/OMO state.
- Existing macOS checkout is not a delivery workspace. All implementation,
  builds, probes, and captures run on DGX.
- Host has no Rust or Xvfb in its default environment. Install the requested
  toolchain and Godot using official distributions; retain PATH explicitly.

## Execution and evidence ledger

| Part | Deliverable | State | Evidence |
|---|---|---|---|
| 0 | arm64 extension, baseline gates | complete | 1729 debug + release tests; clippy and Godot gates pass |
| 1 | tallies, shared scenes, probe, captures, frozen legacy pin | complete (capture exception) | 1748 tests, both Clippy checks, strict14 and smoke900 PASS; four legacy repeats deterministic |
| 2 | default-legacy rules, world generation, terrain/building FFI | complete | 1764 tests; both Clippy checks; strict14/smoke900; live Godot 48 checks |
| 3 | gated survival/growth loop, 64-seed tuning | implemented; survival gate blocked | four 64-seed twin batches PASS; 3-day zero-dehydration test fails |
| 4 | deterministic art, truthful renderers/HUD, visual QA | implemented locally; visual exception, global gate blocked | UI322 PASS/GPU96 unobservable; strict17; smoke900; ledger230 |
| 5 | full verification, Korean report, handoff | verified partial delivery; no push/PR | debug+release1781 PASS/1 FAIL each; fresh68 twins PASS; gh absent |

P0–P2 passed the owner gates before push; remote landing was checked against local HEAD. P3–P5 remain local because the retained survival gate fails. No legacy behavioral assertion may be weakened.

## Environment exception: captures

`xvfb-run` is absent and `sudo -n true` reports a password requirement. Per
Part 0 item 5, all screenshot capture steps are skipped on this host; no
password request and no substituted screenshots. Capture tooling will still
be delivered. Headless smoke and strict checks remain mandatory. Visual
acceptance cannot be claimed from static checks.

## Native legacy precision and measurement

V7 `Hunger.value` is f32, whereas thirst, sleep, social needs and health are
f64. Preserve the existing component and its exact legacy trajectory. The
fingerprint hashes native `Hunger.value.to_bits()` (4 little-endian bytes),
and native 8-byte bits for the f64 values. Metric aggregation widens hunger
to f64 and sorts agents before summation. No FFI f32 snapshot downcasts are
used for fingerprints.

The remote clone has no Graft CLI/index; the local macOS graph describes an
older checkout. Use bounded source reads on the remote clone rather than
using that stale graph as evidence.

## Part 1 boundary

Completed 2026-10-10 12:28:46 KST; Part 1 began after Part 0 push at ~12:18.
Full debug tests: 1748 passed, zero failed (baseline1729 +19 new tests),
110 result groups. Both Clippy checks: zero warnings. Strict GDScript14 and
smoke900: PASS; capture/camera standalone check-only: PASS. Forbidden math
scan and added-production unwrap scan: zero hits. Independent read-only
review found no blockers. Four 10-day legacy repeats matched every sampled
fingerprint. Raw evidence remains ignored; summaries are committed.
# Sparse berry edge repair

The exact recipe fails for seeds 247, 338, 345, 383, 426, 728, 759 and 773 in the deterministic 1,156-seed geometry audit (0..=1024, u64::MAX, 2^63, 2^63-1 and 128 high hashed seeds). Camp selection itself succeeded for every audited seed. The failures are fewer than 12 Forest/Plain berry candidates or fewer than three centres separated by six tiles.

When and only when the original candidates fail, exchange a local Plain edge tile in camp radius 6..=16 with a distant Forest tile outside radius16. Rank both deterministically by hash salt9 then y,x; while centres are missing require each new edge to be at least six tiles from the current greedy centres. Never exchange building or water tiles. Re-run the original hash-salt8 cluster recipe after every swap. This preserves total Plain/Forest/Hill/Mountain/River counts exactly, elevation bytes, river geometry, camp, buildings, resident positions and resources. It changes the strict moisture-quantile classification only for deficient seeds, an explicit necessary relaxation to satisfy the requested exact bush count.

All eight failing audited seeds recover with 1..7 swaps and retain exact global class counts. Original successful audited seeds perform zero swaps. Evidence: p2-extended-geometry-results.json and p2-repair-geometry-results.json. These are standalone Python algorithm reproductions, not compiled Rust evidence or a proof over all 2^64 seeds. Runtime harness includes all eight regression seeds and passed the Part 2 full gate.

## Generated camp formation

The required tick-2 all-24-members invariant conflicts with the legacy
lexicographically-first resident anchor: movement precedes formation and a
ring-three camp can span more than radius five from its westernmost resident.
For worlds with shelter planning enabled, prepend real registered Hearth
tiles (sorted y,x) to the formation candidates. Preserve all thresholds,
private birth bookkeeping, causal events, and the exact default candidate
path. This narrow Part 2 rule read is necessary before Part 3 behavior wiring;
no settlement state is fabricated by worldgen or GDScript. A focused test
checks both legacy and generated anchors; worldgen invariants verify 24
members by tick two for normal and repaired regression seeds.

`reset_world` emits its notification deferred: synchronous listeners would
reenter snapshot getters while Rust still holds the mutable node borrow.
Reset preserves simulation speed and rebuilds only this node's engine.
Building snapshots contain registered buildings only; queued camp buildings
appear after the first simulation tick, with no invented render-only ids.

## Part 2 boundary

Completed 2026-10-10 12:43:22 KST; began after Part 1 push at ~12:29.
1764 debug tests passed, zero failed (Part1 +16); 111 result groups.
Both Clippy invocations, forbidden math, production unwrap scan, strict14,
and smoke900 pass. Actual Godot main-scene FFI/reset test passed 48 checks
and three deferred reset callbacks, including reentrant snapshot reads.
The 16-seed untuned 10-day probe is a diagnostic only: rules are defined but
Part 3 behavior is not wired yet; median dehydration deaths272 and alive103.5
are not survival acceptance. Renderer ledger records eight replaced static
test functions and three wording-only provenance corrections; no behavioral
predicate, frozen pin, or test retirement changed. Captures remain skipped
under the documented environment exception.

## Part 3 scope conflict and tuning stop

Initial seed1 trace identifies all 11 dehydration deaths as normal social Seeking Agent.
No construction or GatherFood state occurs among thirst>=80 samples. A resident
stands on infinite shore water at thirst100 while following a nonreciprocating
peer; normal social seeking has no urgent-need preemption. Legacy pin passes.
A real escape transition would change social behavior forbidden by sections8.3
and12, so do not add it. Three allowed knob rounds preserve determinism but do
not solve dehydration. Keep round2 under the stated priority order: it passes
non-extinction and has fewer deaths than the other non-extinct configuration.
The new survival assertion remains unchanged and failing; do not re-pin, ignore,
or delete it to make the gate green. Part3 cannot be pushed under the every-push
green rule. Continue Part4 locally and retain all evidence, then report this
explicit conflict and the last green remote boundary. Night-rhythm stretch
is skipped because T1 and T6 do not pass.

## Part 4 local boundary

Truthful terrain, registered buildings, depleted berry stages, simulation-time
HUD, localized inspector/feed, selected-only paths, daylight and agent
interpolation/shadows are implemented. Static renderer source pins were
replaced under the explicit ledger policy: 230 baseline assertion occurrences
map to 113 replacement function contracts; no tests were retired.
Actual headless main-scene QA passed 322 checks. An explicit nonzero sentinel
proved MultiMesh GPU transform readback unavailable in the dummy renderer:
96 coordinate checks are NOT_OBSERVABLE, never counted as passing. Reset
CPU position caches and interpolation origins pass. Strict check passed
17 files and 14 FFI calls; smoke900 passed. The fresh final full debug/release
runs are recorded separately in Part 5. All-target Clippy is clean after
removing orphan comments for deleted static-test helpers.
No Xvfb and no passwordless sudo: exercise the Part 0 item 5 capture exception.
No screenshot, contact sheet, visual-quality acceptance, or ten-minute 4x
render performance claim is made. Part 4 stays LOCAL because Part 3 retains
the genuine survival failure and every pushed head must be green.

## Final partial delivery

Both final native suites run with --no-fail-fast:1781 passed,1 failed,0 ignored,113 result groups each. All expected1782 tests accounted for. Only required three-day survival fails seed1:dehydration11 versus0. Seeds2/3 inside that failed function were not reached. Both Clippy checks, strict17/14, smoke900, diff check pass; forbidden math0, changed-production unwrap calls0. Fresh68 twin runs and full legacy/selected-round sample comparison pass.

No gh executable: draft PR skipped. Last green remote remains c438026. To preserve unpushed implementation without violating green-push policy, export local P3–P5 commits as a verified incremental git bundle plus complete evidence and report in an external handoff archive. Download only that delivery copy to macOS; all implementation remains on DGX. This is the explicit partial-delivery end state, not a claim that all goals were achieved.

## Pass 2

Start:2026-10-10 16:49:46 KST. Eight-hour boundary:2026-10-11 00:49:46 KST.
Authoritative goal:/home/hyunlord/work/revitalize-pass2-goal.txt.
PASS2_BASE_SHA=c70b0c67e7103f42b300ccc1ed892e8143803c89.
Pass1 P3–P5 are now secured on origin/codex/revitalize-2026-10-wip at that SHA.
Green branch remains c43802600db209b0b32b20fbaecb5f7f8ddf8cf5 until a full green gate.
All commits immediately push WIP, including failures; green only fast-forwards.
Persistent handoffs copied to /Users/rexxa/work/revitalize-handoff-2026-10-10/
and /home/hyunlord/work/revitalize-handoff-2026-10-10/; SHA256 verified.

Initial game_v8 values:move_interval_ticks8,idle_pause_chance0.4,
idle_tether_radius8; Provisioned cooldown360,food_cost3,max_mean_hunger60,
max_mean_thirst60; housing residents4,max_shelters12,plan_interval60,
rings2..5,required_progress240,max_builders4. Worldgen bushes8/capacity4.

Execution plan and gates:
- Part0 complete: remote safety line and persistent copies.
- Part1 in progress: every-tick death census, display-only social tally,
  schema2 additive metrics; measure unmodified P5 behavior before any fixes.
- Part2 queued: exact rule-gated social bounds and need fallthrough, clock
  offset480, new staged regressions; unchanged survival and legacy pins.
  Generic critical preemption only if post-fix16-seed census meets5% trigger.
- Part3 queued:64-seed10day and16-seed30day twins; at most3 permitted tuning
  rounds; retain best by ordered targets, stretch only if every target passes.
- Part4 queued: Mac Aqua tierA detected; fresh dedicated captureclone only.
  Clock-based final/baseline captures, entity sidecars, V1–V6, displayed418 UI
  harness and direct image inspection. If tierfails15min, follow B/C/D.
- Part5 queued: separate mutation worktree, exhaustive token mutants for
  canonical ledger contracts, per-mutant restore, strengthen survivors and
  explicit reasons for any survivor. Never replace source-text tests wholesale.
- Part6 queued:final full gate/probes, report, WIP push, green fast-forward
  only with gate evidence; draft PR via authenticated Mac gh then DGX.

Independent agents prepare code in owned paths or ignored staged copies;
leader integrates, runs heavyweight checks serially, commits and verifies
remote heads. Max6 active children. No unavailable Graft graph is used as
source evidence: DGX has no graft/rg, so bounded grep/source reads are used.

### Pass 2 Part 1 schema compatibility

Adding the requested public Tallies.social_completed field makes the frozen
legacy test full Rust struct literal fail to compile. Preserve every original
expected value and the fingerprint, tick, population and replay assertions;
mechanically represent the original four-counter comparison as a tuple.
This is a source compatibility adapter, not a re-pin: the test file is not
byte-unchanged, and no equality implementation ignores the new counter.
Other two full literals explicitly set the added field to0; the new social
unit test independently verifies exact per-pair counts and unchanged outputs.
The social counter test passed; full legacy trajectory verification follows.

### Part1 completed

20 probe unit tests, social counter regression, legacy/tallies tests and all-target Clippy PASS.64 twins PASS;7,744 original behavioral rows identical. Partner-alive is measured before lethal phase; simultaneous deaths can report alive. Part2 review completed two rounds with objections resolved. Part1 metrics prose follows in a supplemental commit because report generation raised a formatting exception; the code and complete JSON/Markdown summaries were already committed and pushed safely. Boundary UTC:2026-10-10T08:05:01.159403+00:00.

### Part2 post-fix census and bounded critical preemption

16 seeds x10days, deterministic twins PASS: dehydration0, starvation536. Death states Idle260, SeekingFood166, SeekingWater82, SeekingAgent19, SeekingSleep9. Food unavailable at342/536 deaths; all166 SeekingFood deaths had positive food supply somewhere, and27/82 SeekingWater deaths did. Food and Water seeking exceed the literal5% trigger. Add critical_preemption legacyNone/game75 only for these observed states, and only when the highest actionable need targets a different activity. Resetting a seek for its own need would create a new starvation livelock; this anti-livelock restriction is an explicit interpretation of section7.4. No Gather/Construction interruption is justified by this census.

New7 regression tests and legacy pin PASS;21 probe unit tests PASS. Unchanged3-day survival test now passes dehydration and population checks for all3 seeds but fails seed3 raw idle share18130/165503 (10.95% versus10%). Do not weaken it: Part3 allowed birth/resource-demand tuning follows.

Part2 critical competing-need preemption implemented;10 new regression tests and all-target Clippy PASS. Full green promotion withheld because unchanged3-day raw-idle regression remains red; full gate will run after bounded Part3 tuning. Part2 boundary UTC:2026-10-10T08:09:22.257385+00:00. No existing behavioral assertion was changed in this Part.

### Part3 tuning round1 and critical-preemption correction

Round1: birthcooldown1440,foodcost3,residents6; social240/480 andcritical75 unchanged.64 ten-day and16 thirty-day twins PASS; unchanged3day survival now PASS. Ten-day dehydrationmedian0/max3,starvationmedian0/p900,max8; alive median49/min37. Housing maxoverflow29 (30day33) still fails. Full64 pre-tuning cohort exposed critical-preemption regression: samefirst16 seeds went from0 dehydration before critical rule to63 after it. Code can interrupt water even when thirst100. A bounded fix protects a current need that is already critical; same-target protection alone was insufficient. No claim of observed oscillation: census records states, not every transition. Remaining hungry Food seekers may still ignore critical thirst under unchanged Hunger-first policy; broad priority redesign remains outside scope.

Critical-preemption correction passed11regressions and finalreviewround2. CorrectedR1 sameknobs rerun:64/16 twins PASS and dehydration0 in every10day/30day seed. R2 changes only birthcost6/residents8 (cooldown1440); bothtwins and unchanged3day survival PASS. R3 is final allowed tuning round:cost10/residents8/cooldown1440; otherparameters remain240/480/75. Three tuning rounds cap is respected; re-running R1 after implementation bug correction is separately identified, not a fourth parameter search. Finalchoice follows declared target priority. Night-rest stretch is skipped unless alltargets pass. Time UTC:2026-10-10T08:15:32.996026+00:00.

### Part3 retained configuration

Retain round3: birthcooldown1440,foodcost10,residents8,socialchase240/retry480,critical75. All three corrected configurations meet higher-priority T8/T1/T3/T2/T4 at10days. Round3 has lowest maximum same-tick housing overflow (30day25 vs26/33;10day15 vs19/29), lower starvation tail and30daymedian6.5 vs18/39. T10housing remains FAIL and lower-priorityT5new sheltersmedian1 FAIL; do not hide either, change worldgen, or add a fourth round. No night-rest stretch. Legacy/default rules remain unchanged; unchanged3day regression PASS forrounds1/2/3. All64/16 twins PASS perround. Part3 fullgate next; boundary UTC:2026-10-10T08:16:06.986323+00:00.

Part3 fullgate: debug/release1805pass1fail, sole existing named-target-arms staticaudit rejected the new wildcard fallback. Clippy both, forbiddenmath, strict, bridgebuild, import, smoke900 allPASS. Replace own fallback with explicit exhaustiveTargetKindarms and availabilityfilter, preserving behavior; no test re-pin. Targeted37socialcomponent+11survival testsPASS. Green staysc438026 until next full gate; no claim ofgreen basedonlytargetedchecks.

### Part4 actual displayed evidence, round0

TierA Mac84aeef80:10 complete1920x1080PNG frames and sidecars,45/46visual checksPASS. V2FAIL11/23agents (47.8261% vs90%); V3night/day0.504151 andHearth/night1.490472PASS; V1/V4/V5/V6PASS. DisplayedUI418/418PASS, zero unobservable; actualGPUcoordinate probe available. Forty artifact hashes verifiedafterDGXtransfer. Leader directlyopenedF2/F2N/F4: riverbankvillage recognizable, selectedvillagerWater target readable, darkneutralvillagersblendwithground and coincidentstockpilelabels overlap. Round1 will fix realrendercontrast and truthful labelcollision; no threshold/metadata manipulation orsprite replacement.

Fresh committed-head84aeef80 probes:64game and4legacy deterministic twinsPASS. Comparealloldsamplefields excludingonlygit_sha/schema_version:7,744game rows identicalretainedround3;484legacy rows identicalPass1P5. Current sourceclock/rules/schemaadapter preserveslegacybehavior. TimeUTC:2026-10-10T08:28:37.224174+00:00.

Visualround1: preserve texture/silhouette/alpha/darkoutline, lift opaque palette output towardwhite0.65 before unchangedcanvaslighting. Statehue remains represented; no sprite replacement. Coincidentnearest-stockpile labels now sum realsettlementstocks byanchor and showone localizedlabel. Runtime fixture conserves26food acrossanchors15/11; no-stockpile caseempty.88 unchanged staticRusttests and GDScriptcheckPASS; no ledgerrepin. Realcapture determinesacceptance, notofflinefixture.

### Part4 second and final visual correction

Round1 real displayed capture passed46/46 visual checks and418/418 UI checks, with23/23 visible agents. Direct image inspection found colored activity markers lacked a key. Add a compact row inside the existing HUD: redFood, blueWater, yellowRest, grayOther target, whiteBusy. These describe target markers; white includes consuming/sleeping/construction, so use localized Busy rather than misleading Working. Three new keys are present in en/ko and compiled. Targeted64 unchanged static tests, strict17/FFI14, and actual headless en/ko legend colors/text/live locale switching PASS. This is the second and final allowed visual correction; real displayed recapture follows. No simulation or FFI changes, no static assertion re-pin.

### Part4 completed with actual final display evidence

Final visual round2 exact code005ca150:46/46 visual checks PASS;V2=23/23 agents;V3 night/day0.503267 andHearth/night1.473976; displayedUI418/418 PASS, zero unobservable.39 transferred artifact hashes verified. Leader directly opened fullF2/F2N/F4: village is the riverbank huts/hearth/stockpile; coloredtarget markers now have a readable legend, selectedvillager0 is On the way toWater. Noon HUD reports Berries0 andstoredFood63: harvestable food is depleted, but a single frame does not establish imminent famine or adequate future supply. NightFood79/Berries1, fire visible. Two visual correction rounds exhausted; no further cosmetic expansion. Commit14 JPEGs(960x540quality85), contact sheet, detailedQA and provenance; fullPNGs remain ignored. Part4 boundaryUTC:2026-10-10T08:42:00Z (approximate boundary; exact commit time recorded bygit). Green promotion awaits final whole gate. Macgh authentication hyunlord confirmed; draftPR follows finalverification.

Supplemental startup validation (no third visual fix): exact tick0 displayed Day1 08:00 and bright terrain, but no village exists before settlement formation. The driver waited0.4s+5frames+postdraw, so this is a preformation state rather than an undrawn frame. A separate natural startup after about1real second showed tick30/08:30, one24-member settlement, Hearth+3Shelters+Stockpile and visible residents; unmodified defaultcamera naturally centered(824,324),zoom4. Leader opened both fullPNGs. Checklist1 is partial at the exact initial instant; the clock and natural postformation centered village are directly observed. No artificial world initialization or camera repositioning was introduced. Preserve both supplementalJPEGs and metadata rather than claiming tick0 already contains a village.

### Part5 audit baseline correction

Isolated mutation execution found an existing full-file HUD snapshot still pinned to105lines after the localized legend extended it to159. Earlier64 targeted static tests did not include this guard. Preserve both strict line-count and SHA256 equalities, update only their reviewed expected values, and record both assertions in TEST_LEDGER. Do not count a failing unmutated baseline as a killed mutant. Audit worktree will advance to this committed correction and rerun against clean source.

### Part5 completed

All597 primary mutants rerun on clean committed68bb8fff:593 killed,4 survived,0invalid,0baseline-invalid; raw99.329983%, not100%. Two survivors preserve whitespace-equivalent accepted literals, two remove an optional connect call from an allowlist guard. Do not add arbitrary spelling or required-presence assertions to inflate a metric. Three separately reported semantic supplements all killed. Required reset-subscription presence remains outside the proven allowlist contract. StrictHUD snapshot repin is logged; no behavioral assertion weakened.117functions fully accounted,114source contracts/3behavioralN/A, unresolved0. Isolatedtree clean,13sourcehashes restored, no active journals; runner/offline checksPASS. MainCargo slot released for finalfullgate. No dependency manifest changes or addedHashMap/HashSet/unwrap/SystemTime patterns in three simulationproductioncrates sincePass1base.

### Part6 code verification complete

Full gate p6-full at0ec143c027a2570ac97f7b5f78356345f892f0cb PASS: debug/release each1806passed,0failed,0ignored(114resultgroups); bothClippy,bridge,forbiddenmath0,Godotimport,strict17/FFI14,smoke900,diffcheck allPASS. Finallegacy4 andgame64 ten-day deterministictwinsPASS. Legacy484oldfieldrows reproducePass1P5 exactly;game7744rows and11deathrecords exactly reproduce retainedR3. Finalgame median0.0008098773672545105ms/agenttick;10daydehydration0,totalstarvation11. Read-only observers remain outside measured engine time. Currentcapturedproduction source identity matches005ca150 across scripts/shaders/scenes/localization and4Rustproductioncrate source trees. Green fast-forwarded fromc438026 to0ec143c0 only after fullgate andprobes, remotels-remote verified. The final report/probe-summary publication is documentation-only; rerun the fullgate and freshrawprobes on its exact committedHEAD before the finalgreenpromotion/PR. No additional tuning orvisualfix rounds.

## Pass 3

- Start: 2026-10-10T20:06:32+09:00; 10-hour boundary: 2026-10-11T06:06:32+09:00.
- PASS3_BASE_SHA: `0941e1b76ee250a017295fe923f7c0841dafaaa7` (both remote branches verified).
- Part 0: clean DGX WIP; release build and game v8 64x10 deterministic reproduction passed before any code change. Every summary simulation metric equals p2-6_final_game after excluding timing metadata. Frozen tick-1440 fingerprints recorded in probe/p3-0_v8_repro/frozen_fingerprints.json.
- Execution plan: (1) additive v9 plumbing and schema3 instruments, (2) hearth-only formation and owned capacity, (3) sustainable supply and strict meals, (4) beds/night/evening/children plus additive FFI, (5) at most three 10/60-day tuning rounds, (6) Mac capture under shared window lock with no renderer edits, (7) exact-head full gate, final probes, Korean report and PR510 update. Each Part commits and pushes WIP; green only fast-forwards after full gate.
- No writes to art-owned paths, .omo/.omx, protected harness directories, or owner checkouts. No image generation. Shared-file Rust-only assertion/struct adapters require explicit ledger rows.
- Graft unavailable on DGX (command lookup empty); bounded named-file source lookup is used. No graph claims are used as source proof.
- Parallel native agents have disjoint write ownership; leader coordinates shared APIs and serial build/probe slots. Baseline completed before implementation authorization.
- Part 1 shared API: food_economy supplies deterministic source_supply_per_day, supply_per_day, need_per_day, members_need_per_day, adult_need_per_day and sustainable_adults. Global probe supply deduplicates the union of settlement catchments; overlapping v8 catchments do not multiply world food.
- Existing day checkpoints retain elapsed-day semantics. Clock observations use one-based display days and integer-minute conversion; coincident sample kinds are explicit, and old cumulative Idle ratios count periodic observations once. New owned housing overflow is separate from the legacy global housing metric and starts at first formation.
- New fields/types for future Parts are introduced inertly in Part 1; game_v9 initially reproduces game_v8. No v9 behavior is enabled before baseline instrumentation is validated and committed.
- Part 1 verification: core neutral rules2/2, worldgen2/2, probe25/25, ground counter2/2, settlement14/14, bridge tooling5/5 pass; workspace all-target Clippy passes after removing a duplicate unused test import. v8 schema3 64x10 reproduces all old summary values and 7744 sample/11 death rows; 32x60 clock-sampled twins pass. Baseline60day: total starvation656/dehydration2; day60 population median46, stock3, starvation13.5, settlements6; free meals median762. Raw schema additions are observational only.
- Part 1 elapsed boundary: 2026-10-10T20:14 KST (under10-hour limit). Full gate reserved for green promotion; WIP commit records targeted checks and long-run baseline, not a green declaration.
- Instrument review round1 found an early-maturation blind spot: Child removal was counted without checking its age. The observer now accepts a configured transition only at completed ticks due+1..due+60 (systems execute before the completed-tick increment), rejects an overdue still-Child at due+60, and separates death. Boundary/early/late/death fixtures plus staged metrics:3/3 PASS. This is instrument correction, not a behavior change; v8 has no children.
- Part2: v9 now uses Hearth anchors, Owned housing and headroom2. Formation claims nearby unowned buildings immediately; completed planned shelters inherit site ownership; dissolution removes only its entries. NearFormation/v8 retains old count order/formula through the shared helper. Both planner and birth gate call housing_capacity, with shelter_count shared for max-shelter limits.
- Part2 validation: new village6/6 (including deterministic8seed formation, overlap ownership, headroom, completion, frozen4 fingerprints, dissolution), existing loop7/7, tallies2/2, legacy pin1/1, scoped Clippy PASS. Leader independently reviewed diff and reran village6/6. No existing behavioral assertion changed. Elapsed under10-hour boundary; next is sustainable food/strict consumption.
- Instrument review round2: no further blocking finding; completed-tick maturation boundaries match engine ordering. Review capped at two rounds.
- Part3: game_v9 now has10 bushes,720-tick births, utilization0.85/reserve0.5/forage16, relief8 and strict food consumption. Supply counts future adults; reserve sums live-member actual hunger rates in sorted AgentId order. Empty/absent ground food returns Idle without relief only for strict rules; v8 leak remains frozen.
- Part3 validation: new food6/6, village6/6, unchanged loop7/7 and legacy pin1/1 PASS; core/systems all-target and food/village Clippy PASS. Evidence logs under .harness/evidence/revitalize-pass3/logs/part3-*. Leader reviewed production diff and new test fixtures. These are targeted checks, not yet sixty-day acceptance or full green gate. Boundary2026-10-10T20:22KST, elapsed approximately16minutes; next Part4 rhythm/children/FFI.
- Part4: enabled exact requested night/evening/child rules. BedAssignment145/1 orders settlements, live members and owned shelter coordinates deterministically; Maturation144/60 defers Child removal and restores founder rates. Newborn child rates are gated; default/v8 remain inert. Assigned-bed arrival cannot consume Sleep on an intermediate shelter. Critical hunger and thirst independently wake sleepers; natural/bias eligibility and probe T6 share the active fatigue threshold.
- Child job/social-initiation/combat/gather exclusions and child/evening idle tethers implemented; additive target_kinds/life_stages byte arrays derive directly from snapshot rows. agent_rows_split and rendering paths untouched. Newborns after20:00 can wait until the next20:00 or shelter-set change for assignment, using the specified fallback sleep target meanwhile.
- Part4 validation:167 targeted tests PASS (rhythm6,food6,village6,unchanged v8clock1,systems104,probe30,bridge14); workspace all-target Clippy PASS. Independent bounded reviews found no material defect; no runtime fix or assertion-repin needed. Logs p4-*.log. Full gate/long-run targets remain pending.
- Probe provenance now records full git SHA (metadata only). Evaluator review found mixed-revision input acceptance; fixed once with per-row/seed/round full-SHA binding to leader receipt and identical coincident fingerprints;9 Python tests PASS and final review clear. Tools publish with Part5. Part4 implementation commit2026-10-10T20:26:26+09:00,19minutes54seconds elapsed, below10h.
- Part5 round1 investigation journal (user-authorized evidence retained here instead of forbidden root/.omo paths): observed10day57 dehydration deaths, allSeekingSleep, no starvation. H1: seeking does not preempt critical needs; distinguish active thirst/availability/state transitions. H2: resource targeting overwrites assigned bed; distinguish SeekTarget vs bed before/after targeting. H3: invalid/unreachable bed or movement stalling; distinguish bed terrain/walkability/position/movement interval. Native agents inspect independently; no production edits before observed mechanism. Temporary bounded trace is planned only in new rhythm harness, log p5-round1-sleep-preemption-red.log; remove diagnostic prints after locking regression. No debugger/port or environment overrides; no unrelated processes touched.
- Round1 results: U1/U3/U4/U5/U7/U8/U9/U10/U11 PASS, U2/U6 FAIL. Runtime RED seed33agent12 at9483: SeekingSleep, position=(24,25), target=(24,25), bed=(25,24), thirst100;1146 critical-thirst seeking ticks. This confirms stale bed target after reassignment, not terrain blocking. Repair only synchronization to current assigned bed; leave generic preemption and all tuning parameters unchanged. Bound this as correction round1, next complete cohort is round2; no extra uncounted parameter search.
- Bed-target repair: identical seed33 RED repeated twice (12dehydration); focused target-refresh fixture RED. Guarded current-bed refresh GREEN: seed33zero deaths, critical Sleep-seeking ticks0. One fixture initially called movement at nonmovement tick780; corrected fixture to784 with explicit cadence assertion, no production expansion. Permanent regressions retain real10day execution and focused reassignment; temporary eprintln trace removed.118 targeted tests and all-target Clippy PASS; bounded review clear. Original whole-cohort QA repeats as round2 with unchanged knobs.
- Part5 completed2026-10-10T20:38:20.065073+09:00: round2 U1–U11 allPASS on3450163d. Retain original requested knobs, stop aftertwo cohorts/onebugfix, no third tuning search. Exact current-head v8/legacy reproduction and named pin green; real64x10/32x60 QA confirms zero deaths. Probe performance passes, not a renderer FPS claim. Full gate and screenshots follow before final green promotion.

- Part6 begins: capture destination is optionally configurable with the existing default preserved; shared window-lock wrapper matches the authorized goal verbatim. bash syntax checks pass. Mac uses only the dedicated capture clone; Pass2 images remain untouched.

- Part6 complete: Mac capture019bba07,10JPEG960x540, V1–V6 all42checks PASS; leader inspected d2noon/d2night/d6noon. OneHUDsettlement/onecentral label/compactcamp;22:00 sprites stack onfour shelters withsomeoutside, soindividualsleepcount is not visually established. Previous untracked capture files preserved inside capture clone evidence; no stale lock removed. No renderer edits.
- Preliminary fullgate at019bba07 PASS: debug1850/release1850,0failed/ignored (+44vsPass2each), bothClippy, forbiddenmath0, bridgebuild, Godotimport/strict/smoke no nonaudioerrors. Rawlogs published under verification/p3-7-code. Read-onlyscopeaudit clear; completed missing mechanical/schema testledger entries. Final probes and publication-head verification follow.

- Part7 final probes complete 2026-10-10T20:54:16.070863+09:00: producer6db6db6a, legacy4/v8game64/v9ten64/v9sixty32 twinsPASS, U1–U11 allPASS; v9allraw simulation values equalRound2, legacy/v8 frozen comparisons exact. No tuning afterRound2 and noRust changes after3450163d. Final report/evidence publication is docs-only; runfullgate again onthat exactWIPhead beforegreenFF, thenrecordfinalSHA inPR510 andfinalresponse. A report cannot embed its owncommitSHA; evidenceproducerSHA andpublicationSHA are explicitly distinguished.

## Pass 4

Executor: Claude Code (Opus 5.5, 1M context), unattended, on the Mac clone `/Users/rexxa/work/new-world-pass4`.

- `PASS4_BASE_SHA` = `a8611e07aabbb4043f5fac2e8dd14b375a0ada5d` (both `codex/revitalize-2026-10-wip` and `codex/revitalize-2026-10`).
- Run start: 2026-10-10 22:27:19 KST.
- Host: `Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64`.
- First-command check (0.2 item 5): no `.git/hooks/pre-commit` or `post-commit`; `git ls-files -v .claude/settings.json` → `S`; contents `{}`. Nothing to repair.
- The working tree also carries an unrelated `.gitignore` edit and a new `.ignore` written by the local graft indexer before the run started; they are not part of this goal and are never staged.
- Part 0 start: 2026-10-10 22:27 KST.
- `tools/revitalize/run_bounded.sh <seconds> <command...>` added (Python `subprocess.run(timeout=…)`, 124 on timeout). `probe.sh` made portable: on Darwin it skips the `free -g` memory wait and checks disk with `df -Pk .`; paths resolve through Python `os.path.realpath` instead of GNU `realpath -m`; an exported `CARGO_BUILD_JOBS` is kept (defaults to 10 as before); usage names `game_v10`. Linux control flow is unchanged apart from honouring an exported job count, which the goal asks for.
- Part 0.3 frozen v9 fingerprints (`sim_test::probe::fingerprint` of `game_v9_scene(seed)` after 1,440 ticks, computed at `e136ba8a` before any Rust change): seed 0 `c022f6555945fcad`, seed 1 `5ca9b7e597238ee3`, seed 2 `d8afca881c554360`, seed 3 `48cf47e1341f5048`. These are the constants of every Pass 4 `*_v9_unchanged` test.
- `probe.sh` second portability fix: macOS ships bash 3.2, where `"${workers_override[@]}"` on an empty array fails under `set -u`; the call now uses `${workers_override[@]+"${workers_override[@]}"}` (identical on bash 5).
- Part 0 probes were rerun from a detached worktree of `e136ba8a` in the session scratchpad, because `probe.sh` rebuilds the binary from the working tree and Part 1 edits had started in the clone. Only the summaries were copied back. Reproduction results are in METRICS.md `## Pass 4 baseline` (v9 and v8 identical to Pass 3).
- This clone had never been opened by Godot, so the headless GDScript parse test (`harness_p12_beta_a25_gdscript_parses_cleanly_in_godot_headless`) failed with `Could not find type "WorldSimNode"`. Building `sim-bridge` and running `Godot --headless --path . --import --editor --quit` once (the same step `capture.sh` uses) creates the ignored `.godot/` cache; it is environment setup, not a code change.
- Part 0 end: 2026-10-10 22:47 KST (commit `b11a8d15` at 22:47:16).
- Part 1 start: 2026-10-10 22:47 KST.
- Part 1 introduces the Pass 4 rule fields (`AgentRules::forage`, `SocietyRules::stock_target`, `SocietyRules::aging`, `WorldRules::seasons`) and their structs, the `Life`/`Elder` components and `DeathReason::OldAge` inertly (every preset, including `game_v10`, keeps them `None`/unused), together with pure helpers (`resource_regen::season_at`, `food_economy::{stock_target, stock_in_hand, season_supply_per_day, annual_mean_supply, winter_supply_per_day}`, `agent::haul_due`). This follows the Pass 3 precedent of inert Part 1 plumbing, so schema 4 and the evaluator are written once; each later Part switches its behaviour on in `game_v10()` only.
- `game_v10()` = `game_v9()` with sustain utilization 0.705 (12 bushes × 12 = 144; `(m + 1) × 2.4 ≤ 101.52` keeps the plateau at 42). `WorldGenConfig::game_v10(seed)` = v9 with 12 bushes. Both live constructors in `world_node.rs` now build v10.
- `regen_lost_at_cap` counts, for each finite food source per pulse, `cur + amount − max` when a pulse overshoots and the whole pulse when the source is already full; sentinel sources count nothing. It is display-only and counted in every scene.
- Death census: the cause comes from the recorded `DeathReason` (the `AgentDied` chronicle at the death tile, falling back to the recent-deaths buffer by tile and tick) only for old age; the three Pass 3 causes keep their saturation inference so v8/v9 census keys stay comparable. The cross-check now covers four tallies.
- Schema 4 adds 15 sample keys: the 11 required ones plus `days_per_season`, `haul_deposits_total`, `food_supply_season_per_day`, `food_supply_winter_per_day` (needed by the W7 and Seasons computations). `foraging_now` counts `Seeking{GatherFood}` agents while the forage rule is on (under the rule every gather seeker is routed by the forage choice rule); `hauling_now` uses the same `haul_due` predicate as movement. `sustainable_adults` uses the annual-mean supply when seasons are on, mirroring the production gate.
- Seasons summary attribution: an elapsed day is attributed to the season of its closing checkpoint (display 08:00). `evaluate_pass4.py` works on raw periodic and clock rows, which carry their own season.
- Part 1 verification: debug workspace 1,854 passed / 0 failed; both clippy variants clean; forbidden-math grep 0 hits. Part 1 end: 2026-10-10 22:54 KST (commit `8b33728f` at 22:53:57).
- Part 2 start: 2026-10-10 22:54 KST.
- Part 2 rules in `game_v10()` only: `ForageRule {360, 1080, 16, 4, 1}` and `StockTargetRule {1.5, true, 2.0}`. The forage behaviour is active only when both rules are present.
- Forage arm: inserted in the Idle cascade after Combat and before Gather. Claims are counted once before the agent loop from every `Seeking{GatherFood}` agent's `SeekTarget` and incremented as the loop assigns foragers; the agent loop keeps the existing hecs query order (deterministic since the a15 fix), so earlier agents in that order pick first. Target and stock in hand are computed once per settlement per decision tick, in sorted settlement id order, before the loop.
- Post-pass under the forage rule: a forager's chosen bush becomes its `SeekTarget`; a gatherer whose `SeekTarget` tile holds food keeps it; a fresh opportunistic gatherer still gets the nearest tile within `GATHER_SCAN_RADIUS`; a gatherer whose tile is empty is re-targeted in AgentId order by the same choice rule (claims = targets held after the pass, the empty tile excluded) or becomes Idle without a target.
- Opportunistic Gather under the rule also needs carried food < haul threshold and, for members, `stock_in_hand < hoard_cap_factor × target`; non-members (before formation) are bounded only by the haul threshold.
- Haul home: in `AgentMovementSystem`, an Idle member without `Child` for whom `haul_due` holds takes a directed signum step home and draws no RNG; the step is recorded in the display-only `SimResources::haulers` set, and `StockpileDepositSystem` counts `haul_deposits` when a member in that set deposits (removing it). `despawn_agent` also drops the agent from the set.
- Decision beyond the letter of section 7: under the forage rule, a member carrying at least a haul load is not assigned as a builder (it delivers first). Without this, v10's immediate shelter site drafted the test hauler 12 tiles out and it never walked home; the rule only affects members already carrying a full load.
- Part 2 verification: the six `harness_revitalize_pass4_forage` tests pass, plus three new module unit tests; both clippy variants clean.
- Timing correction: the Part 0–2 boundary times first written into this file were estimates; they were replaced with times read from `date` and the commit timestamps (`git log --date=format:%H:%M:%S`). Every later boundary is read from `date`.
- Part 2 end: 2026-10-10 23:01 KST (commit `d5b43c1b` at 23:00:51). Part 3 start: 2026-10-10 23:01 KST.
- Part 3: `AgingRule {66_240, 83_520, 11_520, 8_640, 57_600, 4_320, 4}` in `game_v10()`. `AGING_RNG_SALT = 0x4C1F_E5A1_0000_0001`; a grep of every `0x…_…` 64-bit constant in `rust/crates/*/src` shows the other salts are `BIRTH_RNG_SALT 0xB117_0000_5EED_0001`, `BOOTSTRAP_RNG_BASE 0xA5A5_A5A5_0000_0001`, `BOOTSTRAP_NEED_STAGGER_SALT` / camp needs salt `0x5EED_5A66_E0DE_0001`, camp resident salt `0xA9E7_0000_0000_0001`, so it is distinct. The aging stream is a fresh `MovementRng` value (splitmix64) keyed by `world_seed ^ (id × 0x9E37_79B9_7F4A_7C15 + salt)`; the movement `MovementRng` component is never touched.
- `AgingSystem` (priority 146, interval 60) is registered in `register_rhythm_systems` for every engine and returns immediately without the aging rule, so legacy/v8/v9 behaviour and fingerprints are unchanged. Deaths and promotions are applied in AgentId order after the query.
- Founders get `Life` in `camp::populate` (world seed = `cfg.seed`); newborns get `Life { born_tick: tick }` in `run_births` next to `Child`.
- Elders: a `worker = adult && !elder` flag gates the Construction arm (natural and bias paths), builder candidacy, Combat, Forage and opportunistic Gather; Social, needs, sleep and eating are unchanged. Their idle tether is `elder_tether_radius` around the formation tile at all hours, replacing the evening tether for them (the goal says "at all hours").
- FFI (additive): `life_stages` reports 2 for a live `Elder` (child still wins), `get_agent_snapshot` gains `ages_days` (whole days, -1 without `Life`) built by `collect_agent_ages_days` over the same `Agent + Position` archetype order; `get_tallies` has `deaths_old_age` (Part 1); `get_recent_deaths` reports reason 3 through `DeathReason::as_u8`.
- Part 3 verification: five `harness_revitalize_pass4_aging` tests, the six forage tests, four aging unit tests and the new bridge snapshot test pass.
- Part 3 end: 2026-10-10 23:09 KST.
- Part 4: `SeasonRule {7, [750, 1250, 1000, 500]}` and utilization 0.81 in `game_v10()` (annual mean 126/day; 0.81 × 126 = 102.06 admits 42 residents and refuses 43; 42 × 2.4 = 100.8 and 43 × 2.4 = 103.2 are far from the boundary). Regen uses `SimResources::food_regen_acc` (thousandths, zero without seasons): `acc += permille[season]; units = acc / 1000; acc %= 1000`, and a zero-unit pulse skips the food channel entirely. The season comes from `resources.current_tick` plus the clock offset (regen pulses at multiples of 120 align with display-day boundaries, 12 per day).
- The sustain gate calls `annual_mean_supply`, which returns exactly `supply_per_day` without seasons (v8/v9 bit-identical). `get_time_snapshot` adds `season`, `day_of_season` (1-based) and `days_per_season` derived from the snapshot's own `day_index` (0/0/0 without seasons).
- W11 evaluation: each round is compared with the same-host Pass 3 game_v9 baseline of its own duration (10 days: `p4-0_v9_repro`; 120 days: `p4-1_v9_120day`), because per-agent-tick cost grows with run length in v9 too (0.000778 → 0.000883).
- The gate script runs `cargo build -p sim-bridge` and a headless Godot import before `cargo test`, because a fresh worktree otherwise fails the headless GDScript parse test (environment, not code). The first gate attempt on Part 2 (`d5b43c1b`) stopped at that environment failure and was superseded by a gate on the Part 3 head.
- Part 4 end: 2026-10-10 23:19 KST.
- Part 5 start: 2026-10-10 23:20 KST. Three tuning rounds (round 1 six knobs; round 2 two knobs on top of round 1; round 3 one knob on top of round 0), all inside the section 10 ranges; every round recorded with its numbers in METRICS.md.
- Kept configuration: round 0, i.e. the Part 4 `game_v10()` (rules.rs restored byte-for-byte from `e72d7e28`, whose full gate is green). Reason: target priority follows the table order, and round 0 is the only configuration passing W1–W7; rounds 1–3 each fail W5 (seed 22) and round 2 also W6.
- Three strikes — W8 (noon idle ≤ 0.45; forage trips ≥ 10/day in summer and autumn): fails in all four configurations. Evidence: in summer no food remains on the ground from 10:00 to 20:00 (all 32 seeds), so after the 08:00–10:00 harvest 34–35 of 42 residents are idle with nothing to forage while stock in hand is still below target; elders (share ≈ 0.26) and children (≈ 0.10) idle by design, and haulers walking home are in the Idle state. Autumn trips are zero because the summer surplus already lifts the store above the autumn target. Hypothesis: noon idleness is bounded by daytime regrowth, not by labour; only more food per day or a different job (not a listed knob) can change it.
- Three strikes — W9 evening near hearth ≥ 0.30 in every seed: minimum 0.257–0.281 in every round (median 0.53–0.57 passes; night in shelter passes). In the low seeds the whole membership's mean home distance at 19:30 is 6–7 tiles with only 3–4 haulers. Hypothesis: with seasons the bushes are empty most of the day, every regrown unit draws all hungry residents (global need availability) across the map, and the store only feeds critically hungry residents standing at home, so in seeds with distant bushes residents are strung out in the evening. Changing where residents eat is outside the permitted knobs.
- Three strikes — W10 day-10 home distance (median ≤ 7.0, every seed ≤ 10): the median passes in rounds 1–3 (forage start 08:00) but those rounds break W5; the maximum is seed 7 (bushes 12–16 tiles away) at 10.9–11.1 in every round, and was 11.28 in Pass 3 v9 as well. Kept round 0: median 7.012, max 11.02.
- Part 5 end: 2026-10-10 23:36 KST.
- Part 6 start: 2026-10-10 23:37 KST. The tmux session runs in launchd's Background session (`launchctl managername` → `Background`), so `capture.sh` refused with "Tier A requires an Aqua session" (exit 77). The console user is `rexxa` and `launchctl asuser 501 launchctl managername` reports `Aqua`, so the same single command ran inside the user's GUI session: `with_window_lock.sh 1500 launchctl asuser 501 env CAPTURE_DOCS_DIR=docs/revitalize/shots/pass4 … bash tools/revitalize/capture.sh final` (window lock held for the whole command; no GDScript or renderer edits). Ten 960 × 540 JPEGs and `visual_qa.md` (42/42 checks, V1–V6 all PASS) are committed; the capture uses the live game (v10, default seed).
- Part 6 end: 2026-10-10 23:39 KST.
- Part 7 start: 2026-10-10 23:45 KST (after the Part 6 commit).
- Independent review (built-in `general-purpose` subagent, fresh context, read-only; given `git diff a8611e07..HEAD -- rust/ tools/revitalize/`, sections 4 and 7–10, and the W table). Round 1 findings and dispositions:
  1. Medium — `agent_decision.rs` post-pass: under the forage rule a fresh `Seeking{GatherFood}` agent whose Gather-arm tile was emptied later in the same tick got no target and froze, ignoring needs until food regrew within 8 tiles. **Fixed**: it now becomes Idle; regression test `harness_forage_gatherer_whose_tile_vanished_idles_instead_of_freezing`. This changes v10 behaviour, so every final v10 number comes from probes run after the fix.
  2. Low — `haulers` was cleared only by a deposit or death, so a later passive deposit could count as a haul. **Fixed**: a member outside the deposit radius that did not take a haul step is removed on each movement tick (display-only).
  3. Low — haul (movement) and the probe gated on `forage` only, the decision system on `forage && stock_target`. **Fixed**: one gate everywhere.
  4. Low — evaluator W8 labelled each elapsed day with its closing checkpoint's season. **Fixed**: opening-checkpoint attribution (`window_season`); the probe's Seasons table uses the same attribution (and its supply column comes from the opening row).
  5. Low — evaluator W6 replacement started at the day-30 checkpoint. **Fixed**: subtracts the day-29 checkpoint (days 30–120 inclusive).
  6. Low — weak tests. **Fixed**: the re-target test now asserts after every decision pass that a gatherer's target holds food; the season-target test checks independent numbers (151.2 / 352.8); new `regen_channel_counts_regrowth_lost_at_the_ceiling`; new world test `harness_haul_home_after_the_forage_window`.
  7. Low risk — `Seeking{GatherFood}` is not pre-empted by critical needs. **Not changed**: Pass 3 pre-empts only Food/Water seeks, section 7 does not ask for it, W2/W3 pass in every probe; recorded as a known limitation.
- Review round 2 (same reviewer, re-check of the fixes): all six fixes confirmed correct with no new defect; F7 accepted as a recorded risk. Noted and accepted: a hauler pulled away by a seek keeps its `haulers` flag until it is idle outside the deposit radius again (it was carrying a load home, display-only); `harness_haul_home_after_the_forage_window` assumes the highest-id member is an adult with no food within 8 tiles of its start tile and fails loudly if that changes. Review capped at two rounds.
- Part 7 final: full gate green on code head `0dc9a38d` (debug and release 1,881 / 0 / 0, both clippy variants, forbidden math 0, bridge build, Godot import, strict check, smoke without errors, Python 17 OK); green branch fast-forwarded to it. Final probes (seven runs, all determinism PASS): legacy, v8, v9 10-day and v9 60-day equal to Pass 3 on every shared key (only the summary `schema_version` differs); v10 W1–W7 and W11 PASS, W8, W9, W10 FAIL (three strikes). `REPORT_PASS4.md` written in Korean in the section 16 format.
- Elapsed working time ≈ 1 h 55 min (22:27 → about 00:20); the 10-hour rule never triggered; no usage-limit pause; no context compaction; nothing dropped from the drop order (Part 4 and the elder behaviours of 8.4 are both implemented).
- Part 7 end: 2026-10-11 00:16 KST (before the final docs-head gate and the PR #510 update recorded in the final response).

## Pass 5

Run by Claude Code (Opus 5.5, 1M context) from `docs/revitalize/PASS5_GOAL.md`, unattended.

- `PASS5_BASE_SHA` = `5cbcbc4f9bd02bf3f00695bae810368b93fe6300`. Start 2026-10-11 00:56:05 KST. Host `Darwin hyunlordui-MacBookPro.local 27.0.0 Darwin Kernel Version 27.0.0: Tue Aug 11 21:05:48 PDT 2026; root:xnu-13432.1.9~1/RELEASE_ARM64_T6041 arm64`.
- Section 0.2 check: no `.git/hooks/pre-commit` or `post-commit`; `git ls-files -v .claude/settings.json` starts with `S`; the file holds `{}`. Nothing to record or remove.
- The working tree also carries an unstaged `.gitignore` change and an untracked `.ignore` (graft index files, not part of this run); never staged.
- Part 0 probes ran from a copy of the base release probe binary (`scratchpad/bin/probe_base`), so Part 1 edits could start while they ran; only summaries were copied back.
- `p4-7_final_v10_10day` was produced without `--clock-samples`; a first Part 0 rerun with clocks differed only in the rhythm block, so the stored `p5-0_v10_10day` is the clock-free rerun (0 differences). Clock samples do not change simulation values (they only add raw rows).
- Part 0.3 fingerprints use seeds 0..4 half-open (four seeds), as Pass 4 did for v9.
- Part 0.4 used a throwaway integration test (deleted, not committed) because schema 4 has no per-state breakdown (eating at a bush, social, sleeping).

### Part 1 — instruments and the v11 skeleton
- All Pass 5 types land in Part 1 as inert substrate so the probe and FFI code are written once: `SocietyRules::meals: Option<MealRule>`, `SocietyRules::firewood: Option<FirewoodRule>`, `AgentRules::preempt_gathering: bool` (None/false in legacy, v8, v9, v10 and — until Parts 2–4 — v11); `TargetKind::GatherWood` (FFI code 7); `components::Cold`; `sim_engine::FireState`; `SimResources::{wood_tiles, wood_source_max, wood_regen_acc, fires}` (all `BTreeMap`/integer, empty or zero without the rule); six `Tallies`. `game_v11()` = `game_v10()` in Part 1.
- `wood_tiles` / `wood_source_max` are `BTreeMap<(u32, u32), u8>` (section 4.1 asks new collections to be ordered), not `HashMap` like the food maps they mirror.
- Deadwood placement (8.1) lands in Part 1: `WorldGenConfig::{deadwood_spots, deadwood_capacity, deadwood_min_radius, deadwood_max_radius}` (0 everywhere but v11: 20, 6, 5, 18), `camp::deadwood` after `populate`, stream `MovementRng::new(seed ^ DEADWOOD_SALT)` with `DEADWOOD_SALT = 0xDEAD_0D00_F1E1_0005` (distinct from `0xA9E7_0000_0000_0001`, `0x5EED_5A66_E0DE_0001`, `BIRTH_RNG_SALT 0xB117_0000_5EED_0001`, `AGING_RNG_SALT 0x4C1F_E5A1_0000_0001` and the `h64` salt 8). The distance is measured from the Hearth building tile (the formation tile under `SettlementAnchor::Hearth`). Forest candidates are shuffled and taken first, then Plain; a spot needs Chebyshev ≥ 2 from every spot already taken; water, buildings and bushes (`food_source_max` and `food_tiles`) excluded. Nothing reads the wood maps until Part 3, so the Part 1 v11 world runs bit-for-bit like v10 (checked: `game_v11_scene(s)` fingerprints after 1,440 ticks equal the v10 constants for seeds 0–3, with 20 spots placed).
- `burn_per_day(s) = burn_per_hour[s] × burn hours`, where burn hours are computed from the configured window (13 for 18:00–07:00), so the formula stays right if the window ever moves; `firewood::wood_target` is identical for every settlement (it depends only on the rule and the season).
- Haul home generalised in `movement::hauls_home(forage, firewood, food, wood, in_forage_window, in_wood_window, distance)`; without the firewood rule it is exactly Pass 4's `haul_due`. The probe's schema-4 `hauling_now` uses it, so under v11 it also counts wood haulers (unchanged meaning: members walking home with a load).
- Tallies `meals_from_store` and `ground_meals` are counted in every scene (display-only, never read by simulation), so v10 has a baseline for X4: store relief meals (`StockpileConsumeSystem`) and resolved `Consuming { Food }` meals.
- Schema 5 adds 15 keys (all never null): the 12 listed in section 6, `cold_member_hours_total` (the cumulative hourly tally, needed for the summary's cold member-hours) and the two work keys. `adult_work_share` and `idle_share` are written on every row (not only clock rows), which keeps the raw row shape uniform. `idle_share` = residents Idle **and not hauling** / all residents; Pass 4's `noon_idle_share` (= `state_idle / alive`) counts haulers as idle, which differed by the hauling share (≈ 0.025 at noon in v10). The Pass 5 baseline `p5-1_v10_120day` reports both. `fire_lit` = 1 when at least one settlement exists and every settlement's fire is lit.
- `evaluate_pass5.py` needs the configured winter burn per day (default 39 = 3 × 13) and the cold window (default 1320,360) as arguments because samples carry the derived wood target, not the rule. X7 uses periodic samples (every 120 ticks: 22:00, 00:00, 02:00, 04:00 fall in the cold window) as `Σ cold_members / Σ settled_agents` per seed over winter cold-window samples. X4's per-seed clause uses each seed's summed store meals / summed meals from day 3.
