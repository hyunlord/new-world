//! V7 Stage 1.5 — newborn mobility regression harness (`fix-newborn-movementrng`).
//!
//! Settlement births (`SettlementSystem::run_births`) used to spawn each
//! newborn WITHOUT a `MovementRng` and with all four need `growth_rate`s at
//! 0.0. `AgentMovementSystem::tick` iterates `MovementRng` as a REQUIRED
//! (non-`Option`) query term, so a newborn lacking it is never iterated and is
//! permanently frozen; with zero growth rates its needs never rise so it never
//! enters the `Seeking → Consuming` gathering loop either. This was masked
//! until Stage-1 (`9dce85e1`) removed the settlement-migration freeze; the
//! Stage-1 probe then measured 3/6 newborns frozen.
//!
//! The fix gives newborns the SAME movement/needs component set bootstrap gives
//! every agent (`bootstrap_spawn_agents` in `world_node.rs`): a `MovementRng`
//! seeded deterministically from the unique `new_agent_id`, plus nonzero growth
//! rates mirroring the bootstrap values (Hunger 0.05 / Thirst 0.08 / Sleep 0.03
//! / Social 0.04). Initial need VALUES stay 0.0.
//!
//! Setup MIRRORS the production windowed scene (`harness_settlement_migration_
//! unfreeze.rs`): the bootstrap 64-agent lattice (ids 0..63) PLUS the 3 startup
//! buildings at (32,32),(24,32),(40,32), which form a settlement early and
//! trigger births after `BIRTH_COOLDOWN_TICKS`. Newborns are the agents whose
//! id is NOT in the pre-birth snapshot (the diff-of-id-set form the plan
//! prefers, robust to any future bootstrap-count change).
//!
//! Thresholds here are LOCKED by the test plan (`fix-newborn-movementrng`,
//! plan_attempt 2, seed 42, 64 agents) and must NOT be tuned to match the
//! implementation.
//!
//! Run:
//!   cargo test -p sim-test --test harness_newborn_mobile -- --nocapture

use std::any::TypeId;
use std::collections::{HashMap, HashSet};

use hecs::Entity;
use sim_bridge::ffi::world_node::{bootstrap_spawn_agents, enqueue_building_placed};
use sim_core::causal::event::CausalEvent;
use sim_core::components::{
    Agent, AgentId, AgentState, Hunger, Position, SeekTarget, Sleep, Social, TargetKind, Thirst,
};
use sim_core::material::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::register_default_runtime_systems;
use sim_systems::runtime::agent::MovementRng;
use sim_systems::runtime::influence::BuildingStampSystem;

const W: u32 = 64;
const H: u32 = 64;

/// `(value, growth_rate)` pairs for Hunger / Thirst / Sleep / Social. Hunger is
/// `f32`; the other three are `f64`.
type NeedSnapshot = ((f32, f32), (f64, f64), (f64, f64), (f64, f64));

/// One lightweight per-tick query row:
/// `(entity, id, tile, state, has-MovementRng, thirst-value)`.
type AgentRow = (Entity, AgentId, (u32, u32), AgentState, bool, f64);

// ── Per-agent observation records ───────────────────────────────────────────

/// One per-tick observation of a single agent.
#[derive(Clone, Copy)]
struct Obs {
    tick: u64,
    pos: (u32, u32),
    state: AgentState,
    has_rng: bool,
    thirst_value: f64,
}

/// Details captured the FIRST tick an agent is observed (its "birth tick" for a
/// newborn). Need VALUES + growth_rates read live from components; `type_sig`
/// is the full set of component `TypeId`s on the entity at that tick.
#[derive(Clone)]
struct FirstObs {
    tick: u64,
    has_rng: bool,
    /// (value, growth_rate) for Hunger / Thirst / Sleep / Social.
    hunger: (f32, f32),
    thirst: (f64, f64),
    sleep: (f64, f64),
    social: (f64, f64),
    type_sig: HashSet<TypeId>,
    /// How many settlements list this agent as a member at its first tick.
    settlement_membership_count: usize,
}

