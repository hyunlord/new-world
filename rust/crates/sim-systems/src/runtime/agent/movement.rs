//! AgentMovementSystem: moves agents one tile per move interval (seek step, haul home, play, tethered idle walk).
//!
//! Priority: 120 · Interval: 1 tick (acts every `rules.agent.move_interval_ticks`) · Gate: none
//! Reads: `AgentState`, `SeekTarget`, `Agent`, `Child`, `Elder`, `Inventory`, `Lineage`, `Founding`, `Temperament`, `PorterTrip`, `Activity`; `SimResources.settlements`, `tile_grid`, `rules`
//! Writes: `Position`, `MovementRng`; `SimResources.haulers`
//! Events: none
//!
//! V7 Phase 4-β / Section 16-β — `AgentMovementSystem` (priority 120, every tick).
//!
//! Per-tick motion with three state-keyed branches:
//!   - `Idle` (or no `AgentState`): Brownian `(dx, dy)` step in `{-1, 0, +1}^2`
//!     derived from the agent's own [`MovementRng`] state. Determinism comes
//!     from per-agent seeded `splitmix64` — replaying with the same seeds
//!     reproduces every Brownian trajectory byte-for-byte.
//!   - `Seeking { .. }` **with** a [`SeekTarget`] (Section 16-β): a one-tile
//!     **directed** signum step toward `SeekTarget.tile` (pure coordinate
//!     math — RNG-free, inherently deterministic). Without a `SeekTarget` the
//!     agent freezes (ConstructionSite/Agent seeks are co-located).
//!   - `Consuming { .. }`: freeze (the 2-tick consume commit reads the tile).
//!
//! `AgentState::suppresses_movement()` (in `agent_state.rs`) stays locked for
//! the 5 external truth-table harnesses; it now means "suppresses *Brownian*
//! motion" — a directed step is not Brownian, so `Seeking → true` is still
//! consistent with β doing a directed walk.
//!
//! # Priority ordering
//!
//! ```text
//! 90   BuildingStampSystem
//! 100  InfluenceUpdateSystem
//! 110  AgentInfluenceSampleSystem  ← agents read current-side influence
//! 120  AgentMovementSystem         ← agents move AFTER reading influence
//! 130  HungerDecaySystem           ← LANDED in V7 Phase 5-α
//! 1000 InfluenceVisualizationSystem
//! ```
//!
//! Reading influence before moving keeps the `InfluenceSample` value
//! consistent with the agent's pre-move tile — a property β does not yet
//! exploit but γ/δ will need when motion becomes influence-guided.
//!
//! # Why inline splitmix64 (not the `rand` crate)?
//!
//! Phase 4-β scope is "agents move and tests prove determinism." Pulling in
//! `rand` along with `rand_chacha` would be net-new transitive dependency
//! surface for a 12-line PRNG; planning §2.2 explicitly authorises a
//! "deterministic seeded RNG" without dictating the crate. A per-agent `u64`
//! state advanced by `splitmix64` covers every β requirement, ships with
//! zero new deps, and keeps the migration to a proper `rand_chacha`-backed
//! source one type-swap away if a later phase needs it.

use sim_core::components::Lineage;
use sim_core::rng::{
    keyed_step, keyed_unit_f64, STREAM_MOVE_DX, STREAM_MOVE_DY, STREAM_MOVE_PAUSE,
    STREAM_ROUTINE_PLAY,
};
use std::collections::BTreeMap;

use crate::runtime::decision::clock_in_window;
use hecs::World;
use sim_core::components::{
    Activity, ActivityKind, Agent, AgentId, AgentState, Child, Elder, Founding, Inventory,
    PorterTrip, Position, ResourceKind, SeekTarget, TargetKind, Temperament,
    SETTLEMENT_PROXIMITY_RADIUS,
};
use sim_core::rules::{FirewoodRule, ForageRule};
use sim_engine::{RuntimeSystem, SimResources};

/// Per-agent PRNG state used by [`AgentMovementSystem`] to compute the
/// next Brownian step. Each agent owns its own `MovementRng`, so trajectories
/// are independent and reproducible from the initial seed.
///
/// The state is advanced via `splitmix64`, a 64-bit mixing function whose
/// output stream passes the standard PractRand statistical battery for the
/// step counts a single simulation will ever realistically consume.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct MovementRng {
    state: u64,
}

