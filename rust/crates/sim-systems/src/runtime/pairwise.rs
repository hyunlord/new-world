use std::cmp::Ordering;
use std::collections::HashMap;

use hecs::{Entity, World};
use rand::Rng;
use sim_core::causal_log::{CausalEvent, CauseRef};
use sim_core::components::{Behavior, Identity, Needs, Personality, Position, Social, Stress};
use sim_core::config;
use sim_core::enums::{ActionType, HexacoAxis, InteractionType, NeedType};
use sim_core::ids::{EntityId, SettlementId};
use sim_engine::{SimEvent, SimEventType, SimResources, SimSystem};

/// Warm-tier runtime system that creates and updates social edges from nearby contact.
#[derive(Debug, Clone)]
pub struct PairwiseInteractionSystem {
    priority: u32,
    tick_interval: u64,
}

impl PairwiseInteractionSystem {
    /// Creates a new pairwise interaction runtime system.
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct AgentSnapshot {
    entity: Entity,
    id: EntityId,
    x: f64,
    y: f64,
    settlement_id: Option<SettlementId>,
    action: ActionType,
    target_x: Option<i32>,
    target_y: Option<i32>,
    carry: f64,
    hunger: f64,
    stress: f64,
    agreeableness: f64,
}

#[derive(Debug, Clone, Copy)]
struct CandidatePair {
    left: usize,
    right: usize,
    distance_sq: f64,
}

#[derive(Debug, Clone, Copy)]
struct PendingInteraction {
    source_entity: Entity,
    source_id: EntityId,
    target_id: EntityId,
    interaction_type: InteractionType,
}

#[derive(Debug, Clone, Copy)]
struct InteractionDeltas {
    trust: f64,
    affinity: f64,
    familiarity: f64,
}

impl SimSystem for PairwiseInteractionSystem {
    fn name(&self) -> &'static str {
        "pairwise_interaction_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let snapshots = collect_snapshots(world);
        if snapshots.len() < 2 {
            return;
        }

        let mut candidates = collect_candidate_pairs(&snapshots);
        candidates.sort_by(|left, right| {
            left.distance_sq
                .partial_cmp(&right.distance_sq)
                .unwrap_or(Ordering::Equal)
                .then_with(|| snapshots[left.left].id.0.cmp(&snapshots[right.left].id.0))
                .then_with(|| snapshots[left.right].id.0.cmp(&snapshots[right.right].id.0))
        });

        let mut pending = Vec::new();
        let mut outgoing_counts = HashMap::<EntityId, u32>::new();
        for candidate in candidates {
            let left = snapshots[candidate.left];
            let right = snapshots[candidate.right];
            maybe_queue_interaction(
                &mut pending,
                &mut outgoing_counts,
                left,
                right,
                &mut resources.rng,
            );
            maybe_queue_interaction(
                &mut pending,
                &mut outgoing_counts,
                right,
                left,
                &mut resources.rng,
            );
        }

        for interaction in pending {
            let deltas = interaction_deltas(interaction.interaction_type);
            let applied = apply_interaction(world, interaction, tick, deltas);
            if !applied {
                continue;
            }

            push_pairwise_causal(
                resources,
                interaction.source_id,
                tick,
                interaction.target_id,
                interaction.interaction_type,
                deltas.trust,
            );
            push_pairwise_event(
                resources,
                interaction.source_entity,
                interaction.target_id,
                tick,
                interaction.interaction_type,
                deltas.trust,
            );
        }
    }
}

