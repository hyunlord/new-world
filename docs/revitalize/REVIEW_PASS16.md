# Pass 16 independent review

Two rounds at most (PASS16_GOAL.md 12.1), each by a fresh built-in `general-purpose` subagent, read-only.

Round 1: 2026-10-15 09:43–09:52 KST, HEAD `38f75d4c` (before the evaluation results existed; items 1–9 of 12.1 except the T verdicts, which round 2 checks).

## Round 1 — the reviewer's report (verbatim)

Base is `da8e45c7`. The Pass 16 commits are 579faf9b, 1362addd, 01f0fe02, aa254bc1, 7ee8b86c, 9a9888fd, 4296c166 and 38f75d4c. I did not build or run anything; I read the code and recomputed the scripted observer test by hand. No item fails and there is no High finding; there is one Medium and nine Low.

## Per-item table

| # | Item | Verdict | Evidence |
|---|---|---|---|
| 1 | v19 pins committed alone, before any other Rust change; rules pinned field by field | PASS | `1362addd` touches only `rust/crates/sim-test/tests/harness_revitalize_pass16_frozen.rs` and `docs/revitalize/DECISIONS.md`. `git diff da8e45c7 1362addd -- rust` is that one file. The next commit, `01f0fe02`, is the first other Rust change. The pin file has not changed since (`git diff 1362addd HEAD` on it is empty). Every `AgentRules` and `SocietyRules` field and every `AgingRule`/`FirewoodRule`/`FissionRule` leaf has its own `assert_eq!`/`assert!` (frozen.rs L42–255; I checked coverage against the struct definitions). There is no `WorldRules {` or `AgentRules {` literal. |
| 2 | Only new field is `AgentRules::temperament`; `get_agent_detail` keeps 9 keys; frozen tests changed by one line | PASS | In `git diff -U0 da8e45c7 HEAD -- rust` the only struct-body change is `AgentRules` (rules.rs L52). The `RunSpec` hunk is a doc comment, and the `PerfArgs` hunk is the `USAGE` const after the struct. `--agent-trace` lives in the new type `ProbeOptions` (agent_trace.rs L40–65), not in `ProbeArgs`. `agent_detail_to_dict` is untouched and still has 9 `dict.set` calls. Each of `harness_revitalize_pass{6..13}_frozen.rs` differs by exactly `+ temperament: None,`. |
| 3 | Arithmetic of 6.1–6.2, no forbidden math, the new stream | PASS | Details below the table. |
| 4 | Hook placement, and the `None` path equal to the old one | PASS (Low L7) | Details below the table. |
| 5 | Observer definitions, scripted test, outputs unchanged when off | PASS (Low L1, L4, L5) | Details below the table. |
| 6 | Pre-registration order; tuning seeds | PASS | The T table was committed in `9a9888fd` at 09:27:53 +0900. DECISIONS records the evaluation launch at 09:41:17 (PID 1605407, code `4296c166`). The DGX job launched at 09:19:59 ran only the inert check (seeds 0..32) and then tuning. Both `p16-3_tuning1/eval.json` and `p16-3_tuning2/eval.json` cover seeds 16100–16115 (16 worlds). |
| 7 | Allowed paths | PASS | `git diff --name-only da8e45c7..HEAD` lists nothing outside `rust/`, `tools/revitalize/` and `docs/revitalize/`. Nothing is under `rust/crates/sim-test/src/worldmodel/`, `scripts/`, `scenes/`, `assets/` or `localization/`. The only "worldmodel" match is the log `docs/revitalize/verification/p15_gate/python_worldmodel.log`, which is allowed. |
| 8 | `evaluate_pass16.py` follows section 9 | PASS (Low L2, L3) | Details below the table. |
| 9 | Other correctness bugs | FINDING (M1, plus Lows) | See the findings sections. |

