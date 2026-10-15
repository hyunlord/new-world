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
| M1 | **Fixed.** I1–I3 and the 120-day inert check are rerun on the final Rust commit in Part 6 (results in DECISIONS.md and `probe/p16-6_final/`, `probe/p16-6_inert120/`), and the gate runs the v19 pins and `harness_v20_inert_equals_v19_at_40320_ticks` on it. |
| L1 | **Kept, documented.** The definition is the pre-registered one (members of a settlement, the hearth of the first settlement listing the person); changing it after the evaluation would be a post-hoc metric. `eval.md` (notes) and the report name the confound; it acts only in v20 and only on the few Idle samples of a two-day trek. |
| L2 | **Fixed.** `evaluate_pass16.py` reports T2 both with (primary, stricter) and without the wood ρ; both pass. |
| L3 | **Fixed.** The number of valid bootstrap draws is listed next to every interval (2,000 for every evaluation interval). |
| L4 | **Reported.** `component_mismatches` is 0 in all three evaluation scenes and both tuning rounds (`eval.md`, trace bookkeeping); `born_event_misses` 0. |
| L5 | **Documented** in `eval.md`'s notes (affects every scene alike). |
| L6 | No change (documented in DECISIONS). |
| L7 | **Kept.** Removing the duplicate lookups would change `sim-systems` after the evaluation build; the cost is two BTreeSet lookups per agent per tick only while shelter planning runs, and the performance rule (8.5) measures it as it is. Noted for a later pass. |
| L8 | **Fixed.** `game_v20_is_game_v19_with_temperament` also asserts `game_v19_nofission(s)` and `game_v18(s)` carry no rule (test-only change in `rules.rs`). |
| L9 | No change this pass (the gate's grep is Pass 15's, as 4.5 asks); noted for a later pass. |