/// Everything one run of the engine yields, sufficient for all assertions.
struct Run {
    /// Agent ids present immediately after bootstrap, before tick 1.
    pre_birth_ids: HashSet<AgentId>,
    /// Per-agent full position/state/rng/thirst trajectory, in tick order.
    traj: HashMap<AgentId, Vec<Obs>>,
    /// Per-agent first-observation snapshot.
    first: HashMap<AgentId, FirstObs>,
    /// RAW number of `AgentBorn` causal events accumulated tick-by-tick (each
    /// birth is pushed to exactly one tile → counted exactly once, immune to
    /// 32-event ring-buffer eviction). PRIMARY quantity A9(a) compares.
    born_event_count: usize,
    /// Distinct agent ids that appeared in an `AgentBorn` event (SECONDARY
    /// uniqueness check: equals `born_event_count` when no id repeats).
    born_ids: HashSet<AgentId>,
}

impl Run {
    /// Newborn set = (all observed ids) − (pre-birth ids). Sorted by
    /// (first-seen tick, id) so "earliest-born" is deterministic.
    fn newborns(&self) -> Vec<AgentId> {
        let mut ids: Vec<AgentId> = self
            .traj
            .keys()
            .copied()
            .filter(|id| !self.pre_birth_ids.contains(id))
            .collect();
        ids.sort_by_key(|id| {
            let t = self.first.get(id).map(|f| f.tick).unwrap_or(u64::MAX);
            (t, *id)
        });
        ids
    }

    /// First-observed (birth) tick of `id`, or `u64::MAX` if never seen.
    fn birth_tick(&self, id: AgentId) -> u64 {
        self.first.get(&id).map(|f| f.tick).unwrap_or(u64::MAX)
    }

    /// Last-observed tick of `id`, or 0 if never seen.
    fn last_tick(&self, id: AgentId) -> u64 {
        self.traj
            .get(&id)
            .and_then(|o| o.last())
            .map(|o| o.tick)
            .unwrap_or(0)
    }

    /// Ticks `id` was alive (last − first observation). 0 if seen once/never.
    fn lifespan(&self, id: AgentId) -> u64 {
        self.last_tick(id).saturating_sub(self.birth_tick(id))
    }

    /// Maximum Manhattan displacement of `id` from its birth tile over EVERY
    /// observed tick (so a Brownian wander-and-return cannot read as frozen).
    fn max_disp(&self, id: AgentId) -> u64 {
        let obs = match self.traj.get(&id) {
            Some(o) if !o.is_empty() => o,
            _ => return 0,
        };
        let (bx, by) = obs[0].pos;
        obs.iter()
            .map(|o| {
                let dx = (o.pos.0 as i64 - bx as i64).unsigned_abs();
                let dy = (o.pos.1 as i64 - by as i64).unsigned_abs();
                dx + dy
            })
            .max()
            .unwrap_or(0)
    }

    /// `true` if `id` ever carried a `MovementRng` at any observed tick.
    fn ever_has_rng(&self, id: AgentId) -> bool {
        self.traj
            .get(&id)
            .map(|o| o.iter().any(|x| x.has_rng))
            .unwrap_or(false)
    }

    /// `true` if `id` ever entered a `Seeking` state targeting a need resource
    /// (Food / Water / Sleep).
    fn ever_seeking_resource(&self, id: AgentId) -> bool {
        self.traj
            .get(&id)
            .map(|o| {
                o.iter().any(|x| {
                    matches!(
                        x.state,
                        AgentState::Seeking {
                            target: TargetKind::Food | TargetKind::Water | TargetKind::Sleep
                        }
                    )
                })
            })
            .unwrap_or(false)
    }

    /// Final observed tile of `id`.
    fn final_pos(&self, id: AgentId) -> Option<(u32, u32)> {
        self.traj.get(&id).and_then(|o| o.last()).map(|o| o.pos)
    }
}

// ── Setup helpers (mirror harness_settlement_migration_unfreeze.rs) ──────────