### Item 3 detail
- **Founder draw** (`sim-core/src/components/temperament.rs` L57–71). The slot is `4*a as u32 + s` for s in 1..=3. The draw uses tick 0, replicate 0 and n = 1001, and the sum is divided by 3 in integers.
- **Child** (temperament.rs L80–113). The mid-parent is an integer mean. With one known parent, that parent's axis is the mid-parent. With none, the child is `fresh`. The value is `500 + m/2 + f*935/1000` in `i32`, clamped to 0..1000.
- **`TemperamentRule` methods** (`sim-core/src/rules.rs` L85–159):
  - `tether`: `max(2, base + span·d/500)`.
  - `bedtime`: `start − span·d/500` (see L6 for the extra wrap).
  - `social_rate`: `base × f64(1000 + span·d/500) / 1000`.
  - `food_haul` / `wood_haul`: clamped to 1..8.
  - `departure_key`: `unit − w·ns/1e6`.
  - Every method returns its base value when `effects` is false.
- **Tests:** the extreme and mean cases (d = −500, 0, +500) are covered in `temperament_functions_at_the_extremes_and_the_mean`. The `effects: false` cases are covered in `temperament_without_effects_returns_every_base`.
- **Stream:** `STREAM_TEMPERAMENT = 0x5445_4D50_524D_4E54` is distinct and is the 17th entry of `STREAMS` (rng.rs L88, L91–109).
- **Forbidden math:** the only math function in the added sim-core/sim-systems/sim-engine lines is `var.sqrt()` inside a test, and sqrt is allowed. A wider grep of the three crates (`.log(`, `ln_1p`, `exp_m1`, `asin`, `acos`, `atan`, `sinh`, `cosh`, `powf`, `powi`) finds nothing.

### Item 4 detail
**HA** (`runtime/decision/agent_decision.rs`)
- `person_night_rest` (L247–259) equals the old global filter when there is no lean.
- `rest_for` (L670–678) returns the old global pair `(night_rest, fatigue_threshold)` whenever the rule or the component is missing.
- It is applied at:
  - the builder query (L743/757);
  - `social_eligible_by_pos` (L902/911);
  - both peer maps (L961/968 and L985/987);
  - the top of the main loop (L1080).
- The main-loop binding shadows every in-loop read, including the meal-wait threshold uses (~L1339–1345) and the `Consuming{Sleep}` branch (L2147). The main loop runs from L1060 to L2204.
- The post-pass only reads the unfiltered rule (L1924, L2332, L2343).
- `beds.rs` L23/L92 and the probe's `fatigue_threshold_at` are unchanged.

**P** (workers only)
- In the decision system: L744–751, L1088–1105, L1575, L1600, L1632 and L2111.
- In movement: `runtime/agent/movement.rs` L297–311, filtered by `elder.is_none()`. Children are already excluded by the `(…, None, None)` pattern.
- The remaining reads are test fixtures: firewood.rs L325, movement.rs L422, agent_decision.rs L2571.

**NS**
- Tether: only the adult fallback, movement.rs L359–365. The child branch, the elder branch and `evening_radius` are untouched.
- Departure: fission.rs L557–588 is gated on `rule.effects`, and the `else` branch is the old code verbatim.

**RD and insertion**
- The `Social` rate is mutated right after the draw: `worldgen/camp.rs` L159–167 and `runtime/settlement/settlement_system.rs` L896–907.
- The insert sits inside the same `if let Some(lineage)` block as `Hardiness`.
- The parent lookup (settlement_system.rs L835–850) is gated and immutable, and it draws nothing.

**v19 equivalence:** there are no new early returns or query filters. The only query change is the added `Option<&Temperament>`.

