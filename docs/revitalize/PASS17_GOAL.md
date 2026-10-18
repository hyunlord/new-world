# GOAL: WorldSim — Revitalize Pass 17: a day worth watching and families that hold — daily routine v1 (`game_v21`) and family v1 (`game_v22`)

Repository `hyunlord/new-world`. Continue the simulation line: `codex/revitalize-2026-10` = `adfdecfece79ab019964729319389fd602685fab` (Pass 16, gate green), `codex/revitalize-2026-10-wip` = `2c079df265ef7c16baf8ee5bb119591091b5c3eb` (Pass 16's report and gate logs after it). Draft PR #510.
This run is executed by **Codex**, unattended, on the Mac, in the simulation clone `/Users/rexxa/work/new-world-pass4`. An art run (Art Pass 9, Codex) may still be working in `/Users/rexxa/work/new-world-art-prod` and measuring frame times; section 1.2 is how the two runs share the Mac. A world-model run (WM-2) uses the DGX; this pass does not use the DGX.

**Two stages, two scenes, one system each.** Stage A gives people something to do in the daytime (`game_v21` = `game_v20` + routine). Stage B gives them partners, real parents and households (`game_v22` = `game_v21` + family). Stage B starts only after Stage A's verdicts are committed. Every older scene — `game_v19` (the research world) and `game_v20` above all — stays bit-for-bit as it is. The live game moves to `game_v22` at the end (or to `game_v21` if Stage B is dropped).

---

# ⚠️ 0. AUTONOMY — READ FIRST
1. **Never ask the owner anything.** Decide, implement, record the decision and its reason in `docs/revitalize/DECISIONS.md` under `## Pass 17`, with a RUNLOG table (Part, start, end, notes).
2. **First action:** save this document unchanged as `docs/revitalize/PASS17_GOAL.md` (or verify the owner's copy is this document), commit it in Part 0, and re-read the section of the current Part at every Part start and after any context compaction.
3. **Do not stop between Parts.** Parts: **0** = section 5; **1** = 6.1–6.3 and 6.5 (Stage A code); **2** = 6.4 and 6.6 (Stage A tuning, proofs, performance); **3** = 6.7–6.8 (Stage A pre-registration and evaluation); **4** = 7.1–7.5 (Stage B code); **5** = 7.4 e and 7.6 (Stage B tuning, proofs, performance); **6** = 7.7–7.8 (Stage B pre-registration and evaluation); **8** = section 8. Push to the WIP branch after every Part.
4. **Review loops: two rounds maximum** (a fresh reviewer each round that has not seen your reasoning: a separate Codex session or sub-agent given only this document, the diff and the outputs).
5. **On failure: work around, note it, continue.** A rule that misses its pre-registered bar is a valid result reported as a failure; a missing number is not.
6. **Time rule:** past **16 hours** of working time (pauses excluded and recorded), finish the current step, push, go to Part 8.
7. **Drop order:** Stage B's family-cohesive fission (7.4 d) → Stage B's household beds (7.4 c) → Stage B entirely (then the live game moves to `game_v21`) → Stage A's tuning round 2. Never drop Part 0, Stage A, the identity checks, the performance rule or Part 8.
8. **This brief supersedes the repository's session-start instructions** (`CLAUDE.md`, `AGENTS.md`): do not install hooks, do not run `harness_pipeline.sh`, never push to `lead/main` or `main`, never force-push. The clone's `.claude/settings.json` is neutralized with `--skip-worktree`: never edit, stage or commit it. Leave the local `.gitignore` edit and `.ignore` uncommitted. Stage explicit paths only.

---

# ⚠️ 1. MACHINE

## 1.1 The Mac
- Only `/Users/rexxa/work/new-world-pass4` (and one base worktree `../new-world-p17-base`, 5.3). Never touch `/Users/rexxa/work/new-world-art-prod`, `/Users/rexxa/work/new-world-wm1`, `~/github/new-world-wt/lead` or any other checkout; never stop a process you did not start; no DGX, no ComfyUI, no dev servers.
- `export CARGO_BUILD_JOBS=6`; cargo tests `-- --test-threads=6`. Probes `--workers 4`. Data outside the repository in `/Users/rexxa/work/p17-data/`. `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.

## 1.2 Sharing the Mac with the art run (both briefs carry this protocol)
- **Quiet lock:** the directory `/Users/rexxa/work/.mac_quiet.lock` (`mkdir` is the atomic acquire); write `owner`, `pid`, `purpose`, `started` into `info.txt` inside it; remove it when done. Hold it **only while measuring time** (the performance rule, 6.6 and 7.6), at most 60 minutes per hold, at least 10 minutes between two of your holds.
- **While the other run holds it:** start no cargo build or test, no probe, no Godot, no gate; poll every 5 minutes. A lock older than 2 hours is stale: remove it, record it, continue.
- **Before taking it:** wait until no `cargo`, `rustc`, `worldsim_*` or `Godot` process of the other run runs (`pgrep -x`) and the 1-minute load average (`sysctl -n vm.loadavg`) is < 4; after 60 minutes of waiting, take it anyway and mark the runs `contended`.
- Record every hold and wait in the RUNLOG.

---

# 2. GIT
- Commits to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose gate (4.5) is green. Never force-push. `git remote -v && git branch --show-current` before every push; `git ls-remote` after.
- Commit messages `revitalize(p17-N): …` (N = Part).
- **May change:** `rust/**`, `tools/revitalize/**`, `docs/revitalize/**`. **Must not change:** `scripts/**`, `scenes/**`, `assets/**`, `localization/**`, `docs/art/**`, `tools/art_pipeline/**`, `tools/worldmodel/**`, `docs/worldmodel/**`, and in `rust/crates/sim-test/src/worldmodel/**` anything at all (the recorder and its `schema.rs` legend strings: changing a legend changes every episode's `meta.json`, including `game_v19`'s).
- **New fields in existing public structs: only the two rules fields of 5.2.** New components, new getters, new types are fine. **Do not add fields to `AgentSnapshotRow`, `AgentDetailRow` or their dictionaries** (`get_agent_detail` must keep exactly 9 keys, `harness_p14_gamma_click_inspector.rs` L478–506): add separate getters (6.5, 7.5). Before Part 1, `git grep` the art branch (`origin/codex/art-production-2026-10-wip`, read-only) for exhaustive Rust `match`es on `TargetKind` / `AgentState` and for struct literals of anything you change, and record the result — the art line merges this pass later.

---

# 3. CONTEXT (planner's review with two code surveys at `2c079df2`; verify each fact before relying on it)

## 3.1 Why people barely move
In the live seed of `game_v19` (60 days, hourly samples) the share of people **Idle** is 39 % at 08:00, 63 % at 10:00, 71 % at 12:00 and 77–79 % from 14:00 to 20:00; 84–97 % sleep from 22:00 to 06:00. Over 32 worlds the noon idle share has median 0.79 (`rhythm.noon_idle_share`). An idle person takes a keyed random step every 8 ticks with a 0.4 pause chance, tethered to the hearth (`movement.rs` L332–375): about 2 tiles of net movement per game hour. The owner, watching the game, saw "people barely move".
- **Why they are idle:** a member with no need, no assignment, no lonely co-located peer and stores at target falls out of every arm of the idle cascade (`agent_decision.rs`, `Idle` arm L1131; meal wait L1141–1158; forage needs `plan.in_hand < plan.target` L1576; firewood the same with `wood_plans` L1601; gather bounded by `hoard_cap_factor` 1.5 × target L1631–1636; settlement arm only for non-members L1673). `worker = adult && !elder` (L1086). Children only serve needs and walk (tether 3); elders are not workers (tether 4).
- **What exists to host activities:** buildings `Hearth`, `Shelter`, `Stockpile` (`building_kind.rs` L6–12; camp layout `camp.rs` L84–89). The fire burns stockpile wood by itself (`fire.rs`, no agent tends it). Construction uses no materials. Nothing is crafted at runtime. There is no "workspot" concept.
- **Mechanics an activity can reuse:** `Seeking{target}` walks one diagonal signum step per move tick to a `SeekTarget` (`movement.rs` L271–281; without a SeekTarget it freezes); Seeking → Consuming happens on the `has_resource` edge (`agent_decision.rs` L1947–2021); food/water/gather consumption finishes in one tick; there is **no "stay N minutes" timer** — a new component is needed. Need decay does **not** depend on state (`hunger_decay.rs` L54–72 etc.), so an activity costs no extra food. Meals are served only within `camp_radius` 8 of the hearth (`stockpile_consume.rs` L102–104); the meal wait runs only from Idle (L1141). The builder pool (L775–777) and the social and combat peer maps (L903, L962, L986) consider only `Idle` agents.
- **Exhaustive matches on `TargetKind` / `AgentState`** (a new variant must be handled): `agent_decision.rs` L1777–1784, L1897–1946, L2039–2200, and the post-pass L2241–2410 (its wildcard clears the SeekTarget — a new Seeking target without its own arm freezes); `world_node.rs` `state_tag` L1512–1517 and L1846–1853, `target_kind` L1547–1556 and L1854–1865 (round-trip test L3261–3263); probe `fingerprint.rs` L18–27, `deaths.rs` L37–50, `pass12.rs` L62–78. The recorder copies `state_tag` / `target_kind` from the bridge's snapshot (`export.rs` L233–238), so a new code never appears in a `game_v19` recording as long as only new rules construct it.
- **The art branch** (`46193a03`) maps `state_tag` 3 with an unknown target to "eat" (`agent_activity.gd` L27–37) and an unknown `state_tag` to "idle"; the inspector clamps unknown values to its last label. Hence 6.5 gives activities their own `state_tag` (4), which the art branch shows as idle until the art line draws them.
- **Performance:** `AgentDecisionSystem` is 54–69 % of every tick (`docs/revitalize/PERF_BUDGET.md`); the 06:01 wake-up spike sets p99.9. Many people choosing activities on the same clock edge would add to it — stagger them.

## 3.2 Families today
- **Births** (`settlement_system.rs` `run_births` L710–958): per settlement, a cooldown of 720 ticks (half a day) and gates on housing, stock and mean needs; the rate never reads the parents. "Parents" are the **two lowest-AgentId members**, with no adult, elder or proximity filter (L806–814). The newborn spawns at the settlement's formation tile (L794–798); its lineage key comes from `parent_ids[0]` (`Lineage::child`, which bumps that parent's `births`), and lifespan, hardiness, the fresh temperament draw and the fission rank key all follow from that key; its temperament inherits from both (L837–850, L899–908). `AgentBorn { parent_ids }` is recorded only in the causal log (`event.rs` L513–528).
- **No sex, no partner, no household, no stored parent ids** anywhere. Life: child 4 days (`maturation.rs`), elder from 46 days, lifespan ≈ 50–66 days (`aging.rs`, `rules.rs` L1125–1134).
- **Relationships:** `resources.relationships: HashMap<RelationshipKey, RelationshipState { familiarity, hostility }>` (`sim-engine/src/lib.rs` L362; key a canonical (min, max) pair, `relationship.rs` L34–46); each completed interaction adds `FAMILIARITY_BUMP` 0.1 (saturating at 1.0 after 10); never decays; purged on death (`survival/mod.rs` L73–78). Iterate it in sorted key order only (it is a `HashMap`).
- **Beds:** `BedAssignmentSystem` (`beds.rs`, priority 145) recomputes at 20:00 or when shelters change: members sorted by AgentId, member i → shelter i / `residents_per_shelter` (8) (L52–73) → `resources.bed_of` (`sim-engine/src/lib.rs` L240).
- **Fission group** (`fission.rs` L550–600): non-elders ranked (v20: by `departure_key`, `rules.rs` L154), size `clamp(round(members × 0.35), 6, 14)`, first `min_adults` 4 adults, then by rank; families are not kept together.
- **Probe:** `--agent-trace` (`sim-test/src/probe/agent_trace.rs`) writes `_people.jsonl`, `_agents.jsonl`, `_fissions.jsonl`; it recovers parents from the `AgentBorn` event in the tile ring (8 events per tile) and counts misses.

## 3.3 The design (why this shape)
- **Daily routine:** every non-sleeping, need-satisfied person who would stay Idle in the daytime takes up one of a few **visible, economy-neutral activities near home** for a while, then rests a little, then picks again. Neutral on purpose: this stage must make the village *look* alive without moving the food and wood balance (a later material/crafting pass can give the activities products). The four temperament axes shape which activities a person prefers and how long they keep at them — **persistence finally gets a lever with a footprint (how long one sticks to a task)**, replacing Pass 16's weak haul lever for the evaluation of P.
- **Family:** adults pair with someone they already know (familiarity from completed interactions), births go to a couple instead of the two lowest ids, children know their mother and father, couples and their young children sleep in the same shelter, and a splitting village takes whole households. Birth **timing and rate stay exactly the existing rule**: only who the parents are changes, so the population dynamics stay comparable.

---

# 4. RULES

## 4.1 Nothing old changes
`game_v8` … `game_v20` (with `_nofission` / `_neutral` / `_inert` variants), the recorder and the stress world behave bit-for-bit as before: proved by the identity checks I1–I4 on the base build (5.3) and the final build — I1–I3 as Pass 16 (`game_v19` and `game_v16` probe summaries `--seeds 0..32`, 120 days, identical except `timing`; recorder `game_v19` `--seeds 12000..12002` array sha256 identical), **I4:** `game_v20` probe summary `--seeds 0..32`, 120 days, identical except `timing` — plus Pass 16's pins (`harness_revitalize_pass16_frozen.rs`) and a new pin file for `game_v20` (5.1).

## 4.2 Seeds
Stage A: tuning **17100–17115**, evaluation **17000–17031**. Stage B: tuning **18100–18115**, evaluation **18000–18031**. Not used before; not in tests (tests use seeds < 1000). The live game keeps `DEFAULT_GAME_SEED` 20,261,010.

## 4.3 Arithmetic
Integers and plain `f64` multiplication/division of integers; no forbidden math (`exp`, `ln`, `pow*`, trig, …; the gate greps for them). Every per-person value is a pure function in `sim-core`, unit-tested at the extremes. Keyed draws follow the identity-trait convention (subject = lineage key; replicate 0 for traits drawn once; tick and slot for daily choices) with new stream constants added to `STREAMS` (`rng.rs`; its distinctness test covers them).

## 4.4 Pre-registration
Each stage's rule table (6.7, 7.7) is committed to METRICS.md under `## Pass 17` **before any evaluation-seed probe of that stage runs**. Tuning uses the tuning seeds only; at most one tuning round per stage after the first (6.4, 7.4 e).

## 4.5 The gate
`tools/revitalize/gate_pass17.sh <log_dir>` = `gate_pass16.sh`'s steps unchanged (debug and release `cargo test --workspace`, both clippy runs, `forbidden_math`, bridge build, Godot import, strict check, smoke, both Python discoveries with `WM_BIN=/nonexistent PROBE_BIN=/nonexistent`, `perf_quick`), with `allowed_paths` checking section 2's list against `adfdecfe`. Under 1.2's rules.

---

# 5. PART 0 — Pins, rules fields, base identity
## 5.1 Pin `game_v20` first
`rust/crates/sim-test/tests/harness_revitalize_pass17_frozen.rs` following Pass 16's pin file: for `game_v20` and `game_v20_inert`, seeds 0–4, the probe fingerprint and `state_digest` after 1,440 and 40,320 ticks from the **unchanged** code, and `WorldRules::game_v20(s)` field by field with individual `assert_eq!`s (never a struct literal). Commit it alone before any other Rust change.
## 5.2 The two rules fields
`AgentRules` gains `pub routine: Option<RoutineRule>`; `SocietyRules` gains `pub family: Option<FamilyRule>`; both `None` in `WorldRules::default` and `game_v8` (hence in every existing scene). Update every whole-struct literal (`rules.rs` ≈ L607/623, L1182/1201, L1541/1560 and any other `AgentRules {` / `SocietyRules {` — grep) and the frozen tests that write them out (`harness_revitalize_pass{6,…,13}_frozen.rs`, one line each, as Passes 13 and 16 did).
## 5.3 Base identity
Worktree `../new-world-p17-base` detached at `adfdecfe`; release build; I1–I4 into `docs/revitalize/probe/p17-0_base/` (summaries committed; recorder outputs in `p17-data`, their sha256 list committed). Push.

---

# 6. STAGE A — Daily routine v1 (`game_v21`)

## 6.1 Activities (new types in `sim-core`)
`pub enum ActivityKind { TendFire, Craft, Repair, HearthCircle, Explore, Play, SitByFire }`, a new `TargetKind::Activity` variant, a component `Activity { kind: ActivityKind, settlement: SettlementId, place: (u32, u32), chosen_tick: u64, arrived_tick: Option<u64>, until_tick: Option<u64> }` present only while a person walks to or does an activity, and `ActivityRest { until_tick: u64 }` for the pause after one ends or fails.

| kind | who | candidate place tiles (walkable, not a building, ≤ 8 from the person's settlement hearth unless noted) |
|---|---|---|
| `TendFire` | workers | Chebyshev distance 1 from the hearth |
| `Craft` | workers | distance 1 from the settlement's stockpile (none in a daughter camp, which has no stockpile → `Craft` has no place there) |
| `Repair` | workers | distance 1 from the settlement's shelters |
| `HearthCircle` | non-child adults (workers and elders) | distance 2 from the hearth |
| `Explore` | workers | distance 10–14 from the hearth (the only kind outside the meal radius) |
| `Play` | children | distance 2–3 from the hearth |
| `SitByFire` | elders | distance 1–2 from the hearth |

**Places and occupancy:** a place tile holds at most **2** people with that tile as their activity place. Build the occupancy map **once per tick** before the decision loop (from existing `Activity` components, in AgentId order). A person's place = the candidate tile with the fewest occupants (< 2), ties by a keyed rank `keyed_u64(seed, STREAM_ROUTINE_PLACE, lineage key, tick, 0, rep)` then `(y, x)`; occupancy is updated as each person (in AgentId order) takes a place within the tick. If the drawn kind has no place, the person **redraws once** among kinds that have a place; if none has, the person gets `ActivityRest` for 30 minutes (no per-tick retries).
**Determinism:** every insert or removal of `Activity` / `ActivityRest` is applied in AgentId order from a sorted `Vec` (never a `HashSet` / `HashMap` iteration — the a15 archetype-order lesson, `agent_decision.rs` ≈ L2480–2501). Stale `Activity` components are removed at the top of each decision tick (6.2).

## 6.2 When, what, how long (pure functions next to `RoutineRule`)
`RoutineRule { day_start_minute: 420, day_end_minute: 1080, base_duration_minutes: 90, duration_span_minutes: 60, min_duration_minutes: 20, walk_timeout_minutes: 150, rest_minutes_min: 20, rest_minutes_max: 60, adult_weights: [u16; 5], elder_weights: [u16; 2] }` with `game_v21` values `adult_weights` (TendFire, Craft, Repair, HearthCircle, Explore) = `[180, 180, 140, 200, 120]`, `elder_weights` (HearthCircle, SitByFire) = `[500, 500]`; children always `Play`.
- **Entry point:** in `AgentDecisionSystem`, a new arm **after Gather and before the settlement `else`** (≈ L1652–1655): a member who reached this point Idle, between its `day_start_minute + (lineage key mod 60)` (staggered start) and `day_end_minute` (clock with `clock_offset_ticks`), **not carrying a haul load** (`no_trips` / `loaded` false — a loaded worker must deposit first), with no `ActivityRest` in force, starts an activity: it chooses the kind and place (6.1), inserts `Activity`, and becomes `Seeking{Activity}` with a `SeekTarget` at the place. Every arm before it is untouched.
- **Choice:** a keyed draw `keyed_below(seed, STREAM_ROUTINE, lineage key, tick, 0, rep, total)` over the weights adjusted by temperament (d_x = x − 500, `i32`): `TendFire` × (1000 + d_ha)/1000, `Craft` and `Repair` × (1000 + d_p)/1000, `HearthCircle` × (1000 + d_rd)/1000, `Explore` × max(0, 1000 + 2·d_ns)/1000.
- **Duration** (counted from **arrival**): `max(min_duration, base + span × d_p / 500)` minutes — 30 … 150 at the defaults; persistent people keep at it longer. On arrival (`Seeking{Activity}` → `Consuming{Activity}` on the `has_resource` edge, which must treat "standing on the place tile" as having the resource) set `arrived_tick` and `until_tick = arrived_tick + duration`. A walker that has not arrived `walk_timeout_minutes` after `chosen_tick` gives up (rest follows).
- **Rest after:** when an activity ends for any reason, `ActivityRest { until_tick: now + r }` with r a keyed draw in [rest_min, rest_max] minutes.
- **Leaving an activity — implementation:** the decision loop is a `match *state { Idle => …, Seeking{..} => …, Consuming{..} => … }` (≈ L1115); a `Seeking`/`Consuming` arm cannot fall through into the Idle arm, and copying the Idle arm's social or work code would duplicate the bias-flip and causal-event blocks (≈ L1309–1566). So: **before `match *state`**, for a person in `Seeking{Activity}` or `Consuming{Activity}`, test exits (1)–(5) below; on a hit set `*state = AgentState::Idle`, queue the `Activity` removal (AgentId-ordered), and let the **unchanged Idle arm run in that same iteration** — it handles the need, work, builder, social (filling `social_seek_entities` and `social_chase_since`, emitting its events) and meal-wait responses itself. Exits (6)–(7) stay inside the activity arms. The exits:
  (1) a need breach by the decision system's own thresholds — hunger > 50, thirst > 50, fatigue above the person's own threshold (`rest_for`) — with the Idle arm's need-availability filter;
  (2) **work:** one of the Idle arm's forage, firewood or gather arms would fire for this person now — its **full** predicate, including `worker`, `no_trips`, the clock window, the store/hoard condition **and** the tile lookup (`choose_bush`, `choose_spot`, `nearest_gather_tile` returning a tile under the current claims) — not the store condition alone (most people are idle today exactly because the hoard cap allows work but no tile is near);
  (3) the person is in `assigned_builders`;
  (4) the person is social-eligible and a social-eligible peer is on its tile;
  (5) the meal-wait condition (≈ L1141) within the camp radius;
  (6) `until_tick` or `day_end_minute` reached;
  (7) its `settlement` differs from the person's current settlement.
  Exits (6)–(7) set `ActivityRest`; exits (1)–(5) set none, **but** if the Idle arm then reaches the activity arm again in the same iteration (no response fired), it sets a 1-tick `ActivityRest` instead of starting a new activity (no per-tick churn).
- **Stale activities:** at the **top of the decision tick, before the occupancy map is built**, remove any `Activity` whose holder is not in `Seeking{Activity}` / `Consuming{Activity}` (e.g. a combat defender forced Idle by `CombatSystem`, priority 137, ≈ L253–254), in AgentId order.
- **Places:** "the settlement hearth" = the settlement's `formation_tile` (anchored to the hearth, `settlement_system.rs` ≈ L419–425). Draw slots: kind choice slot 0, the no-place redraw slot 1, the rest duration slot 2, the place rank `STREAM_ROUTINE_PLACE` slot 0. Elders' `HearthCircle` weight is also scaled by (1000 + d_rd)/1000. Count no-place redraws in the trace (crowding can push people into `Explore`; report it beside R3-NS).
- **Being found by others:** under `routine.is_some()` a person in an activity is a candidate in the builder pool (L775–777) and the social and combat peer maps (L903, L962, L986) exactly as if Idle; being chosen by someone else ends its activity and it follows the Idle arm's response that tick.
- **Play:** a playing child stays `Consuming{Activity}` but steps every move tick: add a case in `movement.rs` (which freezes every `Consuming`, L268) — a keyed step `keyed_step(seed, STREAM_ROUTINE_PLAY, key, tick, …)` kept within Chebyshev 1 of its place.
- **Economy:** activities consume and produce nothing. Count `activities_started[kind]` and `activity_minutes[kind]` in the probe trace only — **no field in `Tallies`** (struct literals of `Tallies` exist in `world_node.rs` ≈ L3003 and `harness_revitalize_tallies.rs` ≈ L54/127/153).
- Handle `TargetKind::Activity` explicitly in every exhaustive match of 3.1 (the post-pass needs its own arm that keeps the SeekTarget, or the walker freezes).

## 6.3 Scenes and wiring
`WorldRules::game_v21(seed)` = `game_v20(seed)` with `agent.routine = Some(V21_ROUTINE)`; `WorldGenConfig::game_v21` = `game_v20`'s. Wire into `scenes.rs` and the probe's scene lists (flip any assertion that `game_v21` is unknown; add it to `twin_checked`), not into the recorder. `worldsim_perf stress --rules` gains `game_v21`.

## 6.4 Tuning (seeds 17100–17115, 60 days, `--agent-trace`)
Evaluate R1–R3 on the tuning seeds. One round 2 is allowed: for R1 too idle, shorten the rest window (20–60 → 10–30) or lengthen `base_duration_minutes` (90 → 120); for R1 too busy, the reverse (the kind weights do not change the start probability, only the mix); for an R3 miss, raise that axis's weight factor or `duration_span_minutes` 60 → 90 (the 20-minute floor holds). Record both rounds; freeze.

## 6.5 Bridge
- `state_tag` **4 = "activity"** for `Consuming{Activity}` and `Seeking{Activity}` in `collect_agent_snapshot` and `collect_agent_detail` (the art branch shows unknown state 4 as idle until the art line draws activities); `target_kind` **8** for `Activity`. Update the round-trip test. These codes never appear in `game_v20` or older.
- New `#[func] get_activity_snapshot() -> VarDictionary` → packed arrays `{ids, kinds, place_x, place_y, until_tick}` over people with an `Activity` (pure collector + unit tests), for the art line.

## 6.6 Proofs and performance (Stage A)
Final identity I1–I4 against the base; the v20 and v19 pins pass. Performance (quiet lock), as Pass 16 measured it: three interleaved repeats each of `worldsim_perf stress --rules game_v21` and `--rules game_v20` (seed 15100, n = 500, 1 day, fed); compare **medians** of the three: fails if the mean tick rises by more than **15 %** or exceeds 4.0 ms, or the p99.9 rises by more than **35 %** (single-run p99.9 is noisy: Pass 16 saw 0.941 vs 0.573 ms between repeats); `quick` before/after; `AgentDecisionSystem`'s three-size stress slope (`--seeds 15100..15101 --agents 100,500,1000 --days 2 --feed fed`) vs 1.06 (fails ≥ 1.7).

## 6.7 Stage A rule table (pre-registered before 6.8)
Over the 32 evaluation worlds, 120 days, `game_v21` vs `game_v20` paired by seed; ρ = Spearman (average ranks) over non-elder adults with ≥ 10 adult days (people with no activity minute count as 0 share and are included; for R3-P people with ≥ 3 completed activities); 95 % world-level bootstrap (B = 2,000, `default_rng(0)`). All bars apply to **point estimates**; intervals are reported (and must exclude 0 where stated).
| # | question | rule |
|---|---|---|
| R1 | the day is alive | share of non-elder adults **Idle** (state_tag 0) at the hourly samples 08:00–17:00, pooled: v21 ≤ **0.45** and ≥ **0.10** (planner's estimate 0.20–0.30; v20 ≈ 0.65 reported beside it) |
| R2 | nothing else broke | paired v21 − v20: mean alive at day 120 within **±10 %**; non-old-age deaths per world ≤ **+1.5**; shelters completed per world ≥ **0.9 ×** v20's; social interactions per non-elder adult day ≥ **0.9 ×** v20's; mean food and wood in store (daily 12:00 samples) within **±15 %** |
| R3-P | persistence shows | ρ(P, mean minutes spent in completed activities) ≥ **0.25**, interval excludes 0 |
| R3-NS | explorers explore | ρ(NS, share of activity minutes in `Explore`) ≥ **0.25**, interval excludes 0 |
| R3-RD | the sociable gather | ρ(RD, share of activity minutes in `HearthCircle`) ≥ **0.20**, interval excludes 0 |
| R4 | old worlds untouched | I1–I4 and the pins identical |
| R5 | budget | 6.6 passes |
"Activity minutes" = ticks in `Consuming{Activity}` (not walking); a **completed** activity (R3-P) is one that ended by exit (6) after arrival.

## 6.8 Evaluation
`game_v21` and `game_v20` on 17000–17031, 120 days, `--agent-trace` (extend the trace: per person, activity minutes by kind, activity count, mean duration; per hour, state counts by life stage). `tools/revitalize/evaluate_pass17.py` → `docs/revitalize/probe/p17-a_eval/eval.md` + `eval.json`. Commit the verdicts before Stage B starts.

---

# 7. STAGE B — Family v1 (`game_v22`)

## 7.1 Components (new, `sim-core`)
- `Sex { female: bool }`: drawn once per person, `keyed_below(seed, STREAM_SEX, lineage key, 0, 0, 0, 2) == 1`, inserted for founders and newborns **right next to** `Hardiness` and `Temperament` (same insertion point and order — archetype order matters).
- `Kin { mother: Option<AgentId>, father: Option<AgentId>, partner: Option<AgentId>, last_birth_tick: Option<u64> }` inserted at the same point (founders: no parents). Fallback births (7.3) set `mother`/`father` to the two ids the old rule names, marked in the trace as fallback.

## 7.2 Pairing (`FamilyRule { min_familiarity_permille: 300, pair_minute: 1200 }`)
A new `PairingSystem` registered in every scene, a no-op unless `family.is_some()` (no draws, no writes), at **priority 144, registered after `MaturationSystem`** (stable sort: it runs after maturation and before `BedAssignmentSystem` at 145). At 20:00 (clock with offset), first clear partners who died (`partner` of a living person pointing to a dead or missing agent → `None`), then per settlement in id order: candidates are unpaired, non-child, non-elder members, never close kin of each other (a person's mother or father, their children, anyone sharing a mother or a father with them, from `Kin`); for each unpaired female in AgentId order, the partner is the unpaired male candidate with the highest familiarity to her (`relationships` read in sorted key order), ties to the lower AgentId, provided familiarity ≥ 0.300 (three completed interactions); otherwise she stays unpaired. Pairs are mutual and last until a partner dies.

## 7.3 Births with real parents
In `run_births`, **when the existing gates allow a birth (unchanged timing and rate)**, the parents are: among the settlement's couples whose female is a non-elder adult, the one whose `last_birth_tick` is oldest (never = oldest), ties to the lower mother AgentId; mother = the female, father = her partner. The child's lineage key comes from the **mother** (`Lineage::child` on her), its temperament from both (`Temperament::child`), `Kin { mother, father }`, the mother's `last_birth_tick` updated, `AgentBorn { parent_ids: [mother, father] }`. A couple qualifies only if **both partners are alive** at the birth tick (`run_births` runs every tick; dead partners are cleared only at 20:00). With no qualifying couple, the old rule (two lowest ids) applies and the birth is a **fallback**. Under `family == None` the old code path is verbatim. Check that nothing else reads `parent_ids[0]` with a meaning that changes (grep; record).

## 7.4 Households
A **household** = a couple plus their Child-stage children (mother's household when the father is dead); a person in no couple and not a child of one is a household of one.
c. **Beds:** in `BedAssignmentSystem`, under `family.is_some()`: households in order of their smallest AgentId; **first-fit packing** — a household starts in the current shelter only if it fits in the remaining slots of `residents_per_shelter`, otherwise in the next shelter; households larger than a shelter fill shelters in order; members beyond total capacity get no bed as today. Couples thus share a shelter unless capacity runs out. (`housing_capacity` = shelters × `residents_per_shelter`, independent of `bed_of`, so empty first-fit slots do not block births; people without a bed still sleep on the nearest sleep tile — report how many.)
d. **Fission:** under `family.is_some()`, the group is built household by household: first the seed — households of the best-ranked adults, in rank order, until the group holds at least `min_adults` adults (a seed household may take the group above k, up to `group_max`; a household that would exceed `group_max` is skipped) — then households in rank order of their best-ranked member, each added only if the whole household fits in the remaining size k; skipped households stay. If the result is below `group_min` after all households were tried, top up with individuals in rank order (as today) and count those splits. Elders are excluded as today. Per fission, the number of couples in the parent settlement at departure and the number split go to the **trace** (not to `FissionRecord`).
e. **Tuning** (seeds 18100–18115, 60 days): if F1 misses, lower `min_familiarity_permille` 300 → 200 once; record.

## 7.5 Bridge
`#[func] get_agent_family(entity_bits: i64) -> VarDictionary` → `{found, female, mother, father, partner, children}` (ids as ints, −1 when none; `children` a packed array of living children's ids), pure collector + unit tests. `WorldSimNode::init` and `reset_world` build `game_v22` (or `game_v21` if Stage B is dropped); `reset_world_stress` stays on `game_v19`.

## 7.6 Proofs and performance (Stage B)
I1–I4 again, plus **I5:** `game_v21` probe summary `--seeds 0..32`, 120 days, identical between the Stage A final commit and the Stage B final commit. Performance: `--rules game_v22` vs `game_v21` as 6.6 (same procedure and thresholds).
Wiring: `game_v22` into `scenes.rs`, the probe's scene lists and `twin_checked`, `perf.rs`'s `--rules`, and flip any assertion that it is unknown (`probe/args.rs` ≈ L156).

## 7.7 Stage B rule table (pre-registered before 7.8)
Over 18000–18031, 120 days, `game_v22` vs `game_v21` paired by seed; point-estimate bars, intervals reported.
| # | question | rule |
|---|---|---|
| F1 | people pair | share of non-elder adults with a partner at day 60 (pooled) ≥ **0.50** |
| F2 | real parents | share of births **from day 20 on** with a couple as parents ≥ **0.90** (all births and fallbacks by day reported; early births precede the first pairs) |
| F3 | siblings resemble | (mean over the four axes of the Pearson correlation of temperament between full siblings) minus (the same between unrelated same-settlement pairs) ≥ **0.06**, interval excludes 0 — pairs entered in both orders; expected ≈ 0.125 from the inheritance formula (siblings share only half the mid-parent value; each child has its own fresh draw) |
| F4 | households sleep together | share of couple-nights (20:00 samples, both alive, both with a bed) where both `bed_of` tiles are the same shelter ≥ **0.90** |
| F5 | splits keep families | couples split by completed fissions ÷ couples in the parent settlements at those departures ≤ **0.10** |
| F6 | nothing else broke | R2's rules applied to `game_v22` − `game_v21` |
| F7 | old worlds untouched | I1–I5 and the pins identical |
| F8 | budget | 7.6 passes |

## 7.8 Evaluation
`game_v22` and `game_v21` on 18000–18031, 120 days, `--agent-trace` (extend: per person sex, partner and pairing tick, mother, father, household key, shelter per night; per fission households kept or split). `evaluate_pass17.py --stage b` → `docs/revitalize/probe/p17-b_eval/eval.md` + `eval.json`.

---

# 8. PART 8 — Docs, review, gate, report
1. `docs/revitalize/ROUTINE.md` and `docs/revitalize/FAMILY.md` (English): the mechanics with formulas and final values, what is not modelled yet (activities produce nothing; no courtship, divorce or remarriage rules beyond 7.2; close kin are excluded from pairing only as far as `Kin` records it — founders' unknown kinship is not modelled; no fertility age window; sex has no other effect yet), and a note for the art line (state 4 / target 8, `get_activity_snapshot`, `get_agent_family`, the activities and family links to draw).
2. Independent review (two rounds maximum): identity I1–I5 and pins; no new field outside 5.2; every exhaustive match handles the new target; the activity arm sits after Gather and before the settlement arm; Idle-equivalence for builders, peers and the meal wait; no tally field added; the birth gates untouched under `family`; one person's activity minutes and one family recomputed by hand from the trace; pre-registration order per stage; tuning seeds only for tuning; verdicts follow 6.7 and 7.7; allowed paths; quiet-lock record. Findings in `docs/revitalize/REVIEW_PASS17.md`.
3. Gate (4.5) → `docs/revitalize/verification/p17_gate/`; fast-forward `codex/revitalize-2026-10` if green; a Pass 17 section in PR #510's description; remove `../new-world-p17-base`.
4. `docs/revitalize/REPORT_PASS17.md` in Korean (section 10 format) and the same text in the final message.

---

# 9. 인게임 확인사항 (오너용)
- 낮(7시~18시)에 쉬는 사람이 확 줄었는지: 화덕 옆에 앉아 불을 보는 사람, 저장고 옆에서 무언가 만드는 사람, 쉼터 옆에서 손보는 사람, 화덕 둘레에 모여 앉은 사람들, 마을 밖으로 멀리 나갔다 돌아오는 사람.
- 아이들이 화덕 근처에서 뛰어다니고, 노인들이 불 옆에 앉아 있는지.
- 같은 사람이 하루 종일 같은 일만 하지 않고, 쉬었다가 다른 일을 고르는지. 끈기 높은 사람이 한 자리에 더 오래 붙어 있는지.
- 아트 라인이 그리기 전까지 새 활동은 제대로 된 그림이 없다(이 브랜치 UI에서는 상태 색이 '일하는 중'처럼, 아트 브랜치에서는 쉬는 모습처럼 보일 수 있다). 활동 장소로 걸어가 머무는 움직임으로 확인한다.
- 저녁 8시 이후 부부가 같은 쉼터로 들어가는지, 아이가 엄마와 같은 쉼터에서 자는지.
- 마을이 갈라질 때 가족이 통째로 함께 떠나는지.
- 굶주림이나 추위로 죽는 사람이 늘지 않았는지, 쉼터 건설이 느려지지 않았는지. 콘솔 에러가 없는지.

---

# 10. 구현 완료 보고 (REPORT_PASS17.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장, 두 단계 각각)

### 구현 내용
단계 A(일과)와 단계 B(가족)에서 무엇을 만들었는가? (구성요소, 규칙, 장면, 결정 팔, 침대·분열 변경, 브리지 게터, 추적)

### 구현 방법
활동 선택·지속시간·휴식 공식과 최종 값, 유휴 동등 처리, 짝짓기 규칙, 실제 부모 선택, 가구 침대·분열, 동작 불변 증명(I1~I5, 고정값), 사전 등록과 조정

### 기능 설명
게임에서 하루가 어떻게 달라졌는가? 가족은 어떻게 보이는가? 무엇이 아직 없는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
R1~R5 표와 F1~F8 표(값과 구간), 시간대별 상태 비율(v20·v21·v22), 활동 종류별 시간, 건강 표, 조정 라운드, 성능 규칙 결과

### 환경
맥 사양, 조용함 잠금 기록과 대기

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md Pass 17 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 대기 구간

### 확인된 제한사항

### 게이트 결과
(단계별 결과, 테스트 수치, 허용 경로 밖 변경 없음)

### 정직한 판단
화면이 살아났는가? 가족이 의미 있게 작동하는가? 다음 게임 패스는 무엇이어야 하는가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #510 update, and the time spent per Part.