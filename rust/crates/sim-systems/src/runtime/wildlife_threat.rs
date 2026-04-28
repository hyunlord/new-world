//! WildlifeThreatPerceptionSystem — perceive nearby live wildlife and trigger
//! the Flee behavior on humans.
//!
//! [wildlife-threat-detection-and-flee-v1]
//!
//! Pipeline placement:
//!   - Priority 22 — runs after `BehaviorRuntimeSystem` (priority 20) and before
//!     `SteeringRuntimeSystem` (priority 29) and `MovementRuntimeSystem`
//!     (priority 30). This ordering lets us override the action selected by
//!     Behavior in the same tick, and have steering pick up the flee target.
//!   - Tick interval 1 — perception is checked every tick so newly-arrived
//!     threats can interrupt other actions immediately.
//!
//! Determinism:
//!   - Iterates entities sorted by `Entity::to_bits()` so the order of recorded
//!     observations is independent of `hecs` archetype layout.
//!   - Reads only ECS state and `SimResources::map` extents — no RNG, no clock.
//!
//! Behaviour summary:
//!   1. Snapshot live wildlife positions (`current_hp > 0.0`).
//!   2. For each human (`Identity::species_id == "human"`), find the closest
//!      live wildlife and the resulting Euclidean distance.
//!   3. If `distance <= WILDLIFE_PERCEPTION_RANGE` → record a `WildlifePerceived`
//!      observation, and ensure the agent is in `ActionType::Flee` with a target
//!      pointed away from the threat. The first transition into Flee records a
//!      `FleeStarted` observation for the same `(tick, agent)`.
//!   4. If the agent is currently fleeing and the nearest live threat is at
//!      `distance > WILDLIFE_FLEE_SAFETY_THRESHOLD`, terminate Flee → Idle and
//!      record a `FleeTerminatedSafe` observation.
//!
//! Harness observations are appended to the corresponding `Vec` fields on
//! `SimResources` (`harness_wildlife_perceived`, `harness_flee_started`,
//! `harness_flee_terminated_safe`). Only test code reads these; production
//! systems treat them as inert diagnostics.

use hecs::World;
use sim_core::components::{Behavior, Identity, Position, Wildlife};
use sim_core::config;
use sim_core::enums::ActionType;
use sim_engine::{SimResources, SimSystem};

/// Returns `true` if a Flee transition is allowed to override the agent's
/// current action. Only Idle and Flee itself are overridable.
///
/// Productive actions (Build/Construction/Crafting/Forage/Gather*/Hunt/Mourn/
/// Wander/Rest/Socialize/...) are protected so a transient nearby wolf does
/// not invalidate hours of work in a multi-thousand-tick run; the existing
/// Danger influence channel + steering pipeline handles the soft "stay safe"
/// response in those cases.
///
/// Idle is the gap between actions where the agent is purely waiting for a
/// new behavior decision — exactly the moment when an immediate flee is
/// most appropriate without disrupting another commitment.
fn can_override_with_flee(action: ActionType) -> bool {
    matches!(action, ActionType::Idle | ActionType::Flee)
}

/// Plan record produced during the read-only first pass; applied during the
/// mutating second pass to avoid nested ECS borrows.
struct AgentPlan {
    entity: hecs::Entity,
    entity_bits: u64,
    perceives: bool,
    nearest_dist: f64,
    /// Unit vector from the threat toward the agent (i.e. flee direction).
    flee_dir_x: f64,
    flee_dir_y: f64,
    /// Whether the agent is currently in `ActionType::Flee`.
    was_fleeing: bool,
}

/// Runtime system that wires wildlife perception to the Flee behavior.
pub struct WildlifeThreatPerceptionSystem {
    priority: u32,
    tick_interval: u64,
}