### Item 5 detail
**Definitions** (`sim-test/src/probe/agent_trace.rs`)
- Non-elder adult: no `Child` and no `Elder` component (L269).
- Idle sample: taken when `sys_tick % 10 == 0`, the clock is in 360..1080, the person is Idle and is a settlement member (L276–282).
- `night_of`: from 18:00 it gives the minute itself, and before 03:00 it gives 1440+m (L124–136, L283–285).
- Social completion: `prev − now ≥ min(25, prev)` (L290).
- Deposit: a drop within `SETTLEMENT_PROXIMITY_RADIUS` (5, the same radius the deposit system uses) of the hearth, recorded with the previous tick's carried amount (L293–305).
- `departed`: only for fissions with `founded_tick` set and not cancelled; cancelled ones are counted (L335–347).
- v19 would-be temperament: read from the `AgentBorn` ring at the newborn's tile and never inserted (the tracer takes `&SimEngine`). Movement (priority 120) runs before the birth (138), so the tile is right.

**Hand recomputation of `scripted_adult_counters`**
- System ticks run 0..865, so there are 866 adult ticks and `adult_days` = 866/1440 ✓.
- Idle samples:
  - Tick 0 reads 05:59, so it is not sampled.
  - Tick 10 is sampled at distance 2.
  - Ticks 20..720 are sampled at distance 1, since 720 + 359 = 1079 < 1080. That is 71 samples.
  - Total: 72 samples summing to 73 ✓.
- Deposits:
  - At tick 12 the food drop happens at distance 6 > 5, so it does not count.
  - At tick 13 food goes 4→0 and wood 2→0 at distance 1.
  - Result: food (sum 4, count 1) and wood (sum 2, count 1) ✓.
- Social:
  - At tick 15 loneliness drops 40→12, which is 28 ≥ 25, so it counts.
  - At tick 17 the drop is 0 < 12, so it does not.
  - Result: 1 ✓.
- Bed minute: tick 861 + offset 359 = 1220, which is night 0, minute 1220 ✓.

**Outputs with the flag off:** the tracer exists only when `options.agent_trace` is set (runner.rs L171), and `run` is `run_with(args, &ProbeOptions::default())`. Nothing else in the runner changes.

### Item 8 detail
- Spearman uses average ranks (L55–78) and pools across worlds (L159–164).
- The minimum-sample filters are in L137–156.
- The bootstrap uses B = 2000 with a fresh `default_rng(0)` per statistic, resamples worlds and takes a percentile interval (L91–106).
- T1 applies the bars and the "interval excludes 0" rule (L234–246).
- T2 requires |ρ| ≤ 0.10 and |dep| ≤ 40 (L249–254).
- T3 uses ±15 % alive and +1.5 non-old-age deaths (L278–308).
- T4 requires each axis's slope in [0.35, 0.65] and each SD within ±20 % (L311–348), as pre-registered in METRICS.

## High

None.

## Medium

**M1 — The final-build identity (T5) evidence is one Rust commit behind the final code.**
- *Where:* DECISIONS.md, Pass 16 Part 2. I1–I3 ran with the `7ee8b86c` binaries, and the 120-day inert check ran with `P16_CODE_COMMIT = 7ee8b86c`. Later, `4296c166` changed `sim-core/src/rules.rs` L845–856 (`V20_TEMPERAMENT`: food span 2→3, wood 1→2), and the evaluation ran on `4296c166`.
- *Problem:* spec 7.1 and T5 require I1–I3 and the inert checks on the final build. The recorded evidence is for the build before it.
- *Why it matters:* logically nothing changes, because v19 never reads the rule and inert's haul functions return the base value when `effects` is false. But the audit trail for T5 does not cover the build being reported.
- *Suggested fix:* either rerun I1–I3 on HEAD, or record the argument explicitly in DECISIONS and REPORT (the change touches constants only and is gated on `effects`). Also cite the Part 6 gate run of the v19 pins and `harness_v20_inert_equals_v19_at_40320_ticks` on HEAD.

## Low

