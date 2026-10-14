# GOAL: WorldSim — Revitalize Pass 16: people who differ — temperament v1 (`game_v20`)

Repository `hyunlord/new-world`. Continue the simulation line: `codex/revitalize-2026-10` = `da8e45c7858b7973330ebecef8ff35ae5ee936ee` (Pass 15, gate green), `codex/revitalize-2026-10-wip` = `92688a95d7239d724c9d52fa4a7cd35131591cc7` (Pass 15's report, review and gate logs after it). Draft PR #510.
This run is executed by **Claude Code on the Mac** in the simulation clone, unattended. **At the same time an art run (Art Pass 9, Codex) works on the same Mac** in `/Users/rexxa/work/new-world-art-prod` and measures frame times there; section 1.2 is how the two runs share the Mac. A world-model run (WM-2) trains on the DGX's GPU; this pass's probes run on the DGX's CPU, gently (section 1.3).

**This is the first game-track pass** (`WorldSim_통합_로드맵_2026-10.md` sections 5–7: the research world is frozen on `game_v19`; the game grows in new scene versions). It adds the first piece of individuality: every person gets a temperament of four axes, drawn at birth and partly inherited, and four of their everyday choices lean with it. It is a new scene, `game_v20`; **every older scene stays bit-for-bit as it is**, `game_v19` above all (the research dataset is recorded on it). The live game switches to `game_v20`.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 92688a95
cp ~/Downloads/claude_goal_worldsim_pass16.md docs/revitalize/PASS16_GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS16_GOAL.md>
```
The clone's hooks are neutralized as before (`.claude/settings.json` = `{}`, `--skip-worktree`); never edit, stage or commit it and never clear the flag. Leave the local `.gitignore` edit and `.ignore` uncommitted. Stage explicit paths only; never stage outputs or gate logs of earlier passes.

**This document is `docs/revitalize/PASS16_GOAL.md`.** Commit it unchanged in Part 0. Re-read the relevant section at the start of every Part and after every compaction or pause. The owner drives the run with `/goal`: a small model reads only the transcript, so the section 13 report, the T table, commit hashes and `git ls-remote` heads must appear in your final message.

## 0.2 Overrides of CLAUDE.md
As Pass 15's 0.2: no `install_hooks.sh`, never `lead/main` or `main`, no `harness_pipeline.sh` (4.5 is the gate), built-in `general-purpose` subagents only, a blocking hook other than `/goal` → record, push, report. CLAUDE.md's temperament pipeline (38-D genes → TCI → HEXACO → …) is the long-term design; this pass builds only its first, reduced step (section 3.2) — say so in the docs. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 16`, with the RUNLOG table (Part, start, end, notes).
2. **Do not end your turn between Parts or while a DGX job runs.** Poll long jobs every 10–20 minutes with short calls.
3. Review loops: two rounds maximum. Push after every Part.
4. On failure: work around, note it, continue. A hook that does not move behaviour is a valid result; a missing number is not.
5. **Time rule:** past **12 hours** of working time (pauses excluded and recorded), finish the current step, push, go to Part 6.
6. **Drop order:** the second tuning round (8.3) → the 120-day evaluation beyond seeds 16000–16015 (keep 16 for every scene) → the fission-departure hook (6.4 e; keep the other four; T1-dep then reads "dropped"). Never drop Part 0's v19 pins, the identity checks, the inert control, T1–T3, the performance rule or Part 6.

---

# ⚠️ 1. MACHINES

## 1.1 Mac and DGX
- **Mac** (`/Users/rexxa/work/new-world-pass4`): edit, build, test, the gate, the performance rule's timed runs. Never touch `/Users/rexxa/work/new-world-art-prod`, `/Users/rexxa/work/new-world-wm1`, `~/github/new-world-wt/lead` or any other checkout; never stop a process you did not start.
- **DGX** (`ssh hyunlord@100.70.109.50`; a throwaway `ssh -F` config or the full address; never edit `~/.ssh/config`): your code clone `/home/hyunlord/work/new-world-p16` (fresh clone, detached at a pushed WIP commit before every job; `P16_CODE_COMMIT` recorded), `cargo build --release -p sim-test` there; data `/home/hyunlord/work/worldsim-wm-data/p16/` (everything else under `worldsim-wm-data/` is read-only to you; `v2/` belongs to WM-2); numpy from `/home/hyunlord/work/wm1-venv/bin/python` (never install into it). **Forbidden:** `/home/hyunlord/work/new-world-wm1`, `/home/hyunlord/work/wm1-runs/**`, `~/github/worldsim-training`, ComfyUI (8188), dev servers 3100/3200, any process you did not start.

## 1.2 Sharing the Mac with the art run (both briefs carry this protocol)
- **Quiet lock:** the directory `/Users/rexxa/work/.mac_quiet.lock` (`mkdir` is the atomic acquire). Write `owner`, `pid`, `purpose`, `started` (ISO time) into `info.txt` inside it; remove it when done. Hold it **only while measuring time** (the performance rule's runs, 8.5), at most 60 minutes per hold. Leave at least **10 minutes** between two of your holds so the other run can build and test.
- **While the other run holds it:** do not start a cargo build or test, a Godot run, a probe or the gate on the Mac; wait, polling every 5 minutes. A lock older than 2 hours is stale: remove it, record it, continue.
- **Before taking it:** wait until no `cargo`, `rustc`, `worldsim_*` or `Godot` process of the other run is running (`pgrep -x` on the names) and the 1-minute load average (`sysctl -n vm.loadavg`) is < 4; after 60 minutes of waiting, take it anyway and mark the runs `contended`.
- Record every hold and every wait in DECISIONS' RUNLOG.

## 1.3 Being gentle on the DGX
Before every DGX job: `nproc`, `uptime`, `free -h`, `nvidia-smi`, `pgrep -fl "m1_tf|m1_gru|worldsim_"`. Every probe under `nice -n 10` with `--workers` = min(4, nproc − 12); never more than one probe job of yours at a time; never the GPU. Detached jobs (`setsid nohup … > log 2>&1 &`, PID and `.exit` recorded). Every probe sets `WORLDSIM_GIT_SHA` to the code commit it was built from.

---

# 2. GIT
- Commits to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose gate (4.5) is green. Never force-push. `git remote -v && git branch --show-current` before every push; `git ls-remote` after.
- Commit messages `revitalize(p16-N): …`.
- **May change:** `rust/**` (the simulation line owns it), `tools/revitalize/**`, `docs/revitalize/**`. **Must not change:** `scripts/**`, `scenes/**`, `assets/**`, `localization/**`, `docs/art/**`, `tools/art_pipeline/**`, `tools/worldmodel/**`, `docs/worldmodel/**`, the recorder's schema (`rust/crates/sim-test/src/worldmodel/**` — `game_v20` is not recorded).
- **No new field in an existing public struct except the one rules field of 5.2.** The art branch builds some sim structs as literals (Art Pass 7 broke on a new field); `AgentRules` literals exist only in the eight frozen tests listed in 5.2, which travel with the merge. New components, new getters and new types are fine. In particular **do not add a key to `get_agent_detail`'s dictionary** (`harness_p14_gamma_click_inspector.rs` L478–506 requires exactly 9 keys) — add a separate getter (6.6).

---

# 3. CONTEXT

## 3.1 What the code does today (planner's review with a code survey; verify each fact before relying on it)
- **Decisions are deterministic.** `AgentDecisionSystem` (`sim-systems/src/runtime/decision/agent_decision.rs`, priority 125, every tick, `tick` L599–2433) has no random draw; ties break by distance, coordinates and AgentId. The idle cascade (L1059–1666) tries, in order: meal wait, survival needs (hunger/thirst > 50, fatigue > threshold), construction, social (loneliness > `SOCIAL_THRESHOLD` 50), combat, memory-bias flip, forage (06:00–18:00), firewood (10:00–17:00, urgent from 09:00), gather (no clock window), migration.
- **The only per-agent random behaviour** is the idle random walk in `runtime/agent/movement.rs` L308–349: a keyed pause chance and steps tethered to the hearth (`idle_tether_radius` 8; evening gather radius 3; child 3; elder 4).
- **Fission picks who leaves by lottery** (`runtime/settlement/fission.rs` L550–574): non-elders ranked by `keyed_u64(seed, STREAM_FISSION, lineage key, tick, 0, rep)` ascending; group size `clamp(round(members × 0.35), 6, 14)`; the first `min_adults` 4 adults, then by rank.
- **Identity traits already exist as a pattern:** `Hardiness` (`sim-core/src/components/hardiness.rs` L19–35) is drawn from the lineage key, tick 0, replicate 0 (`keyed_below(seed, STREAM_HARDINESS, key, 0, 0, 0, 601)`), inserted for founders in `worldgen/camp.rs` L147–155 and for newborns in `settlement_system.rs` L873–880. Noise forks never redraw it.
- **Births have no real parents yet.** `SettlementSystem` (`settlement_system.rs` L801–908) names as "parents" the **two lowest-AgentId members** of the settlement (L806–814), uses only the first one's lineage (L821–834), and records both only in `CausalEvent::AgentBorn { parent_ids }` (L901–907). Real couples are the family pass's job; this pass inherits from those two and says so.
- **Draw helpers** (`sim-core/src/rng.rs`): `keyed_u64` L117, `keyed_unit_f64` L122, `keyed_below` L133; streams L26–83 and `STREAMS: [u64; 16]` (L86, a distinctness test at L168). No normal-distribution helper; lifespan uses the mean of three keyed uniforms (`aging.rs` L65–70). The gate forbids `exp/ln/pow/sin/…` in the sim crates; integer and `sqrt` arithmetic are fine.
- **What else the hooks touch** (reviewer's survey): loneliness grows at a per-agent rate stored in the `Social` component (`Social::new(0.0, 0.04)` for founders, `camp.rs` ≈ L137; `BIRTH_SOCIAL_RATE` 0.04 for newborns, `settlement_system.rs` ≈ L102) and each completed interaction removes `SOCIAL_CONSUME_AMOUNT` 30 (`social_interaction_system.rs` ≈ L313) — so completions per day are set by the growth **rate**, not by the threshold; night rest makes the fatigue threshold 0 from 21:00 to 06:00 (`NightRestRule { start_minute: 1260, end_minute: 360, … }`, `rules.rs` ≈ L995; `fatigue_threshold_at` `agent_decision.rs` L242, computed once per tick at L638); a carried load is hauled home at `haul_threshold` 4 food / 2 wood (`rules.rs` ≈ L897, L904; read at `agent_decision.rs` L699, L701, L1031–1032, L1503, L1528, L1560, L2039, L2499, `movement.rs` L110, L130–131, L398, `settlement/firewood.rs` L325), inventory capacity 10 (`inventory.rs` L23). Gather trips are rarely abandoned (`gather_preempted_total` median 1 per world in 120 days), so `critical_preemption` is not a usable lever. Elders (≈ 12 of 66 alive at day 120) never forage or gather wood and keep tether 4. **No fission happens before day 28** in any v19 seed 0–31.
- **ECS order matters:** `agent_decision.rs` L2408–2420 documents a contended split that depended on query iteration order; hecs iterates by archetype, so where a new component is inserted can reorder agents. Insert `Temperament` exactly where `Hardiness` is inserted (founders and newborns), never from a separate loop or system.
- **Rules layering:** `WorldRules { agent, society, clock_offset_ticks, seasons }` (`rules.rs` L6–15), optional rules as `Option<…Rule>` gated by plain field reads; `game_v19_with` L720–734. Whole-struct `AgentRules` literals exist in `rules.rs` at ≈ L499 (`WorldRules::default`; `AgentRules` has no `Default` impl), ≈ L1030 (`game_v8`) and ≈ L1388 (`legacy_has_no_opt_in_behavior`), and in eight frozen tests: `sim-test/tests/harness_revitalize_pass{6,7,8,9,10,11,12,13}_frozen.rs`. Pass 13 added fields to all of them (precedent: `harness_revitalize_pass13_frozen.rs` L160, L184). The probe summary's `rules` block is hand-built (`probe/pass8.rs` ≈ L37–51), so a new rules field adds no summary key; I1/I3 stay comparable.
- **No pins of `game_v19` exist yet** (`harness_revitalize_pass13_frozen.rs` pins v18 at L241, L281). Part 0 adds them before any change.
- **Performance:** `AgentDecisionSystem` is 54–69 % of every tick (`docs/revitalize/PERF_BUDGET.md` 1.2); the rule of its section 2 applies to this pass (8.5). Do not add per-agent `BTreeMap` lookups in the decision loop; read `Temperament` in the queries that already visit each agent.

## 3.2 The design (why this shape)
- **Cloninger's temperament model** (Cloninger, Svrakic & Przybeck 1993, *Arch Gen Psychiatry* 50:975) describes four heritable dimensions: **novelty seeking (NS)** — exploration, impulsiveness; **harm avoidance (HA)** — caution, worry and *fatigability* (its HA4 subscale); **reward dependence (RD)** — sociability, attachment, need for others' approval; **persistence (P)** — perseverance despite fatigue. Twin studies put their heritability around 0.3–0.6. This pass uses the four axes as permille values (0–1,000, mean 500), with a heritability of 0.5.
- **One visible lean per axis, on an existing quantity with a real footprint** (the code survey ruled out levers that almost never fire): NS → how far a person wanders when idle (and who goes with a splitting village); HA → how early one goes to bed (fatigability); RD → how fast one gets lonely (need for company), hence how often one seeks it; P → how full a load one gathers before carrying it home. Each changes a value the code already has, per person, so the mechanism is readable and cheap.
- **Magnitudes are moderate on purpose:** the village must stay healthy (T3). One tuning round is allowed if T1 fails for an axis (8.3).

---

# 4. RULES

## 4.1 Nothing old changes
Every scene `game_v8` … `game_v19` (and `_nofission` / `_neutral` variants), the recorder and the stress world behave bit-for-bit as before: proved by the v19 pins (5.1) and identity checks I1–I3 (as Pass 15: probe summaries of `game_v19` and `game_v16`, `--seeds 0..32` (32 worlds), 120 days, identical except `timing`; recorder `game_v19` 12000–12001 array sha256 identical) run on the base and the final build.

## 4.2 Seeds
`game_v20` evaluation: **16000–16031** (32 worlds). Tuning round(s): **16100–16115** (16 worlds). Not used before; not in tests (tests use seeds < 1000). The live game keeps `DEFAULT_GAME_SEED` 20,261,010.

## 4.3 Units and arithmetic
All temperament values are integers in permille (0–1,000). Every derived threshold is computed with integer arithmetic or plain `f64` multiplication/division of integers (no forbidden math), and is a pure function of (rule constants, the agent's temperament) — written once in `sim-core` (6.2) and unit-tested.

## 4.4 Pre-registration
The T table (section 9) is committed in METRICS.md under `## Pass 16` **before any `game_v20` probe on seeds 16000–16031 runs**. Tuning uses 16100–16115 only.

## 4.5 The gate
`tools/revitalize/gate_pass16.sh <log_dir>` = `gate_pass15.sh`'s steps unchanged (including `forbidden_math` and `perf_quick`), with `allowed_paths` checking section 2's list against `da8e45c7`. Gate on the Mac under 1.2's rules (not while the art run holds the quiet lock).

---

# 5. PART 0 — Pins first, then the base identity

## 5.1 Pin `game_v19` before touching anything
Add `rust/crates/sim-test/tests/harness_revitalize_pass16_frozen.rs` following `harness_revitalize_pass13_frozen.rs`'s pattern: for `game_v19` and `game_v19_nofission`, seeds 0–4, the probe fingerprint and `state_digest` after 1,440 and 40,320 ticks, values recorded from the **unchanged** code (commit them in a commit of their own before any other Rust change). Also pin `WorldRules::game_v19(s)` for s in 0–4 **field by field with individual `assert_eq!`s** — never as a struct literal (a literal would become a ninth file needing 5.2's field) — so 5.2's new field cannot silently change v19.

## 5.2 The one rules field
`AgentRules` gains `pub temperament: Option<TemperamentRule>` (default `None` in `Default` and `game_v8`, hence `None` in every existing scene). Update the whole-struct literals: `rules.rs` (≈ L499, L1030, L1388; grep for any other `AgentRules {`) and the eight frozen tests listed in 3.1 (`temperament: None`, one line each, as Pass 13 did). Nothing else changes in those tests.

## 5.3 Base identity
In a separate worktree `../new-world-p16-base` detached at `da8e45c7`, build the release binaries (Mac, under 1.2) and run I1–I3 into `docs/revitalize/probe/p16-0_base/` (summaries committed; recorder outputs under `/Users/rexxa/work/p16-data/`, their sha256 list committed). Record Pass 15's base files for comparison (`p15-0_final/`). Push.

---

# 6. PART 1 — Temperament and `game_v20` (Rust)

## 6.1 The component (`sim-core/src/components/temperament.rs`)
`#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)] pub struct Temperament { pub ns: u16, pub ha: u16, pub rd: u16, pub p: u16 }` (permille). Doc comments cite section 3.2.
- **Founder draw** `Temperament::founder(seed: u64, lineage_key: u64) -> Self`: for axis a ∈ {0 NS, 1 HA, 2 RD, 3 P}, `value_a = (u1 + u2 + u3) / 3` (integer division) with `u_s = keyed_below(seed, STREAM_TEMPERAMENT, lineage_key, 0, 4·a + s, 0, 1001)` for s ∈ {1, 2, 3} (replicate 0: an identity trait, never redrawn by a noise fork). Mean 500, standard deviation ≈ 167.
- **Newborn** `Temperament::child(seed: u64, child_key: u64, parents: [Option<Temperament>; 2]) -> Self`: `fresh = Temperament::founder(seed, child_key)`; if both parents are known, `mid_a = (pa_a + pb_a) / 2`, else if one is known, `mid_a` = that one's, else the child is `fresh`. Then `value_a = clamp(500 + (mid_a − 500) / 2 + (fresh_a − 500) × 935 / 1000, 0, 1000)` (integer arithmetic, `i32`; heritability 0.5; the factor 0.935 = √(1 − 0.5²/2) keeps the population's variance stable across generations when parents are independent draws). Unit tests: bounds; determinism; founder mean within 500 ± 5 and SD within 167 ± 5 over 20,000 keys; child slope on mid-parent ≈ 0.5 (regression over 20,000 synthetic triples within 0.45–0.55); child SD within 167 ± 10 for independent parents.
- New stream `STREAM_TEMPERAMENT` (an ASCII-tag constant like the others, distinct) added to `STREAMS` (length 16 → 17; the distinctness test covers it).

## 6.2 The rule and the per-person values (`sim-core/src/rules.rs`, pure functions next to `TemperamentRule`)
`#[derive(Clone, Copy, Debug, PartialEq, serde::Serialize, serde::Deserialize)] pub struct TemperamentRule { pub effects: bool, pub ns_tether_span: i32, pub ha_bedtime_span_minutes: i32, pub rd_social_rate_span_permille: i32, pub p_food_haul_span: i32, pub p_wood_haul_span: i32, pub ns_departure_weight: u32 }` with `game_v20` values `effects: true, ns_tether_span: 4, ha_bedtime_span_minutes: 60, rd_social_rate_span_permille: 500, p_food_haul_span: 2, p_wood_haul_span: 1, ns_departure_weight: 1000`. With `d_x = x − 500` as `i32` (−500 … +500) and `/` the `i32` division (truncation toward zero, symmetric):
- **NS → idle tether:** `tether(base, t) = max(2, base + span × d_ns / 500)` → base 8 gives 4 … 12. Applies to the adult branch's fallback `rules.idle_tether_radius` (`movement.rs` adult branch `evening_radius.or(rules.idle_tether_radius)`) whatever the hour; the evening gather radius, children and elders are unchanged.
- **HA → bedtime (fatigability):** `bedtime(start, t) = start − span × d_ha / 500` minutes → 21:00 gives 20:00 (most cautious) … 22:00 (boldest). For every person (children and elders included) **their own night window** — bedtime to the unchanged end 06:00, on the clock with `clock_offset_ticks` — replaces **every clock-filtered night-rest read** for that person: the filtered `night_rest` of L636–637 and the threshold of L638 through each of its uses (L707, L858, L1051, L1065, L1267–1273, L1674, L1697, L1744, L1784), **the sleep branch at L2075** (otherwise an early sleeper gets the legacy one-shot rest, fatigue −30 then Idle, and naps on and off until 21:00), and the peer filters at L913 and L930 (the peer's own window). The **unfiltered** reads of `rules.agent.night_rest` stay as they are (L1852, L2260, L2271; `beds.rs` L23, L92). The probe's global `fatigue_threshold_at` (`probe/metrics.rs` L67, L108, L204) is not changed; note in the report that v20's idle-breach numbers count bold people as breached between 21:00 and 22:00. Waking, `wake_preempt` and the meal wait are unchanged. A unit test: a cautious person who goes to sleep at 20:00 stays in `Consuming{Sleep}` until woken by the usual rules (no 20:00–21:00 nap cycling).
- **RD → loneliness rate:** `social_rate(base, t) = base × (1000 + span × d_rd / 500) / 1000` → 0.04 gives 0.02 … 0.06 per tick; set once by **mutating the `Social` component's growth rate right after the temperament is drawn** (it is created only at `camp.rs` L137 and `settlement_system.rs` L861, in the bundle inserted before the draw; maturation does not reset it; mutation changes no archetype), as a multiplication of `f64` by an integer ratio. The threshold (50) and the consumed amount (30) are unchanged — the rate is what sets how often a person seeks company.
- **P → load before going home:** `food_haul(base, t) = clamp(base + span_f × d_p / 500, 1, 8)` → 4 gives 2 … 6; `wood_haul(base, t) = clamp(base + span_w × d_p / 500, 1, 8)` → 2 gives 1 … 3 (inventory capacity is 10). Workers only (the only people who haul). Applies at every production read, all per-person: decision L699, L701 (builder check of the agent's own inventory), L1031–1032 (`loaded`), L1503, L1528, L1560 (forage / wood / gather continuation), L2039 (wood pickup amount); movement's `hauls_home` L110, L130–131 (`firewood.rs` L325, `movement.rs` L398 and decision L2499 are test fixtures). Record the list as found. Note: a food pickup takes up to the inventory's room, limited by the bush (≈ 4 per bush), so the food threshold acts mainly on whether a person goes on to a further bush before hauling home.
- **NS → who leaves with a split:** the rank key becomes `(keyed_unit_f64(seed, STREAM_FISSION, lineage key, tick, 0, rep) − weight × ns / 1,000,000, AgentId)`, sorted with `f64::total_cmp` and the AgentId tie-break (weight 1,000: NS 1,000 subtracts 1.0 — about half the U(0,1) range per 500 permille); size and the four-adult rule unchanged. With effects off the old `Vec<(u64, AgentId, bool)>` path and `sort_unstable()` stay **verbatim**.
- With `effects: false` every function returns the base value: that is `game_v20_inert` (6.3), the placebo.
Unit tests for each function at d = −500, 0, +500 (exact expected integers) and for `effects: false`.

## 6.3 Scenes
- `WorldRules::game_v20(seed)` = `game_v19(seed)` with `agent.temperament = Some(V20_TEMPERAMENT)`; `game_v20_inert(seed)` the same with `effects: false`. `WorldGenConfig::game_v20(seed)` = `game_v19(seed)` (no worldgen change). Rules tests: `game_v20(s) == game_v19(s)` except `agent.temperament`.
- Wire both into `sim-test/src/scenes.rs` and the probe's scene names (`probe/args.rs`, `probe/runner.rs`, as Pass 13 wired `game_v19`); **not** into the recorder.
- **Inert equals v19:** a test that `game_v20_inert` and `game_v19` of seeds 0–4 have the same probe fingerprint and `state_digest` after 40,320 ticks; and, because no fission happens before day 28, an **I1-style 120-day check** in Part 2: the probe summaries of `game_v20_inert` and `game_v19`, `--seeds 0..32`, identical except `timing` and the scene name. Together they prove that drawing and storing temperament changes nothing by itself.
- Probe wiring: flip the existing assertion that `game_v20` is an unknown scene (`probe/args.rs` ≈ L153) and add the new scenes wherever the runner lists scenes (e.g. `twin_checked`, `runner.rs` ≈ L407).

## 6.4 Wiring the behaviour
a. **Insert** `Temperament` exactly where `Hardiness` is inserted (founders `worldgen/camp.rs`, newborns `settlement_system.rs`), only when `rules.agent.temperament.is_some()` and keyed draws are on; newborns from the child's lineage key and the `Temperament` of the two `parent_ids` the birth already names. In the same places, with effects on, set the `Social` rate (RD).
b. **NS tether** in `movement.rs`'s idle walk.
c. **HA bedtime, P haul** in `agent_decision.rs` and `movement.rs` at the sites 6.2 names — read `Option<&Temperament>` in the queries that already visit each agent; no new per-agent map lookups.
d. With `temperament == None` the code path must be the old one exactly (the pins and the inert checks prove it).
e. **NS departure** in `fission.rs` L550–574.

## 6.5 Per-person measurement (`sim-test/src/probe/`)
A probe flag `--agent-trace` (off by default; existing outputs unchanged when off) writes `<scene>_s<seed>_agents.jsonl`: one line per agent who was ever a non-elder adult, with id, lineage key, founder or born tick, the temperament (when the component exists; for `game_v19` runs, the values `Temperament::founder` / `child` **would** give — computed in the probe from the lineage key and the `AgentBorn` parents, read on the birth tick at the newborn's tile because each tile's causal ring keeps only 8 events; count and report misses — never inserted), non-elder adult days observed, and these counters, kept by an observer of components and state transitions (no change to systems), all over **non-elder adult** time only:
- `idle_dist_sum`, `idle_samples`: Chebyshev distance from the person's settlement hearth, sampled every 10 ticks while Idle between 06:00 and 18:00 (members of a settlement);
- `bed_minutes` (list): for each night, the clock minute (from `tick + clock_offset_ticks`; minutes after 18:00 as 1,080…1,439, after midnight as 1,440 + m) of the first tick between 18:00 and 03:00 at which the person is `Consuming{Sleep}`; nights without such a tick are not counted;
- `social_completed`: interactions the person completed — a transition out of `Consuming{Agent}` with a loneliness drop of ≥ min(25, loneliness before the tick) (decay runs after the interaction, so a full interaction drops 30 minus the growth rate; memory-bias partners can start below 25);
- `deposit_food` and `deposit_wood` (sum and count each): at every deposit — a tick where the person's carried food (or wood) drops while within the settlement's deposit radius of its stockpile — the amount carried on the tick before (food and wood recorded separately; a drop elsewhere, e.g. eating, is not a deposit);
- `departed`: whether the person left in a completed fission (`FissionRecord.members`; cancelled fissions are excluded and counted) and, for every completed fission, the eligible (non-elder) members at the departure tick.
Unit tests of the observer on a scripted engine (one agent's counters checked by hand for each kind).

## 6.6 The bridge
`world_node.rs`: `#[func] fn get_agent_temperament(&self, entity_bits: i64) -> VarDictionary` → `{found: bool, ns, ha, rd, p}` (ints; `found: false` and zeros when the agent or the component is missing), backed by a pure `collect_agent_temperament(world, entity_bits) -> Option<Temperament>` with unit tests. The live game (`WorldSimNode::init` ≈ L187–191 and `reset_world` ≈ L228–233) builds `game_v20`; `reset_world_stress` stays on `game_v19` (Pass 15's benchmark). Displaying temperament is the art line's job (a note for it in the report).

Commit, push (debug and release `cargo test --workspace` green first, under 1.2).

---

# 7. PART 2 — Proofs
1. Final-build identity I1–I3 against the base (5.3): identical; the v19 pins (5.1) pass; both inert checks of 6.3 (fingerprint and digest at 40,320 ticks; the 120-day summaries on DGX, under 1.3) identical. A difference stops the pass until fixed.
2. Commit the T table (section 9) to METRICS.md (4.4). Push.

---

# 8. PART 3 — Runs and tuning

## 8.1 Builds on the DGX
Clone, detach at the pushed commit, `cargo build --release -p sim-test` (record sha256 of `worldsim_probe`).

## 8.2 Tuning round 1 (seeds 16100–16115, 60 days, `--agent-trace`)
`game_v20` and `game_v20_inert`. Compute T1's correlations and T3's health on these seeds only (`tools/revitalize/evaluate_pass16.py`, numpy from the venv on the DGX or standard library on the Mac). T1-dep is not evaluated on tuning runs. If every other T1 axis meets its bar and T3 holds, freeze the spans.

## 8.3 Tuning round 2 (only if needed; once)
For an axis that misses T1's bar on the tuning seeds, raise its span once: tether 4 → 6; bedtime 60 → 120; social rate 500 → 750 permille (0.01 … 0.07); food haul 2 → 3 and wood haul 1 → 2 (the wood value clamps at 1 from below). The departure weight is **not** tuned (60-day runs see almost no fission). For a T3 failure, halve the span of the axis whose ablation (rerun with that axis's span 0) recovers the most health. Rerun 8.2's seeds. Freeze whatever results; record both rounds. **No tuning after 8.4 starts.**

## 8.4 Evaluation (seeds 16000–16031, 120 days, `--agent-trace`)
`game_v20`, `game_v20_inert` and `game_v19` (the probe's usual flags as Pass 13 round 3 plus `--agent-trace`), under 1.3. Then `evaluate_pass16.py` → `docs/revitalize/probe/p16-4_eval/eval.md` + `eval.json`.

## 8.5 The performance rule (`PERF_BUDGET.md` section 2, on the Mac, quiet lock held)
`worldsim_perf stress` parses `--scene` but ignores it (`perf.rs` ≈ L103 builds `game_v19` rules; ≈ L738 labels `stress_v19`). Add `--rules game_v19|game_v20|game_v20_inert` (default `game_v19`, today's behaviour; the label records it). Then, same day, same Mac, seed 15100, n = 500, 1 day, fed:
(a) **code cost:** `game_v20_inert` vs `game_v19` — fails if the mean tick rises by more than **15 %**; (a′) **behaviour + code:** `game_v20` vs `game_v19` — reported, fails only above 4.0 ms; (a″) `worldsim_perf quick` before (base build) and after (final build) as `PERF_BUDGET.md` asks.
(b) the three-size stress slope of `AgentDecisionSystem` under `game_v20` rules: `--seeds 15100..15101 --agents 100,500,1000 --days 2 --feed fed` (the reference's own design), against Pass 15's reference 1.06.

---

# 9. THE T TABLE (pre-registered; committed before 8.4)
Correlations are Spearman's ρ (average ranks for ties) over **non-elder adults** pooled across the 32 evaluation worlds (≥ 10 non-elder adult days; for idle distance ≥ 50 samples; for bedtime ≥ 10 nights; for loads ≥ 10 food deposits), with a 95 % interval from a world-level bootstrap (B = 2,000, `default_rng(0)`).

| # | question | rule |
|---|---|---|
| T1-NS | wanderers wander | ρ(NS, mean idle distance) ≥ **0.25**, interval excludes 0, in `game_v20` |
| T1-HA | the cautious go to bed earlier | ρ(HA, mean bed minute) ≤ **−0.25**, interval excludes 0 |
| T1-RD | the sociable socialise more | ρ(RD, social interactions per non-elder adult day) ≥ **0.25**, interval excludes 0 |
| T1-P | the persistent bring fuller loads | ρ(P, mean food load at deposit) ≥ **0.25**, interval excludes 0; ρ(P, mean wood load at deposit) reported (no verdict) |
| T1-dep | explorers leave | mean NS of departers − mean NS of eligible stayers (pooled over completed fissions) ≥ **80** permille, interval excludes 0 (planner's estimate at weight 1,000: ≈ 135) |
| T2 | the placebo shows nothing | in `game_v20_inert` **and** in `game_v19` (probe-computed values) every T1 statistic has \|ρ\| ≤ **0.10** (departure difference \|·\| ≤ 40) |
| T3 | the village stays healthy | paired over the 32 worlds, `game_v20` − `game_v19`: mean alive at day 120 within **±15 %** of v19's; **non-old-age deaths** (starvation, dehydration, exposure, combat) per world ≤ **+1.5** (old-age deaths, ≈ 80 per world, are reported, not judged); fissions per world and first-split day reported with intervals |
| T4 | inheritance works | slope of child on mid-parent (all births with two known parents) within **[0.35, 0.65]**; each axis's SD over alive agents at day 120, pooled across worlds, within **±20 %** of the founders' |
| T5 | the old worlds are untouched | I1–I3, the v19 pins and both inert checks identical |
| T6 | the budget holds | 8.5 (a) and (b) pass |

---

# 10. PART 4 — Analysis
`docs/revitalize/probe/p16-4_eval/eval.md`: the T table with values and intervals; per axis a binned plot-free table (axis quintile → behaviour mean) for v20, inert and v19; health tables; what the tuning rounds changed; per-world founder temperament means vs outcomes (alive at day 120, first split day) as a descriptive table (no verdict — the seed of a later "founders shape the village" question). Commit, push.

# 11. PART 5 — Docs
`docs/revitalize/TEMPERAMENT.md` (English): the model, the four leans with their formulas and final spans, what is not yet modelled (real parents, the full pipeline of CLAUDE.md, mutation by life events, display), and a short note for the art line (the getter, suggested display: four bars or the dominant axis, all text through `Locale.ltr`). Commit, push.

# 12. PART 6 — Review, gate, report
1. Independent review (fresh `general-purpose` subagent, two rounds maximum): the pins were committed before any other Rust change; v19 identity (I1–I3, pins, inert = v19); no new field outside 5.2; the arithmetic of 6.1–6.2 (bounds, integer math, no forbidden math); each hook sits where 6.2 says and nowhere else (NS only on the adult fallback tether; HA only on the start of the night-rest window; RD only on the `Social` rate right after the draw; P only at the listed load sites, workers only; HA on every clock-filtered night-rest read and no unfiltered one; the departure path verbatim with effects off); the observer's counters on one agent recomputed by hand from a short traced run; the pre-registration order; the tuning rounds used only 16100–16115; T verdicts follow section 9; the quiet-lock holds; allowed paths. Findings in `docs/revitalize/REVIEW_PASS16.md`.
2. Gate (4.5) → `docs/revitalize/verification/p16_gate/`; fast-forward `codex/revitalize-2026-10` if green; a Pass 16 section in PR #510's description; remove `../new-world-p16-base`.
3. `docs/revitalize/REPORT_PASS16.md` in Korean (section 13 format) and the same text in the final message.

---

# 13. 인게임 확인사항 (오너용)
- 게임이 `game_v20`으로 시작한다. 같은 시드라도 이전과 장면이 달라진다(사람마다 선택이 달라졌기 때문).
- 낮에 할 일 없이 쉬는 사람 중 일부는 화덕에서 멀리(최대 약 12칸) 돌아다니고, 일부는 화덕 곁(약 4칸)에 머무는지.
- 밤에 일찍(저녁 8시쯤) 잠자리에 드는 사람과 늦게(10시쯤)까지 깨어 있는 사람이 섞여 있는지.
- 덤불이나 숲에서 한 번에 많이 들고 오는 사람과 조금만 들고 자주 오가는 사람이 섞여 있는지.
- 사교적인 사람이 더 자주 다른 사람과 붙어 이야기하는지(같은 칸에 붙어 있는 장면).
- 마을이 갈라질 때 떠나는 무리에 돌아다니기 좋아하던 사람이 많은지.
- 마을이 예전처럼 유지되는지(굶어 죽는 사람이 갑자기 늘지 않는지). 콘솔 에러가 없는지.
- 기질은 아직 화면에 표시되지 않는다(다음 아트 패스의 일). 숫자는 `get_agent_temperament`로만 볼 수 있다.

---

# 14. 구현 완료 보고 (REPORT_PASS16.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
왜 이 작업을 했는가? (문제 → 해결 방향 한 문장)

### 구현 내용
무엇을 만들었는가? (기질 구성요소, 규칙, game_v20/inert, 네 가지 기울기(배회 반경·취침 시각·외로움 속도·운반량)와 분열 이탈, 개인 추적, 브리지 게터)

### 구현 방법
추첨과 유전 공식, 사람별 값 공식과 최종 폭, haul_threshold 사용처 분류, 동작 불변 증명(v19 고정값, I1~I3, inert=v19), 관찰자 계측, 사전 등록과 조정 라운드

### 기능 설명
게임에서 사람들이 어떻게 달라졌는가? 무엇이 아직 없는가?

### 변경된 파일 목록
(커밋별)

### 측정 결과
T1~T6 표(값과 구간), 축 5분위별 행동 평균(v20·inert·v19), 건강 표(생존·사망·분열), 유전 기울기와 분산, 조정 라운드 기록, 성능 규칙 결과(v20 대 v19 평균 틱, 의사결정 시스템 기울기)

### 환경
맥·DGX 사양, 조용함 잠금 기록과 대기, DGX 부하

### 독립 리뷰
지적 사항과 처리 결과

### 자율 결정 목록
DECISIONS.md Pass 16 요약

### 실행 기록
Part별 시작·종료 시각과 소요 시간, 대기 구간

### 확인된 제한사항
(가짜 부모(가장 낮은 id 두 명) 문제 포함)

### 게이트 결과
(단계별 결과, 테스트 수치, 허용 경로 밖 변경 없음)

### 정직한 판단
사람마다 다른 행동이 실제로 보이는가? 마을은 건강한가? 다음은 가족(진짜 부모)인가, 아니면 다른 것이 먼저인가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #510 update, the DGX paths, and the time spent per Part.