impl MovementRng {
    /// Construct a new RNG seeded with `seed`.
    ///
    /// `seed == 0` is a valid seed — splitmix64 escapes the all-zero state
    /// on its first call because the additive constant is non-zero.
    pub fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Advance the internal state once and return the mixed output.
    ///
    /// Reference: Sebastiano Vigna, "Further scramblings of Marsaglia's
    /// xorshift generators." This is the canonical splitmix64.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    }

    /// Return the next Brownian step axis value in `{-1, 0, +1}`.
    ///
    /// Uses a `u64 % 3` reduction on a fresh 64-bit sample; the bias from
    /// `2^64 mod 3 == 1` is on the order of 1 part in 2^63 and is well below
    /// the tolerance of any β assertion.
    ///
    /// `pub` so harness tests can probe a seed's first-step output and
    /// fail loudly if a chosen test seed accidentally yields `(0,0)`
    /// (which would silently hide a Consuming-tick movement-freeze
    /// regression).
    pub fn next_step(&mut self) -> i32 {
        ((self.next_u64() % 3) as i32) - 1
    }
}

/// Whether an Idle settlement member carrying `carried` food walks straight
/// home under the forage rule: a full load (`>= haul_threshold`), or any food
/// outside the forage window, while farther than the deposit radius.
pub fn haul_due(
    rule: &ForageRule,
    carried: u32,
    in_forage_window: bool,
    home_distance: u32,
) -> bool {
    home_distance > SETTLEMENT_PROXIMITY_RADIUS
        && (carried >= rule.haul_threshold || (carried >= 1 && !in_forage_window))
}

/// Pass 5 haul home generalised to wood: a member carrying a full food load,
/// a full wood load, or any food or wood outside both the forage and firewood
/// windows, while farther than the deposit radius. Without the firewood rule
/// this is exactly [`haul_due`] (false without the forage rule).
pub fn hauls_home(
    forage: Option<&ForageRule>,
    firewood: Option<&FirewoodRule>,
    carried_food: u32,
    carried_wood: u32,
    in_forage_window: bool,
    in_wood_window: bool,
    home_distance: u32,
) -> bool {
    match firewood {
        None => {
            forage.is_some_and(|rule| haul_due(rule, carried_food, in_forage_window, home_distance))
        }
        Some(wood) => {
            home_distance > SETTLEMENT_PROXIMITY_RADIUS
                && (forage.is_some_and(|rule| carried_food >= rule.haul_threshold)
                    || carried_wood >= wood.haul_threshold
                    || ((carried_food >= 1 || carried_wood >= 1)
                        && !in_forage_window
                        && !in_wood_window))
        }
    }
}

/// Phase 4-β movement system.
///
/// On every tick, queries `(&mut Position, &mut MovementRng)` and mutates each
/// agent's position by a Brownian `(dx, dy) ∈ {-1, 0, +1}^2` step, clamped to
/// the tile-grid bounds so coordinates never underflow `u32` or escape the
/// grid.
pub struct AgentMovementSystem;

impl AgentMovementSystem {
    /// Construct a new movement system.
    pub fn new() -> Self {
        Self
    }
}

impl Default for AgentMovementSystem {
    fn default() -> Self {
        Self::new()
    }
}

impl RuntimeSystem for AgentMovementSystem {
    fn name(&self) -> &str {
        "AgentMovementSystem"
    }

    fn priority(&self) -> u32 {
        120
    }