**L1 — Trek distances confound T1-NS in v20.**
- *Where:* `sim-test/src/probe/agent_trace.rs` L208–218 and L276–282. Membership is only dropped on death (`settlement_system.rs` L283–285).
- *Problem:* departers stay in the parent village's `member_agents` while walking to the new site (up to `settle_timeout_days` = 2). If they are Idle on the way, their distance from the parent hearth (about 16 tiles or more) is sampled.
- *Why it matters:* the v20 departure hook favours high NS, so these samples correlate with NS through departure rather than the tether. The effect is probably small (a few trek days against about 120 days of samples), and the placebo is unaffected. It is allowed by the spec's wording but is a confound.
- *Suggested fix:* skip idle samples while the person carries `Founding`, or report T1-NS without those samples as a sensitivity row.

**L2 — T2 includes the wood ρ.**
- *Where:* `tools/revitalize/evaluate_pass16.py` L250.
- *Problem:* `p_wood` counts toward the T2 verdict, but section 9 reports the wood ρ "(no verdict)" in T1. "Every T1 statistic" is ambiguous here.
- *Why it matters:* a chance |ρ_wood| > 0.10 in the inert or v19 runs would fail T2 on a statistic that has no verdict in T1.
- *Suggested fix:* state the reading in eval.md, or report T2 both with and without `p_wood`.

**L3 — The bootstrap silently drops NaN draws.**
- *Where:* `tools/revitalize/evaluate_pass16.py` L101.
- *Problem:* resamples with fewer than 3 pairs, or with no departers, are dropped, so an interval can rest on fewer than 2,000 draws. Section 9 does not say this.
- *Why it matters:* the reader cannot tell how many draws an interval is based on.
- *Suggested fix:* record the number of valid draws next to each interval.

**L4 — Rare parent-alive mismatch between the observer and the engine.**
- *Where:* `agent_trace.rs` L259–264, compared with `settlement_system.rs` L835–850.
- *Problem:* the tracer treats a parent as known if they were alive at the end of the previous tick. The engine needs the parent alive at the birth itself. A parent who dies earlier in the birth tick (before priority 138) is known to the tracer but not to the engine.
- *Why it matters:* in v20 this appears as `component_mismatches` (0 in tuning). In v19 it would go unseen and would bias T4 and the placebo only marginally.
- *Suggested fix:* report `component_mismatches` from the evaluation runs; optionally mirror the engine's timing.

**L5 — The deposit amount can undercount.**
- *Where:* `agent_trace.rs` L298–305.
- *Problem:* the counted load is `prev.food` / `prev.wood`. A pickup inside the radius by the decision system (priority 125) on the same tick the deposit system (priority 141) drains is not included.
- *Why it matters:* the measured load is slightly lower than what was deposited. This matches the spec's wording ("the amount carried on the tick before").
- *Suggested fix:* none required; add one sentence to eval.md.

**L6 — `bedtime` wraps the day.**
- *Where:* `rules.rs` L113–118.
- *Problem:* it adds `rem_euclid(1440)`, which the spec formula does not have.
- *Why it matters:* it never triggers at the frozen spans (bedtimes 1,200–1,320). It is documented in DECISIONS.
- *Suggested fix:* none.

**L7 — Duplicate set lookups in the decision builder.**
- *Where:* `agent_decision.rs` L744.
- *Problem:* `haul_for(!children.contains(..) && !elders.contains(..))` repeats per agent the two BTreeSet lookups the same closure already makes at L753.
- *Why it matters:* this is not a correctness bug, but it goes against 3.1's "no per-agent lookups" guidance and adds a small cost in the 8.5 performance runs.
- *Suggested fix:* compute the worker flag once and reuse it.

**L8 — `game_v19_nofission` is not asserted to have no temperament.**
- *Where:* `rules.rs` L1654 (`game_v20_is_game_v19_with_temperament`).
- *Problem:* only `game_v19(s).agent.temperament == None` is asserted. The field-by-field pin predates the field and cannot cover it.
- *Why it matters:* setting the rule on the nofission scene would go unnoticed by the rules tests. The fingerprint pins would catch it only if behaviour changed, and the inert rule changes none.
- *Suggested fix:* add `assert_eq!(WorldRules::game_v19_nofission(seed).agent.temperament, None)` to that test.

