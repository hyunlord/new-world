# GOAL: WorldSim — Revitalize Pass 20: villages that help and villages that quarrel (`game_v25`: porters, kin aid and exchange; forage disputes, relations and estrangement)

Repository `hyunlord/new-world`. Continue the simulation line: `codex/revitalize-2026-10` = `5cb16639b2334800aee9b907a7f3b26f0c657aaf` (Pass 19, gate green), `codex/revitalize-2026-10-wip` = `bbba1dbdfcb21966474fb07e16bef7f7f04ae79b`. Draft PR #510.
This run is executed by **Claude Code**, unattended, on the Mac, in `/Users/rexxa/work/new-world-pass4`. **The owner launches it only after Art Pass 11 has finished**, so the Mac is yours; the quiet-lock protocol (1.2) stays in force in case another run starts. No DGX.

Roadmap step 5 (v1.0): villages that relate to each other. Today a world's 1–3 villages are kin (a camp and the daughters its splits founded, 22–28 tiles from the camp) but they never meet: nobody can carry anything to another village, and nothing records how villages feel about each other. They touch in one place: **daughters can reach some of their parent's bushes** (camp bushes lie 6–16 tiles from the camp hearth; forage radius 16) and bush claims are world-wide, so foragers of two villages already compete. Pass 20 adds, in a new scene `game_v25` = `game_v24` + a `RelationsRule`:
1. **Porters** — an adult carries food or wood to another village and comes home: **kin aid** (a well-fed village helps a short one) and **exchange** (food for wood when each has what the other lacks).
2. **Disputes** — foragers of two villages going for one bush dispute it; some disputes become **quarrels**; both lower the **relation** between the villages; a low relation makes them **estranged** (no porters, and each avoids bushes nearer the other's hearth) until they **reconcile**.
3. Everything is written to the chronicle and exposed through the bridge for the art line.
Fights, raids and deaths are not in v1. The performance baseline moves from `game_v20` to `game_v24` (the owner's decision, 2026-10-16).

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # bbba1dbd
cp ~/Downloads/claude_goal_worldsim_pass20.md docs/revitalize/PASS20_GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS20_GOAL.md>
```
The clone's hooks are neutralized as before (`.claude/settings.json` = `{}`, `--skip-worktree`); never edit, stage or commit it and never clear the flag. Leave the local `.gitignore` edit and `.ignore` uncommitted. Stage explicit paths only.

**This document is `docs/revitalize/PASS20_GOAL.md`.** Commit it unchanged in Part 0. Re-read the relevant section at the start of every Part and after every compaction or pause. The owner drives the run with `/goal`: the section 11 report, the T table, commit hashes and `git ls-remote` heads must appear in your final message.

## 0.2 Overrides of CLAUDE.md
As Pass 19's 0.2 (no `install_hooks.sh`, never `lead/main` or `main`, no `harness_pipeline.sh` — 4.4 is the gate, built-in `general-purpose` subagents only, a blocking hook other than `/goal` → record, push, report). First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`. Pass 19's files (`PASS19_GOAL.md`, `REPORT_PASS19.md`, `DECISIONS.md` Pass 19, `tools/revitalize/*_p19*`, `run_watched_p19.sh`) are the reference for every procedure cited here.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions in `docs/revitalize/DECISIONS.md` under `## Pass 20`, with a RUNLOG table.
2. **Do not end your turn between Parts or while a measurement or probe batch runs.**
3. **Parts:** 0 = section 5; 1 = section 6 (rules and mechanics); 2 = section 7 (diagnostic, evaluation); 3 = section 8 (docs, review, gate, report). Push after every Part.
4. **Review loops: two rounds maximum.**
5. **On failure: work around, note it, continue.** A bar missed is reported as missed.
6. **Time rule:** past **12 hours** of working time (pauses excluded and recorded), finish the current step, push, go to Part 3. **Checkpoints:** if Part 1 is not pushed by hour 7, drop exchange and the porter snapshot; run the tuning round (7.3) only if ≥ 3 h remain.
7. **Drop order:** exchange (6.5 b) → the bridge's porter snapshot (6.8) → the tuning round (7.3). Never drop Parts 0 and 3, identity, kin aid, disputes, relations, estrangement, the chronicle entries, the diagnostic (7.1) or T1–T7.
8. **Mac sleep:** if the Mac sleeps (a closed lid stopped other runs for ≈ 2 h on 2026-10-16), record the pause and continue.

---

# ⚠️ 1. MACHINE
## 1.1 The Mac
Only `/Users/rexxa/work/new-world-pass4` and one base worktree `../new-world-p20-base` (5.2). Never touch other checkouts or processes you did not start; no DGX. `CARGO_BUILD_JOBS=6`, tests `--test-threads=6`, probes `--workers 4`. Data in `/Users/rexxa/work/p20-data/`.
## 1.2 Sharing the Mac
As Pass 19's 1.2: quiet lock `/Users/rexxa/work/.mac_quiet.lock` only while measuring time; the process-group watcher (`run_watched_p19.sh` copied to `_p20`, owner string, environment names and data path renamed) stops your non-measuring work while a foreign lock is held.

---

# 2. GIT
- Commits to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose gate (4.4) is green; `git ls-remote` after every push. Messages `revitalize(p20-N): …`.
- **May change:** `rust/**`, `tools/revitalize/**`, `docs/revitalize/**`. **Must not change:** `scripts/**`, `scenes/**`, `assets/**`, `localization/**`, `docs/art/**`, `tools/art_pipeline/**`, `tools/worldmodel/**`, `docs/worldmodel/**`, `rust/crates/sim-test/src/worldmodel/**` (the recorder: no schema change). Pass 19's tools are not edited: copy to `_p20` names (e.g. `evaluate_pass19_chronicle.py` indexes kinds by position and would raise on kinds ≥ 17).
- **New public items allowed:** `RelationsRule`, `V25_RELATIONS` and `SocietyRules::relations: Option<RelationsRule>` (`None` in every existing scene); `WorldRules::game_v25`, `WorldGenConfig::game_v25`, `game_v25_scene` and the probe / perf wiring (3.4); `RelationsSystem` and its registration; the components / types `PorterTrip`, `PorterKind`, `PorterStage`; `RelationState`, `SettlementRelations` and the field `SimResources::settlement_relations`; pub helpers for food days, wood nights and base relation; `ChronicleKind` variants 17–21 (appended; 0–16 unchanged); named `Tallies` fields; one RNG stream `STREAM_RELATIONS`; the bridge getters of 6.8. **No new `TargetKind` variant.** New probe-summary keys appear only when `relations.is_some()`; the bridge's `tallies_to_dict` (`world_node.rs` ≈ L2901) output is unchanged; no change to existing getters' output for `game_v24` and older.

---

# 3. CONTEXT (planner's survey at `5cb16639`, paths under `rust/crates/`; verify each fact)

## 3.1 Villages
- `Settlement` (`sim-core/src/components/settlement.rs` ≈ L92–151): `formation_tile` = the hearth, `member_agents` (std `HashSet`), `stockpile: BTreeMap<ResourceKind, u32>` (`store` ≈ L175, `withdraw` ≈ L188), `parent: Option<SettlementId>`.
- Fission (`rules.rs` ≈ L695–713): `max_settlements` 4; site = the free wild-region centre nearest the parent (`settlement/fission.rs` ≈ L595–606); region centres Chebyshev 22–28 from the camp hearth and ≥ 20 apart (`worldgen/camp.rs` ≈ L312–316, L396–410) — so siblings can be up to ≈ 56 tiles apart. On the Pass 19 seeds (19200–19231, 120 days): 9 worlds with 1 settlement, 13 with 2, 10 with 3. A daughter forms at priority 138 and its `parent` is set at 148 in the same tick (`settlement_system.rs` ≈ L553; `fission.rs` `found` ≈ L288–351); settlements are removed at `settlement_system.rs` ≈ L1198 and `fission.rs` ≈ L433.
- Movement: one Chebyshev step per `move_interval_ticks` 8 (`rules.rs` ≈ L1261), no pathfinding. **Membership is sticky** under the fission rule (`settlement_system.rs` ≈ L268–276).

## 3.2 Food, wood, carrying
- Forage within `forage_radius` 16 of the **own** hearth (`choose_bush` ≈ `agent_decision.rs` L93–122; arm ≈ L1930–1961); claims are world-wide, counted from Seeking agents with a `SeekTarget` (≈ L951–955), cap 2; bush capacity 3–5 (`variety.rs` ≈ L41); `Consuming{GatherFood}` lasts one tick, the pickup is unconditional then Idle (≈ L2611–2647); rows sorted by AgentId (≈ L1199); retarget ≈ L3037–3060.
- `Inventory` cap 10; deposit: `StockpileDepositSystem` (`settlement/stockpile_deposit.rs` ≈ L91–135, priority 141, every 10 ticks) takes **all** carried food (and wood under the firewood rule) from any member within 5 tiles of **their own** hearth.
- Need: `members_need_per_day` returns f64 (`food_economy.rs` ≈ L178–190). Fire: per-hour burn from `night_rate` is 0 outside 18:00–07:00 (`firewood.rs` ≈ L197–204); **`tonight_burn(resources, &rule)`** (≈ L236–239) = the coming night's `winter_burn_per_hour` × 13; per-hour burn 1–4 by season and severity (13–52 a night); winter-start woodpiles median 162 (daughters) / 193 (parents) (`DAUGHTER_DIAGNOSIS.md` (d)). `stock_in_hand` / `wood_in_hand` count members' carried loads (`firewood.rs` ≈ L291–304).
- Founding walkers' pattern to copy: the leader targets itself (`TargetKind::Agent(own id)`) with arms at `agent_decision.rs` ≈ L1438–1458 (Idle), ≈ L2172–2189 (Seeking{Agent}), and the post-pass ≈ L2848–2855; exemptions: tethers and haul-home (`movement.rs` ≈ L337), local food (≈ L723), Cold (`fire.rs` ≈ L143–145). Without a porter Seeking arm the social chase timeout (≈ L2287–2320, 240 ticks) idles the walker; without a post-pass arm the self-target clears the `SeekTarget` (≈ L3001–3016) and the walker freezes (`movement.rs` ≈ L322–333).
- `assigned_builders` is local to `AgentDecisionSystem` (≈ L824); builders are in `ConstructionSite` states. `routine.interrupt` is `pub(in crate::runtime::decision)` (`routine/lifecycle.rs` ≈ L130); a stale `Activity` on an agent no longer in an activity state is ended as interrupted by the next `RoutineTick::new` (`routine/mod.rs` ≈ L90–108).
- Hunger thresholds: decision `HUNGER_THRESHOLD` 50 (≈ L485); meal threshold 30 (`rules.rs` ≈ L1113).

## 3.3 Relations, combat, chronicle
- Agent-pair familiarity/hostility exists (`sim-engine/src/lib.rs` ≈ L362); **no settlement relation exists**; `CombatSystem` cannot start in these scenes (not used in v1).
- Chronicle (`sim-core/src/chronicle.rs`): kinds 0–16 (≈ L18–64), `ALL` (≈ L68–86), `as_str` (≈ L94–114), stability test (≈ L425–434); `runtime/chronicle_log.rs` `record` runs a query (call it after loops); `CHRONICLE.md`.

## 3.4 Wiring a new scene
`rules.rs` (`SocietyRules` ≈ L176–204, `Default` ≈ L628–641, `game_v8` ≈ L1280–1309, after `game_v24` ≈ L941–945); the struct literals in `sim-test/tests/harness_revitalize_pass{6..13}_frozen.rs` (`family: None` pattern); `worldgen/mod.rs` ≈ L142–144; `sim-test/src/scenes.rs` ≈ L254–262; `probe/args.rs` ≈ L69 (fix its stale usage strings ≈ L45, L72 that omit `game_v24`); `probe/runner.rs` ≈ L179, L501, L514; `perf.rs` ≈ L95–137, L693, L719–720, tests ≈ L980–995; `sim-bridge/src/ffi/world_node.rs` `build_game_engine` ≈ L197–199, init comment ≈ L228, test `live_game_is_game_v24` ≈ L2935–2938; systems in `sim-systems/src/lib.rs` ≈ L315 (**free priorities 149+**; 143 is `ShelterPlanningSystem`); `rng.rs` `STREAMS` ≈ L100 and `keyed_below` (`slot` is u32, ≈ L131); `Tallies` (`sim-engine/src/lib.rs` ≈ L67–141).

## 3.5 Performance
`PERF_BUDGET.md`: mean ≤ +15 %, p99.9 ≤ +35 %, mean ≤ 4.0 ms, slope < 1.7; Pass 19's five-repeat procedure (`perf_p19.sh h5`, `evaluate_perf_p19.py`); v24 mean 0.3723 ms, p99.9 0.767 ms. The stress world never splits; `worldsim_perf natural --scene X --seeds A..B --days D` exists (`perf.rs` ≈ L95–104; v19 natural took 11.7–18.2 s per 120-day world).

---

# 4. RULES
## 4.1 Identity
As Pass 19's 4.1 with **I8 = `game_v24`** (copy `identity_p19.sh` → `identity_p20.sh`): probe summaries (`--seeds 0..32`, 120 days) identical except timing between base (`5cb16639`) and candidate for I1–I8; recorder I2; all pins; new pins (5.1); `--chronicle-out` of I7/I8 identical too. Three identity sets: base, Part 1 build, final.
## 4.2 Seeds
`game_v25` tuning and diagnostic **19500–19515** (60 days), evaluation **19400–19431** (120 days, paired with `game_v24`). Check unused as Pass 19's 4.2 (`grep -rnE -- '--seeds +19(4|5)[0-9]{2}' docs/revitalize tools/revitalize`, and `find docs/revitalize -name '*_s19[45][0-9][0-9][._]*'`); shift by +1,000 if used.
## 4.3 Arithmetic and draws
Integers and permille (f64 helpers converted as 6.2 says); no forbidden math; draws only through keyed `STREAM_RELATIONS`; every map iterated in key order (sort locally where the code holds a `HashMap`/`HashSet`).
## 4.4 The gate
`tools/revitalize/gate_pass20.sh` = `gate_pass19.sh`'s 15 steps, `allowed_paths` against `5cb16639`, with the watcher.
## 4.5 Performance baseline (from this pass)
`PERF_BUDGET.md` gains a Pass 20 section: later scenes are judged against **`game_v24`** (the v21 routine's cost is accepted), same bars and procedure, plus a **natural** multi-village measurement (T6).

---

# 5. PART 0 — Pins and base
5.1 `rust/crates/sim-test/tests/harness_revitalize_pass20_frozen.rs`: `game_v24`, seeds 0–4, fingerprint and `state_digest` at 1,440, 40,320, 86,400 ticks (one `#[test]` per seed) and **field-by-field** asserts of `WorldRules::game_v24(s)` (no struct literal, so adding `relations` does not touch this file; put `relations.is_none()` asserts in a separate test added in Part 1). Commit alone before any other Rust change.
5.2 Worktree `../new-world-p20-base` at `5cb16639`; release build; I1–I8 into `docs/revitalize/probe/p20-0_base/`. Push.

---

# 6. PART 1 — `game_v25`

## 6.1 The rule
`RelationsRule` (beside `FamilyRule`; derives as `RoutineRule`; integers):
`decision_minute 480`, `latest_arrival_minute 900`, `porter_max 2` (per sender per day), `aid_giver_days_permille 4000`, `aid_keep_days_permille 3000`, `aid_receiver_days_permille 1500`, `wood_giver_nights 3`, `wood_keep_nights 2`, `wood_receiver_nights 1`, `trade_food_surplus_days_permille 3000`, `trade_food_short_days_permille 2000`, `trade_wood_surplus_nights 4`, `trade_wood_short_nights 2`, `relation_kin_parent 300`, `relation_kin_sibling 150`, `relation_other 0`, `relation_drift_permille 10`, `delivery_gain 40`, `dispute_loss 10`, `quarrel_loss 100`, `estrange_at -300`, `reconcile_at -150`, `porter_hunger_max 30`.
`game_v25(seed)` = `game_v24(seed)` with `society.relations = Some(V25_RELATIONS)`. Wire per 3.4. Live game (`init`, `reset_world`) builds `game_v25`; `reset_world_stress` stays `game_v19`.

## 6.2 Measures (a snapshot for every village at `decision_minute`, integers)
- need_milli = floor(`members_need_per_day` × 1000) as u64; **food days** D_f = stock_food × 1,000,000 / max(1, need_milli) (permille of a day); food keep = ceil(need_milli × `aid_keep_days_permille` / 1,000,000).
- burn_night = max(1, `tonight_burn(resources, &rule)` as u32); **wood nights** D_w = stock_wood / burn_night; wood keep = `wood_keep_nights` × burn_night.
- Pairs: every unordered pair of living settlements; **base** = `relation_kin_parent` if one is the other's `parent`, `relation_kin_sibling` if they share a parent, else `relation_other`. A pair is **in reach** if Chebyshev(hearths) × `move_interval_ticks` ≤ (`latest_arrival_minute` − `decision_minute`) × 3 / 4 (ticks are minutes).

## 6.3 Relations (`SettlementRelations`: `BTreeMap<(SettlementId, SettlementId), RelationState { value: i32, estranged: bool, base: i32 }>`, lower id first)
- `RelationsSystem` (priority **149**, every tick) syncs the map every tick: adds a missing pair at its base value (the daughter's `parent` is set at 148 of the founding tick), removes pairs with a missing id.
- At `decision_minute`, before porters: value += (base − value) × `relation_drift_permille` / 1000 (integer); every change is clamped to [−1,000, 1,000]; estrangement is evaluated **only here, after drift**: `estranged` becomes true at value ≤ `estrange_at`, false at value ≥ `reconcile_at`; each change chronicled (6.7).

## 6.4 Who can carry
Candidates of a sender: members who are adults (no `Child`, no `Elder`), not carrying `Founding` or `PorterTrip`, empty inventory, hunger < `porter_hunger_max`, state Idle or in an `Activity` (not a `ConstructionSite` state); ranked by RD descending, then AgentId. An `Activity` holder is taken by setting the state to Idle and dropping its `SeekTarget`, leaving the `Activity` component — the next `RoutineTick::new` records it ended as interrupted.

## 6.5 Trips (decided at `decision_minute`)
Process ordered pairs (g, r) of non-estranged, in-reach pairs in id order, at most one trip per ordered pair per day; the sender's stock is debited after each departure and `porter_max` counts across all of g's trips that day. For each pair, first **aid**, then **exchange**:
- **a. Kin aid:** food if D_f(g) ≥ `aid_giver_days_permille` and D_f(r) < `aid_receiver_days_permille`, with surplus = stock_food(g) − food keep(g); else wood if D_w(g) ≥ `wood_giver_nights` and D_w(r) < `wood_receiver_nights`, with surplus = stock_wood(g) − wood keep(g). amount = min(10 × remaining porter slots, surplus, 10 × available candidates); porters = ceil(amount / 10); no trip if amount ≤ 0.
- **b. Exchange (both trips or none; uses both ordered pairs' daily slots):** if D_f(g) ≥ `trade_food_surplus_days_permille` and D_w(g) < `trade_wood_short_nights` while D_w(r) ≥ `trade_wood_surplus_nights` and D_f(r) < `trade_food_short_days_permille`: g sends food, r sends wood, each amount computed as in a. with the keep lines of 6.2.
- **The trip:** at departure the amount moves from the sender's stockpile into the porters' inventories; each porter gets `PorterTrip { from, to, kind, amount, stage: Outbound, depart_tick }`, and targets itself (`TargetKind::Agent(own id)`, the founding leader's pattern) while `RelationsSystem` keeps its `SeekTarget` at the receiver's hearth. **Arms (copying founding's):** a `PorterTrip` Idle arm before the routine / forage / builder arms; a `PorterTrip` Seeking{Agent} arm before the generic one (so the social chase timeout does not apply); a `PorterTrip` post-pass arm before the generic Agent arm. **Exemptions while carrying `PorterTrip`:** `StockpileDepositSystem` skips the porter (else it redeposits the load at home within 10 ticks); tethers and haul-home; local food; Cold; fission ranking excludes porters (a porter never also gets `Founding`). Not exempt: meals (a porter eats at home). Within 5 tiles of the receiver's hearth: the load is stored in the receiver's stockpile by `RelationsSystem`, `stage = Return`, target home. At home `RelationsSystem` stores any remaining load into the sender's stockpile (booked as returned) and removes `PorterTrip`. Not arrived by `latest_arrival_minute`, or the receiver dissolved: turn back (a failed trip); sender dissolved: the load is booked as lost. A porter who dies loses the load (booked as lost). `stock_in_hand` / `wood_in_hand` count a porter's load as the sender's (documented, not changed).
- A delivered trip adds `delivery_gain` to the pair's relation (clamped).

## 6.6 Disputes and quarrels
- Before the decision loop, snapshot every member in `Consuming{GatherFood}` (tile = position) or `Seeking{GatherFood}` with a `SeekTarget` (tile = target), with its settlement. At a pickup on tile X, the **rival** = the lowest-AgentId snapshot entry at X from another settlement; skip if either has no settlement or the pair has no relation entry. Each unordered pair of foragers is evaluated **at most once per day**. A dispute costs `dispute_loss`.
- **Quarrel:** `keyed_below(seed, STREAM_RELATIONS, lineage_key(lower AgentId of the two), tick, 0, noise_replicate_for(tick), 1000) < p`, p = clamp((NS_a + NS_b − HA_a − HA_b + 1000) / 4, 0, 500) (bold, novelty-seeking pairs quarrel; cautious ones step back; 250 without temperament). On a quarrel the relation −`quarrel_loss`, the picking forager is set Idle without the pickup, and a per-tick `quarrelled` set makes the rival's pickup that tick a no-op. The `Quarrel` chronicle entry is written after the loop.
- **Estranged villages:** no trips between them; `bush_eligible` / `choose_bush` gain an `avoid: &[(u32, u32)]` list of estranged hearths (built from `SettlementRelations`) and skip bushes whose Chebyshev distance to an avoided hearth is smaller than to the own hearth — at all three call sites: the exit predicate (≈ L1309), the forage arm (≈ L1945) and the retarget (≈ L3046).

## 6.7 Chronicle (appended kinds; 0–16 unchanged)
| # | kind | subject / other | settlement / settlement2 | cause | value |
|---|---|---|---|---|---|
| 17 | `PorterDeparted` | porter / – | sender / receiver | 0 aid food, 1 aid wood, 2 exchange food, 3 exchange wood | amount carried |
| 18 | `PorterDelivered` | porter / – | receiver / sender | as 17; a failed trip: cause = 10 + the departure cause, value 0, written at turn-back | amount delivered |
| 19 | `Quarrel` | forager a / forager b | a's village / b's village | 0 | relation after |
| 20 | `VillagesEstranged` | – | lower id / higher id | 0 | relation |
| 21 | `VillagesReconciled` | – | lower id / higher id | 0 | relation |
One entry per porter. Update `ALL`, `as_str`, the stability test and `CHRONICLE.md`; copy the chronicle evaluator to `_p20` with kind names. Disputes are counted (tallies), not chronicled.

## 6.8 Bridge (pure collectors + `#[func]`s)
`get_settlement_relations()` → packed `{a, b, value, estranged, base}` in key order; `get_porter_snapshot()` → packed `{ids (entity bits), from, to, kind (0 food / 1 wood), amount, stage (0 out / 1 back)}` by AgentId. Unit tests. No GDScript. (Porters appear as `Seeking{Agent(self)}` in existing snapshots until the art line reads this getter.)

## 6.9 Checks of Part 1
I1–I8 identical on the Part 1 build; a `game_v25` determinism twin; unit tests for every rule (measures, reach, aid, exchange both-or-none, keep lines, porter choice, the trip incl. failure and loss paths, the deposit exemption, dispute rival and once-per-day, quarrel draw and no-op, drift, clamp, hysteresis, the avoid filter at three sites, chronicle kinds); an **integration test**: two villages with the relation set to −290 and one forced quarrel → estrangement chronicled, no trips, the avoid filter applies. Push.

---

# 7. PART 2 — Diagnostic and evaluation
7.1 **Diagnostic (mandatory, before the rule table):** on 19500–19515, 60 days, `game_v25`: per world, the 08:00 pair-days meeting the aid and exchange conditions, in-reach pairs, trips, cross-village rival encounters (disputes), quarrels, relation trajectories → `docs/revitalize/probe/p20-2_diag/diag.md`.
7.2 **Rule table** (committed to METRICS.md under `## Pass 20` after 7.1 and before 7.4; `game_v25` vs `game_v24`, seeds 19400–19431, 120 days, paired; Pass 17's definitions and bootstrap; "multi-village world" = a world with ≥ 2 settlements alive at once in `game_v25`; a village's **region** = the nearest wild-region centre to its formation tile, or "camp"):
| # | rule |
|---|---|
| T1 | porters happen: ≥ **0.6** of multi-village worlds have ≥ 1 delivered trip; failed-trip share ≤ **0.1**; deliveries per world, food and wood moved, lost loads reported |
| T2 | aid matters: **T2a** per multi-village world, non-old-age deaths in daughter villages `game_v25` − `game_v24` ≤ 0 (point estimate; interval reported); daughters alive at day 118 summed over the 32 worlds ≥ `game_v24`'s. **T2b** (reported): the same for region-matched villages that received aid, with the match rate |
| T3 | conflict exists but does not rule: ≥ **0.3** of multi-village worlds have ≥ 1 dispute and ≥ **0.15** ≥ 1 quarrel; estranged share of the pairs alive at day 120 (pooled) ≤ **0.5**; estrangements may be 0 in play (reported) |
| T4 | health: Pass 17's R2 rules for `game_v25` − `game_v24`; F1 and F5 within ± 0.05 |
| T5 | identity: I1–I8 and all pins identical on the final build |
| T6 | performance against `game_v24` (4.5): stress five-repeat (mean ≤ +15 %, p99.9 ≤ +35 %, ≤ 4.0 ms, slope < 1.7); **natural**: `worldsim_perf natural` on the 4 lowest evaluation seeds with ≥ 2 settlements alive at day 60 in `game_v25`, 120 days, `game_v25` vs `game_v24`, three interleaved repeats under the quiet lock: mean tick ratio ≤ 1.15, p99.9 ratio ≤ 1.35; ms per agent-tick reported (populations differ) |
| T7 | chronicle and accounting: every departure, delivery, quarrel, estrangement and reconciliation in the tallies has exactly one entry; per trip, withdrawn = delivered + returned + lost; C1–C4 of Pass 19 still pass on `game_v25` |
7.3 **One tuning round** (19500–19515, 60 days) only if 7.1 shows T1 or T3 out of reach: levers `aid_giver_days_permille` 4000 → 3000, `aid_receiver_days_permille` 1500 → 2000, `quarrel_loss` 100 → 150, `relation_drift_permille` 10 → 5; recorded; frozen afterwards; the rule table is committed after it.
7.4 Evaluation (`game_v25` and `game_v24` on 19400–19431, 120 days, `--agent-trace --chronicle-out`) → `docs/revitalize/probe/p20-2_eval/eval.md` + `eval.json`; T6 → `docs/revitalize/perf/p20-2/`. Push.

---

# 8. PART 3 — Docs, review, gate, report
1. `docs/revitalize/RELATIONS.md` (the rules, why each number, what the player sees); `CHRONICLE.md` (kinds 17–21); `PERF_BUDGET.md` (4.5, Pass 20 numbers); `FAMILY.md` link.
2. Independent review (fresh `general-purpose` subagent, two rounds max): identity I1–I8 and pins; that nothing changes a non-v25 scene; trip accounting; the porter arms and exemptions (deposit especially); dispute snapshot and quarrel draw (keyed, ordered, once per pair-day); drift, clamp and hysteresis; no new `TargetKind`; chronicle completeness; T verdicts; quiet-lock record. Findings in `docs/revitalize/REVIEW_PASS20.md`.
3. Gate (4.4) → `docs/revitalize/verification/p20_gate/`; fast-forward `codex/revitalize-2026-10`; PR #510 section; remove `../new-world-p20-base`.
4. `docs/revitalize/REPORT_PASS20.md` in Korean (section 11 format) and the same text in the final message.

---

# 9. Accounting invariants (tests)
- Per trip: withdrawn from the sender = delivered + returned home + lost (death or dissolved sender); a test over the evaluation worlds sums every trip.
- Members never change village through a trip; `PorterTrip` exists only in `game_v25`.
- Relations exist exactly for living pairs; values stay in [−1,000, 1,000].

---

# 10. 인게임 확인사항 (오너용)
- 게임이 `game_v25`로 시작한다. 마을이 둘 이상 생긴 뒤, 짐을 든 사람이 다른 마을까지 걸어갔다가 돌아오는지(모자란 딸 마을에 부모 마을이 먹을 것을 보내는 모습). 짐꾼의 화면 표시는 다음 아트 패스에서 다듬는다.
- 연대기 창(C 키)에 새 항목이 오류 없이 들어오는지. 짐꾼·다툼·사이가 틀어짐/화해의 문구는 다음 아트 패스의 일이다.
- 마을이 많아져도 2배속에서 끊김이 없는지, 콘솔 에러가 없는지.

---

# 11. 구현 완료 보고 (REPORT_PASS20.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
### 구현 내용
### 구현 방법
(관계 규칙과 수치의 이유, 짐꾼 왕복·예외·재고 회계, 원조·교환 조건과 도달 거리, 분쟁 스냅샷과 말다툼 추첨, 이동·고정·히스테리시스, 연대기 종류, 브리지, 성능 기준선 이동과 자연 측정)
### 기능 설명
### 변경된 파일 목록
(커밋별)
### 측정 결과
진단(7.1) 요약, T1~T7 표(값과 구간), 세계당 배달·이동량·실패·분실, 분쟁·말다툼·소원·화해 수, 딸 마을 사망과 생존, 건강 표, 성능(스트레스·자연), 조정 기록
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
마을들이 이제 서로 돕고 다투는가? 원조가 실제로 사람을 살리는가? 갈등이 게임을 지배하지 않는가? 다음(v2: 싸움·습격, 또는 교역 확장)은 무엇인가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #510 update, and the time per Part.