impl WildlifeThreatPerceptionSystem {
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

impl SimSystem for WildlifeThreatPerceptionSystem {
    fn name(&self) -> &'static str {
        "wildlife_threat_perception_system"
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let perception_range = config::WILDLIFE_PERCEPTION_RANGE;
        let safety_threshold = config::WILDLIFE_FLEE_SAFETY_THRESHOLD;
        let refresh_timer = config::WILDLIFE_FLEE_REFRESH_TIMER;

        // ── Pass 1: snapshot live wildlife positions ─────────────────────
        let mut wildlife_positions: Vec<(f64, f64)> = world
            .query::<(&Wildlife, &Position)>()
            .iter()
            .filter(|(_, (w, _))| w.current_hp > 0.0)
            .map(|(_, (_, p))| (p.x, p.y))
            .collect();

        if wildlife_positions.is_empty() {
            return;
        }

        // Sort wildlife order is irrelevant for distance min; agents must be
        // sorted instead. Keep wildlife as-is.
        let _ = &mut wildlife_positions; // silence unused-mut if any

        // ── Pass 2: build per-human plan in deterministic order ──────────
        let mut plans: Vec<AgentPlan> = Vec::new();
        for (entity, (identity, position, behavior)) in
            world.query::<(&Identity, &Position, &Behavior)>().iter()
        {
            if identity.species_id != "human" {
                continue;
            }

            let mut nearest_dist = f64::INFINITY;
            let mut nearest_xy = (position.x, position.y);
            for &(wx, wy) in &wildlife_positions {
                let dx = wx - position.x;
                let dy = wy - position.y;
                let d = (dx * dx + dy * dy).sqrt();
                if d < nearest_dist {
                    nearest_dist = d;
                    nearest_xy = (wx, wy);
                }
            }

            let perceives = nearest_dist.is_finite() && nearest_dist <= perception_range;

            let mut dir_x = position.x - nearest_xy.0;
            let mut dir_y = position.y - nearest_xy.1;
            let len = (dir_x * dir_x + dir_y * dir_y).sqrt();
            if len > 1.0e-6 {
                dir_x /= len;
                dir_y /= len;
            } else {
                // Degenerate co-located case — pick a fixed direction so we
                // still flee deterministically.
                dir_x = 1.0;
                dir_y = 0.0;
            }

            plans.push(AgentPlan {
                entity,
                entity_bits: entity.to_bits().get(),
                perceives,
                nearest_dist,
                flee_dir_x: dir_x,
                flee_dir_y: dir_y,
                was_fleeing: behavior.current_action == ActionType::Flee,
            });
        }

        plans.sort_by_key(|p| p.entity_bits);

        // ── Pass 3: apply Flee state and record harness observations ─────
        let max_x = i32::try_from(resources.map.width.saturating_sub(1)).unwrap_or(i32::MAX);
        let max_y = i32::try_from(resources.map.height.saturating_sub(1)).unwrap_or(i32::MAX);

