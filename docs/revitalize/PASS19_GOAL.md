# GOAL: WorldSim — Revitalize Pass 19: a chronicle of every life (write-only, behaviour-preserving), the end of the 18:00 stampede (`game_v24`), and a scale check

Repository `hyunlord/new-world`. Continue the simulation line: `codex/revitalize-2026-10` = `4ef34b97d165720d030469e4e6f0ab042a0d5214` (Pass 18, gate green), `codex/revitalize-2026-10-wip` = `20a1d07892f7127668a9c2d3c4e649d4a34d352e` (Pass 18's final gate record on top). Draft PR #510.
This run is executed by **Claude Code**, unattended, on the Mac, in `/Users/rexxa/work/new-world-pass4`. An art run (Codex, `/Users/rexxa/work/new-world-art-prod`) may be measuring frame times at the same time; section 1.2 is how the two runs share the Mac. No DGX.

Three parts, in this order:
1. **The chronicle** — an append-only record of every person's life (birth and parents, founding, pairing, coming of age, old age, leaving with a split and why, settling a daughter village, widowhood, death and why) and of every village's (formed, split, dissolved), plus a **person table** that outlives death (today a dead person's entity is despawned at once and everything about them is gone, so a family tree cannot show a dead grandmother). Nothing in the simulation reads it: every scene stays **bit-identical**. The bridge exposes it for the art line's chronicle and "why?" UI (roadmap step 4).
2. **`game_v24`** = `game_v23` + each person ends their day's activity at their own time (17:00–18:59, later for the persistent) instead of everyone at 18:00. This removes the 18:00 tick spike that kept Pass 18's R5 tail failing (+37–63 % in six of eight runs), and makes temperament visible at dusk. The live game moves to `game_v24`.
3. **A scale check** (roadmap step 3) — `game_v24` at 500, 1,000 and 2,000 people: tick cost, its slope per system, memory, and what it means for the live game's speeds. Findings only; fixes go to a later pass.

---

# ⚠️ 0. THIS IS A CLAUDE CODE RUN — READ FIRST

## 0.1 How the owner launched you (for your information; do not repeat it)
```bash
cd /Users/rexxa/work/new-world-pass4
git fetch origin && git checkout codex/revitalize-2026-10-wip
git merge --ff-only origin/codex/revitalize-2026-10-wip   # 20a1d078
cp ~/Downloads/claude_goal_worldsim_pass19.md docs/revitalize/PASS19_GOAL.md
claude --dangerously-skip-permissions
# then: /goal <short completion condition naming docs/revitalize/PASS19_GOAL.md>
```
The clone's hooks are neutralized as before (`.claude/settings.json` = `{}`, `--skip-worktree`); never edit, stage or commit it and never clear the flag. Leave the local `.gitignore` edit and `.ignore` uncommitted. Stage explicit paths only; never stage outputs or gate logs of earlier passes.

**This document is `docs/revitalize/PASS19_GOAL.md`.** Commit it unchanged in Part 0. Re-read the relevant section at the start of every Part and after every compaction or pause. The owner drives the run with `/goal`: a small model reads only the transcript, so the section 11 report, the C/H/S tables, commit hashes and `git ls-remote` heads must appear in your final message.

## 0.2 Overrides of CLAUDE.md
As Pass 18's 0.2: no `install_hooks.sh`, never `lead/main` or `main`, no `harness_pipeline.sh` (4.4 is the gate), built-in `general-purpose` subagents only (for the independent reviews: a fresh subagent given only this document, the diff and the outputs), a blocking hook other than `/goal` → record, push, report. First command: `ls .git/hooks/pre-commit .git/hooks/post-commit 2>/dev/null; git ls-files -v .claude/settings.json; cat .claude/settings.json`. Pass 18 was run by Claude Code in this clone: `docs/revitalize/PASS18_GOAL.md`, `REPORT_PASS18.md`, `DECISIONS.md` (Pass 18) and `tools/revitalize/*_p18*` are the reference for every procedure cited here.

## 0.3 Autonomy
1. **Never ask the owner anything.** Decisions and reasons in `docs/revitalize/DECISIONS.md` under `## Pass 19`, with a RUNLOG table (Part, start, end, notes).
2. **Do not end your turn between Parts or while a measurement or probe batch runs.** Poll long jobs every 10–20 minutes with short calls.
3. **Parts:** 0 = section 5; 1 = section 6 (chronicle); 2 = section 7 (`game_v24`); 3 = section 8 (scale); 4 = section 9 (review, gate, report). Push after every Part.
4. **Review loops: two rounds maximum.**
5. **On failure: work around, note it, continue.** A bar missed is reported as missed; a missing number is not acceptable.
6. **Time rule:** past **12 hours** of working time (pauses excluded and recorded), finish the current step, push, go to Part 4.
7. **Drop order:** the 2,000-person runs of Part 3 → the `subprofile` attribution of Part 3 → `get_chronicle_since` (6.5). Never drop Parts 0, 1, 2, 4, the identity checks, C1–C4 or H1–H6.

---

# ⚠️ 1. MACHINE
## 1.1 The Mac
Only `/Users/rexxa/work/new-world-pass4` and one base worktree `../new-world-p19-base` (5.2). Never touch other checkouts or processes you did not start; no DGX. `CARGO_BUILD_JOBS=6`, tests `--test-threads=6`, probes `--workers 4`. Data in `/Users/rexxa/work/p19-data/`.
## 1.2 Sharing the Mac with the art run (as Pass 18's 1.2, tightened)
Quiet lock `/Users/rexxa/work/.mac_quiet.lock` (`mkdir`; `info.txt` with owner, pid, purpose, started; removed when done). Hold it **only while measuring time** (6.6, 7.6, Part 3), ≤ 60 minutes per hold, ≥ 10 minutes between your holds. While the other run holds it: **no cargo build, test, clippy or gate step of any kind, no probe, no Godot**. Pass 18's gate already checked the lock before every step (`gate_pass18.sh` ≈ L16–27 via `wait_mac_p18.sh`) and still ran its debug test step through an art hold (04:39:25–04:39:56), because a step that has started keeps running. So in Pass 19 every gate step, probe batch and build runs in its own process group, and a watcher polling every 10 s sends `kill -STOP -<pgid>` when a foreign lock appears and `kill -CONT -<pgid>` when it goes; stopped intervals are logged, and a timed step that was stopped (`perf_quick`) is re-run. The p19 copies of `wait_mac_p18.sh` / `quiet_lock_p18.sh` rename the owner string (`pass19-sim…`), the `P18_QUIET_*` environment names and the data path. Poll every 5 minutes; a lock older than 2 hours is stale (remove, record). Before taking it: no `cargo`/`rustc`/`worldsim_*`/`Godot` process of the other run (`pgrep -x`) and 1-minute load < 4; after 60 minutes of waiting take it anyway and mark the runs `contended`. Record every hold and wait. Reviewer subagents run nothing while you hold the lock.

---

# 2. GIT
- Commits to `codex/revitalize-2026-10-wip`; `codex/revitalize-2026-10` moves only by fast-forward to a WIP commit whose gate (4.4) is green; `git ls-remote` after every push. Messages `revitalize(p19-N): …`.
- **May change:** `rust/**`, `tools/revitalize/**`, `docs/revitalize/**`. **Must not change:** `scripts/**`, `scenes/**`, `assets/**`, `localization/**`, `docs/art/**`, `tools/art_pipeline/**`, `tools/worldmodel/**`, `docs/worldmodel/**`, `rust/crates/sim-test/src/worldmodel/**`.
- **New public items allowed:** the new module `sim_core::chronicle` (6.1); one new public field `SimResources::chronicle` (`sim-engine/src/lib.rs`, the struct at ≈ L228; it has no derives and one constructor, `SimResources::new` ≈ L616); one new public field `RoutineRule::end_stagger`, the struct `EndStagger` and the method `RoutineRule::end_minute` (7.2; the only struct literal is `V21_ROUTINE` in `rules.rs` ≈ L883 — the test at `rules/routine.rs` ≈ L198 uses `..rule`); `WorldRules::game_v24`, `WorldGenConfig::game_v24`, the scene and `stress_v24` label wiring (7.1); `households::select_departure_explained` and `DepartCause` (6.3); the probe flag `--chronicle-out` (6.4); the bridge getters of 6.5 and their pure collectors in a new `sim-bridge/src/ffi/chronicle.rs`. No other new public field in an existing public struct; no change to existing getters' output; `FamilyEvent` and `FissionRecord` unchanged; `households::select_departure`'s signature and result unchanged (6.3). Private signatures may gain parameters where 6.2 or 7.3 need them (e.g. `run_formation_scan`, `can_enter`, `deadline`).
- The word "chronicle" already appears in comments and test names about `causal_log` (e.g. "Chronicle the death at its tile", `survival/mod.rs` ≈ L103; tests `harness_p*_chronicle.rs`). They are unrelated to this pass: leave them as they are.

---

# 3. CONTEXT (planner's survey at `20a1d078`, paths under `rust/crates/`; verify each fact)

## 3.1 Where lives happen today
- **Birth:** `SettlementSystem` (priority 138) → `run_births` (`sim-systems/src/runtime/settlement/settlement_system.rs` ≈ L710–998): parents from `pairing::birth_parents` (a mutual couple, `agent/pairing.rs` ≈ L158–199) or the fallback (two lowest member ids, ≈ L802–820); spawn ≈ L861–865 with `resources.issue_agent_id()`; `Kin { mother, father, .. }` ≈ L918–927; `Child { born_tick }` ≈ L950; `Life` ≈ L952–962; roster ≈ L984; `FamilyEvent::Born` ≈ L933–945. Founders come from worldgen (`sim-systems/src/worldgen/camp.rs` ≈ L144–190) with `Kin::default()` and a negative `Life.born_tick`; no birth event.
- **Pairing:** `PairingSystem` (`agent/pairing.rs` ≈ L12–134, priority 144) at clock minute 1200 (20:00); familiarity from `resources.relationships`; both `Kin.partner` set ≈ L114–128; `FamilyEvent::Paired`. A partner who died is cleared at the next 20:00 run (≈ L57–62); no divorce; elders never pair again; fission never clears it.
- **Death:** every path ends in `despawn_agent` (`survival/mod.rs` ≈ L56–129), which **despawns the entity immediately** (≈ L65) and then cleans maps and rosters and pushes `CausalEvent::AgentDied`. Reasons: starvation / dehydration (`survival/mod.rs` ≈ L155–211, `StarvationSystem` priority 139), old age (`agent/aging.rs` ≈ L106–126, priority 146, every 60 ticks), exposure (`settlement/fire.rs` ≈ L189–233, priority 147, every 60), combat (`combat/combat_system.rs` ≈ L219–242; practically unreachable in these scenes).
- **Life stages:** `MaturationSystem` (`agent/maturation.rs`, priority 144, every 60) removes `Child` at `child_ticks` = 5,760; `AgingSystem` inserts the `Elder` marker at `elder_age_ticks` = 66,240. No transition event exists.
- **Fission:** `FissionSystem` (`settlement/fission.rs`, priority 148) decides at 06:00 in `check` (≈ L441–720): ranking ≈ L557–588 (elders excluded), k ≈ L589, `households::select_departure` (`agent/households.rs` ≈ L78–160; phases: household build incl. v23's elder partners ≈ L95–110, seed ≈ L117–132, fill ≈ L133–144, top-up ≈ L145–152) returning only `Option<BTreeSet<AgentId>>`; apply ≈ L628–680 (`Founding` component, `FissionRecord` into `resources.fission_log`); `advance` / `stamp_camp`; the daughter is minted by `SettlementSystem`'s formation scan (≈ L463, `issue_settlement_id`) and adopted by `FissionSystem::found` (≈ L266–308); `cancel` ≈ L244–262. Remnant merge (≈ L414–438) dissolves a ≤ 2-member settlement; its people become `SettlementMigrant`s until proximity sync admits them.
- **Membership:** `sync_membership` (`settlement_system.rs` ≈ L258–316) admits non-members within radius 5 (cap 50); `resolve_exclusivity` (≈ L335–393) keeps the nearest roster.
- **IDs:** `AgentId = u64`, monotonic, never reused (`sim-engine/src/lib.rs` ≈ L707–709); `SettlementId = u32`, monotonic. hecs `Entity` slots are recycled; the bridge takes entity bits as input and returns links as AgentIds (`sim-bridge/src/ffi/family.rs` ≈ L28–69).

## 3.2 Why the chronicle must be its own store
`resources.causal_log` is a ring of 8 events **per tile** (`sim-core/src/causal/ring_buffer.rs` ≈ L24) that memory (`memory/memory_system.rs` ≈ L199–228), decisions (`agent_decision.rs` ≈ L1872, 2470, 2503), social, combat and settlement history (`settlement_system.rs` ≈ L588) read back; a push evicts the oldest event of that tile and `issue_event_id` shifts every later id. `FamilyObservations` holds only the current tick's events (probe-only). `Memory` is capped at 32 and salience-evicted. None of them is a life record, and adding to any of them changes behaviour.

## 3.3 The 18:00 edge
`RoutineRule` (`sim-core/src/rules/routine.rs` ≈ L9–30; `V21_ROUTINE` in `rules.rs` ≈ L883–894): `day_start_minute` 420, `day_end_minute` 1080. Per person: `start_minute(key) = 420 + key % 60` (≈ L79–81), `duration_minutes(t) = max(20, 90 + 60 × (P − 500) / 500)` (≈ L71–76). `can_enter` (`decision/routine/mod.rs` ≈ L172–186) requires `start_minute(key) ≤ minute < day_end_minute`; `until_tick` is set only on arrival (`routine/lifecycle.rs` ≈ L241–256), unclamped; `deadline` (`lifecycle.rs` ≈ L183–207) ends an activity when `tick ≥ until_tick` **or** `minute ≥ day_end_minute`, so every active person exits on the tick the clock reaches 18:00 (`lifecycle::apply`, ≈ L40–64, fuses the moves). Pass 18 measured that tick at 0.82–0.84 ms against `game_v20`'s 0.49 ms, the cause of the R5 tail (`docs/revitalize/PERF_BUDGET.md`, "Pass 18"). Bedtime is per person already (`TemperamentRule::bedtime`, `rules.rs` ≈ L109–114: 20:00–22:00 by HA, applied by `person_night_rest`, `agent_decision.rs` ≈ L303–315) — the model to copy.

## 3.4 Scale today
`worldsim_perf stress --rules game_v23` builds `WorldGenConfig::stress_v19(seed, n, side)` with `WorldRules::game_v23` (`sim-test/src/perf.rs` ≈ L101–122); `fed` gifts 3 × alive food at 06:00. Settlements are capped at 50 members by proximity sync and the birth gate (`SETTLEMENT_MAX_POP`), but the formation scan adds every founder in its radius without a cap. Pass 18's slope runs used 100 / 500 / 1,000 people.

---

# 4. RULES
## 4.1 Identity
As Pass 18's 4.1 — probe summaries (`tools/revitalize/identity_p18.sh` flags), `--seeds 0..32`, 120 days, identical except `timing` / `wall_seconds` / `ms_per_agent_tick` between the base build (`20a1d078`) and a candidate: **I1** v19, **I2** recorder `game_v19` `--seeds 12000..12002` array sha256, **I3** v16, **I4** v20, **I5** v21, **I6** v22, and new **I7** v23 (copy `identity_p18.sh` to `identity_p19.sh` and add I7); all existing pins; the new pins (5.1). `rust/Cargo.lock` is git-ignored: copy the main clone's lock file into the base worktree and record `cargo tree -i hecs` in both.
## 4.2 Seeds
`game_v24` evaluation **19200–19231** (paired with `game_v23` on the same seeds; the `game_v23` runs of 6.6 on these seeds are reused as 7.6's baseline — run them once); smoke **19300–19303**. Before using them, `grep -rnE -- '--seeds +19(2|3)[0-9]{2}' docs/revitalize tools/revitalize` and `find docs/revitalize -name '*_s19[23][0-9][0-9][._]*'` must both be empty (they are at `20a1d078`; else shift both ranges by +1,000 and record).
## 4.3 Arithmetic and draws
As Pass 17 (integers, no forbidden math, keyed draws, new streams in `STREAMS`). The chronicle and the end-time rule use no draws.
## 4.4 The gate
`tools/revitalize/gate_pass19.sh` = `gate_pass18.sh`'s 15 steps unchanged, `allowed_paths` against `20a1d078`, with 1.2's process-group watcher.
## 4.5 Time
Identity sets are the long pole (one I1–I6 set took ≈ 70 minutes in Pass 18; I7 adds ≈ 23): run them three times only — base (5.2), the Part 1 build (6.6) and the final build (H6). The gate takes ≈ 70 minutes per run.

---

# 5. PART 0 — Pins and base
5.1 `rust/crates/sim-test/tests/harness_revitalize_pass19_frozen.rs`: for `game_v23`, seeds 0–4, probe fingerprint **and** `state_digest` after 1,440, 40,320 and 86,400 ticks from the unchanged code, one `#[test]` per seed (as Pass 18's file), and field-by-field asserts of `WorldRules::game_v23(s)` (no struct literal). Commit alone before any other Rust change.
5.2 Worktree `../new-world-p19-base` at `20a1d078`; release build; I1–I7 into `docs/revitalize/probe/p19-0_base/` (summaries committed; recorder outputs in `p19-data`, sha256 list committed). Push.

---

# 6. PART 1 — The chronicle (every scene bit-identical)

## 6.1 The store (`sim-core/src/chronicle.rs`, `pub mod chronicle` in `sim-core/src/lib.rs`)
```rust
#[repr(u8)] pub enum ChronicleKind {
    Born = 0,            // subject child; other mother; third father; settlement; cause 0 couple / 1 fallback parents
    FoundingMember = 1,  // subject; settlement (an original camp: formation of a settlement with no fission behind it)
    Paired = 2,          // subject the woman; other the man; settlement; value familiarity permille
    CameOfAge = 3,       // subject (Child removed)
    BecameElder = 4,     // subject (Elder inserted)
    Departed = 5,        // subject; settlement = the parent; other = the group leader; cause DepartCause; value fission id
    DepartureCancelled = 6, // subject; settlement = the parent; value fission id
    Settled = 7,         // subject; settlement = the daughter; settlement2 = the parent; value fission id
    Joined = 8,          // subject; settlement; cause 0 from no roster / 1 from another roster / 3 present when a daughter formed
    LeftSettlement = 9,  // subject; settlement; cause 0 remnant dissolved / 2 roster change (exclusivity or proximity)
    Widowed = 10,        // subject the survivor; other the deceased partner
    Died = 11,           // subject; settlement (last roster, if any); cause DeathReason::as_u8; value age in ticks (−1 without Life); other = Kin.partner as recorded (a partner who died earlier stays set until the next 20:00)
    SettlementFormed = 12,   // no subject; settlement; value founding member count
    FissionStarted = 13,     // no subject; settlement = parent; value fission id; other = leader
    DaughterFounded = 14,    // no subject; settlement = daughter; settlement2 = parent; value fission id
    FissionCancelled = 15,   // no subject; settlement = parent; value fission id
    SettlementDissolved = 16,// no subject; settlement; cause 0 empty, 1 remnant merge
}
#[repr(u8)] pub enum DepartCause { Ranked = 0 /* no family rule */, Seed = 1, Household = 2, ElderPartner = 3, TopUp = 4 }
pub struct ChronicleEntry { pub tick: u64, pub kind: ChronicleKind, pub subject: Option<AgentId>, pub other: Option<AgentId>,
    pub third: Option<AgentId>, pub settlement: Option<SettlementId>, pub settlement2: Option<SettlementId>, pub cause: u8, pub value: i64 }
pub struct PersonRecord { pub female: Option<bool>, pub born_tick: Option<i64>, pub mother: Option<AgentId>, pub father: Option<AgentId>,
    pub temperament: Option<[u16; 4]>, pub died_tick: Option<u64>, pub death_reason: Option<u8> }
pub struct Chronicle { entries: Vec<ChronicleEntry>, by_agent: BTreeMap<AgentId, Vec<u32>>,
    by_settlement: BTreeMap<SettlementId, Vec<u32>>, people: BTreeMap<AgentId, PersonRecord> }
```
`Chronicle::push(entry)` appends and indexes the entry under every `Some` agent (subject, other, third; once per distinct id) and settlement (settlement, settlement2). `ensure_person(id, record_fn)` creates a person record the first time an id appears (from the live components: `Sex`, `Life.born_tick`, `Kin` mother/father, `Temperament`; any absent component → `None`); it is called for every `Some` subject / other / third of every entry before the push (when that agent is alive), and for every founder in `worldgen::camp` populate after its components are inserted (every scene), so every agent that ever lived has a record. `close_person(id, tick, reason)` fills death (creating the record from the components read before despawn if it is still missing). Read accessors: `entries()`, `of_agent(id)`, `of_settlement(id)`, `person(id)`, `len()`. Every type `Debug + Clone`; enums carry stable `as_u8`. Unit tests for indexing, order and the person table.

## 6.2 Where it is written
**Order rule:** entries of one site are appended in AgentId order (then SettlementId). Wherever the code iterates a `HashMap` / `HashSet` (`SimResources::settlements` is a std `HashMap`, `sim-engine/src/lib.rs` ≈ L403; `Settlement::member_agents` a std `HashSet`, `components/settlement.rs` ≈ L106), collect into a local `Vec` and sort by (SettlementId, AgentId) before appending — a local sort is invisible to the simulation. **Borrow rule:** no `world.get` / `ensure_person` while a `world.query` iterator is alive; buffer `(entry, ids)` inside loops and push after them (Pairing, `check`). "Alive" = the entity is present and not `BodyHealth::is_dead()`.
- `run_births`: at the end of the per-birth block (after `Life` is inserted ≈ L952–962, before the roster insert ≈ L984), `ensure_person(child)` and `Born` (cause 1 if the fallback parents were used); the parents' records are ensured too.
- **Formation and membership** — in `SettlementSystem::tick`, by diff, not inside `sync_membership` (in scenes without fission, sync re-admits agents to overlapping rosters every tick and `resolve_exclusivity` drops them again, `settlement_system.rs` ≈ L272–276, L1108; recording each step would grow without bound): snapshot each agent's roster before `sync_membership`; after `resolve_exclusivity`, (1) for each settlement in `newly_formed_this_tick`: `SettlementFormed` (value = roster size after exclusivity); if any roster member carries `Founding` it is a daughter — no `FoundingMember`, members without `Founding` get `Joined` cause 3; otherwise each member gets `FoundingMember` (an agent that already has a `FoundingMember` gets `Joined` cause 1 instead — `run_dissolutions` lets an area re-form in non-fission scenes, ≈ L1061–1068); (2) for every other agent whose roster changed (agents carrying `Founding` skipped: their move is `Settled`): `LeftSettlement` cause 2 for the old one and `Joined` (cause 0 if it had none, 1 otherwise) for the new one. `run_formation_scan` has no `world` (≈ L398–405): do this in `tick`, not inside the scan.
- `PairingSystem`: `resources` stays borrowed through the loop (`pairing.rs` ≈ L62–79): collect (settlement, woman, man, familiarity) beside `events` and push `Paired` afterwards; value = `(familiarity × 1000.0).round() as i64`.
- `MaturationSystem`: `CameOfAge` when it removes `Child`; `AgingSystem`: `BecameElder` when it inserts `Elder`.
- `FissionSystem::check`: `FissionStarted`, then `Departed` per member in AgentId order with its `DepartCause` (6.3; the no-family path, `fission.rs` ≈ L614–622, labels everyone `Ranked`); `cancel` (iterates all `group.members`, dead ones included, ≈ L246): `FissionCancelled` and `DepartureCancelled` per **living** member; `found` (skips the dead, ≈ L272): `DaughterFounded` and `Settled` per member.
- Dissolutions: `SettlementSystem::run_dissolutions` (≈ L1011) and `FissionSystem::remnants` with zero members (≈ L419–421): `SettlementDissolved` cause 0. Remnant merge (≈ L414–438): take a sorted copy of `member_agents` **before** `member_agents.clear()` (≈ L427): `LeftSettlement` cause 0 per member, then `SettlementDissolved` cause 1.
- `despawn_agent` (`&mut World, &mut SimResources`; every caller is outside query borrows): **read before `world.despawn`** the agent's `Life`, `Kin` (partner) and roster settlement; after it, `close_person` and `Died`; if the recorded partner is alive, `Widowed` for the partner at the same tick. In `AgingSystem`'s sorted pass a partner who dies later in the same pass is still alive at the first death, so `Widowed` followed by that partner's own `Died` at the same tick is intended.
- **Never** push to `causal_log`, call `issue_event_id`, or change iteration order (no new query that inserts or removes components); **no simulation state may depend on a chronicle read** (`ensure_person` reads the table; nothing else does). The chronicle is recorded in **every** scene (it is an observation; identity proves it).

## 6.3 Departure causes without changing who leaves
Add `households::select_departure_explained(...) -> Option<BTreeMap<AgentId, DepartCause>>` holding the four phases' logic, labelling each member on its **first** insertion only (`entry().or_insert`; the top-up calls `chosen.insert` on ids already chosen, `households.rs` ≈ L145–152): in the seed phase the iterated ranked adult → `Seed`; every other ranked member of a household added by the seed or the fill → `Household`; an unranked member of such a household (a kept elder partner, v23+) → `ElderPartner`; a top-up single → `TopUp`. `select_departure` becomes `select_departure_explained(...).map(|m| m.into_keys().collect())` — same signature, same result. The no-family path is in `check`, not here; it labels everyone `Ranked`. Tests: every existing `select_departure` test unchanged and passing; a property test over 2,000 random (ranked, households, couples, k) inputs from a fixed seed asserting the key sets are equal to a frozen copy of Pass 18's function kept in the test module.

## 6.4 Probe output
`worldsim_probe --chronicle-out DIR` writes, per seed, `chronicle_<seed>.json` (entries in order, the person table) at the end of the run. Off by default; it never changes the summary.

## 6.5 Bridge (`sim-bridge/src/ffi/chronicle.rs` pure collectors + `#[func]`s in `world_node.rs`; all ids as i64, `None` → −1; packed arrays)
- `get_agent_id(entity_bits: i64) -> i64` (−1 if not found).
- `get_person(agent_id: i64) -> Dictionary {found, alive, female (−1/0/1), born_tick, died_tick, death_reason, mother, father, ns, ha, rd, p}` from the person table (works for the dead).
- `get_person_chronicle(agent_id: i64) -> Dictionary {found, tick: PackedInt64Array, kind: PackedInt32Array, subject, other, third: PackedInt64Array, settlement, settlement2: PackedInt64Array, cause: PackedInt32Array, value: PackedInt64Array}` — that person's entries in chronicle order.
- `get_settlement_chronicle(settlement_id: i64)` — the same shape for a village.
- `get_chronicle_since(start: i64, max: i64)` — the same shape plus `next` and `total`, `max` clamped to 1..=512 (the HUD's feed).
Unit tests on the collectors (dead person, founder with no parents, unknown id, the clamp). No GDScript in this pass.

## 6.6 Checks (Part 1)
- I1–I7 and all pins identical on the Part 1 build.
- **C-rules** on 32 worlds, `game_v23`, seeds 19200–19231, 120 days, `--agent-trace --chronicle-out` (the existing traces are the reference):

| # | rule |
|---|---|
| C1 | completeness: every `FamilyEvent::Born` / `Paired` / `Departed` member, every death of the deaths observer, every `FissionRecord` start / found / cancel has **exactly one** matching chronicle entry (same tick, ids, settlement); every `CameOfAge` / `BecameElder` tick t satisfies 0 ≤ t − (born_tick + threshold) < 60. Mismatches = 0. |
| C2 | causes: departure causes sum to the departed members of every fission; under `game_v23` the `ElderPartner` count per fission = the trace's `elders_departed` (`tools/revitalize/evaluate_pass18.py` ≈ L65); death reasons equal the deaths observer's. |
| C3 | person table: every agent that ever lived has a record; every dead one has `died_tick` and `death_reason` equal to the deaths observer's; every born one has the parents of `FamilyEvent::Born`. |
| C4 | determinism: two runs of the same seed give byte-identical chronicle files; the probe's twin check (`probe/runner.rs` ≈ L482) also compares the two engines' chronicles. |
| C5 | size (reported): entries and bytes per person-day (bytes = `entries × size_of::<ChronicleEntry>()` plus the index and person table, and the `--chronicle-out` JSON length); a projection to 10,000 people × 10 game years. |
| C6 | cost (reported, quiet lock): `stress --rules game_v23 --agents 500` three interleaved repeats, base build vs Part 1 build; mean within ±3 % expected (identity guarantees behaviour; this measures only the writes). |

Write `docs/revitalize/CHRONICLE.md` (what is recorded, why each cause, the getters, what the UI can build from it). Push.

---

# 7. PART 2 — `game_v24`: everyone ends their day at their own time

7.1 `WorldRules::game_v24(seed)` = `game_v23(seed)` with the routine rule of 7.2; `WorldGenConfig::game_v24` = `game_v23`'s. Wire into `scenes.rs`, the probe's scene lists (flip any "unknown scene" assertion), `twin_checked`, `perf.rs --rules`. Live game (`WorldSimNode::init`, `reset_world`) builds `game_v24`; `reset_world_stress` stays `game_v19`.
7.2 `RoutineRule` gains `end_stagger: Option<EndStagger>` with `EndStagger { base_minute: 1065, key_span: 30, p_span_minutes: 45 }` (fields `u32`; derives `Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize` as `RoutineRule`'s; `None` in `game_v21`–`game_v23`, `Some` in `game_v24`). One helper `RoutineRule::end_minute(&self, lineage_key, Option<&Temperament>) -> u32`, computed in `i64`: `None` → `day_end_minute`; `Some(s)` → `s.base_minute + (key / 60) % s.key_span + s.p_span_minutes × (P − 500) / 500` (truncating toward zero; an absent temperament = P 500, as `arrive` does, `lifecycle.rs` ≈ L248) — 1,020 (17:00, P = 0, key term 0) … 1,139 (18:59, P = 1,000, key term 29). `key / 60` keeps the end independent of the start minute (`key % 60`). P is the mean of three uniform draws (sd ≈ 167, `components/temperament.rs` ≈ L57–71), so the P term's sd (≈ 15 min) dominates the key term's (≈ 9 min): expected ρ(P, end) ≈ 0.85. (The planner's first draft, 1050 / 60 / 30, gives ρ ≈ 0.45 by simulation — below H2's bar.) **No tuning of `EndStagger` after METRICS.md is committed; a missed bar is reported as missed.**
7.3 **Every** reader of `day_end_minute` goes through `end_minute`: today `decision/routine/mod.rs` ≈ L183 (`can_enter`) and `lifecycle.rs` ≈ L196 (`deadline`); grep and list them in DECISIONS.md. `deadline(id, member, walking)` and `can_enter` have neither the key nor the temperament: their `pub(super)` signatures gain them (both call sites, `agent_decision.rs` ≈ L2199 and ≈ L2558, and the routine arm ≈ L2056–2079 have `lineage` and `temperament` in scope). The person's window becomes `start_minute(key) ≤ minute < end_minute(key, t)`. Nothing else changes; I5–I7 must stay identical (the `None` path).
7.4 Expected: ends spread over 17:00–18:59, peaked near 18:00; the persistent work up to an hour later, the least persistent stop up to an hour earlier. **Other 18:00 windows stay where they are:** early enders fall through to the normal cascade, where forage and wood trips run until 18:00 (`forage.end_minute` 1080, read at `agent_decision.rs` ≈ L884; the forage arm precedes the routine arm, ≈ L1931 < L2056), so they may make trips they never made in `game_v23` (H3's stores row will show it); late enders join the evening tether (`evening_gather.start_minute` 1080, `movement.rs` ≈ L180) when their activity ends. Bedtime (≥ 20:00), pairing (20:00) and fission (06:00) are unchanged. The probe's trace windows (`agent_trace.rs` ≈ L131 `night_of` ≥ 1080, ≈ L298 idle tether 360..1080) are not changed either; report what they show.
**Event definitions for H1/H2:** an activity end = a routine trace row with `event == "ended"` (`probe/pass17_activity.rs` ≈ L252; not the `"end…"` observation row ≈ L150), all life stages; a non-interrupted day-end exit = `ended` with `completed == true` and `tick < until_tick`; its minute = `(tick + clock_offset) % 1440`.
7.5 **Rule table** (committed to METRICS.md under `## Pass 19` before 7.6 runs; `game_v24` vs `game_v23`, seeds 19200–19231, 120 days, paired by seed, Pass 17's definitions and bootstrap):

| # | rule |
|---|---|
| H1 | the edge: per world-day, the largest number of activity ends in one tick between 16:00 and 20:00 (routine trace); median over world-days in `game_v24` ≤ **0.15 ×** `game_v23`'s |
| H2 | temperament at dusk: Spearman ρ between P and a person's mean clock minute of non-interrupted day-end exits (people with ≥ 5 such exits) ≥ **0.5** (interval reported) |
| H3 | health: Pass 17's R2 rules for `game_v24` − `game_v23` (alive ±10 %, non-old-age deaths ≤ +1.5, shelters ≥ 0.9×, social ≥ 0.9×, stores ±15 %) and family F1, F5 within ± 0.05 of `game_v23`'s |
| H4 | daytime: adult daytime Idle share (Pass 17 R1's definition, hourly 08:00–17:00) ≤ `game_v23`'s + 0.02; the 17:00–19:00 adult Idle share reported beside it (R1 does not see the change) |
| H5 | performance: `game_v24` vs `game_v20`, Pass 18's stress procedure but **five** interleaved repeats per rule (medians of five, declared before measuring, so one machine hiccup cannot decide the tail): mean ≤ +15 %, p99.9 ≤ +35 %, mean ≤ 4.0 ms, `AgentDecisionSystem` slope < 1.7; the 18:00-window ticks' cost reported beside v20's and v23's |
| H6 | identity: I1–I7 and all pins identical on the final build |

7.6 Evaluation (the `game_v23` side is 6.6's runs on the same seeds) → `docs/revitalize/probe/p19-2_eval/eval.md` + `eval.json`; H5 under the quiet lock → `docs/revitalize/perf/p19-2/`. `perf.rs` maps unknown labels to `game_v19` silently (≈ L113–134) and its parser rejects `game_v24` today (≈ L704): add a test that `--rules game_v24` yields the `stress_v24` label and `WorldRules::game_v24`. Pass 18's mean ratio was already +13.7–17.7 % against the 15 % bar, so H5's mean may miss even if the tail is fixed: report it as it is. Push.

---

# 8. PART 3 — Scale check (findings only)
1. `worldsim_perf stress --rules game_v24 --seeds 15100..15101 --agents 500,1000,2000 --side 64 --days 2 --feed fed` (side 64 for all sizes, comparable with PERF_BUDGET), plus one 2,000-person run at `--side 128` to separate map-size cost — three interleaved repeats under the quiet lock (split into holds ≤ 60 minutes), **one process per (size, repeat)**; and the 2,000 run once more with the `subprofile` build (its scopes exist only inside the decision system, `sim-systems/Cargo.toml` ≈ L14–16). **The stress world is one camp of N founders around a single hearth** (`worldgen/camp.rs` ≈ L106–118; the formation scan adds them all without a cap), not villages of ≤ 50: say so in SCALE.md and read the slopes as a single-camp upper bound; record the settlement count and sizes each run ends with.
2. Per size: mean and p99.9 tick, the per-system cost table (`TickProfile`), each system's slope over 500 → 2,000; peak RSS (`/usr/bin/time -l`, macOS BSD time, bytes); chronicle bytes at the end (C5's definition).
3. The live game's speed: a fixed 30 ticks per second at 1× (`FIXED_DT = 1/30`, `MAX_ITERS_PER_FRAME = 5`, `sim-bridge/src/ffi/world_node.rs` ≈ L166–169); the UI offers 0.25 / 0.5 / 1 / 2× on keys 1–4 (`scripts/ui/camera_controller.gd` ≈ L96); `set_sim_speed` clamps at 4× (`world_node.rs` ≈ L175). Verify, then convert: simulation CPU per real second at 1×, 2× (60 ticks/s, the UI's fastest) and 4× for each size, and whether `MAX_ITERS_PER_FRAME` caps the speed actually reached.
4. **S-rules** (reported verdicts, no fix this pass):

| # | rule |
|---|---|
| S1 | every system with ≥ 5 % of the tick at 2,000 people has slope < 1.7 over 500 → 2,000 (else it is listed as a finding, with the decision system's scope breakdown when it is that system) |
| S2 | mean tick at 1,000 people ≤ 4.0 ms |
| S3 | simulation CPU at 2× (60 ticks/s, the UI's fastest) with 1,000 people ≤ 50 % of one core; 4× reported |
| S4 | peak RSS at 2,000 people reported; chronicle growth reported |

5. `docs/revitalize/SCALE.md`: the tables, the findings ranked by cost, and what a fix pass should target first. Push.

---

# 9. PART 4 — Docs, review, gate, report
1. Update `ROUTINE.md` (`game_v24` end times), `FAMILY.md` (the chronicle's family entries), `PERF_BUDGET.md` (Pass 19 references; the five-repeat procedure).
2. Independent review (fresh `general-purpose` subagent, two rounds max): identity I1–I7 and pins; that no system reads the chronicle and nothing in Part 1 touches `causal_log`, event ids or component sets; `select_departure` equivalence; C1–C4 on one world by hand (a birth, a pairing, a fission departure with each cause, a death and its widow); the `end_minute` readers; H verdicts; the quiet-lock record. Findings in `docs/revitalize/REVIEW_PASS19.md`.
3. Gate (4.4) → `docs/revitalize/verification/p19_gate/`; fast-forward `codex/revitalize-2026-10`; PR #510 section; remove `../new-world-p19-base`.
4. `docs/revitalize/REPORT_PASS19.md` in Korean (section 11 format) and the same text in the final message.

---

# 10. 인게임 확인사항 (오너용)
- 게임이 `game_v24`로 시작한다.
- 저녁에 사람들이 18:00 한순간에 일제히 돌아오지 않고, 17:00부터 19:00 사이에 한 명씩 흩어져 돌아오는지. 끈기(P)가 높은 사람이 늦게까지 남는지(인스펙터의 기질 막대와 비교).
- 연대기는 이번 패스에 데이터만 생긴다(화면은 다음 아트 패스). 화면에서 달라지는 것은 위 하나뿐이다.
- 2배속(키 4, 게임 UI의 최고 속도)에서 끊김이 없는지, 콘솔 에러가 없는지.

---

# 11. 구현 완료 보고 (REPORT_PASS19.md와 최종 메시지에 이 형식 그대로, 한국어로)
```
## 구현 완료 보고

### 구현 의도
### 구현 내용
### 구현 방법
(연대기 저장소와 기록 위치, 분열 이유 표시가 출발 집합을 바꾸지 않는다는 증명, 사람 기록부, 브리지 조회, game_v24 종료 시각 규칙, 규모 측정 방법)
### 기능 설명
### 변경된 파일 목록
(커밋별)
### 측정 결과
C1~C6 표, H1~H6 표(값과 구간), 18:00 전후 틱 비용 비교(v20·v23·v24), 규모 표(500/1,000/2,000명: 평균·p99.9·시스템별 비용과 기울기·RSS·연대기 크기·배속별 CPU), S1~S4 판정
### 환경
### 독립 리뷰
### 자율 결정 목록
### 실행 기록
### 확인된 제한사항
### 게이트 결과
### 정직한 판단
연대기는 UI가 "왜?"를 답할 만큼 충분한가? 18:00 꼬리는 해결됐는가? 몇 명까지 부드럽게 돌아가는가, 무엇이 먼저 무너지는가? 다음은 무엇인가?
```
Final message also includes: every commit hash with its Part, both branch heads with `git ls-remote` confirmation, the PR #510 update, and the time per Part.