fn collect_snapshots(world: &World) -> Vec<AgentSnapshot> {
    let mut snapshots = Vec::new();
    let mut query = world.query::<(
        &Position,
        &Identity,
        Option<&Behavior>,
        Option<&Needs>,
        Option<&Stress>,
        Option<&Personality>,
    )>();

    for (entity, (position, identity, behavior_opt, needs_opt, stress_opt, personality_opt)) in
        &mut query
    {
        let hunger = needs_opt
            .map(|needs| needs.get(NeedType::Hunger))
            .unwrap_or(1.0)
            .clamp(0.0, 1.0);
        let stress = stress_opt
            .map(|stress_component| stress_component.level)
            .unwrap_or(0.0)
            .clamp(0.0, 1.0);
        let agreeableness = personality_opt
            .map(|personality| personality.axis(HexacoAxis::A))
            .unwrap_or(0.5)
            .clamp(0.0, 1.0);

        snapshots.push(AgentSnapshot {
            entity,
            id: EntityId(entity.id() as u64),
            x: position.x,
            y: position.y,
            settlement_id: identity.settlement_id,
            action: behavior_opt
                .map(|behavior| behavior.current_action)
                .unwrap_or(ActionType::Idle),
            target_x: behavior_opt.and_then(|behavior| behavior.action_target_x),
            target_y: behavior_opt.and_then(|behavior| behavior.action_target_y),
            carry: behavior_opt
                .map(|behavior| f64::from(behavior.carry))
                .unwrap_or(0.0),
            hunger,
            stress,
            agreeableness,
        });
    }

    snapshots.sort_by_key(|snapshot| snapshot.id.0);
    snapshots
}

fn collect_candidate_pairs(snapshots: &[AgentSnapshot]) -> Vec<CandidatePair> {
    let mut pairs = Vec::new();
    let distance_sq_limit = config::INTERACTION_MAX_DISTANCE * config::INTERACTION_MAX_DISTANCE;

    for left in 0..snapshots.len() {
        for right in (left + 1)..snapshots.len() {
            let dx = snapshots[left].x - snapshots[right].x;
            let dy = snapshots[left].y - snapshots[right].y;
            let distance_sq = dx * dx + dy * dy;
            if distance_sq <= distance_sq_limit {
                pairs.push(CandidatePair {
                    left,
                    right,
                    distance_sq,
                });
            }
        }
    }

    pairs
}

fn maybe_queue_interaction(
    pending: &mut Vec<PendingInteraction>,
    outgoing_counts: &mut HashMap<EntityId, u32>,
    source: AgentSnapshot,
    target: AgentSnapshot,
    rng: &mut impl Rng,
) {
    let current_count = outgoing_counts.get(&source.id).copied().unwrap_or(0);
    if current_count >= config::INTERACTION_MAX_PER_TICK {
        return;
    }

    let same_settlement =
        source.settlement_id.is_some() && source.settlement_id == target.settlement_id;
    let interaction_type = select_interaction_type(source, target, same_settlement, rng);
    pending.push(PendingInteraction {
        source_entity: source.entity,
        source_id: source.id,
        target_id: target.id,
        interaction_type,
    });
    outgoing_counts.insert(source.id, current_count + 1);
}

fn select_interaction_type(
    source: AgentSnapshot,
    target: AgentSnapshot,
    same_settlement: bool,
    rng: &mut impl Rng,
) -> InteractionType {
    if same_settlement && source.action == target.action && is_productive_action(source.action) {
        return InteractionType::Cooperate;
    }

    if same_settlement && source.carry > 0.0 && target.hunger < 0.3 {
        return InteractionType::Share;
    }

    if is_resource_competition(source, target) && rng.gen_bool(0.10) {
        return InteractionType::Conflict;
    }

    if same_settlement && target.stress > 0.7 && source.agreeableness > 0.5 {
        return InteractionType::Comfort;
    }

    InteractionType::Talk
}

fn is_productive_action(action: ActionType) -> bool {
    matches!(
        action,
        ActionType::Forage
            | ActionType::Build
            | ActionType::GatherWood
            | ActionType::GatherStone
            | ActionType::Hunt
    )
}

fn is_resource_competition(source: AgentSnapshot, target: AgentSnapshot) -> bool {
    matches!(
        source.action,
        ActionType::Forage
            | ActionType::GatherWood
            | ActionType::GatherStone
            | ActionType::GatherHerbs
            | ActionType::Hunt
    ) && source.action == target.action
        && source.target_x.is_some()
        && source.target_x == target.target_x
        && source.target_y.is_some()
        && source.target_y == target.target_y
}