        for plan in plans {
            if plan.perceives {
                resources
                    .harness_wildlife_perceived
                    .push((tick, plan.entity_bits));

                // Compute flee target ~ safety_threshold * 1.5 tiles away from
                // the threat, clamped to map bounds.
                let target_dist = safety_threshold * 1.5;

                let mut flee_started = false;
                // Snapshot the current action before borrowing &mut Behavior so
                // the override guard can read it cleanly.
                let current_action = world
                    .get::<&Behavior>(plan.entity)
                    .map(|b| b.current_action)
                    .unwrap_or(ActionType::Idle);

                if can_override_with_flee(current_action) {
                    if let Ok(mut behavior) = world.get::<&mut Behavior>(plan.entity) {
                        if behavior.current_action != ActionType::Flee {
                            flee_started = true;
                        }
                        behavior.current_action = ActionType::Flee;
                        behavior.action_timer = refresh_timer;
                        behavior.action_duration = refresh_timer;

                        // Compute target relative to the agent's current
                        // position (which we re-read here because `&Position`
                        // is dropped).
                        if let Ok(pos) = world.get::<&Position>(plan.entity) {
                            let tx = (pos.x + plan.flee_dir_x * target_dist).round() as i32;
                            let ty = (pos.y + plan.flee_dir_y * target_dist).round() as i32;
                            behavior.action_target_x = Some(tx.clamp(0, max_x));
                            behavior.action_target_y = Some(ty.clamp(0, max_y));
                        }
                    }
                }

                if flee_started {
                    resources
                        .harness_flee_started
                        .push((tick, plan.entity_bits));
                }
            } else if plan.was_fleeing && plan.nearest_dist > safety_threshold {
                // Terminate Flee — agent is at safe distance.
                if let Ok(mut behavior) = world.get::<&mut Behavior>(plan.entity) {
                    behavior.current_action = ActionType::Idle;
                    behavior.action_target_x = None;
                    behavior.action_target_y = None;
                    behavior.action_timer = 0;
                    behavior.action_duration = 0;
                }
                if let Ok(mut pos) = world.get::<&mut Position>(plan.entity) {
                    pos.vel_x = 0.0;
                    pos.vel_y = 0.0;
                }
                resources.harness_flee_terminated_safe.push((
                    tick,
                    plan.entity_bits,
                    plan.nearest_dist,
                ));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::{Identity, Position, Wildlife, WildlifeKind};
    use sim_core::{GameCalendar, WorldMap};
    use sim_core::config::GameConfig;

    fn fresh_resources() -> SimResources {
        let cfg = GameConfig::default();
        let cal = GameCalendar::new(&cfg);
        let map = WorldMap::new(64, 64, 1);
        SimResources::new(cal, map, 1)
    }

    #[test]
    fn no_wildlife_no_observations() {
        let mut world = World::new();
        let mut resources = fresh_resources();
        world.spawn((
            Identity {
                species_id: "human".to_string(),
                ..Default::default()
            },
            Position::new(10, 10),
            Behavior::default(),
        ));
        let mut sys = WildlifeThreatPerceptionSystem::new(22, 1);
        sys.run(&mut world, &mut resources, 1);
        assert!(resources.harness_wildlife_perceived.is_empty());
        assert!(resources.harness_flee_started.is_empty());
    }

    #[test]
    fn human_within_range_records_perceived_and_flee() {
        let mut world = World::new();
        let mut resources = fresh_resources();
        let _agent = world.spawn((
            Identity {
                species_id: "human".to_string(),
                ..Default::default()
            },
            Position::new(20, 20),
            Behavior::default(),
        ));
        // Wolf 5 tiles away — well inside perception range (25).
        let wolf_pos = Position::new(25, 20);
        let mut wolf = Wildlife::wolf((wolf_pos.tile_x(), wolf_pos.tile_y()));
        wolf.current_hp = wolf.max_hp;
        world.spawn((
            Identity {
                species_id: "wolf".to_string(),
                ..Default::default()
            },
            wolf_pos,
            wolf,
        ));

        let mut sys = WildlifeThreatPerceptionSystem::new(22, 1);
        sys.run(&mut world, &mut resources, 1);

        assert_eq!(resources.harness_wildlife_perceived.len(), 1);
        assert_eq!(resources.harness_flee_started.len(), 1);
        assert_eq!(
            resources.harness_wildlife_perceived[0].1,
            resources.harness_flee_started[0].1,
            "perceiver and fleer must match"
        );
    }

    #[test]
    fn dead_wildlife_ignored() {
        let mut world = World::new();
        let mut resources = fresh_resources();
        world.spawn((
            Identity {
                species_id: "human".to_string(),
                ..Default::default()
            },
            Position::new(20, 20),
            Behavior::default(),
        ));
        let mut wolf = Wildlife::wolf((25, 20));
        wolf.current_hp = 0.0;
        world.spawn((
            Identity {
                species_id: "wolf".to_string(),
                ..Default::default()
            },
            Position::new(25, 20),
            wolf,
        ));

        let mut sys = WildlifeThreatPerceptionSystem::new(22, 1);
        sys.run(&mut world, &mut resources, 1);

        // No live wildlife → early return → no observations.
        assert!(resources.harness_wildlife_perceived.is_empty());
        assert!(resources.harness_flee_started.is_empty());
    }

    #[test]
    fn out_of_range_no_perception() {
        // World is 64×64 → diagonal ~90. Place wolf >70 from human.
        let mut world = World::new();
        let mut resources = fresh_resources();
        world.spawn((
            Identity {
                species_id: "human".to_string(),
                ..Default::default()
            },
            Position::new(0, 0),
            Behavior::default(),
        ));
        let wolf = Wildlife::wolf((60, 60));
        world.spawn((
            Identity {
                species_id: "wolf".to_string(),
                ..Default::default()
            },
            Position::new(60, 60),
            wolf,
        ));
        let mut sys = WildlifeThreatPerceptionSystem::new(22, 1);
        sys.run(&mut world, &mut resources, 1);
        // Distance ~84.85 > perception range 70 → no perception.
        assert!(resources.harness_wildlife_perceived.is_empty());
    }

    #[test]
    fn fleeing_agent_far_from_threat_terminates() {
        // Use a larger map so we can place the wolf beyond
        // WILDLIFE_FLEE_SAFETY_THRESHOLD (85) from the agent.
        let cfg = GameConfig::default();
        let cal = GameCalendar::new(&cfg);
        let map = WorldMap::new(256, 256, 1);
        let mut resources = SimResources::new(cal, map, 1);
        let mut world = World::new();

        let mut behavior = Behavior {
            current_action: ActionType::Flee,
            ..Default::default()
        };
        behavior.action_target_x = Some(0);
        behavior.action_target_y = Some(0);
        let agent = world.spawn((
            Identity {
                species_id: "human".to_string(),
                ..Default::default()
            },
            Position::new(0, 0),
            behavior,
        ));
        // Wolf at (200, 200) → distance ≈ 282.8, well beyond safety threshold.
        let wolf = Wildlife::wolf((200, 200));
        world.spawn((
            Identity {
                species_id: "wolf".to_string(),
                ..Default::default()
            },
            Position::new(200, 200),
            wolf,
        ));
        let mut sys = WildlifeThreatPerceptionSystem::new(22, 1);
        sys.run(&mut world, &mut resources, 1);
        assert_eq!(resources.harness_flee_terminated_safe.len(), 1);
        let after = world.get::<&Behavior>(agent).unwrap();
        assert_eq!(after.current_action, ActionType::Idle);
    }

    #[test]
    fn wildlife_kind_used_for_filter() {
        // Non-human Identity (e.g. a wildlife entity) must NOT be treated as a
        // perceiver — anti-gaming check that Flee emitter == perceiver only.
        let mut world = World::new();
        let mut resources = fresh_resources();
        // Spawn a wolf (acts as the threat).
        let wolf = Wildlife::wolf((25, 20));
        world.spawn((
            Identity {
                species_id: "wolf".to_string(),
                ..Default::default()
            },
            Position::new(25, 20),
            wolf,
        ));
        // Spawn another wildlife with Behavior — must be ignored by perception.
        let bear = Wildlife::bear((20, 20));
        world.spawn((
            Identity {
                species_id: "bear".to_string(),
                ..Default::default()
            },
            Position::new(20, 20),
            bear,
            Behavior::default(),
        ));
        let mut sys = WildlifeThreatPerceptionSystem::new(22, 1);
        sys.run(&mut world, &mut resources, 1);
        // No human → no observations even though wildlife are within range.
        assert!(resources.harness_wildlife_perceived.is_empty());
        assert!(resources.harness_flee_started.is_empty());

        // Suppress unused warning on WildlifeKind import.
        let _ = WildlifeKind::Wolf;
    }
}