    fn tick_interval(&self) -> u64 {
        1
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let rules = resources.rules.agent;
        if !resources
            .current_tick
            .is_multiple_of(rules.move_interval_ticks.max(1))
        {
            return;
        }
        let child_rule = resources.rules.society.life_stage;
        let evening_radius = rules
            .evening_gather
            .filter(|rule| clock_in_window(resources, rule.start_minute, rule.end_minute))
            .map(|rule| rule.tether_radius);
        // Same gate as the decision system: foraging needs a store target too.
        let forage = rules
            .forage
            .filter(|_| resources.rules.society.stock_target.is_some());
        let forage_window = forage
            .is_some_and(|rule| clock_in_window(resources, rule.start_minute, rule.end_minute));
        // Pass 5: wood loads are hauled home too under the firewood rule.
        let firewood = resources.rules.society.firewood;
        let wood_window = firewood.is_some_and(|rule| {
            clock_in_window(resources, rule.gather_start_minute, rule.gather_end_minute)
        });
        let mut home: BTreeMap<AgentId, (u32, u32)> = BTreeMap::new();
        // Pass 4: elders keep to the camp at all hours. Pass 6: inside the
        // evening-gather window they follow the evening tether when it is tighter.
        let elder_radius = resources.rules.society.aging.map(|rule| {
            match (rule.elder_evening_tether, evening_radius) {
                (true, Some(evening)) => rule.elder_tether_radius.min(evening),
                _ => rule.elder_tether_radius,
            }
        });
        if rules.idle_tether_radius.is_some()
            || child_rule.is_some()
            || evening_radius.is_some()
            || forage.is_some()
            || elder_radius.is_some()
            || firewood.is_some()
        {
            let mut settlements: Vec<_> = resources.settlements.iter().collect();
            settlements.sort_by_key(|(id, _)| **id);
            for (_, settlement) in settlements {
                for id in &settlement.member_agents {
                    home.entry(*id).or_insert(settlement.formation_tile);
                }
            }
        }
        let w = resources.tile_grid.width;
        let h = resources.tile_grid.height;
        if w == 0 || h == 0 {
            return;
        }
        let max_x = (w - 1) as i64;
        let max_y = (h - 1) as i64;
        let mut haul_steps: Vec<AgentId> = Vec::new();
        let mut not_hauling: Vec<AgentId> = Vec::new();
        // Pass 9: under keyed draws an idle step is a pure function of (seed,
        // lineage key, tick, replicate); the sequential stream is not advanced.
        let keyed = rules.keyed_draws;
        let seed = resources.world_seed;
        let tick = resources.current_tick;
        let replicate = resources.noise_replicate_for(tick);
        let playing = rules.routine.map(|_| {
            world
                .query::<(&Agent, &Activity, &Child, &Lineage)>()
                .iter()
                .filter(|(_, (_, activity, _, _))| activity.kind == ActivityKind::Play)
                .map(|(_, (agent, activity, _, lineage))| (agent.id, (activity.place, lineage.key)))
                .collect::<BTreeMap<_, _>>()
        });
        // Pass 16 (`game_v20`): a resident's temperament leans its idle tether
        // (NS) and, for workers, its haul loads (P). None before v20.
        let temperament_rule = rules.temperament;
        for (
            _,
            (
                pos,
                rng,
                state,
                seek,
                agent,
                child,
                inventory,
                elder,
                lineage,
                founding,
                temperament,
                porter,
            ),
        ) in world
            .query::<(
                &mut Position,
                &mut MovementRng,
                Option<&AgentState>,
                Option<&SeekTarget>,
                Option<&Agent>,
                Option<&Child>,
                Option<&Inventory>,
                Option<&Elder>,
                Option<&Lineage>,
                Option<&Founding>,
                Option<&Temperament>,
                Option<&PorterTrip>,
            )>()
            .iter()
        {
            // Pass 20: a porter on its trip is treated as a founder on its
            // way — no haul home, no tether (no `PorterTrip` outside `game_v25`).
            let walker = founding.is_some() || porter.is_some();
            let lean = temperament_rule.zip(temperament);
            // V7 Section 16-β: three movement branches keyed directly on the
            // FSM state (NOT via `suppresses_movement()` — that predicate
            // stays locked in `agent_state.rs` for the 5 external truth-table
            // harnesses; the movement loop decides its own behavior):
            //
            //   1. `Consuming { .. }` → FREEZE. The decision system's 2-tick
            //      consume commit reads `pos.x / pos.y` to locate the resource
            //      tile; a step between `Seeking→Consuming` (this tick) and
            //      `Consuming→Idle` (next tick) would commit on a tile the
            //      agent merely wandered onto. Per Section-3.10 the consume
            //      must decrement at the tile the agent actually reached.
            //
            //   2. `Seeking { .. }` → DIRECTED STEP toward `SeekTarget.tile`
            //      when one is attached (α attaches it for Food/Water/Sleep),
            //      else FREEZE (defensive: ConstructionSite/Agent seeks carry
            //      no SeekTarget and stay co-located). A directed step is one
            //      signum tile per axis — pure coordinate math, RNG-free, so
            //      it is inherently deterministic and never advances the
            //      Brownian stream. `suppresses_movement()` still reports
            //      `true` for `Seeking` and that remains correct: it now means
            //      "suppresses *Brownian* motion" — a directed step is not
            //      Brownian.
            //
            //   3. `Idle` (or no `AgentState`) → BROWNIAN `{-1,0,+1}^2` step,
            //      consuming the per-agent RNG stream (unchanged).
            if let Some(s) = state {
                if matches!(
                    s,
                    AgentState::Consuming {
                        target: TargetKind::Activity
                    }
                ) {
                    if let Some((place, key)) =
                        agent.and_then(|agent| playing.as_ref()?.get(&agent.id))
                    {
                        let dx = i64::from(keyed_step(
                            seed,
                            STREAM_ROUTINE_PLAY,
                            *key,
                            tick,
                            0,
                            replicate,
                        ));
                        let dy = i64::from(keyed_step(
                            seed,
                            STREAM_ROUTINE_PLAY,
                            *key,
                            tick,
                            1,
                            replicate,
                        ));
                        let nx = (i64::from(pos.x) + dx).clamp(
                            i64::from(place.0.saturating_sub(1)),
                            i64::from(place.0.saturating_add(1)).min(max_x),
                        );
                        let ny = (i64::from(pos.y) + dy).clamp(
                            i64::from(place.1.saturating_sub(1)),
                            i64::from(place.1.saturating_add(1)).min(max_y),
                        );
                        pos.x = nx as u32;
                        pos.y = ny as u32;
                    }
                    continue;
                }
                if matches!(s, AgentState::Consuming { .. }) {
                    continue;
                }
                if matches!(s, AgentState::Seeking { .. }) {
                    if let Some(target) = seek {
                        // Directed signum step (no RNG — determinism preserved).
                        let dx = (target.tile.0 as i64 - pos.x as i64).signum();
                        let dy = (target.tile.1 as i64 - pos.y as i64).signum();
                        pos.x = (pos.x as i64 + dx).clamp(0, max_x) as u32;
                        pos.y = (pos.y as i64 + dy).clamp(0, max_y) as u32;
                    }
                    // No Brownian for Seeking either way (target → directed
                    // above; no target → freeze).
                    continue;
                }
            }
            // Pass 4 haul home: a loaded idle member walks straight to its
            // formation tile instead of a Brownian step (no RNG draw).
            // Pass 11: a founder on its way never hauls home (no `Founding`
            // marker exists outside `game_v17`).
            if let (true, Some(agent), None, false) =
                (forage.is_some() || firewood.is_some(), agent, child, walker)
            {
                if let Some(&anchor) = home.get(&agent.id) {
                    let carried = inventory.map_or(0, |inv| inv.get(ResourceKind::Food));
                    let wood = if firewood.is_some() {
                        inventory.map_or(0, |inv| inv.get(ResourceKind::Wood))
                    } else {
                        0
                    };
                    let distance = pos.x.abs_diff(anchor.0).max(pos.y.abs_diff(anchor.1));
                    // Pass 16 P → this worker's own haul loads (elders never haul).
                    let (forage_p, firewood_p) = match lean.filter(|_| elder.is_none()) {
                        Some((rule, t)) => (
                            forage.map(|mut f| {
                                f.haul_threshold = rule.food_haul(f.haul_threshold, t);
                                f
                            }),
                            firewood.map(|mut w| {
                                w.haul_threshold = rule.wood_haul(w.haul_threshold, t);
                                w
                            }),
                        ),
                        None => (forage, firewood),
                    };
                    if hauls_home(
                        forage_p.as_ref(),
                        firewood_p.as_ref(),
                        carried,
                        wood,
                        forage_window,
                        wood_window,
                        distance,
                    ) {
                        pos.x = (pos.x as i64 + (anchor.0 as i64 - pos.x as i64).signum())
                            .clamp(0, max_x) as u32;
                        pos.y = (pos.y as i64 + (anchor.1 as i64 - pos.y as i64).signum())
                            .clamp(0, max_y) as u32;
                        haul_steps.push(agent.id);
                        continue;
                    }
                    // Display-only: a member that stopped hauling before reaching
                    // the deposit radius has not arrived by hauling.
                    if distance > SETTLEMENT_PROXIMITY_RADIUS {
                        not_hauling.push(agent.id);
                    }
                }
            }
            // Idle / no AgentState: Brownian motion. v8–v14 (and any mover
            // without a lineage key) draw from the agent's sequential stream;
            // v15 draws each number from its key (section 7.1).
            let (dx, dy) = match lineage.filter(|_| keyed) {
                Some(lineage) => {
                    let key = lineage.key;
                    if rules.idle_pause_chance > 0.0
                        && keyed_unit_f64(seed, STREAM_MOVE_PAUSE, key, tick, 0, replicate)
                            < rules.idle_pause_chance
                    {
                        continue;
                    }
                    (
                        keyed_step(seed, STREAM_MOVE_DX, key, tick, 0, replicate) as i64,
                        keyed_step(seed, STREAM_MOVE_DY, key, tick, 0, replicate) as i64,
                    )
                }
                None => {
                    if rules.idle_pause_chance > 0.0 {
                        let value = (rng.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
                        if value < rules.idle_pause_chance {
                            continue;
                        }
                    }
                    (rng.next_step() as i64, rng.next_step() as i64)
                }
            };
            let mut nx = (pos.x as i64 + dx).clamp(0, max_x);
            let mut ny = (pos.y as i64 + dy).clamp(0, max_y);
            if let (Some(radius), Some(anchor)) = (
                if child.is_some() {
                    child_rule
                        .map(|rule| rule.tether_radius)
                        .or(rules.idle_tether_radius)
                } else if elder.is_some() && elder_radius.is_some() {
                    // Pass 4: elders keep to the camp (Pass 6: evening tether inside its window).
                    elder_radius
                } else {
                    // Pass 16 NS → an adult's own fallback tether (the evening
                    // gather radius, children and elders are unchanged).
                    evening_radius.or(match (rules.idle_tether_radius, lean) {
                        (Some(base), Some((rule, t))) => Some(rule.tether(base, t)),
                        (base, _) => base,
                    })
                },
                // Pass 11: no tether pulls a founder on its way back home.
                agent.filter(|_| !walker).and_then(|a| home.get(&a.id)),
            ) {
                if (nx as u32)
                    .abs_diff(anchor.0)
                    .max((ny as u32).abs_diff(anchor.1))
                    > radius
                {
                    nx = (pos.x as i64 + (anchor.0 as i64 - pos.x as i64).signum()).clamp(0, max_x);
                    ny = (pos.y as i64 + (anchor.1 as i64 - pos.y as i64).signum()).clamp(0, max_y);
                }
            }
            pos.x = nx as u32;
            pos.y = ny as u32;
        }
        for id in not_hauling {
            resources.haulers.remove(&id);
        }
        resources.haulers.extend(haul_steps);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::material::MaterialRegistry;
    use sim_engine::SimEngine;

    fn fresh_engine() -> SimEngine {
        SimEngine::new(32, 32, MaterialRegistry::new())
    }

    #[test]
    fn hauls_home_generalises_haul_due_to_wood() {
        use sim_core::rules::WorldRules;
        let forage = WorldRules::baseline(0).agent.forage.expect("forage");
        let wood = WorldRules::baseline(0).society.firewood.expect("firewood");
        let far = SETTLEMENT_PROXIMITY_RADIUS + 1;
        // Without the firewood rule: exactly haul_due, wood ignored.
        for (food, window) in [(0, true), (1, true), (4, true), (1, false), (0, false)] {
            assert_eq!(
                hauls_home(Some(&forage), None, food, 9, window, false, far),
                haul_due(&forage, food, window, far)
            );
        }
        assert!(!hauls_home(None, None, 9, 9, false, false, far));
        let h = |food, carried_wood, fw, ww| {
            hauls_home(Some(&forage), Some(&wood), food, carried_wood, fw, ww, far)
        };
        // Full loads go home inside the windows.
        assert!(h(4, 0, true, true) && h(0, 2, true, true));
        // Partial loads wait while either window is open...
        assert!(
            !h(1, 0, true, true)
                && !h(0, 1, true, false)
                && !h(0, 1, false, true)
                && !h(3, 1, true, true)
        );
        // ...and go home once both are closed.
        assert!(h(1, 0, false, false) && h(0, 1, false, false));
        assert!(!h(0, 0, false, false));
        // Never inside the deposit radius.
        assert!(!hauls_home(
            Some(&forage),
            Some(&wood),
            4,
            2,
            false,
            false,
            SETTLEMENT_PROXIMITY_RADIUS
        ));
    }

    #[test]
    fn haul_due_needs_a_load_or_the_end_of_the_window_and_distance() {
        let rule = ForageRule {
            start_minute: 360,
            end_minute: 1080,
            forage_radius: 16,
            haul_threshold: 4,
            max_foragers_per_bush: 1,
        };
        let far = SETTLEMENT_PROXIMITY_RADIUS + 1;
        assert!(haul_due(&rule, 4, true, far));
        assert!(!haul_due(&rule, 3, true, far));
        assert!(haul_due(&rule, 1, false, far));
        assert!(!haul_due(&rule, 0, false, far));
        assert!(!haul_due(&rule, 9, true, SETTLEMENT_PROXIMITY_RADIUS));
    }

    #[test]
    fn metadata() {
        let s = AgentMovementSystem::new();
        assert_eq!(s.name(), "AgentMovementSystem");
        assert_eq!(s.priority(), 120);
        assert_eq!(s.tick_interval(), 1);
    }

    #[test]
    fn keyed_mix_is_one_movement_rng_output() {
        // Pass 9: `sim_core::rng::mix` is the splitmix64 step this stream uses.
        for x in [0u64, 1, 42, u64::MAX, 0x9E37_79B9_7F4A_7C15] {
            assert_eq!(sim_core::rng::mix(x), MovementRng::new(x).next_u64());
        }
    }

    #[test]
    fn splitmix_escapes_zero_seed() {
        let mut r = MovementRng::new(0);
        assert_ne!(r.next_u64(), 0);
    }

    #[test]
    fn same_seed_produces_same_stream() {
        let mut a = MovementRng::new(42);
        let mut b = MovementRng::new(42);
        for _ in 0..16 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn agent_moves_after_one_tick() {
        let mut e = fresh_engine();
        let id = e.world.spawn((Position::new(10, 10), MovementRng::new(42)));
        let mut sys = AgentMovementSystem::new();
        sys.tick(&mut e.world, &mut e.resources);
        let p = *e.world.get::<&Position>(id).unwrap();
        // (dx, dy) ∈ {-1,0,+1}^2 → distance ≤ 1 on each axis
        assert!(p.x.abs_diff(10) <= 1);
        assert!(p.y.abs_diff(10) <= 1);
    }

    #[test]
    fn boundary_clamp_low() {
        let mut e = fresh_engine();
        // Spawn at (0,0) and run many ticks; coords must stay in-bounds.
        let id = e.world.spawn((Position::new(0, 0), MovementRng::new(7)));
        let mut sys = AgentMovementSystem::new();
        for _ in 0..200 {
            sys.tick(&mut e.world, &mut e.resources);
            let p = *e.world.get::<&Position>(id).unwrap();
            assert!(p.x < 32);
            assert!(p.y < 32);
        }
    }

    #[test]
    fn seeking_without_target_freezes() {
        // Section 16-β: a Seeking agent WITHOUT a SeekTarget freezes (no
        // Brownian — suppressed; no directed step — no goal). α attaches a
        // SeekTarget for Food/Water/Sleep; ConstructionSite/Agent seeks get
        // none and must stay co-located.
        use sim_core::components::{AgentState, TargetKind};
        let mut e = fresh_engine();
        let id = e.world.spawn((
            Position::new(10, 10),
            MovementRng::new(42),
            AgentState::Seeking {
                target: TargetKind::Food,
            },
        ));
        let mut sys = AgentMovementSystem::new();
        for _ in 0..32 {
            sys.tick(&mut e.world, &mut e.resources);
        }
        let p = *e.world.get::<&Position>(id).unwrap();
        assert_eq!(p.x, 10);
        assert_eq!(p.y, 10);
    }

    #[test]
    fn seeking_with_target_moves_toward() {
        // Section 16-β: a Seeking agent WITH a SeekTarget takes one signum
        // step per tick toward the goal, then holds once arrived (signum 0).
        use sim_core::components::{AgentState, SeekTarget, TargetKind};
        let mut e = fresh_engine();
        let id = e.world.spawn((
            Position::new(10, 10),
            MovementRng::new(42),
            AgentState::Seeking {
                target: TargetKind::Food,
            },
            SeekTarget::new((15, 10)),
        ));
        let mut sys = AgentMovementSystem::new();
        // One tick: dx=signum(15-10)=+1, dy=signum(10-10)=0 → (11,10).
        sys.tick(&mut e.world, &mut e.resources);
        let p = *e.world.get::<&Position>(id).unwrap();
        assert_eq!(
            (p.x, p.y),
            (11, 10),
            "one signum step toward goal, y unchanged"
        );
        // Four more ticks reach (15,10); further ticks hold (signum 0).
        for _ in 0..8 {
            sys.tick(&mut e.world, &mut e.resources);
        }
        let p = *e.world.get::<&Position>(id).unwrap();
        assert_eq!((p.x, p.y), (15, 10), "reaches target then holds station");
    }

    #[test]
    fn idle_state_still_moves() {
        use sim_core::components::AgentState;
        let mut e = fresh_engine();
        let id = e.world.spawn((
            Position::new(10, 10),
            MovementRng::new(42),
            AgentState::Idle,
        ));
        let mut sys = AgentMovementSystem::new();
        let mut moved = false;
        for _ in 0..32 {
            sys.tick(&mut e.world, &mut e.resources);
            let p = *e.world.get::<&Position>(id).unwrap();
            if p.x != 10 || p.y != 10 {
                moved = true;
                break;
            }
        }
        assert!(moved, "Idle agents must still execute Brownian motion");
    }

    #[test]
    fn consuming_state_freezes_movement_in_runtime() {
        // Consuming agents must NOT move in the runtime, even though the
        // public `AgentState::suppresses_movement()` predicate returns
        // false for them (locked Assertion-3 truth table). Reason: the
        // decision system's Consuming-tick reads `pos.x / pos.y` to find
        // the resource tile, so any Brownian step between
        // `Seeking→Consuming` and `Consuming→Idle` would commit on the
        // wrong tile. Use a movement-producing seed (verified separately
        // — seed=1 yields `(1,0)` on its first step) so the test would
        // observably fail if the freeze regressed.
        use sim_core::components::{AgentState, TargetKind};
        let mut e = fresh_engine();
        let id = e.world.spawn((
            Position::new(10, 10),
            MovementRng::new(1),
            AgentState::Consuming {
                target: TargetKind::Water,
            },
        ));
        let mut sys = AgentMovementSystem::new();
        for _ in 0..32 {
            sys.tick(&mut e.world, &mut e.resources);
        }
        let p = *e.world.get::<&Position>(id).unwrap();
        assert_eq!(
            (p.x, p.y),
            (10, 10),
            "Consuming agents must NOT execute Brownian motion (would corrupt the Consuming-tick commit position)"
        );
        // Sanity: the chosen seed actually produces movement when not
        // suppressed — a fresh RNG with the same seed yields non-zero step.
        let mut probe = MovementRng::new(1);
        let dx = probe.next_step();
        let dy = probe.next_step();
        assert!(
            dx != 0 || dy != 0,
            "seed 1 must produce a non-zero step (regression guard against an accidentally-(0,0) seed)"
        );
        // And the public predicate STILL reports false (Assertion-3 contract).
        assert!(!AgentState::Consuming {
            target: TargetKind::Water
        }
        .suppresses_movement());
        assert!(!AgentState::Consuming {
            target: TargetKind::Food
        }
        .suppresses_movement());
    }

    #[test]
    fn distinct_seeds_diverge() {
        let mut e = fresh_engine();
        let a = e.world.spawn((Position::new(16, 16), MovementRng::new(1)));
        let b = e.world.spawn((Position::new(16, 16), MovementRng::new(2)));
        let mut sys = AgentMovementSystem::new();
        // After enough ticks two independent walks must visit different cells
        // somewhere along the way. Compare full trajectories, not endpoints.
        let mut diverged = false;
        for _ in 0..32 {
            sys.tick(&mut e.world, &mut e.resources);
            let pa = *e.world.get::<&Position>(a).unwrap();
            let pb = *e.world.get::<&Position>(b).unwrap();
            if pa != pb {
                diverged = true;
                break;
            }
        }
        assert!(
            diverged,
            "distinct seeds should produce distinct trajectories"
        );
    }
}