/// Production bootstrap path: `new` → `register_default_runtime_systems`
/// → `bootstrap_spawn_agents` (= 64 agents, ids 0..63).
fn bootstrapped() -> SimEngine {
    let mut e = SimEngine::new(W, H, MaterialRegistry::new());
    register_default_runtime_systems(&mut e);
    bootstrap_spawn_agents(&mut e);
    e
}

/// Place the 3 startup buildings at the REAL scene coordinates
/// (32,32),(24,32),(40,32) via the FFI queue + a `BuildingStampSystem` drain.
/// Mirrors `world_renderer.gd::_ready()` — this is what makes a settlement form
/// early and triggers births in the bootstrap scene.
fn place_startup_buildings(e: &mut SimEngine) {
    for (x, y) in [(32i32, 32i32), (24, 32), (40, 32)] {
        let ok = enqueue_building_placed(&mut e.resources, x, y, 8);
        assert!(ok, "startup building ({x},{y}) must enqueue within bounds");
    }
    let mut bss = BuildingStampSystem::new();
    bss.tick(&mut e.world, &mut e.resources);
}

/// Read the four need (value, growth_rate) pairs from a live entity. A missing
/// component yields a sentinel `-1.0` pair so exact comparisons fail loudly.
fn read_needs(e: &SimEngine, ent: Entity) -> NeedSnapshot {
    let h = e
        .world
        .get::<&Hunger>(ent)
        .map(|c| (c.value, c.growth_rate))
        .unwrap_or((-1.0, -1.0));
    let t = e
        .world
        .get::<&Thirst>(ent)
        .map(|c| (c.value, c.growth_rate))
        .unwrap_or((-1.0, -1.0));
    let s = e
        .world
        .get::<&Sleep>(ent)
        .map(|c| (c.fatigue, c.growth_rate))
        .unwrap_or((-1.0, -1.0));
    let so = e
        .world
        .get::<&Social>(ent)
        .map(|c| (c.loneliness, c.growth_rate))
        .unwrap_or((-1.0, -1.0));
    (h, t, s, so)
}

/// Build a fresh bootstrap + startup-building engine and run it `ticks` ticks,
/// capturing a per-tick trajectory plus first-observation detail for every
/// agent and the accumulated `AgentBorn` event set. Per-tick sampling (every
/// `e.tick()`) is mandatory — coarse sampling would corrupt the birth-tile /
/// movement assertions (plan Assertion 5).
fn collect_run(ticks: u64) -> Run {
    let mut e = bootstrapped();
    place_startup_buildings(&mut e);

    // Pre-birth snapshot (before tick 1).
    let pre_birth_ids: HashSet<AgentId> =
        e.world.query::<&Agent>().iter().map(|(_, a)| a.id).collect();

    let mut traj: HashMap<AgentId, Vec<Obs>> = HashMap::new();
    let mut first: HashMap<AgentId, FirstObs> = HashMap::new();
    let mut born_ids: HashSet<AgentId> = HashSet::new();
    let mut born_event_count: usize = 0;

    for _ in 0..ticks {
        e.tick();
        let cur = e.resources.current_tick;

        // Collect lightweight rows first (drops the query borrow before the
        // per-entity component reads below).
        let rows: Vec<AgentRow> = e
            .world
            .query::<(&Agent, &Position, &AgentState, Option<&MovementRng>, Option<&Thirst>)>()
            .iter()
            .map(|(ent, (a, p, s, r, t))| {
                (
                    ent,
                    a.id,
                    (p.x, p.y),
                    *s,
                    r.is_some(),
                    t.map(|c| c.value).unwrap_or(-1.0),
                )
            })
            .collect();

        for (ent, id, pos, state, has_rng, thirst_value) in rows {
            traj.entry(id).or_default().push(Obs {
                tick: cur,
                pos,
                state,
                has_rng,
                thirst_value,
            });
            if let std::collections::hash_map::Entry::Vacant(slot) = first.entry(id) {
                let (h, t, s, so) = read_needs(&e, ent);
                let type_sig: HashSet<TypeId> = e
                    .world
                    .entity(ent)
                    .ok()
                    .map(|er| er.component_types().collect())
                    .unwrap_or_default();
                let membership = e
                    .resources
                    .settlements
                    .values()
                    .filter(|st| st.member_agents.contains(&id))
                    .count();
                slot.insert(FirstObs {
                    tick: cur,
                    has_rng,
                    hunger: h,
                    thirst: t,
                    sleep: s,
                    social: so,
                    type_sig,
                    settlement_membership_count: membership,
                });
            }
        }

        // Accumulate AgentBorn events emitted on THIS tick, before the per-tile
        // 32-event ring buffer can evict them.
        for (_tile, log) in e.resources.causal_log.iter() {
            for ev in log.iter() {
                if let CausalEvent::AgentBorn { agent, tick, .. } = ev {
                    if *tick == cur {
                        born_event_count += 1;
                        born_ids.insert(*agent);
                    }
                }
            }
        }
    }

    Run {
        pre_birth_ids,
        traj,
        first,
        born_event_count,
        born_ids,
    }
}