**L9 — The gate's forbidden-math grep is narrower than "log*".**
- *Where:* `tools/revitalize/gate_pass16.sh` L47.
- *Problem:* the pattern does not match `.log(`, `ln_1p`, `exp_m1`, `asin`, `acos`, `atan`, `sinh` or `cosh`. It is unchanged from Pass 15, as the spec asks.
- *Why it matters:* the code uses none of these today (I checked by grep), but the gate would not catch them if they appeared.
- *Suggested fix:* none this pass; widen the pattern in a later pass.

## Round 1 — dispositions

| id | disposition |
|---|---|
| M1 | **Planned in Part 6 when round 1 was answered; done since** — see round 2's M1 below for the results and directories. |
| L1 | **Kept, documented.** The definition is the pre-registered one (members of a settlement, the hearth of the first settlement listing the person); changing it after the evaluation would be a post-hoc metric. `eval.md` (notes) and the report name the confound; it acts only in v20 and only on the few Idle samples of a two-day trek. |
| L2 | **Fixed.** `evaluate_pass16.py` reports T2 both with (primary, stricter) and without the wood ρ; both pass. |
| L3 | **Fixed** for every T1, T2 and departure interval (2,000 valid draws each); T3's and T4's intervals cannot be undefined with 32 (or 18) worlds and carry no count (review round 2, L1). |
| L4 | **Reported.** `component_mismatches` is 0 in all three evaluation scenes and both tuning rounds (`eval.md`, trace bookkeeping); `born_event_misses` 0. |
| L5 | **Documented** in `eval.md`'s notes (affects every scene alike). |
| L6 | No change (documented in DECISIONS). |
| L7 | **Kept.** Removing the duplicate lookups would change `sim-systems` after the evaluation build; the cost is two BTreeSet lookups per agent per tick only while shelter planning runs, and the performance rule (8.5) measures it as it is. Noted for a later pass. |
| L8 | **Fixed.** `game_v20_is_game_v19_with_temperament` also asserts `game_v19_nofission(s)` and `game_v18(s)` carry no rule (test-only change in `rules.rs`). |
| L9 | No change this pass (the gate's grep is Pass 15's, as 4.5 asks); noted for a later pass. |

Round 2: 2026-10-15 10:08–10:15 KST, HEAD `9eb221d3` (after the evaluation; no cargo, Godot or probe runs — the art run held the quiet lock; recomputation from the raw traces in Python).

## Round 2 — the reviewer's report (verbatim)

HEAD `9eb221d3`. I ran no cargo, Godot or probe binary and edited nothing. My own Python (scipy and numpy, without importing `evaluate_pass16.py`) recomputed the statistics from the raw traces in `/Users/rexxa/work/p16-data/dgx/`.

Nothing fails on the numbers. Every recomputed value matches `eval.json` and METRICS to 3 decimals, every T verdict follows section 9, and T1-P is correctly a FAIL. The two Medium findings are about documents that report work as done, or results as validated, before the evidence exists.

### Per-item table