fn interaction_deltas(interaction_type: InteractionType) -> InteractionDeltas {
    let familiarity = config::INTERACTION_FAMILIARITY_DELTA;
    match interaction_type {
        InteractionType::Talk => InteractionDeltas {
            trust: config::INTERACTION_TALK_TRUST_DELTA,
            affinity: config::INTERACTION_TALK_AFFINITY_DELTA,
            familiarity,
        },
        InteractionType::Cooperate => InteractionDeltas {
            trust: config::INTERACTION_COOPERATE_TRUST_DELTA,
            affinity: config::INTERACTION_COOPERATE_AFFINITY_DELTA,
            familiarity,
        },
        InteractionType::Share => InteractionDeltas {
            trust: config::INTERACTION_SHARE_TRUST_DELTA,
            affinity: config::INTERACTION_SHARE_AFFINITY_DELTA,
            familiarity,
        },
        InteractionType::Conflict => InteractionDeltas {
            trust: config::INTERACTION_CONFLICT_TRUST_DELTA,
            affinity: config::INTERACTION_CONFLICT_AFFINITY_DELTA,
            familiarity,
        },
        InteractionType::Comfort => InteractionDeltas {
            trust: config::INTERACTION_COMFORT_TRUST_DELTA,
            affinity: config::INTERACTION_COMFORT_AFFINITY_DELTA,
            familiarity,
        },
    }
}

fn apply_interaction(
    world: &mut World,
    interaction: PendingInteraction,
    tick: u64,
    deltas: InteractionDeltas,
) -> bool {
    let Ok(mut social) = world.get::<&mut Social>(interaction.source_entity) else {
        return false;
    };

    let edge = social.get_or_create_edge(interaction.target_id);
    edge.affinity = (edge.affinity + deltas.affinity).clamp(0.0, 100.0);
    edge.trust = (edge.trust + deltas.trust).clamp(0.0, 1.0);
    edge.familiarity = (edge.familiarity + deltas.familiarity).clamp(0.0, 1.0);
    edge.last_interaction_tick = tick;
    edge.update_type();
    social.edges.sort_by_key(|social_edge| social_edge.target.0);
    enforce_edge_cap(&mut social);
    true
}

fn enforce_edge_cap(social: &mut Social) {
    while social.edges.len() > config::SOCIAL_EDGE_CAP {
        let min_index = social
            .edges
            .iter()
            .enumerate()
            .min_by(|(_, left), (_, right)| {
                left.familiarity
                    .partial_cmp(&right.familiarity)
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| left.target.0.cmp(&right.target.0))
            })
            .map(|(index, _)| index);
        let Some(index) = min_index else {
            break;
        };
        social.edges.remove(index);
    }
}

fn push_pairwise_causal(
    resources: &mut SimResources,
    entity_id: EntityId,
    tick: u64,
    target_id: EntityId,
    interaction_type: InteractionType,
    magnitude: f64,
) {
    resources.causal_log.push(
        entity_id,
        CausalEvent {
            tick,
            cause: CauseRef {
                system: "pairwise_interaction_system".to_string(),
                kind: interaction_key(interaction_type).to_string(),
                entity: Some(entity_id),
                building: None,
                settlement: None,
            },
            effect_key: format!("social_edge:{}", target_id.0),
            summary_key: interaction_summary_key(interaction_type).to_string(),
            magnitude,
        },
    );
}

fn push_pairwise_event(
    resources: &mut SimResources,
    actor: Entity,
    target: EntityId,
    tick: u64,
    interaction_type: InteractionType,
    value: f64,
) {
    let event_type = match interaction_type {
        InteractionType::Talk => return,
        InteractionType::Conflict => SimEventType::SocialConflict,
        InteractionType::Cooperate | InteractionType::Share | InteractionType::Comfort => {
            SimEventType::SocialCooperation
        }
    };

    resources.event_store.push(SimEvent {
        tick,
        event_type,
        actor: actor.id(),
        target: u32::try_from(target.0).ok(),
        tags: vec!["social".to_string(), "pairwise".to_string()],
        cause: interaction_key(interaction_type).to_string(),
        value,
    });
}