// ─── Assertion 1: birth_occurs_non_vacuity ──────────────────────────────────
#[test]
fn harness_newborn_birth_occurs_non_vacuity() {
    // Type C — non-vacuity GATE for the whole suite. Lower bound (>=1) must fail
    // loudly when births don't fire (every per-newborn assertion is trivially
    // true on an empty set). Upper bound (<=60) is a runaway/duplication guard
    // (~4x the prototype-observed 15). Count = ids present at end but absent at
    // the pre-birth (tick-0) snapshot — diff form, robust to bootstrap count.
    let run = collect_run(1200);
    let newborn_count = run.newborns().len();
    println!("[newborn A1] newborn_count = {newborn_count}");
    assert!(
        newborn_count >= 1,
        "A1: at least one birth must occur in 1200 ticks (bootstrap + startup \
         buildings); got {newborn_count}. Without a birth the suite is vacuous."
    );
    assert!(
        newborn_count <= 60,
        "A1: newborn count must be <= 60 (runaway/duplication guard); got {newborn_count}"
    );
    println!("[newborn A1] birth occurred, count within [1,60] ({newborn_count}) ✓");
}

// ─── Assertion 2: newborn_has_movement_rng ──────────────────────────────────
#[test]
fn harness_newborn_has_movement_rng() {
    // Type A — direct structural invariant of the fix. AgentMovementSystem
    // iterates MovementRng as a required (non-Option) term; a newborn lacking it
    // is never iterated and is permanently frozen. Count must be exactly 0.
    let run = collect_run(1200);
    let newborns = run.newborns();
    assert!(
        !newborns.is_empty(),
        "A2: newborn set must be non-empty (A1 gate)"
    );
    // Structural: present at the birth tick (spawned with it), AND never lost.
    let missing = newborns
        .iter()
        .filter(|id| {
            let at_birth = run.first.get(id).map(|f| f.has_rng).unwrap_or(false);
            !(at_birth && run.ever_has_rng(**id))
        })
        .count();
    println!("[newborn A2] newborns missing MovementRng = {missing} / {}", newborns.len());
    assert_eq!(
        missing, 0,
        "A2: every newborn must carry a MovementRng (at birth and throughout); \
         {missing} did not"
    );
    println!("[newborn A2] every newborn carries MovementRng ✓");
}