| # | Item | Verdict | Evidence |
|---|---|---|---|
| 1 | Round-1 dispositions done as stated | **FINDING** (M1, L1) | **Done:**<br>• **L2:** `t2_verdict(with_wood)`. Both readings are in `eval.md`/`eval.json` and in the regenerated tuning evals.<br>• **L3:** `VALID_DRAWS` counts are 2000 on every T1, T2 and dep row.<br>• **L8:** `rules.rs` L1655–1656 assert `game_v19_nofission(seed)` and `game_v18(seed)` have `temperament == None`. This is a test-only diff in `9eb221d3`.<br>• **L1/L5:** the notes are in `eval.md`.<br>• **L4:** `component_mismatches` and `born_event_misses` are 0 in all three evaluation scenes and in all three tuning trace sets. I checked the 16-world metas myself.<br>**Not done:**<br>• **M1** is marked "Fixed" and cites `probe/p16-6_final/`, `probe/p16-6_inert120/` and results in DECISIONS. None of these exist, and DECISIONS has no Pass 16 Part 6 entry.<br>• **L3** says the counts are next to "every interval", but the T3 and T4 intervals carry none. |
| 2 | Independent recomputation | **PASS** | Pooled Spearman (scipy average ranks) with the section 9 minimums:<br>• v20: NS 0.3383, HA −0.5780, RD 0.7819, P food 0.2171, P wood 0.3951 (n 4117/4108/4117/4051/3337).<br>• inert = v19: 0.0435, −0.0579, 0.0083, 0.0308, 0.0104.<br>• T1-dep: v20 +115.82 (326 departers / 483 stayers, 25 completed fissions); placebo −15.65 (326/485).<br>• T4 slopes (OLS, integer mid-parent, 3,845 births): NS 0.5002, HA 0.4848, RD 0.4871, P 0.4979, pooled 0.4929. SD changes −1.45 / −0.51 / −1.08 / −4.95 % (777 founders, 2,001 alive).<br>• T3 from the three `summary.json` files (`alive_daily` has 120 entries): alive at the end 62.531 (v20) vs 62.469 (v19) = 62.469 (inert); non-old-age deaths 1.750 vs 1.4375 per world (paired +0.3125); old-age 80.156 vs 80.094; fissions 0.781 for both; first split day +0.278 over 18 worlds.<br>All of these equal `eval.json`. My bootstrap intervals are also identical, but they use the same procedure, so that is only a partial independent check.<br>The committed summaries are byte-identical to the data copies, and all 384 lines of `trace_sha256.txt` verify (`shasum -c`). |
| 3 | One agent's counters | **PASS** | Agent 51, seed 16000, v20 (the most bed nights):<br>• It is born at tick 31141, its parents are [0, 1], and it has 42.0 adult days.<br>• It has 43 bed minutes, all in 1272–1600. 43 nights in 42 days is consistent with an adult span crossing 43 nights.<br>• Food: 180/42 = 4.29. With P 726, the haul threshold at span 3 is 5, so the mean is plausibly below it.<br>• Wood: 30/16 = 1.88.<br>• Idle distance: 9925/1655 = 6.00, at 39.4 samples per adult day (the most possible is 72).<br>• Social: 73, which is 1.74 per day.<br>• Its temperament equals the people file's.<br>• `departed: false`. It is in fission 1's `eligible` list and not among its `members`.<br>Whole-world checks on seed 16000: 0 bed minutes outside 1080..1619, food means 1.57–4.41, wood means 1.0–3.0, idle means 3.0–9.5. Across all 32 worlds, 0 `departed` flags disagree with completed-fission membership, and members ⊆ eligible everywhere.<br>Strongest cross-check: the inert components equal the v19 would-be temperaments and parent temperaments for all 4,608 people. The agents files are identical apart from `temperament_source`. |
| 4 | Verdicts and process | **PASS** | **Bars:**<br>• T1 bars and "excludes 0" are applied correctly. P food 0.217 < 0.25 is a FAIL even though its interval excludes 0.<br>• T2: the worst \|ρ\| is 0.058 and \|dep\| is 15.6, in both inert and v19.<br>• T3: +0.1 % alive and +0.31 deaths ≤ 1.5.<br>• T4: every axis slope is in [0.35, 0.65] and every SD within ±20 %.<br>**T1-P reported as FAIL:** in METRICS, DECISIONS, `eval.md` and the commit subject, but not in TEMPERAMENT.md (M2).<br>**Tuning seeds:** the trace dirs and `summary.seeds` are only 16100–16115.<br>**Code at each run:**<br>• Round 1 ran on `7ee8b86c`; round 2 and the evaluation on `4296c166` (committed 09:39:00).<br>• The evaluation launched at 09:41:17.<br>• After launch, the only Rust change is test-only; the `evaluate_pass16.py` diff only adds reporting, and the verdict logic is unchanged.<br>• The pre-registered table is unchanged since `9a9888fd` (09:27:53); the diff since then only adds lines. |
| 5 | Documents vs code, and with each other | **FINDING** (M2, L2) | `V20_TEMPERAMENT` (`rules.rs` L848–856) is 4 / 60 / 500 / 3 / 2 / 1000. It matches TEMPERAMENT, DECISIONS and METRICS, and the formulas match the method bodies. METRICS' tuning table matches my recomputation (0.397/−0.552/0.789/0.159/−0.010; 0.369/−0.555/0.800/0.331/0.346; alive 46.75/46.56/46.56; deaths 0.50/0.44/0.56). The wrong numbers are listed under L2. |
| 6 | Other | **FINDING** (L3–L6) | See the findings below. |