fn interaction_key(interaction_type: InteractionType) -> &'static str {
    match interaction_type {
        InteractionType::Talk => "talk",
        InteractionType::Cooperate => "cooperate",
        InteractionType::Share => "share",
        InteractionType::Conflict => "conflict",
        InteractionType::Comfort => "comfort",
    }
}

fn interaction_summary_key(interaction_type: InteractionType) -> &'static str {
    match interaction_type {
        InteractionType::Talk => "PAIRWISE_TALK",
        InteractionType::Cooperate => "PAIRWISE_COOPERATE",
        InteractionType::Share => "PAIRWISE_SHARE",
        InteractionType::Conflict => "PAIRWISE_CONFLICT",
        InteractionType::Comfort => "PAIRWISE_COMFORT",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::RelationshipEdge;
    use sim_core::config::GameConfig;
    use sim_core::{GameCalendar, WorldMap};

    fn make_resources() -> (World, SimResources) {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 42);
        (World::new(), SimResources::new(calendar, map, 7))
    }

    fn spawn_agent(
        world: &mut World,
        position: Position,
        settlement_id: Option<SettlementId>,
        action: ActionType,
        hunger: f64,
        stress: f64,
        agreeableness: f64,
        carry: f32,
        target: Option<(i32, i32)>,
    ) -> Entity {
        let mut behavior = Behavior::default();
        behavior.current_action = action;
        behavior.carry = carry;
        if let Some((x, y)) = target {
            behavior.action_target_x = Some(x);
            behavior.action_target_y = Some(y);
        }

        let mut needs = Needs::default();
        needs.set(NeedType::Hunger, hunger);

        let mut personality = Personality::default();
        personality.axes[HexacoAxis::A as usize] = agreeableness;

        let identity = Identity {
            settlement_id,
            ..Identity::default()
        };

        world.spawn((
            behavior,
            identity,
            needs,
            personality,
            position,
            Social::default(),
            Stress {
                level: stress,
                ..Stress::default()
            },
        ))
    }

    #[test]
    fn pairwise_creates_edge_for_nearby_agents() {
        let (mut world, mut resources) = make_resources();
        let source = spawn_agent(
            &mut world,
            Position::new(1, 1),
            Some(SettlementId(1)),
            ActionType::Idle,
            1.0,
            0.0,
            0.5,
            0.0,
            None,
        );
        let target = spawn_agent(
            &mut world,
            Position::new(2, 1),
            Some(SettlementId(1)),
            ActionType::Idle,
            1.0,
            0.0,
            0.5,
            0.0,
            None,
        );

        PairwiseInteractionSystem::new(36, 10).run(&mut world, &mut resources, 10);

        let social = world.get::<&Social>(source).expect("source social");
        assert_eq!(social.edges.len(), 1);
        assert_eq!(social.edges[0].target, EntityId(target.id() as u64));
    }

    #[test]
    fn pairwise_skips_distant_agents() {
        let (mut world, mut resources) = make_resources();
        let source = spawn_agent(
            &mut world,
            Position::new(1, 1),
            Some(SettlementId(1)),
            ActionType::Idle,
            1.0,
            0.0,
            0.5,
            0.0,
            None,
        );
        spawn_agent(
            &mut world,
            Position::new(20, 20),
            Some(SettlementId(1)),
            ActionType::Idle,
            1.0,
            0.0,
            0.5,
            0.0,
            None,
        );

        PairwiseInteractionSystem::new(36, 10).run(&mut world, &mut resources, 10);

        let social = world.get::<&Social>(source).expect("source social");
        assert!(social.edges.is_empty());
    }

    #[test]
    fn pairwise_cooperate_when_same_action() {
        let (mut world, mut resources) = make_resources();
        let source = spawn_agent(
            &mut world,
            Position::new(1, 1),
            Some(SettlementId(1)),
            ActionType::Forage,
            1.0,
            0.0,
            0.5,
            0.0,
            None,
        );
        spawn_agent(
            &mut world,
            Position::new(2, 1),
            Some(SettlementId(1)),
            ActionType::Forage,
            1.0,
            0.0,
            0.5,
            0.0,
            None,
        );

        PairwiseInteractionSystem::new(36, 10).run(&mut world, &mut resources, 10);

        let social = world.get::<&Social>(source).expect("source social");
        assert_eq!(
            social.edges[0].trust,
            config::INTERACTION_COOPERATE_TRUST_DELTA
        );
    }

    #[test]
    fn pairwise_talk_increases_trust() {
        let (mut world, mut resources) = make_resources();
        let source = spawn_agent(
            &mut world,
            Position::new(1, 1),
            Some(SettlementId(1)),
            ActionType::Idle,
            1.0,
            0.0,
            0.4,
            0.0,
            None,
        );
        spawn_agent(
            &mut world,
            Position::new(2, 1),
            Some(SettlementId(2)),
            ActionType::Idle,
            1.0,
            0.0,
            0.4,
            0.0,
            None,
        );

        PairwiseInteractionSystem::new(36, 10).run(&mut world, &mut resources, 10);

        let social = world.get::<&Social>(source).expect("source social");
        assert_eq!(social.edges[0].trust, config::INTERACTION_TALK_TRUST_DELTA);
    }

    #[test]
    fn interaction_type_share_when_other_is_hungry_and_i_have_carry() {
        let source = AgentSnapshot {
            entity: Entity::DANGLING,
            id: EntityId(1),
            x: 0.0,
            y: 0.0,
            settlement_id: Some(SettlementId(1)),
            action: ActionType::Idle,
            target_x: None,
            target_y: None,
            carry: 1.0,
            hunger: 1.0,
            stress: 0.0,
            agreeableness: 0.4,
        };
        let target = AgentSnapshot {
            entity: Entity::DANGLING,
            id: EntityId(2),
            x: 0.0,
            y: 0.0,
            settlement_id: Some(SettlementId(1)),
            action: ActionType::Idle,
            target_x: None,
            target_y: None,
            carry: 0.0,
            hunger: 0.2,
            stress: 0.0,
            agreeableness: 0.4,
        };
        let mut rng = rand::thread_rng();

        assert_eq!(
            select_interaction_type(source, target, true, &mut rng),
            InteractionType::Share
        );
    }

    #[test]
    fn pairwise_enforces_edge_cap() {
        let (mut world, mut resources) = make_resources();
        let central = spawn_agent(
            &mut world,
            Position::new(1, 1),
            Some(SettlementId(1)),
            ActionType::Idle,
            1.0,
            0.0,
            0.5,
            0.0,
            None,
        );

        {
            let mut social = world.get::<&mut Social>(central).expect("central social");
            for id in 100..(100 + config::SOCIAL_EDGE_CAP as u64) {
                let mut edge = RelationshipEdge::new(EntityId(id));
                edge.familiarity = if id == 100 { 0.0 } else { 0.5 };
                social.edges.push(edge);
            }
        }

        spawn_agent(
            &mut world,
            Position::new(2, 1),
            Some(SettlementId(2)),
            ActionType::Idle,
            1.0,
            0.0,
            0.5,
            0.0,
            None,
        );

        PairwiseInteractionSystem::new(36, 10).run(&mut world, &mut resources, 10);

        let social = world.get::<&Social>(central).expect("central social");
        assert_eq!(social.edges.len(), config::SOCIAL_EDGE_CAP);
        assert!(social.find_edge(EntityId(100)).is_none());
    }

    #[test]
    fn enforce_edge_cap_breaks_familiarity_ties_by_target_id() {
        let mut social = Social::default();
        for raw_id in 10..=(10 + config::SOCIAL_EDGE_CAP as u64) {
            let target = EntityId(raw_id);
            let mut edge = RelationshipEdge::new(target);
            edge.familiarity = 0.2;
            social.edges.push(edge);
        }

        enforce_edge_cap(&mut social);
        assert_eq!(social.edges.len(), config::SOCIAL_EDGE_CAP);
        assert!(social.find_edge(EntityId(10)).is_none());
        assert!(social.find_edge(EntityId(11)).is_some());
        assert!(social
            .find_edge(EntityId(10 + config::SOCIAL_EDGE_CAP as u64))
            .is_some());
    }
}