// ─── Assertion 3: newborn_component_completeness ────────────────────────────
#[test]
fn harness_newborn_component_completeness() {
    // Type A — structural all-or-nothing invariant. Every newborn must carry the
    // full movement/needs set the downstream assertions read: Position,
    // AgentState, Hunger, Thirst, Sleep, Social, MovementRng. A newborn missing
    // any one would silently change behavior without a dedicated failure.
    let required: [(TypeId, &str); 7] = [
        (TypeId::of::<Position>(), "Position"),
        (TypeId::of::<AgentState>(), "AgentState"),
        (TypeId::of::<Hunger>(), "Hunger"),
        (TypeId::of::<Thirst>(), "Thirst"),
        (TypeId::of::<Sleep>(), "Sleep"),
        (TypeId::of::<Social>(), "Social"),
        (TypeId::of::<MovementRng>(), "MovementRng"),
    ];
    let run = collect_run(1200);
    let newborns = run.newborns();
    assert!(!newborns.is_empty(), "A3: newborn set must be non-empty (A1 gate)");

    let mut missing_any = 0usize;
    for id in &newborns {
        let sig = match run.first.get(id) {
            Some(f) => &f.type_sig,
            None => {
                missing_any += 1;
                continue;
            }
        };
        let absent: Vec<&str> = required
            .iter()
            .filter(|(tid, _)| !sig.contains(tid))
            .map(|(_, name)| *name)
            .collect();
        if !absent.is_empty() {
            missing_any += 1;
            println!("[newborn A3] newborn {id} missing: {absent:?}");
        }
    }
    println!("[newborn A3] newborns missing any required component = {missing_any}");
    assert_eq!(
        missing_any, 0,
        "A3: every newborn must carry all 7 required components; {missing_any} were incomplete"
    );
    println!("[newborn A3] every newborn carries the full required component set ✓");
}

// ─── Assertion 4: newborn_growth_rate_parity_with_bootstrap ─────────────────
#[test]
fn harness_newborn_growth_rate_parity_with_bootstrap() {
    // Type A — parity invariant read LIVE (never against hardcoded literals,
    // which would be circular). Reference = a bootstrap agent's four growth
    // rates; every newborn's four rates must equal the bootstrap reference
    // exactly (both sides assigned directly from constants — no arithmetic, so
    // == is valid; f32 Hunger vs f64 others compared field-against-same-field).
    let run = collect_run(1200);
    let bootstrap: Vec<&FirstObs> = run
        .pre_birth_ids
        .iter()
        .filter_map(|id| run.first.get(id))
        .collect();
    assert!(
        !bootstrap.is_empty(),
        "A4 precondition: bootstrap agents must have been observed"
    );
    let r_hunger = bootstrap[0].hunger.1;
    let r_thirst = bootstrap[0].thirst.1;
    let r_sleep = bootstrap[0].sleep.1;
    let r_social = bootstrap[0].social.1;
    // Sanity: the bootstrap reference is uniform across the bootstrap set.
    let boot_dev = bootstrap
        .iter()
        .filter(|f| {
            f.hunger.1 != r_hunger
                || f.thirst.1 != r_thirst
                || f.sleep.1 != r_sleep
                || f.social.1 != r_social
        })
        .count();
    assert_eq!(boot_dev, 0, "A4: bootstrap growth rates must be uniform; {boot_dev} deviated");

    let newborns = run.newborns();
    assert!(!newborns.is_empty(), "A4: newborn set must be non-empty (A1 gate)");
    let deviating = newborns
        .iter()
        .filter_map(|id| run.first.get(id))
        .filter(|f| {
            f.hunger.1 != r_hunger
                || f.thirst.1 != r_thirst
                || f.sleep.1 != r_sleep
                || f.social.1 != r_social
        })
        .count();
    println!(
        "[newborn A4] reference R = (h {r_hunger}, t {r_thirst}, s {r_sleep}, so {r_social}); \
         deviating newborns = {deviating}"
    );
    assert_eq!(
        deviating, 0,
        "A4: every newborn's growth_rate 4-tuple must equal the live bootstrap reference; \
         {deviating} deviated"
    );
    println!("[newborn A4] newborn growth rates match bootstrap (live parity) ✓");
}