### High

None.

### Medium

**M1 — The M1 disposition says "Fixed", but the work is not done yet.**
- *Where:* REVIEW_PASS16.md dispositions table (M1 row); METRICS Pass 16 T5 row.
- *Problem:* the disposition says I1–I3 and the 120-day inert check "are rerun on the final Rust commit in Part 6 (results in DECISIONS.md and `probe/p16-6_final/`, `probe/p16-6_inert120/`)". Neither directory exists, and DECISIONS has no Part 6 entry. METRICS already records T5 as "PASS (Part 6 reruns on the final commit)".
- *Also:* the final Rust commit is now `9eb221d3`, because its `rules.rs` test change moves the Rust tree past `4296c166`.
- *Why it matters:* a reader takes the T5 verdict and the round-1 Medium as closed when they are not. This is the same audit-trail gap round 1 raised.
- *Fix:* until the reruns land, mark the disposition as "Planned (Part 6)" and T5 as "pending". Run I1–I3, the inert check and the gate on the last Rust commit (≥ `9eb221d3`), then cite the actual directories.

**M2 — TEMPERAMENT.md presents the P lean as working and omits the evaluation FAIL.**
- *Where:* `docs/revitalize/TEMPERAMENT.md`, the paragraph under "The leans".
- *Problem:* it cites only the tuning-seed ρ (0.331 food, 0.346 wood). The pre-registered evaluation result, food ρ 0.217 [0.147, 0.285], a **FAIL**, is never mentioned. The document was drafted in `38f75d4c`, before the evaluation, and was not updated in `9eb221d3`.
- *Why it matters:* this is the reference document for the model and for the art line. As written, it implies P is validated, which is not an honest report of the result.
- *Fix:* add one sentence giving the evaluation result and the FAIL verdict. Wood ρ 0.395 can be added as reported without a verdict.

### Low

**L1 — The L3 disposition overstates the valid-draw coverage.**
- *Where:* `eval.md` T3 and T4 sections; `eval.json` `t3`/`t4`; `evaluate_pass16.py` `paired()` and `t4_block`.
- *Problem:* the T3 and T4 intervals do not record valid draws. In practice they are always 2000, since the statistics cannot be NaN with 32 or 18 worlds.
- *Fix:* add the counts there, or reword the disposition to "every T1, T2 and departure interval".

**L2 — Two wrong or stale numbers in DECISIONS.**
- *Where:* DECISIONS Pass 16, Part 1 "Per-person values".
- *Problem:*
  - It says `bedtime` wrapping is "never reached at these spans: 1,140–1,380". At span 60 the range is 1,200–1,320 (the round-1 review and TEMPERAMENT agree); 1,140–1,380 would be span 120.
  - The listed tested values "food 2/4/6, wood 1/2/3" are pre-round-2. The tests now assert 1/4/7 and 1/2/4 under `V20_TEMPERAMENT`, with the start spans checked separately.
- *Fix:* correct the range, and add "(round 2: 1/4/7, 1/2/4)".

