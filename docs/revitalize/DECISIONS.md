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