// ─── Assertion 5: newborn_actually_moves ────────────────────────────────────
#[test]
fn harness_newborn_actually_moves() {
    // Type D — THE core freeze guard. MAX displacement from the birth tile over
    // EVERY observed tick (per-tick sampling), so a Brownian wander-and-return
    // cannot false-PASS as frozen. Exclude newborns born within the final K=60
    // ticks (too late to have stepped). Both conjuncts: frozen == 0 AND
    // moved_fraction >= 0.9.
    const K: u64 = 60;
    let total_ticks = 1200u64;
    let run = collect_run(total_ticks);
    let newborns = run.newborns();
    assert!(!newborns.is_empty(), "A5: newborn set must be non-empty (A1 gate)");

    let eligible: Vec<AgentId> = newborns
        .iter()
        .copied()
        .filter(|id| run.birth_tick(*id) + K <= total_ticks)
        .collect();
    assert!(
        !eligible.is_empty(),
        "A5: at least one newborn must be born before tick {} to be eligible",
        total_ticks - K
    );
    let frozen: Vec<AgentId> = eligible
        .iter()
        .copied()
        .filter(|id| run.max_disp(*id) == 0)
        .collect();
    let moved = eligible.len() - frozen.len();
    let moved_fraction = moved as f64 / eligible.len() as f64;
    println!(
        "[newborn A5] eligible = {}, frozen = {}, moved_fraction = {moved_fraction:.3}",
        eligible.len(),
        frozen.len()
    );
    assert!(
        frozen.is_empty(),
        "A5: no eligible newborn may be frozen (max displacement 0); frozen ids: {frozen:?}"
    );
    assert!(
        moved_fraction >= 0.9,
        "A5: moved_fraction must be >= 0.9; got {moved_fraction:.3}"
    );
    println!("[newborn A5] every eligible newborn moved (fraction {moved_fraction:.3} >= 0.9) ✓");
}

// ─── Assertion 6: newborn_need_values_rise_over_time ────────────────────────
#[test]
fn harness_newborn_need_values_rise_over_time() {
    // Type D — threshold-INDEPENDENT consequence check. A4 proves the rate FIELD
    // equals bootstrap's; this proves the rate actually DRIVES the Thirst value
    // upward. For each newborn that lived >= 300 ticks AND has not entered a
    // Water Seeking/Consuming cycle within its first 300 ticks (the value would
    // reset), Thirst must STRICTLY increase between birth and +300 ticks
    // (~+24 raw at 0.08/tick, far above float noise).
    const WINDOW: u64 = 300;
    let run = collect_run(1200);
    let newborns = run.newborns();
    assert!(!newborns.is_empty(), "A6: newborn set must be non-empty (A1 gate)");

    let mut eligible = 0usize;
    let mut non_rising = 0usize;
    for id in &newborns {
        if run.lifespan(*id) < WINDOW {
            continue;
        }
        let obs = match run.traj.get(id) {
            Some(o) if !o.is_empty() => o,
            _ => continue,
        };
        let birth = obs[0].tick;
        // Window observations [birth, birth+WINDOW].
        let window: Vec<&Obs> = obs.iter().filter(|o| o.tick <= birth + WINDOW).collect();
        // Eligibility: no Water Seeking/Consuming in the window (no reset).
        let consumed_water = window.iter().any(|o| {
            matches!(
                o.state,
                AgentState::Seeking { target: TargetKind::Water }
                    | AgentState::Consuming { target: TargetKind::Water }
            )
        });
        if consumed_water {
            continue;
        }
        eligible += 1;
        let birth_thirst = obs[0].thirst_value;
        // Value at the observation closest to (and not beyond) birth+WINDOW.
        let end_thirst = window.last().map(|o| o.thirst_value).unwrap_or(birth_thirst);
        if end_thirst <= birth_thirst {
            non_rising += 1;
            println!(
                "[newborn A6] newborn {id}: thirst {birth_thirst} -> {end_thirst} did NOT rise"
            );
        }
    }
    println!("[newborn A6] eligible (lived >= {WINDOW}, no consume) = {eligible}, non-rising = {non_rising}");
    assert!(
        eligible >= 1,
        "A6: at least one newborn must live >= {WINDOW} ticks without consuming water"
    );
    assert_eq!(
        non_rising, 0,
        "A6: every eligible newborn's Thirst value must strictly rise over {WINDOW} ticks; \
         {non_rising} did not (rate field is dead despite reading correctly)"
    );
    println!("[newborn A6] newborn need values rise over time (rate drives growth) ✓");
}