**L3 — `eval.md` is missing a section 10 item, and the T1-P explanation is partly unsupported.**
- *Where:* `eval.md`; DECISIONS evaluation entry.
- *Problem:*
  - Section 10 asks `eval.md` for "what the tuning rounds changed". That appears only in METRICS and DECISIONS.
  - DECISIONS explains the 60-day vs 120-day gap with "more food comes from winter-thin bushes and the store". My cohort split supports the time-dependence part:

    | cohort | ρ(P, food load) | mean load |
    |---|---|---|
    | founders | 0.334 | 3.12 |
    | born before day 60 | 0.250 | 3.02 |
    | born after day 60 | 0.139 | 2.75 |

  - The "store" part is unsupported. Only `stockpile_deposit.rs` removes carried food, and meals from the store do not pass through the inventory.
- *Fix:* add a tuning paragraph to `eval.md`. Label the explanation a hypothesis and cite the cohort split instead.

**L4 — The placebo HA interval excludes 0.**
- *Where:* `eval.md` T1/T2 table (inert and v19 HA: −0.058 [−0.103, −0.007]).
- *Problem:* the placebo correlation is not zero. It is mostly between-world structure: −0.032 after removing each world's mean and −0.012 as the mean of per-world ρ. T2 still passes, and no verdict is near its bar because of it.
- *Fix:* one note in `eval.md` that pooled ρ carries a small between-world component, with the within-world value as a sensitivity row.

**L5 — The link from the round-2 and evaluation binary to its commit is asserted, not shown.**
- *Where:* DECISIONS tuning round 2 entry; `dgx_probe_p16.sh`.
- *Problem:* round 2 launched at 09:39:24, 24 s after `4296c166` was committed. That is short for push, fetch and a release rebuild. The `git_sha` in the traces is the caller-set `WORLDSIM_GIT_SHA`, not something read from the binary.
- *Evidence it is fine:* the data shows the new spans are in effect (wood ρ 0.395 vs −0.010 at span 1), and inert = v19.
- *Fix:* record the DGX build time, `git rev-parse HEAD` and clean status (or the rsync source) beside the sha256 `0b8707de…`.

**L6 — The evaluator's tests were not extended.**
- *Where:* `tools/revitalize/test_evaluate_pass16.py` (6 tests).
- *Problem:* nothing covers `with_wood=False` or `VALID_DRAWS`, both added after round 1.
- *Fix:* add one assertion for each.

## Round 2 — dispositions

| id | disposition |
|---|---|
| M1 | **Done in Part 6** — the final Rust commit is `9eb221d3` (round 2's L8 test change is the last Rust change). The 120-day inert check was rerun on the DGX at `9eb221d3` (`probe/p16-6_inert120/`) and I1–I3 on the Mac with the `9eb221d3` build (`probe/p16-6_final/`); results and timings in DECISIONS.md (Part 6) and the T5 row of METRICS. The gate (also on the final commit) runs the v19 pins and the 40,320-tick inert test. |
| M2 | **Fixed.** TEMPERAMENT.md states the evaluation result: P fails (food ρ 0.217 [0.147, 0.285]); wood 0.395 reported; the cohort split and a hypothesis labelled as such. |
| L1 | **Fixed** (round-1 L3 disposition reworded). |
| L2 | **Fixed** in DECISIONS (bedtime range 1,200–1,320 at span 60; test values after round 2). |
| L3 | **Fixed.** `eval.md` now has "What the tuning rounds changed" (section 10) and a "T1-P by birth cohort" sensitivity table; DECISIONS labels the explanation a hypothesis and drops the unsupported "store" clause. |
| L4 | **Fixed.** `eval.md` lists the mean per-world ρ as a sensitivity column (placebo HA −0.012) and a note on the between-world component of pooled ρ. |
| L5 | **Recorded.** DGX build logs: round 2's incremental build `build_4296c166.log` finished at 09:39:24 (17 s, `Finished release … in 17.26s`) after `git checkout --detach 4296c166` (fetch at 09:39:0x); the probe batch started the same second. The data confirm the new spans (wood ρ 0.346 vs −0.010). |
| L6 | **Fixed.** Two evaluator tests added (T2 with / without the wood ρ, valid draws, per-world ρ, cohorts); 8 tests OK. |

