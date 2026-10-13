# GOAL: WorldSim — Revitalize Pass 11: villages that split — wild regions, fission and new settlements; a cold that can take anyone; rain that hurts

Repository `hyunlord/new-world`. Continue the Pass 10 line: `codex/revitalize-2026-10-wip` at `8cca033483a6b77d73a6b5904ddd68c2acb9e435` and `codex/revitalize-2026-10` at `14372ea2752842637b73934fc6dfae3da52cf269` (the last code commit; `8cca0334` changes docs only). Draft PR #510 into `lead/main`.
This run is executed by **Claude Code**, unattended, on the Mac. No other heavy job uses the Mac at the same time (a WM run, if any, computes on the DGX). No art run is active.

This is the third simulation pass of the research track and the first step of the integrated roadmap's stage 2 ("several settlements"). **It changes gameplay**: villages can split, and the cold can kill adults. The owner approved both.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 8cca0334
cp ~/Downloads/claude_goal_worldsim_pass11.md docs/revitalize/PASS11_GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS11_GOAL.md>
```
The clone's hooks are neutralized as before (`.claude/settings.json` = `{}`, `--skip-worktree`). Never edit, stage or commit it and never clear that flag. Leave the local `.gitignore` edit and `.ignore` uncommitted. **Never stage gate logs of earlier passes** (Pass 10 committed 165 of them by accident in `00c65bfa`): stage explicit paths only and check `git diff --cached --stat` before every commit.

**This document is `docs/revitalize/PASS11_GOAL.md`.** Commit it unchanged in Part 0. Re-read the relevant section at the start of every Part, after every compaction or pause, and whenever you are unsure of a number.

The owner drives the run with `/goal`: a small model reads **only the transcript**. The evidence it asks for (the section 15 report, the J table, the commit list, the `git ls-remote` heads) must appear in your final message itself.

## 0.2 This document overrides CLAUDE.md where they conflict
As in Pass 10's 0.2: never run `install_hooks.sh`; never check out, commit to or push `lead/main` or `main`; the harness pipeline, `harness_pipeline.sh`, score decomposition, `HARNESS_SKIP`, ENV-BYPASS and `.harness/audit|baseline|reviews/` are not used — 4.5 is the gate; if a hook other than the owner's `/goal` blocks you, record it in DECISIONS.md, push, end with the report; subagents are the built-in `general-purpose` type, never `.claude/agents/harness-*`. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json` — expect no git hooks, a line starting with `S`, `{}`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decide, implement, record decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 11`.
2. **Do not end your turn between Parts.** Keep a task list (one task per Part). After a pause: re-read this file's section for the current Part, the task list, `git log --oneline -5`, background job logs; continue.
3. **Review loops: two rounds maximum.**
4. **Push after every Part to the WIP branch.**
5. **On failure: work around, note it, continue.** An honest miss is acceptable; a missing number is not.
6. **Time rule:** record start time and each Part's start and end (KST). Past **12 hours** of working time (pauses — usage limits, a sleeping Mac — excluded and recorded), finish the current step, push, go to Part 7. Keep the Mac awake for long jobs (`caffeinate -dimsu` in the background, killed at the end; Pass 10 lost 1 h 43 min to sleep).
7. **Three strikes** per failing target: a tuning round that changes knobs counts one strike. After three, keep the best state, record evidence and a hypothesis.
8. **Drop order when time is short:** the 120-day macro-sensitivity forks (12.2) → the `game_v17_nofission` pilot arm (11.3 attribution) → the held-out half of Part 4 (keep the 10-day half). Never drop Parts 0–3, the first pilot (11.1, 11.2, 11.4) or the 35-day fork experiment (12.1).
9. **No image generation, no neural-network training.**
10. **Keep bulky output out of context:** probe runs, recordings and logs go to files; read summaries.
11. **Long commands run in the background** (`nohup bash -c '<cmd>; echo $? > /tmp/p11_<name>.exit' > /tmp/p11_<name>.log 2>&1 &`), polled with short calls; `pgrep -fl` before starting a job again. Gate logs in `docs/revitalize/verification/p11-<N>/`.

---

# ⚠️ 1. MACHINES
- **The Mac** clone `/Users/rexxa/work/new-world-pass4` only. Never read, write or build in `/Users/rexxa/work/new-world-wm1`, `/Users/rexxa/work/new-world-art-prod`, `~/github/new-world-wt/lead` or any other checkout; never kill a process you did not start.
- `export CARGO_BUILD_JOBS=6`; tests `-- --test-threads=6`; probe and recorder `--workers 4`.
- Recordings under `/Users/rexxa/work/worldsim-wm-data/p11/` (outside git). `v0/`, `v1/`, `p9/`, `p10/` are **read-only**.
- Python: system `python3` with numpy only. Headless Godot only (smoke): `GODOT_BIN=/Users/rexxa/Downloads/Godot.app/Contents/MacOS/Godot`.
- Timing runs (J12) back to back with nothing heavy running.
- **DGX:** not used by this pass.

---

# 2. GIT
- Every commit to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose full gate (4.5) is green. Never force-push. Before every push `git remote -v && git branch --show-current`; after it `git ls-remote origin refs/heads/<branch>` must equal `git rev-parse HEAD`.
- Commit messages `revitalize(p11-N): <what changed>`. Stage explicit paths only.
- **May change:** `rust/**`, `docs/revitalize/**`, `tools/revitalize/**`, `docs/worldmodel/**`, `tools/worldmodel/**` — **except** `tools/worldmodel/wm1/**`, `docs/worldmodel/wm1/**`, the v0/v1 datasets, manifests, `BENCH_v0/v1.md`, `bench_v0.py`, `bench_v1.py` and the committed samples.
- **Must not change:** `scripts/**`, `scenes/**`, `shaders/**`, `assets/**`, `localization/**`, `project.godot`, `docs/art/**`, `tools/art_pipeline/**`, static tests that read `.gd`/`.tscn`/`.gdshader`, except minimal Rust-source re-pins allowed in 4.3.
- **Merge notes (list in the report, do not act):** the art branch's `get_intervention_state` getter in `world_node.rs` (do not add a function of that name; keep `world_node.rs` changes to the two constructors, the tallies dictionary and test literals); the Pass 10 UI follow-ups (HUD death sum without exposure, no exposure death notification key, death code 4 drawn in the combat colour); any new UI need this pass discovers (a second settlement's marker, migrants), described, not implemented.

---

# 3. CONTEXT

## 3.1 What Pass 10 and WM-1.1 established (planner's review, with an independent reproduction)
- `game_v16` made seeds different mostly through **static hidden parameters**. On the pilot test seeds the between-seed share of variance η² is food 0.500, alive 0.978, wood 0.063 (the planner reproduced these to three decimals). Persistence predicts `alive` at 168 h with nMAE 0.080: population differences are level offsets set at worldgen (founders 18–30), not diverging histories. Wood and fire stay calendar-locked (R²_abs 0.93–0.96).
- **Exposure deaths are all children.** In the planner's reproduction (16 seeds × 120 days of `game_v16`) all 24 exposure deaths were children (damage per cold hour child 10, adult 4, elder 5: an adult loses 32 per dark night and heals by day). Children do not work and the birth gate refills their places, so exposure added no diversity (|Δη²| ≤ 0.04) and no lasting consequence. Pass 10's report sentence that elders and adults also die is not supported by that sample.
- **Rain saves lives.** Rain puts the fire out without burning wood; the saved wood breaks later dark-night runs, so the rain intervention's mean deaths effect is −0.59. The owner wants rain to be a harmful act.
- **WM-1.1 (final, three seeds):** in v0 the best learned model loses to analog forecasting at 168 h on every split (NONE); every within-village perturbation (agent noise in Pass 9, exposure in Pass 10) is pulled back to the mean trajectory by the supply-aware birth gate and the carrying capacity of one site.

## 3.2 Why fission (planner's diagnosis)
A single village is a strongly self-correcting system: births refill to the site's carrying capacity, stores refill to their targets, children mature in four days. Differences that do not change the *structure* of the world fade. A split is a structural, path-dependent event: when, how many and who leave, where they go, and whether the daughter survives its first winter change the world's future (two carrying capacities instead of one, competition for shared bushes and deadwood, two hearths to keep lit). This is also the integrated roadmap's next game step ("band → village → nation": `claude/WorldSim_통합_로드맵_2026-10.md` in the project). References for the mechanism: village fission at a size or resource threshold is a classic pattern in ethnography and ABM (Chagnon 1975 on Yanomamö fission; the Village Ecodynamics Project ABMs, Kohler et al. 2012); group size near Dunbar's layers (≈ 15 / 50) for the daughter group.

## 3.3 Code facts (planner's survey of `14372ea2`; verify in Part 0)
- **Settlements:** `SettlementSystem` (`runtime/settlement/settlement_system.rs`, priority 138, every tick). Belonging model: members are kept when they wander; new members join by being within `SETTLEMENT_PROXIMITY_RADIUS` 5 of the formation tile; `resolve_exclusivity` keeps a double member in the Manhattan-nearest settlement (ties → lower id). Under generated camps, formation candidates are **Hearth** tiles: a new settlement forms at a Hearth with ≥ 3 agents (`SETTLEMENT_FORMATION_AGENT_THRESHOLD`) and ≥ 2 buildings (`..._BUILDING_THRESHOLD`) within radius 5, outside every existing bubble. Dissolution when no members and no buildings. `SETTLEMENT_MAX_POP` 50.
- **Per-settlement systems** already key on the settlement (stockpiles, `food_economy::annual_mean_supply(resources, sid, radius)`, firewood, fire, shelter planning, housing, meals, births with the `SustainRule` gate). Interventions target `"home"` = the lowest settlement id (`intervention::home_settlement`). The recorder's `settlements.npy` is `[T, S, F]` with S = the number of settlement slots.
- **Worldgen:** `worldgen/camp.rs::populate` places Hearth, three Shelters and a Stockpile around `choose_camp`; berries come from `berries(cfg, r, c)` around the camp, deadwood from `deadwood(cfg, r)` at radius `deadwood_min_radius..=deadwood_max_radius` (5–18) **from the first Hearth**. So today every resource sits around the one camp: a daughter settlement far away would starve.
- **Migration:** a non-member can walk to a settlement as `Seeking { Agent(member) }` with a `SeekTarget` and the `SettlementMigrant` marker (agent_decision.rs ≈ L2311).
- **Exposure:** `ExposureRule { damage_per_cold_hour: [child, adult, elder], heal_block_hours }` applied by `FireSystem` hourly to `Cold` members; `ExposureClock`; deaths `DeathReason::Exposure`.
- **Variety:** `sim-core/src/variety.rs` — `WorldVariety`, `RANGES`, `CLIMATE_PERMILLE`, `draw(seed)` with `keyed_below(seed, STREAM_WORLD_VARIETY, 0, 0, slot, 0, n)` over slots 0–7.

---

# 4. NON-NEGOTIABLE RULES

## 4.1 Determinism (as Pass 9–10)
All simulation logic in Rust; new collections `BTreeMap`/`BTreeSet`; every new loop in AgentId / SettlementId order; no transcendental f64 in simulation crates; no new crates. New random draws are keyed with new, distinct stream constants added to `STREAMS`. Which draws use the noise replicate is specified per draw below; **world draws never use it**.

## 4.2 Versioning — v8 to v16 frozen, v17 new
- `game_v8()` … `game_v16(seed)` (rules and worldgen) keep their behaviour exactly; every new field takes the value that reproduces them bit for bit. `WorldVariety::draw` keeps slots 0–7 unchanged; v17 parameters use new slots 8+.
- New: `WorldRules::game_v17(seed)`, `WorldGenConfig::game_v17(seed)`, scene `game_v17_scene(seed)`; and `game_v17_nofission(seed)` (rules: v17 with fission disabled; worldgen: identical to v17, wild regions included) with its scene. Probe and recorder scene names `game_v17`, `game_v17_nofission` everywhere `game_v16` appears: `probe/args.rs` (`matches!` ≈ L69, its error string ≈ L70, usage ≈ L45), `probe/runner.rs` (`scene_engine`, `twin_checked` ≈ L335 — a scene missing there silently skips the determinism twin — and its test ≈ L347), `tools/revitalize/probe.sh --help`, recorder `export.rs` (`SCENES`, `USAGE`), and `scenes.rs` (set `world_variety` as v16 does). The live game switches to `game_v17` in both `world_node.rs` constructors.
- No new variants of `TargetKind`, `CausalEvent`, `DecisionReason`, `TerrainType`, `DeathReason` or `BuildingKind`. Fission is recorded through a new `Tallies` counter and the recorder, not a new event variant (if a causal record is indispensable, reuse `SettlementFormed` for the daughter and record the parent link in a new `Settlement` field `parent: Option<SettlementId>`).

## 4.3 Legacy invariance and allowed test edits
`WorldRules::default()` unchanged; all Pass 2–10 tests pass unchanged except allowed edits, each with a row in `docs/revitalize/TEST_LEDGER.md` under `## Pass 11`: struct-literal field additions with legacy values; probe schema tests moving to schema 11; recorder schema tests moving to schema 5 (schemas 1–4 still load); Rust-source static assertions pinning code this goal deliberately changes, re-pinned and never weakened. Never change `sim-test/src/probe/fingerprint.rs` or `probe/long_run.rs::trace`.

## 4.4 Tuning discipline
- Tuning seeds 0–63 (10 days) and 0–31 (120 days); held-out 64–127 (10 days) and 32–95 (120 days); **diversity pilot: fresh seeds 22000–22111** (never used before Part 5, not even in unit tests — use seeds < 1000 or ≥ 900000 in tests).
- Scratch exploration on other seeds (≥ 900000) is allowed before a judged round; disclose every scratch configuration in METRICS.md.
- Knobs change only in Part 3 (judged on tuning seeds). Parts 4–6 run on a **frozen** commit recorded in METRICS.md before they start.

## 4.5 The gate
`tools/revitalize/gate_pass11.sh` = `gate_pass10.sh` with the new pass number (debug and release `cargo test --workspace -- --test-threads=6`, both clippy commands with `-D warnings`, the forbidden-math grep, `gdscript_strict_check.sh`, `cargo build -p sim-bridge`, headless smoke `--quit-after 900` with no `SCRIPT ERROR`/`ERROR:`, both Python unittest discoveries).

## 4.6 v16 guard after every behaviour Part
After Parts 1, 2 and 3: re-run `game_v16` and `game_v16_neutral` 64 × 10 and 32 × 120 with Pass 10's final flags and compare with Part 0's runs; every shared value identical (only `schema_version` and new keys may differ).

---

# 5. PART 0 — Baseline and the multi-settlement audit
1. Record `PASS11_BASE_SHA` (`8cca0334`), start time, `uname -a`. Commit `PASS11_GOAL.md`.
2. Release probes with Pass 10's recorded flags: `game_v16` and `game_v16_neutral` 64 × 10 and 32 × 120 against Pass 10's final summaries (`p10-3_round2_*` tuning, `p10-4_heldout_*`); `game_v15`, `game_v14` and legacy against `p10-0_*` / `p10-7_final_*`. Every shared key identical, timing excepted (only `schema_version` and new keys may differ).
3. Extend the frozen pins (`harness_revitalize_pass11_frozen.rs`): `game_v16(seed)` digests for seeds 0–3 at 1,440 and 40,320 ticks, plus field-by-field literals of `game_v16_neutral()` and `WorldVariety::draw` for seeds 0–3.
4. **Multi-settlement audit** `docs/revitalize/MULTI_SETTLEMENT_AUDIT.md`: today one settlement exists in every generated world, so every assumption of "the" settlement is untested. List, with file:line, every place that (a) takes the first / lowest / only settlement, (b) uses the first Hearth or the camp centre as "the" centre, (c) aggregates over settlements in a way that breaks with two (probe metrics: meals at the hearth, evening near the hearth, home distance, store samples; the recorder; `evaluate_pass10.py`; `bench_v1`-style loaders that sum or max over settlements; the FFI tallies and snapshot), (d) assumes resources lie around one camp (worldgen, forage and wood radii, deadwood regrowth, the snow cap, windfall spots), (e) assumes every member is at its own fire (`FireSystem` marks all members cold wherever they are, `fire.rs` ≈ L117–124), (f) re-forms a settlement at an orphaned hearth, (g) ties tethers, builder assignment, plans or haul-home to "home" in a way a walking group would fight (`movement.rs`, `agent_decision.rs`). For each: what it does with two settlements and what v17 needs (fix in Part 2, or harmless, with reason). Commit, push.

---

# 6. PART 1 — A cold that can take anyone; rain that hurts (`game_v17_nofission`, first half)

## 6.1 Hardiness (individual, hidden)
- Component `Hardiness { permille: u16 }` (`sim-core/src/components/hardiness.rs`), attached under v17 to every resident at creation: founders at worldgen, newborns at birth. Value = 700 + `keyed_below(seed, STREAM_HARDINESS, lineage.key, 0, 0, 0, 601)` (uniform 700..=1300). **Replicate 0 always** (an identity trait, not noise). Never derived from `AgentId`.
- Recorder: `agents.npy` gains `hardiness` appended last, marked **hidden** (a latent a model must infer).

## 6.2 Exposure v2
`ExposureRule` gains `hunger_weight: f64` (v16: 0.0) and `use_hardiness: bool` (v16: false). Under v17 the hourly damage of a `Cold` agent is
```
damage = damage_per_cold_hour[stage] × (hardiness_permille / 1000 if use_hardiness else 1)
         × (1 + hunger_weight × clamp((hunger − 50) / 50, 0, 1))
```
**Higher `hardiness_permille` means more damage** (it is a frailty factor; keep the name, document it). Start values: `damage_per_cold_hour` = [8.0, 6.0, 8.0], `hunger_weight` = 0.5, `use_hardiness` = true, `heal_block_hours` = 2. (Orientation: a full dark night is 8 cold hours → an average adult loses 48, the frailest hungry adult up to 6 × 1.3 × 1.5 × 8 = 93.6; children and elders 64 on average, up to 124.8. Hardiness spreads deaths over nights instead of killing every same-stage agent at the same 05:00, which Pass 10's start round did.)

## 6.3 Rain soaks the woodpile
`FirewoodRule` (or the intervention rules slot; record the choice) gains `rain_soak_permille: u32` (v8–v16: 0; v17 start: 300). Under it, `Intervention::Rain` on a settlement also removes `floor(wood_stock × rain_soak_permille / 1000)` from that settlement's stockpile at application (`InterventionRecord` — `sim-engine/src/intervention.rs`, no serde derives — gains `soaked_wood: u64`, 0 outside v17, and `export.rs` writes it into each `interventions.json` row; `applied` keeps meaning the hours). Rain still puts the fire out for its hours. Test: v16 rain unchanged (digest); v17 rain on a 20-wood store leaves 14.

## 6.4 Instruments (Part 1 share)
Probe schema 11 (moved to in Part 2 together with the fission keys): exposure deaths per seed-winter **by stage** (child / adult / elder), from the death census (`deaths.jsonl` rows already carry `life_stage_at_death` with child/adult only, `probe/deaths.rs` ≈ L293: add a new key `stage` with child / adult / elder, captured before despawn, and leave the old key unchanged) and the maximum exposure deaths in one settlement in one hour.

## 6.5 Tests (`harness_revitalize_pass11_exposure.rs`)
`harness_hardiness_keyed_and_ranged` (range, keyed on lineage, unchanged by replicate and by birth order), `harness_exposure_v2_formula` (a constructed agent: damage equals the formula for three hardiness × hunger cases), `harness_exposure_adult_can_die` (an adult with hardiness 1300, hunger fixed at 99 — growth rate 0, below `Hunger::SATURATION` 100 so no starvation damage — and thirst 0 loses ≈ 93 hp on the first full dark night, cannot heal because hunger > 50, and dies on the second with `DeathReason::Exposure`; with hardiness 700 and hunger 0 it survives two), `harness_rain_soaks_only_in_v17`, `harness_v16_exposure_unchanged`. Commit, push; run 4.6.

---

# 7. PART 2 — Wild regions and fission (`game_v17`)

## 7.1 Wild regions (worldgen, hidden)
- `WorldVariety` gains (new slots 8–10; `NEUTRAL` gets the v16-equivalent "0 regions"): `wild_regions: u8` uniform 1..=3; `region_bushes: u32` uniform 5..=9 (per region); `region_deadwood: u32` uniform 6..=12 (per region).
- `WorldGenConfig::game_v17(seed)` = v16's config plus the regions. After the camp, berries and deadwood of v16 are placed **unchanged** (same streams), place each region in order r = 0..wild_regions: its centre is drawn uniformly by `keyed_below(seed, STREAM_REGION, 0, 0, r, 0, n)` among the walkable, non-water tiles inside the map at Chebyshev distance 22–28 from the camp hearth and ≥ 20 from every earlier region centre, listed in (y, x) order (a uniform draw, not a max-distance rule, so regions do not all sit at distance 28); no candidate → that region is skipped and counted. Then `region_bushes` bushes of the world's `berry_capacity` and `region_deadwood` deadwood spots of `deadwood_capacity`, chosen with the **candidate filters** of v16's berries/deadwood (not the functions themselves: `berries()` asserts on a shortfall and `repair_berry_edges` edits terrain, `camp.rs` ≈ L197–224) within radius 2–6 of the centre, from their own keyed streams (slots per region); never panic, count shortfalls in the worldgen report. Region centres are worldgen outputs, not `WorldVariety` fields: emit them (and the region parameters) in the probe's per-seed variety block and `meta.json`'s hidden `world_variety` block only when `wild_regions > 0`, so v16 outputs are unchanged.
- Deadwood regrowth, the snow cap and windfall already iterate all spots (`wood_source_max`) — verify. But the winter fall is a **world total spread round-robin over all spots** and windfall fills spots in key order (`resource_regen/mod.rs` ≈ L190–216, `intervention.rs` ≈ L98–125): regions therefore dilute the camp's winter wood even without fission, and windfall can land far from home. v17 changes the winter fall to a per-region (camp area + each region) budget proportional to its spot count, and windfall to the target's nearest spots first; both behind v17 flags; tests.
- Tests: v16 worlds unchanged (digest); v17 regions in range, at the stated distances, not on water, deterministic, independent of the replicate.

## 7.2 Fission
`SocietyRules` gains `fission: Option<FissionRule>` (None in v8–v16 and in `game_v17_nofission`):
```rust
pub struct FissionRule {
    pub min_members: u32,          // 28: smallest parent that can split
    pub crowding: f64,             // 0.95: members ≥ crowding × sustain capacity
    pub group_share: f64,          // 0.35: share of the parent that leaves
    pub group_min: u32,            // 6
    pub group_max: u32,            // 14
    pub min_adults: u32,           // 4 adults in the group (no elders leave)
    pub cooldown_days: u32,        // 28: per parent, and per world after any fission
    pub min_age_days: u32,         // 14: a settlement must exist this long before it can split
    pub site_clearance: u32,       // 16: no settlement's formation tile within this Chebyshev distance of a site
    pub stock_share: bool,         // true: the group takes its share of food and wood
    pub max_settlements: u32,      // 4
    pub settle_timeout_days: u32,  // 2
    pub remnant_members: u32,      // 2: a settlement with ≤ this many members for 24 h merges away
}
```
**Daily check** at 06:00, for each settlement in SettlementId order (`FissionSystem`, a new system after `SettlementSystem`; record its priority and interval 60):
1. Eligible when: season is spring or summer; settlement age ≥ `min_age_days`; ≥ `cooldown_days` since its last fission and since the world's last fission; members ≥ `min_members`; members ≥ `crowding × capacity` where capacity = the integer the `SustainRule` birth gate admits (`floor(utilization × annual_mean_supply / adult_need_per_day)`, computed by the same functions as the gate); settlements alive < `max_settlements`; at least one free site.
2. **Site:** among the wild-region centres whose distance to every existing formation tile is ≥ `site_clearance`, the one nearest to the parent's formation tile (ties: lower region index). No free site → no fission (the check records `fission_blocked_no_site`).
3. **Group:** size k = clamp(round(members × `group_share`), `group_min`, `group_max`). Rank every non-elder member by `keyed_u64(seed, STREAM_FISSION, lineage.key, tick, 0, r)` with **r = the noise replicate at that tick** (a fork resamples who leaves; the timing is decided by the state). Take adults in rank order until `min_adults`, then the remaining slots in rank order from all non-elders. If fewer than `min_adults` adults exist, no fission.
4. **Departure:** each group member gets a `Founding { site, fission_id, leader, since_tick }` marker (leader = the group's lowest-AgentId adult). The walk (planner's review of `agent_decision.rs`: a self-targeted `Seeking{Agent}` loses its `SeekTarget` in the post-pass ≈ L2196–2213, and the social gate aborts chases on any actionable need and after `max_chase_ticks` ≈ L1627–1650, L1688–1703):
   - every founder, the leader included, holds `Seeking { Agent(leader) }`, and under `Founding` the post-pass sets `SeekTarget = site tile` for every founder (the leader's self-target is allowed only under `Founding`);
   - the social gate and its chase timeout skip `Founding` agents;
   - urgent needs pre-empt the walk as for migrants, but **keep** the `Founding` marker; the next Idle decision re-enters the walk before any work, forage, firewood, meal, social or building arm;
   - `Founding` agents are excluded from builder assignment (≈ L641–653), forage / firewood / meal plans, and from every movement tether and haul-home (`movement.rs` ≈ L189–203, L279–341);
   - `FireSystem` does not mark a `Founding` agent `Cold` for the parent's hearth (it is not at that fire); it can be marked for the daughter's hearth once founded.
   With `stock_share`, the parent's stockpile transfers `floor(stock × k / members)` food and wood into a pending store held by `FissionSystem`; on a cancelled fission the pending store returns to the parent. Food and wood are conserved (test).
5. **Founding:** when ≥ 3 founders stand within radius 5 of the site, or `settle_timeout_days` after departure (then at the leader's tile if it is ≥ `site_clearance` from every formation tile, else the group returns home and the fission is cancelled and counted), place a Hearth at the nearest buildable tile to the site and **two** Shelters nearby (the same placement as `camp.rs::populate`, event queue included; two because births are also capped by owned housing, 8 places per shelter, `settlement_system.rs` ≈ L706–713); the formation scan forms the daughter settlement; **on that tick `FissionSystem` moves every group member explicitly** (removed from the parent's roster, inserted into the daughter's — `resolve_exclusivity` alone would move only founders standing within radius 5); the pending store goes into the daughter's stockpile; `Founding` is removed. Record `Settlement::parent` and `Tallies::fissions` / `fissions_cancelled`. Record in METRICS a per-run count of membership changes made by `resolve_exclusivity` outside fission and migration (agents wandering into another settlement's radius are re-assigned by the existing rule; report how often).
6. **Alive, remnants, dissolution** (today a settlement dissolves only with 0 members **and** no buildings, and nothing ever removes a building, `settlement_system.rs` ≈ L909):
   - A settlement is **alive** while it has ≥ 1 member; every "settlements alive" count, `max_settlements`, J3 and J4 use this definition.
   - A settlement with ≤ `remnant_members` members for 24 consecutive hours: `FissionSystem` removes each remaining member from its roster (they are then non-members and take the existing `SettlementMigrant` arm, which only starts for non-members; its target is the nearest member of any settlement, ≈ L1521–1538 — accept it and say so) and adds the remnant's hearth tile to a `retired_hearths` set that the formation scan skips (otherwise it would re-form at the orphaned hearth).
   - A settlement that reaches 0 members (remnant merge or all members dead) is dissolved explicitly by `FissionSystem` (`SettlementDissolved`, local state cleared as the existing rule does) although its buildings remain; its hearth joins `retired_hearths`.
- Interventions keep targeting `"home"` = the lowest settlement id (the original village).

## 7.3 Instruments
- **Probe schema 11:** per seed: fissions, cancellations, blocked-no-site days, settlements alive at every daily sample, per settlement: founding tick, parent, founders, members at days 30/60/90/118, dissolution tick; Part 1's exposure-by-stage keys. Every per-seed quantity per seed (never from the `rules` block of the first seed).
- **Recorder schema 5:** global `settlements_alive`, `fissions_total`, `deaths_exposure` (exists); `settlements.npy` gains `parent_slot` (−1 none) and `founded_tick`; `agents.npy` gains `hardiness` (hidden, 6.1) and `founding` (0/1). `wm_data.py` accepts schema 5 (`SCHEMAS` and the branch-family check `in (3, 4)` ≈ L148). A settlement slot keeps its index for its lifetime; a dissolved settlement's slot stays NaN afterwards. Document the slot rule in `DATASET` notes.
- `evaluate_pass11.py` (from `evaluate_pass10.py`): v16 baseline scene, per-agent home settlement for every metric that used "the" hearth (Part 0 audit), the J targets.

## 7.4 Tests (`harness_revitalize_pass11_fission.rs`)
`harness_fission_inert_before_v17`, `harness_fission_only_when_crowded` (a constructed world below `crowding` never splits; above it splits once, at 06:00 of a spring or summer day), `harness_fission_group_rules` (size, ≥ 4 adults, no elders, ranked by the keyed draw), `harness_fission_group_resampled_by_replicate` (replicate 3 forked before the check picks a different group in at least one of 8 seeds; replicate 0 is identical to the unforked run), `harness_fission_founds_daughter` (hearth + shelter placed, daughter formed, membership moved, pending store delivered, `parent` set), `harness_fission_timeout_and_cancel`, `harness_remnant_merges`, `harness_fission_order_independent` (spawn order changes nothing). Commit, push; run 4.6; fast-forward green if the gate is green.

---

# 8. PART 3 — Tuning (tuning seeds only)
Run `game_v17` and `game_v17_nofission` on the tuning seeds (64 × 10, 32 × 120, `--check-determinism`), evaluate J1–J9 against `game_v16` on the same seeds. **Knobs and ranges:** `damage_per_cold_hour` child/elder 5–12, adult 3–9; `hunger_weight` 0–1; hardiness range ends ± 100; `heal_block_hours` 0–4; `rain_soak_permille` 150–500; `FissionRule` `min_members` 22–34, `crowding` 0.85–1.0, `group_share` 0.25–0.45, `cooldown_days` 14–42, `site_clearance` 12–20; region ranges ± 2 at either end; `SustainRule::utilization` 0.75–0.85 (v17 only). At most three judged rounds; record every round's knobs and J results in METRICS.md. Freeze: commit the final knobs, record the frozen commit. Commit, push; run 4.6.

# 9. PART 4 — Held-out health
On the frozen commit: both v17 scenes on held-out seeds (64–127 × 10, 32–95 × 120), J1–J9 against `game_v16` on the same seeds; timing for J12. No knob changes. Commit, push.

---

# 10. TARGETS (J)

| # | target |
|---|---|
| J1 | determinism 64/64 and 32/32 for both v17 scenes on tuning and held-out seeds; legacy pin; v8–v16 reproduce Pass 10's final summaries on every shared key |
| J2 | **no collapse** (both scenes, every 120-day seed): total alive ≥ max(8, 0.4 × starting agents) at every daily sample; total alive on day 120 ≥ 0.6 × day 21; no settlement lingers with 1–2 members for more than 48 h |
| J3 | **fission happens and varies** (`game_v17`, 120-day seeds): share of seeds with ≥ 1 fission in [0.50, 0.90]; first-fission day: median in [30, 90] and interquartile range ≥ 14 days; share of seeds with ≥ 2 settlements alive at day 120 in [0.30, 0.85]; no seed above `max_settlements`; cancellations ≤ 0.25 of attempts |
| J4 | **daughters are viable but not guaranteed**: share of daughters alive 28 days after founding ≥ 0.60; report the share that dissolved or merged |
| J5 | **the cold can take anyone** (`game_v17`, held-out + tuning seed-winters with ≥ 1 exposure death pooled): adult share of exposure deaths in [0.15, 0.50]; child share ≤ 0.70; where elders are present in a seed-winter, report their share; over settlement-nights with ≥ 1 exposure death, the median of the maximum exposure deaths in one hour ≤ 2; Pass 10's H3 bounds otherwise (share of seed-winters with ≥ 1 exposure death in [0.15, 0.60], median per seed-winter ≤ 2, maximum ≤ 0.35 × members, mild < normal < harsh); starvation + dehydration 0 at 10 days in every seed, 120-day median ≤ 1, maximum ≤ 6 |
| J6 | **village rhythm kept** (`game_v17_nofission` vs `game_v16`, comparison mode as Pass 10's H5, with Pass 10's per-clause escape table): Pass 6 Y4, Y5, Y6, Y9, Y10, Y11, Y12 and Pass 8 X3 (b)–(e), X4. For `game_v17`, the same clauses evaluated per agent against its own settlement's hearth, reported (not gated) |
| J7 | **diversity with dynamics** (Part 5 pilot, judged on `game_v17`): η² ≥ 0.30 **and** R²_abs ≤ 0.70 for `food_stock`, `wood_stock` and `alive` (as H7); **and** the new dynamic share dyn-η² ≥ 0.25 for `alive` and `food_stock` (11.2) |
| J8 | **interventions have consequences** (Part 6, `game_v17`): placebo identity 100 %; rain: mean deaths effect ≥ 0 and mean dark-night effect > 0; at least two of {rain, cold_snap, rot_wood} have replicate-mean deaths effect > 0 in ≥ 0.25 of seeds; gift_wood's mean deaths effect ≤ 0 |
| J9 | variety and regions realized: every new parameter's mean within ± 10 % of its range centre over the 128 tuning + held-out seeds; region placement failures 0 (or counted and explained) |
| J10 | recorder schema 5 byte identity (two runs; `--workers 1` vs 4); schemas 1–4 still load; Pass 9–10 fork tests pass for v17 |
| J11 | macro sensitivity reported (12.2; no threshold) |
| J12 | ms per agent-tick ≤ 1.3 × `game_v16` (both v17 scenes, back to back on one host) |

---

# 11. PART 5 — The diversity pilot (frozen commit)

## 11.1 Recording
`worldsim_wm` schema 5, 120 days, `--every 60`, no `--agents`, seeds **22000–22111** (`--seeds 22000..22112`), scenes `game_v16`, `game_v17_nofission`, `game_v17` → `/Users/rexxa/work/worldsim-wm-data/p11/pilot/<scene>/`. Roles: 22000–22095 train, 22096–22111 test.

## 11.2 Metrics
`tools/revitalize/diversity_p10.py` extended (keep `calcore_wm11.py` byte-identical; new code in `diversity_p11.py` if cleaner): everything of Pass 10's 11.2 (η², R²_abs, R²_year, divergence day, spread, calendar-family and persistence nMAE at 24 h and 168 h, every-hour starts), plus:
- **dyn-η²** — the between-seed share of variance after removing each episode's own early level: for each episode e and variable j, `a_e[r] = v_e[r] − mean(v_e[rows of display days 3..10])`; dyn-η² is calcore's η² computed on `a` over rows of days ≥ 11. It is near 0 when seeds differ only by constant offsets and grows when histories diverge. Report it for v15 too (re-score Pass 10's pilot recordings read-only: `p10/pilot/game_v15`, `game_v16`) as the reference.
- settlement count (alive, 7.2 step 6) at days 30/60/90/118 per test seed, and its between-seed variance share at each of those days (one value per seed, so report variance and the distribution, not a per-row η²).
- First run the Pass 10 implementation check again (v1 A_test, WM-1.1's 12 numbers to ± 0.002).

## 11.3 Attribution
v16 → v17_nofission isolates exposure v2 + rain + regions; v17_nofission → v17 isolates fission. Report Δη² and Δdyn-η² per variable.

## 11.4 Outputs
`docs/revitalize/probe/p11-5_pilot/` (`diversity.json`, `diversity.md`, exact commands). J7 is judged here, once.

---

# 12. PART 6 — Forks (frozen commit, `game_v17`)

## 12.1 Consequences (35 days)
Regenerate the schedules with `tools/revitalize/pass9_fork_schedules.py` into `p11/schedules` and write `p11_forks.sh` modelled on `docs/revitalize/probe/p10-6_forks/p6_forks.sh` (which hard-codes Pass 10's binary, data root and `WORLDSIM_GIT_SHA`) with `--scene game_v17 --replicates 4` under `p11/`: `--agents`, `--include-factual`, the main families (seeds 12000–12031, 35 days, `--every 60`, fork tick 28,800, the five C_cf variants and the one-unit gift plus the factual), the placebo pass (seven amount-0 kinds on 12000–12007), the unforked runs into `p11/unforked`; `fork_metrics.py --deaths-with-exposure --unforked …`. **Dark nights must be read from the home settlement's slot** (the slot of the lowest id in `settlement_ids.npy`), not the max over settlements as `fork_metrics.py` ≈ L176 and `bench_v1.py` ≈ L91 do (a daughter's lit fire would hide rain on home); add a `--home-slot-fire` option, test it, leave `bench_v1.py` untouched. Report Pass 10's tables plus deaths by stage. J8 is judged here. Outputs `docs/revitalize/probe/p11-6_forks/`.

## 12.2 Macro sensitivity (120 days; drop first)
Seeds 12000–12031, 120 days, `--every 60`, no `--agents`, a schedule directory holding only the `rot_wood` and `gift_wood` files plus `--include-factual`, at the C_cf ticks, `--fork-tick 28800 --replicates 2`. Report: the share of seeds whose settlement count at day 60 / 90 / 118 or first-fission day differs (a) between the factual's two replicates (noise only) and (b) between an intervention and the factual at the same replicate; the mean |Δ| of total alive at day 118 for (a) and (b). This is the first measurement of whether a small act of the god changes the shape of the world. Outputs `docs/revitalize/probe/p11-6_macro/`. Commit, push.

---

# 13. PART 7 — Independent review, final verification, report
1. **Independent review** (fresh `general-purpose` subagent, two rounds maximum): `git diff 8cca0334..HEAD -- rust/ tools/revitalize/ tools/worldmodel/`, sections 4, 6, 7, 10, 11, 12, METRICS.md's Pass 11 rounds, the pilot and fork outputs, MULTI_SETTLEMENT_AUDIT.md. With file:line evidence it checks: v8–v16 invariance; keyed draws and their replicate use (hardiness and regions never, fission group always); no iteration-order dependence in new loops; fission only under the rule's conditions; founders' membership moves and stores conserve (food and wood before = after the transfer); remnant merge; every audit item handled; exposure-by-stage counted correctly; the pilot ran on the frozen commit with fresh seeds; J verdicts follow the rules.
2. Full gate on the final WIP head; totals versus Pass 10 (2,054 debug and release).
3. Final probes: legacy and v8–v16 reproduce Pass 10; both v17 scenes on tuning and held-out seeds; timings for J12; say whether any code changed after Parts 5–6 and rerun what is affected.
4. Fast-forward `codex/revitalize-2026-10` if green; otherwise say exactly what is red.
5. `docs/revitalize/REPORT_PASS11.md` in Korean (section 15 format). Section 14's in-game checks are for the owner: smoke only, list them as "주인 확인 필요". Append `## Pass 11` to METRICS.md and DECISIONS.md.
6. Update PR #510's description with a short Pass 11 section (REST API).

## 13.1 Out of scope
Temperament and personality beyond hardiness; family and kinship links; trade or conflict between settlements; dataset v2 and its benchmark; weather resampling in forks; any learned model; UI and art (merge notes only).

---

# 14. 인게임 확인사항 (게임을 직접 켰을 때 확인할 것)
1. 게임이 game_v17로 시작하고 콘솔에 SCRIPT ERROR가 없다.
2. 처음 마을에서 떨어진 곳(지도 가장자리 쪽 1~3곳)에 덤불과 나무가 모인 땅이 따로 있다.
3. 봄·여름에 마을이 붐비면, 성인 몇 명과 아이들이 무리를 지어 그 땅으로 걸어가 새 모닥불과 쉼터를 세우고 두 번째 마을이 생긴다. 두 마을의 표시(경계·이름)가 화면에 어떻게 보이는지 확인한다(UI 후속 작업 판단용).
4. 혹독한 겨울에 불 꺼진 밤이 이어지면 아이뿐 아니라 배고프고 약한 어른도 죽을 수 있다.
5. 비를 내리면 그 마을의 나무 더미가 줄어든다(비가 더 이상 득이 되지 않는다).
6. 120일 동안 모든 마을이 통째로 사라지는 일은 없다.

---

# 15. 구현 완료 보고 (REPORT_PASS11.md와 최종 메시지에 이 형식 그대로, 한국어로)

```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
Part별로 무엇을 만들었는가? (파일과 핵심 구조)

### 구현 방법
다중 정착지 감사 결과와 수정, 체질(hardiness)과 노출 v2, 비의 나무 젖음, 야생 지역 생성, 분열(조건·장소·무리 선정·이동·정착·잔존 병합), v8~v16 고정, 튜닝 라운드, 다양성 측정(dyn-η² 포함), 분기 실험

### 기능 설명
게임과 연구 관점에서 무엇이 달라졌는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
J1~J12 표, 튜닝 라운드별 손잡이와 결과, 분열 통계(발생 비율·첫 분열일 분포·정착지 수·딸 마을 생존), 단계별 노출 사망, v15/v16/v17_nofission/v17 다양성 표(η²·dyn-η²·R²_abs·정착지 수·기준선 nMAE), 귀속, 분기 실험(사망 효과·VRF·가짜 개입), 거시 민감도

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 멈춘 구간, 백그라운드 작업

### 확인된 제한사항

### Harness 결과
(debug·release cargo test 수치, clippy, strict check, smoke, Python 단위 테스트 두 묶음)

### 정직한 판단
이제 시드마다 역사가 갈라지는가(수준 차이가 아니라)? 분열이 그 원인인가? 신의 작은 개입이 세계의 모양을 바꾸는가? 게임으로서 나아졌는가? 데이터셋 v2로 가도 되는가?
```

Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the compare URL `https://github.com/hyunlord/new-world/compare/lead/main...codex/revitalize-2026-10`, whether PR #510's description was updated, the paths of `diversity.md`, `fork_metrics.md` and the macro-sensitivity report, and the time spent per Part.