// ─── Assertion 7: bootstrap_agents_unaffected ───────────────────────────────
#[test]
fn harness_newborn_bootstrap_agents_unaffected() {
    // Type D — cross-phase collateral-damage tripwire. The change is scoped to
    // the newborn spawn and must not perturb the 64 bootstrap agents Stage-1
    // unfroze. Track, per bootstrap agent (id < 64), the max consecutive run of
    // "position unchanged while in a MOTILE state" (Idle, or Seeking carrying a
    // SeekTarget); a non-motile tick resets the streak. Worst streak must be
    // < 30 (the Stage-1 migration-unfreeze A5b bound; post-Stage-1 worst was 1).
    let mut e = bootstrapped();
    place_startup_buildings(&mut e);

    let pre_birth_ids: HashSet<AgentId> =
        e.world.query::<&Agent>().iter().map(|(_, a)| a.id).collect();

    let mut prev: HashMap<Entity, (u32, u32)> = HashMap::new();
    let mut streak: HashMap<Entity, u32> = HashMap::new();
    let mut worst = 0u32;
    for _ in 0..300u64 {
        e.tick();
        let mut seen: HashSet<Entity> = HashSet::new();
        for (ent, (a, state, pos, seek)) in e
            .world
            .query::<(&Agent, &AgentState, &Position, Option<&SeekTarget>)>()
            .iter()
        {
            if !pre_birth_ids.contains(&a.id) {
                continue; // bootstrap agents only
            }
            seen.insert(ent);
            let here = (pos.x, pos.y);
            let moved = prev.get(&ent).map(|p| *p != here).unwrap_or(true);
            let motile = matches!(state, AgentState::Idle)
                || (matches!(state, AgentState::Seeking { .. }) && seek.is_some());
            let s = streak.entry(ent).or_insert(0);
            if motile && !moved {
                *s += 1;
                if *s > worst {
                    worst = *s;
                }
            } else {
                *s = 0;
            }
            prev.insert(ent, here);
        }
        streak.retain(|k, _| seen.contains(k));
        prev.retain(|k, _| seen.contains(k));
    }
    println!("[newborn A7] worst bootstrap motile-frozen streak = {worst}");
    assert!(
        worst < 30,
        "A7: worst bootstrap motile-state frozen streak must be < 30 ticks; got {worst}"
    );
    println!("[newborn A7] bootstrap agents unaffected (worst streak {worst} < 30) ✓");
}

// ─── Assertion 8: deterministic_birth_seed ──────────────────────────────────
#[test]
fn harness_newborn_deterministic_birth_seed() {
    // Type A — nondeterminism guard. birth_seed is a pure function of
    // new_agent_id (splitmix64 multiply + fixed salt), so adding a seeded RNG
    // must not introduce run-to-run divergence. Two independent engines (each
    // constructed fresh — no shared global RNG/static state) for the same tick
    // count must yield identical {newborn id -> final Position} maps. The
    // non-empty floor prevents a vacuous pass on two empty maps.
    let run1 = collect_run(800);
    let run2 = collect_run(800);

    let map1: HashMap<AgentId, (u32, u32)> = run1
        .newborns()
        .into_iter()
        .filter_map(|id| run1.final_pos(id).map(|p| (id, p)))
        .collect();
    let map2: HashMap<AgentId, (u32, u32)> = run2
        .newborns()
        .into_iter()
        .filter_map(|id| run2.final_pos(id).map(|p| (id, p)))
        .collect();

    assert!(!map1.is_empty(), "A8: non-empty floor — run 1 must produce >= 1 newborn");
    assert!(!map2.is_empty(), "A8: non-empty floor — run 2 must produce >= 1 newborn");
    assert_eq!(
        map1, map2,
        "A8: two independent runs must yield identical {{newborn id -> final position}} maps"
    );
    println!(
        "[newborn A8] deterministic birth seed: {} newborns, identical final-position maps ✓",
        map1.len()
    );
}

// ─── Assertion 9: birth_logic_preserved ─────────────────────────────────────
#[test]
fn harness_newborn_birth_logic_preserved() {
    // Type D — regression guard for p10-β a13/a15/a21/a23 (birth-fires / inside-
    // radius / community-history / parent-chain). (a) accumulated AgentBorn
    // count == distinct newborn ids EVER seen (robust to death/disappearance;
    // death is currently unimplemented so they are equal today). (b) every
    // newborn is enrolled in some settlement AT ITS BIRTH TICK (immune to later
    // dissolution — the ζ trap that would false-fail an end-of-run check).
    let run = collect_run(1200);
    let newborns = run.newborns();
    let newborn_count = newborns.len();
    assert!(newborn_count >= 1, "A9: newborn set must be non-empty (A1 gate)");

    // (a) AgentBorn correspondence.
    let born_events = run.born_event_count;
    let distinct_born = run.born_ids.len();
    println!(
        "[newborn A9] AgentBorn raw events = {born_events}, distinct born ids = {distinct_born}, \
         newborn_count = {newborn_count}"
    );
    assert_eq!(
        born_events, newborn_count,
        "A9(a): raw AgentBorn event count ({born_events}) must equal distinct newborn ids \
         ever seen ({newborn_count})"
    );
    assert_eq!(
        distinct_born, born_events,
        "A9(a'): distinct AgentBorn ids ({distinct_born}) must equal the raw event count \
         ({born_events}) — a mismatch means an id was emitted by two births"
    );

    // (b) Membership AT BIRTH TICK.
    let not_enrolled = newborns
        .iter()
        .filter(|id| {
            run.first
                .get(id)
                .map(|f| f.settlement_membership_count == 0)
                .unwrap_or(true)
        })
        .count();
    println!("[newborn A9] newborns NOT enrolled in any settlement at birth tick = {not_enrolled}");
    assert_eq!(
        not_enrolled, 0,
        "A9(b): every newborn must be enrolled in a settlement at its birth tick; \
         {not_enrolled} were not"
    );
    println!("[newborn A9] birth event + membership-at-birth preserved ✓");
}

// ─── Assertion 10: newborn_enters_seeking ───────────────────────────────────
#[test]
fn harness_newborn_enters_seeking() {
    // Type D — secondary confirmation that the rate -> need -> behavior chain
    // works at the POPULATION level (not one lucky outlier). Among newborns
    // alive >= 700 ticks (generous: ~8750 raw Thirst at 0.08/tick, far above any
    // plausible Seeking threshold), at least HALF must enter a need-driven
    // Seeking{Food/Water/Sleep} state at least once, and the absolute count
    // must be >= 1.
    const MIN_LIFE: u64 = 700;
    let run = collect_run(2000);
    let newborns = run.newborns();
    assert!(!newborns.is_empty(), "A10: newborn set must be non-empty (A1 gate)");

    let eligible: Vec<AgentId> = newborns
        .iter()
        .copied()
        .filter(|id| run.lifespan(*id) >= MIN_LIFE)
        .collect();
    assert!(
        !eligible.is_empty(),
        "A10: at least one newborn must live >= {MIN_LIFE} ticks"
    );
    let seekers = eligible
        .iter()
        .filter(|id| run.ever_seeking_resource(**id))
        .count();
    let fraction = seekers as f64 / eligible.len() as f64;
    println!(
        "[newborn A10] eligible (lived >= {MIN_LIFE}) = {}, seekers = {seekers}, fraction = {fraction:.3}",
        eligible.len()
    );
    assert!(seekers >= 1, "A10: at least one long-lived newborn must reach Seeking; got {seekers}");
    assert!(
        fraction >= 0.5,
        "A10: at least half of long-lived newborns must enter Seeking; fraction {fraction:.3} < 0.5"
    );
    println!("[newborn A10] newborns enter the gathering loop (fraction {fraction:.3} >= 0.5) ✓");
}
