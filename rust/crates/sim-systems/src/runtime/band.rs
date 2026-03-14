use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

use hecs::World;
use sim_core::band::Band;
use sim_core::causal_log::{CausalEvent, CauseRef};
use sim_core::components::{Age, Identity, Needs, Personality, Position, Social, Values};
use sim_core::config;
use sim_core::enums::{HexacoAxis, NeedType};
use sim_core::ids::{BandId, EntityId, SettlementId};
use sim_core::Settlement;
use sim_engine::{SimResources, SimSystem};

/// Cold-tier runtime system that turns pairwise trust into provisional and promoted bands.
#[derive(Debug, Clone)]
pub struct BandFormationSystem {
    priority: u32,
    tick_interval: u64,
}

impl BandFormationSystem {
    /// Creates a new band-formation runtime system.
    pub fn new(priority: u32, tick_interval: u64) -> Self {
        Self {
            priority,
            tick_interval: tick_interval.max(1),
        }
    }
}

#[derive(Debug, Clone)]
struct AgentSnapshot {
    id: EntityId,
    x: f64,
    y: f64,
    settlement_id: Option<SettlementId>,
    band_id: Option<BandId>,
    safety: f64,
    extraversion: f64,
    agreeableness: f64,
    social: Social,
    values: Values,
}

#[derive(Debug, Clone)]
struct ProposedBand {
    existing_band_ids: Vec<BandId>,
    members: Vec<EntityId>,
}

/// Calculates the group-formation score between two agents.
pub fn calculate_gfs(
    a_id: EntityId,
    b_id: EntityId,
    a_pos: (f64, f64),
    b_pos: (f64, f64),
    a_settlement: Option<SettlementId>,
    b_settlement: Option<SettlementId>,
    a_social: &Social,
    b_social: &Social,
    a_values: &Values,
    b_values: &Values,
    a_safety: f64,
    b_safety: f64,
    settlement_resource_score: f64,
    all_socials: &[(EntityId, &Social)],
) -> f64 {
    let dx = a_pos.0 - b_pos.0;
    let dy = a_pos.1 - b_pos.1;
    let distance = (dx * dx + dy * dy).sqrt();
    let distance_proximity =
        (1.0 - distance / config::GFS_PROXIMITY_MAX_DISTANCE).clamp(0.0, 1.0);
    let proximity = if a_settlement.is_some() && a_settlement == b_settlement {
        distance_proximity.sqrt()
    } else {
        distance_proximity
    };

    let kinship = (sim_core::components::social::kinship_r(a_social, b_id, all_socials) * 2.0)
        .clamp(0.0, 1.0);
    let trust = a_social.find_edge(b_id).map(|edge| edge.trust).unwrap_or(0.0);
    let reciprocal_trust = b_social.find_edge(a_id).map(|edge| edge.trust).unwrap_or(0.0);
    let shared_values = a_values.alignment_with(b_values);
    let threat_pressure = 1.0 - ((a_safety + b_safety) * 0.5).clamp(0.0, 1.0);
    let resource_factor = settlement_resource_score.clamp(0.0, 1.0);

    (0.25 * proximity)
        + (0.25 * kinship)
        + (0.20 * ((trust + reciprocal_trust) * 0.5))
        + (0.10 * shared_values)
        + (0.10 * threat_pressure)
        + (0.10 * resource_factor)
}

impl SimSystem for BandFormationSystem {
    fn name(&self) -> &'static str {
        "band_formation_system"
    }

    fn tick_interval(&self) -> u64 {
        self.tick_interval
    }

    fn priority(&self) -> u32 {
        self.priority
    }

    fn run(&mut self, world: &mut World, resources: &mut SimResources, tick: u64) {
        let all_agents = collect_agent_snapshots(world);
        if all_agents.len() < config::BAND_MIN_SIZE_PROVISIONAL {
            cleanup_small_bands(world, resources);
            return;
        }

        let agent_by_id: BTreeMap<EntityId, AgentSnapshot> = all_agents
            .iter()
            .cloned()
            .map(|agent| (agent.id, agent))
            .collect();
        let existing_bands: Vec<Band> = resources.band_store.all().cloned().collect();
        let provisional_ids: BTreeSet<BandId> = existing_bands
            .iter()
            .filter(|band| !band.is_promoted)
            .map(|band| band.id)
            .collect();

        let candidates: Vec<AgentSnapshot> = all_agents
            .iter()
            .filter(|agent| {
                agent.band_id.is_none()
                    || agent
                        .band_id
                        .map(|band_id| provisional_ids.contains(&band_id))
                        .unwrap_or(false)
            })
            .cloned()
            .collect();

        let mut dissolved_band_ids: BTreeSet<BandId> = BTreeSet::new();
        let mut formed_events: Vec<(BandId, Vec<EntityId>)> = Vec::new();
        let mut promoted_events: Vec<(BandId, Vec<EntityId>)> = Vec::new();
        let mut dissolved_events: Vec<(BandId, Vec<EntityId>)> = Vec::new();
        let mut leader_events: Vec<(BandId, EntityId, Vec<EntityId>)> = Vec::new();
        let mut final_bands: BTreeMap<BandId, Band> = BTreeMap::new();

        if candidates.len() >= config::BAND_MIN_SIZE_PROVISIONAL {
            let candidate_by_id: BTreeMap<EntityId, AgentSnapshot> = candidates
                .iter()
                .cloned()
                .map(|agent| (agent.id, agent))
                .collect();
            let social_refs: Vec<(EntityId, &Social)> =
                all_agents.iter().map(|agent| (agent.id, &agent.social)).collect();
            let adjacency = build_adjacency_graph(&candidates, resources, &social_refs);
            let candidate_ids: Vec<EntityId> = candidates.iter().map(|agent| agent.id).collect();
            let components = find_connected_components(&candidate_ids, &adjacency);

            for proposed in components
                .into_iter()
                .filter_map(|component| {
                    build_proposed_band(
                        &component,
                        &candidate_by_id,
                        &adjacency,
                        existing_bands.as_slice(),
                    )
                })
            {
                let primary_band_id = proposed
                    .existing_band_ids
                    .iter()
                    .copied()
                    .min()
                    .unwrap_or_else(|| resources.band_store.allocate_id());
                let existing_primary = existing_bands
                    .iter()
                    .find(|band| band.id == primary_band_id)
                    .cloned();

                let mut band = if let Some(existing) = existing_primary {
                    existing
                } else {
                    formed_events.push((primary_band_id, proposed.members.clone()));
                    Band::new(
                        primary_band_id,
                        generate_band_name(primary_band_id),
                        proposed.members.clone(),
                        tick,
                    )
                };

                band.members = proposed.members.clone();
                band.members.sort_by_key(|member| member.0);
                if !band.is_promoted {
                    band.leader = None;
                }

                for other_id in proposed.existing_band_ids.iter().copied() {
                    if other_id != primary_band_id {
                        dissolved_band_ids.insert(other_id);
                    }
                }

                final_bands.insert(primary_band_id, band);
            }
        }

        for band in &existing_bands {
            if !band.is_promoted {
                if final_bands.contains_key(&band.id) {
                    continue;
                }
                dissolved_band_ids.insert(band.id);
                continue;
            }

            let live_members: Vec<EntityId> = band
                .members
                .iter()
                .copied()
                .filter(|member_id| {
                    agent_by_id
                        .get(member_id)
                        .map(|agent| agent.band_id == Some(band.id))
                        .unwrap_or(false)
                })
                .collect();

            if live_members.len() < config::BAND_MIN_SIZE_PROVISIONAL {
                dissolved_band_ids.insert(band.id);
                continue;
            }

            let mut kept = band.clone();
            kept.members = live_members.clone();
            let leader = elect_leader(&kept.members, &agent_by_id);
            if leader != kept.leader {
                if let Some(leader_id) = leader {
                    leader_events.push((kept.id, leader_id, kept.members.clone()));
                }
                kept.leader = leader;
            }
            final_bands.insert(kept.id, kept);
        }

        for band_id in &dissolved_band_ids {
            if let Some(old_band) = existing_bands.iter().find(|band| band.id == *band_id) {
                dissolved_events.push((old_band.id, old_band.members.clone()));
            }
        }

        for band in final_bands.values_mut() {
            if band.member_count() > config::BAND_MAX_SIZE {
                band.members =
                    truncate_members_to_band_cap(&band.members, &agent_by_id, resources, tick);
                band.members.sort_by_key(|member| member.0);
            }

            if !band.is_promoted
                && band.member_count() >= config::BAND_MIN_SIZE_PROMOTED
                && tick.saturating_sub(band.provisional_since) >= config::BAND_PROMOTION_TICKS
            {
                band.is_promoted = true;
                band.promoted_tick = Some(tick);
                promoted_events.push((band.id, band.members.clone()));
                let leader = elect_leader(&band.members, &agent_by_id);
                if let Some(leader_id) = leader {
                    leader_events.push((band.id, leader_id, band.members.clone()));
                }
                band.leader = leader;
            }
        }

        apply_identity_band_ids(world, &final_bands);

        for band_id in &dissolved_band_ids {
            resources.band_store.remove(*band_id);
        }
        for band in final_bands.values().cloned() {
            resources.band_store.insert(band);
        }

        for (band_id, members) in formed_events {
            push_band_causal(
                resources,
                tick,
                band_id,
                &members,
                "band_formed",
                "BAND_FORMED",
                members.len() as f64,
            );
        }
        for (band_id, members) in promoted_events {
            push_band_causal(
                resources,
                tick,
                band_id,
                &members,
                "band_promoted",
                "BAND_PROMOTED",
                members.len() as f64,
            );
        }
        for (band_id, members) in dissolved_events {
            push_band_causal(
                resources,
                tick,
                band_id,
                &members,
                "band_dissolved",
                "BAND_DISSOLVED",
                members.len() as f64,
            );
        }
        for (band_id, leader_id, members) in leader_events {
            push_band_leader_causal(resources, tick, band_id, leader_id, &members);
        }
    }
}

fn cleanup_small_bands(world: &mut World, resources: &mut SimResources) {
    let all_agents = collect_agent_snapshots(world);
    let live_band_members: BTreeMap<BandId, Vec<EntityId>> = all_agents.iter().fold(
        BTreeMap::new(),
        |mut acc, agent| {
            if let Some(band_id) = agent.band_id {
                acc.entry(band_id).or_default().push(agent.id);
            }
            acc
        },
    );

    let mut dissolved = Vec::new();
    for band in resources.band_store.all() {
        let member_count = live_band_members.get(&band.id).map(Vec::len).unwrap_or(0);
        if member_count < config::BAND_MIN_SIZE_PROVISIONAL {
            dissolved.push((band.id, band.members.clone()));
        }
    }
    if dissolved.is_empty() {
        return;
    }

    apply_identity_band_ids(world, &BTreeMap::new());

    let tick = resources.calendar.tick;
    for (band_id, members) in dissolved {
        resources.band_store.remove(band_id);
        push_band_causal(
            resources,
            tick,
            band_id,
            &members,
            "band_dissolved",
            "BAND_DISSOLVED",
            members.len() as f64,
        );
    }
}

fn collect_agent_snapshots(world: &World) -> Vec<AgentSnapshot> {
    let mut snapshots = Vec::new();
    let mut query = world.query::<(
        &Position,
        &Identity,
        Option<&Age>,
        Option<&Social>,
        Option<&Values>,
        Option<&Needs>,
        Option<&Personality>,
    )>();

    for (entity, (position, identity, age, social, values, needs, personality)) in &mut query {
        if matches!(age, Some(component) if !component.alive) {
            continue;
        }
        let safety = needs
            .map(|needs| needs.get(NeedType::Safety))
            .unwrap_or(1.0)
            .clamp(0.0, 1.0);
        let extraversion = personality
            .map(|personality| personality.axis(HexacoAxis::X))
            .unwrap_or(0.5)
            .clamp(0.0, 1.0);
        let agreeableness = personality
            .map(|personality| personality.axis(HexacoAxis::A))
            .unwrap_or(0.5)
            .clamp(0.0, 1.0);

        snapshots.push(AgentSnapshot {
            id: EntityId(entity.id() as u64),
            x: position.x,
            y: position.y,
            settlement_id: identity.settlement_id,
            band_id: identity.band_id,
            safety,
            extraversion,
            agreeableness,
            social: social.cloned().unwrap_or_default(),
            values: values.cloned().unwrap_or_default(),
        });
    }

    snapshots.sort_by_key(|agent| agent.id.0);
    snapshots
}

fn settlement_resource_score(
    resources: &SimResources,
    a_settlement: Option<SettlementId>,
    b_settlement: Option<SettlementId>,
) -> f64 {
    let mut scores = Vec::new();
    for settlement_id in [a_settlement, b_settlement].into_iter().flatten() {
        if let Some(settlement) = resources.settlements.get(&settlement_id) {
            scores.push(normalized_settlement_resources(settlement));
        }
    }
    if scores.is_empty() {
        0.0
    } else {
        scores.iter().sum::<f64>() / scores.len() as f64
    }
}

fn normalized_settlement_resources(settlement: &Settlement) -> f64 {
    let total_resources =
        settlement.stockpile_food + settlement.stockpile_wood + settlement.stockpile_stone;
    let members = settlement.members.len().max(1) as f64;
    (total_resources / (members * config::BAND_MIN_SIZE_PROMOTED as f64)).clamp(0.0, 1.0)
}

fn build_adjacency_graph(
    candidates: &[AgentSnapshot],
    resources: &SimResources,
    all_socials: &[(EntityId, &Social)],
) -> BTreeMap<EntityId, Vec<EntityId>> {
    let mut adjacency: BTreeMap<EntityId, Vec<EntityId>> = candidates
        .iter()
        .map(|agent| (agent.id, Vec::new()))
        .collect();

    for left in 0..candidates.len() {
        for right in (left + 1)..candidates.len() {
            let a = &candidates[left];
            let b = &candidates[right];
            let resource_score =
                settlement_resource_score(resources, a.settlement_id, b.settlement_id);
            let gfs = calculate_gfs(
                a.id,
                b.id,
                (a.x, a.y),
                (b.x, b.y),
                a.settlement_id,
                b.settlement_id,
                &a.social,
                &b.social,
                &a.values,
                &b.values,
                a.safety,
                b.safety,
                resource_score,
                all_socials,
            );
            if gfs >= config::GFS_THRESHOLD {
                adjacency.entry(a.id).or_default().push(b.id);
                adjacency.entry(b.id).or_default().push(a.id);
            }
        }
    }

    for neighbors in adjacency.values_mut() {
        neighbors.sort_by_key(|neighbor| neighbor.0);
        neighbors.dedup();
    }

    adjacency
}

fn find_connected_components(
    agents: &[EntityId],
    adjacency: &BTreeMap<EntityId, Vec<EntityId>>,
) -> Vec<Vec<EntityId>> {
    let mut visited = BTreeSet::new();
    let mut components = Vec::new();

    for &agent in agents {
        if visited.contains(&agent) {
            continue;
        }

        let mut stack = vec![agent];
        let mut component = Vec::new();
        while let Some(current) = stack.pop() {
            if !visited.insert(current) {
                continue;
            }
            component.push(current);
            if let Some(neighbors) = adjacency.get(&current) {
                for &neighbor in neighbors.iter().rev() {
                    if !visited.contains(&neighbor) {
                        stack.push(neighbor);
                    }
                }
            }
        }
        component.sort_by_key(|member| member.0);
        components.push(component);
    }

    components.sort_by(|left, right| left.first().cmp(&right.first()));
    components
}

fn build_proposed_band(
    component: &[EntityId],
    candidate_by_id: &BTreeMap<EntityId, AgentSnapshot>,
    adjacency: &BTreeMap<EntityId, Vec<EntityId>>,
    existing_bands: &[Band],
) -> Option<ProposedBand> {
    if component.len() < config::BAND_MIN_SIZE_PROVISIONAL {
        return None;
    }

    let component_size = component.len() as f64;
    let mut joined = Vec::new();
    for member_id in component {
        let snapshot = candidate_by_id.get(member_id)?;
        let nearby = nearby_agent_count(snapshot, candidate_by_id.values()) as f64;
        let joined_ratio = (component_size / nearby.max(1.0)).clamp(0.0, 1.0);
        if joined_ratio >= granovetter_threshold(snapshot) {
            joined.push(*member_id);
        }
    }

    joined.sort_by_key(|member| member.0);
    joined.dedup();
    if joined.len() < config::BAND_MIN_SIZE_PROVISIONAL {
        return None;
    }

    if joined.len() > config::BAND_MAX_SIZE {
        joined.sort_by(|left, right| {
            let left_support = adjacency
                .get(left)
                .map(|neighbors| neighbors.len())
                .unwrap_or_default();
            let right_support = adjacency
                .get(right)
                .map(|neighbors| neighbors.len())
                .unwrap_or_default();
            right_support
                .cmp(&left_support)
                .then_with(|| left.0.cmp(&right.0))
        });
        joined.truncate(config::BAND_MAX_SIZE);
        joined.sort_by_key(|member| member.0);
    }

    let mut existing_band_ids: Vec<BandId> = joined
        .iter()
        .filter_map(|member_id| candidate_by_id.get(member_id).and_then(|agent| agent.band_id))
        .filter(|band_id| {
            existing_bands
                .iter()
                .any(|band| band.id == *band_id && !band.is_promoted)
        })
        .collect();
    existing_band_ids.sort_by_key(|band_id| band_id.0);
    existing_band_ids.dedup();

    Some(ProposedBand {
        existing_band_ids,
        members: joined,
    })
}

fn nearby_agent_count<'a>(
    snapshot: &AgentSnapshot,
    all_agents: impl IntoIterator<Item = &'a AgentSnapshot>,
) -> usize {
    let mut count = 1_usize;
    let max_distance_sq = config::GFS_PROXIMITY_MAX_DISTANCE * config::GFS_PROXIMITY_MAX_DISTANCE;
    for other in all_agents {
        if other.id == snapshot.id {
            continue;
        }
        let dx = snapshot.x - other.x;
        let dy = snapshot.y - other.y;
        if (dx * dx + dy * dy) <= max_distance_sq {
            count = count.saturating_add(1);
        }
    }
    count
}

fn granovetter_threshold(agent: &AgentSnapshot) -> f64 {
    (config::BAND_GRANOVETTER_BASE_THRESHOLD + 0.5
        - (agent.agreeableness * 0.3)
        - (agent.extraversion * 0.2))
        .clamp(0.0, 1.0)
}

fn elect_leader(
    members: &[EntityId],
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
) -> Option<EntityId> {
    members
        .iter()
        .filter_map(|member_id| agent_by_id.get(member_id))
        .map(|agent| {
            let avg_trust = if members.len() <= 1 {
                0.0
            } else {
                let trust_sum: f64 = members
                    .iter()
                    .copied()
                    .filter(|other| *other != agent.id)
                    .map(|other| agent.social.find_edge(other).map(|edge| edge.trust).unwrap_or(0.0))
                    .sum();
                trust_sum / (members.len() - 1) as f64
            };
            let score = (agent.extraversion * 0.5) + (avg_trust * 0.5);
            (agent.id, score)
        })
        .max_by(|left, right| {
            left.1
                .partial_cmp(&right.1)
                .unwrap_or(Ordering::Equal)
                .then_with(|| right.0.0.cmp(&left.0.0))
        })
        .map(|(id, _)| id)
}

fn truncate_members_to_band_cap(
    members: &[EntityId],
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
    resources: &SimResources,
    _tick: u64,
) -> Vec<EntityId> {
    let social_refs: Vec<(EntityId, &Social)> = members
        .iter()
        .filter_map(|member_id| agent_by_id.get(member_id).map(|agent| (agent.id, &agent.social)))
        .collect();

    let mut scored: Vec<(EntityId, f64)> = members
        .iter()
        .filter_map(|member_id| agent_by_id.get(member_id))
        .map(|agent| {
            let support_score: f64 = members
                .iter()
                .copied()
                .filter(|other| *other != agent.id)
                .filter_map(|other| agent_by_id.get(&other))
                .map(|other| {
                    calculate_gfs(
                        agent.id,
                        other.id,
                        (agent.x, agent.y),
                        (other.x, other.y),
                        agent.settlement_id,
                        other.settlement_id,
                        &agent.social,
                        &other.social,
                        &agent.values,
                        &other.values,
                        agent.safety,
                        other.safety,
                        settlement_resource_score(
                            resources,
                            agent.settlement_id,
                            other.settlement_id,
                        ),
                        &social_refs,
                    )
                })
                .sum();
            (agent.id, support_score)
        })
        .collect();

    scored.sort_by(|left, right| {
        right
            .1
            .partial_cmp(&left.1)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.0.0.cmp(&right.0.0))
    });
    scored.truncate(config::BAND_MAX_SIZE);
    let mut kept: Vec<EntityId> = scored.into_iter().map(|(id, _)| id).collect();
    kept.sort_by_key(|member| member.0);
    kept
}

fn apply_identity_band_ids(world: &mut World, final_bands: &BTreeMap<BandId, Band>) {
    let mut desired_band_ids = BTreeMap::new();
    for band in final_bands.values() {
        for member in &band.members {
            desired_band_ids.insert(*member, band.id);
        }
    }

    let mut query = world.query::<&mut Identity>();
    for (entity, identity) in &mut query {
        let entity_id = EntityId(entity.id() as u64);
        let desired_band_id = desired_band_ids.get(&entity_id).copied();
        if identity.band_id != desired_band_id {
            identity.band_id = desired_band_id;
        }
    }
}

fn generate_band_name(band_id: BandId) -> String {
    format!("band_{}", band_id.0)
}

fn push_band_causal(
    resources: &mut SimResources,
    tick: u64,
    band_id: BandId,
    members: &[EntityId],
    cause_kind: &str,
    summary_key: &str,
    magnitude: f64,
) {
    for member in members {
        resources.causal_log.push(
            *member,
            CausalEvent {
                tick,
                cause: CauseRef {
                    system: "band_formation_system".to_string(),
                    kind: cause_kind.to_string(),
                    entity: Some(*member),
                    building: None,
                    settlement: None,
                },
                effect_key: format!("band:{}", band_id.0),
                summary_key: summary_key.to_string(),
                magnitude,
            },
        );
    }
}

fn push_band_leader_causal(
    resources: &mut SimResources,
    tick: u64,
    band_id: BandId,
    leader_id: EntityId,
    members: &[EntityId],
) {
    for member in members {
        resources.causal_log.push(
            *member,
            CausalEvent {
                tick,
                cause: CauseRef {
                    system: "band_formation_system".to_string(),
                    kind: "band_leader_elected".to_string(),
                    entity: Some(leader_id),
                    building: None,
                    settlement: None,
                },
                effect_key: format!("band_leader:{}:{}", band_id.0, leader_id.0),
                summary_key: "BAND_LEADER_ELECTED".to_string(),
                magnitude: 1.0,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::Entity;
    use sim_core::band::BandStore;
    use sim_core::{config::GameConfig, GameCalendar, WorldMap};
    use sim_engine::SimResources;

    fn make_resources() -> SimResources {
        let config = GameConfig::default();
        let calendar = GameCalendar::new(&config);
        let map = WorldMap::new(16, 16, 7);
        SimResources::new(calendar, map, 99)
    }

    fn settlement_with_stockpile(id: SettlementId, members: Vec<EntityId>) -> Settlement {
        let mut settlement = Settlement::new(id, format!("S{}", id.0), 0, 0, 0);
        settlement.members = members;
        settlement.stockpile_food = 20.0;
        settlement.stockpile_wood = 20.0;
        settlement.stockpile_stone = 20.0;
        settlement
    }

    fn spawn_agent(
        world: &mut World,
        name: &str,
        x: i32,
        y: i32,
        settlement_id: Option<SettlementId>,
        band_id: Option<BandId>,
        safety: f64,
        extraversion: f64,
        agreeableness: f64,
    ) -> Entity {
        let mut needs = Needs::default();
        needs.set(NeedType::Safety, safety);

        let mut personality = Personality::default();
        personality.axes[HexacoAxis::X as usize] = extraversion;
        personality.axes[HexacoAxis::A as usize] = agreeableness;
        let mut values = Values::default();
        values.values.fill(1.0);

        let identity = Identity {
            name: name.to_string(),
            settlement_id,
            band_id,
            ..Identity::default()
        };

        world.spawn((
            Position::new(x, y),
            identity,
            Age::default(),
            Social::default(),
            values,
            needs,
            personality,
        ))
    }

    fn set_trust(world: &mut World, source: Entity, target: Entity, trust: f64) {
        let target_id = EntityId(target.id() as u64);
        if let Ok(mut social) = world.get::<&mut Social>(source) {
            let edge = social.get_or_create_edge(target_id);
            edge.trust = trust;
            edge.affinity = trust * 100.0;
            edge.familiarity = trust;
            edge.relation_type = if trust >= 0.6 {
                sim_core::enums::RelationType::CloseFriend
            } else {
                sim_core::enums::RelationType::Friend
            };
        }
    }

    #[test]
    fn gfs_same_settlement_high_trust_exceeds_threshold() {
        let a_id = EntityId(1);
        let b_id = EntityId(2);
        let mut a_social = Social::default();
        a_social.get_or_create_edge(b_id).trust = 0.8;
        let mut b_social = Social::default();
        b_social.get_or_create_edge(a_id).trust = 0.8;
        let mut values = Values::default();
        values.values.fill(1.0);
        let all = vec![(a_id, &a_social), (b_id, &b_social)];

        let gfs = calculate_gfs(
            a_id,
            b_id,
            (0.0, 0.0),
            (2.0, 0.0),
            Some(SettlementId(1)),
            Some(SettlementId(1)),
            &a_social,
            &b_social,
            &values,
            &values,
            0.9,
            0.9,
            1.0,
            &all,
        );

        assert!(gfs > config::GFS_THRESHOLD);
    }

    #[test]
    fn gfs_distant_strangers_below_threshold() {
        let a_id = EntityId(1);
        let b_id = EntityId(2);
        let a_social = Social::default();
        let b_social = Social::default();
        let values = Values::default();
        let all = vec![(a_id, &a_social), (b_id, &b_social)];

        let gfs = calculate_gfs(
            a_id,
            b_id,
            (0.0, 0.0),
            (500.0, 0.0),
            None,
            None,
            &a_social,
            &b_social,
            &values,
            &values,
            1.0,
            1.0,
            0.0,
            &all,
        );

        assert!(gfs < config::GFS_THRESHOLD);
    }

    #[test]
    fn gfs_distant_same_settlement_stays_below_threshold() {
        let a_id = EntityId(1);
        let b_id = EntityId(2);
        let mut a_social = Social::default();
        a_social.get_or_create_edge(b_id).trust = 0.8;
        let mut b_social = Social::default();
        b_social.get_or_create_edge(a_id).trust = 0.8;
        let mut values = Values::default();
        values.values.fill(1.0);
        let all = vec![(a_id, &a_social), (b_id, &b_social)];

        let gfs = calculate_gfs(
            a_id,
            b_id,
            (0.0, 0.0),
            (100.0, 0.0),
            Some(SettlementId(1)),
            Some(SettlementId(1)),
            &a_social,
            &b_social,
            &values,
            &values,
            0.9,
            0.9,
            1.0,
            &all,
        );

        assert!(gfs < config::GFS_THRESHOLD);
    }

    #[test]
    fn gfs_kinship_boosts_score() {
        let parent_id = EntityId(1);
        let child_id = EntityId(2);
        let parent_social = Social {
            children: vec![child_id],
            ..Social::default()
        };
        let child_social = Social {
            parents: vec![parent_id],
            ..Social::default()
        };
        let values = Values::default();
        let all = vec![(parent_id, &parent_social), (child_id, &child_social)];

        let gfs = calculate_gfs(
            parent_id,
            child_id,
            (0.0, 0.0),
            (100.0, 0.0),
            None,
            None,
            &parent_social,
            &child_social,
            &values,
            &values,
            0.5,
            0.5,
            0.0,
            &all,
        );

        assert!(gfs >= 0.25);
    }

    #[test]
    fn gfs_grandparent_uses_noncandidate_intermediary_for_kinship() {
        let grandparent_id = EntityId(1);
        let parent_id = EntityId(2);
        let grandchild_id = EntityId(3);
        let grandparent_social = Social {
            children: vec![parent_id],
            ..Social::default()
        };
        let parent_social = Social {
            parents: vec![grandparent_id],
            children: vec![grandchild_id],
            spouse: Some(EntityId(99)),
            ..Social::default()
        };
        let grandchild_social = Social {
            parents: vec![parent_id],
            ..Social::default()
        };
        let mut values = Values::default();
        values.values.fill(1.0);
        let all = vec![
            (grandparent_id, &grandparent_social),
            (parent_id, &parent_social),
            (grandchild_id, &grandchild_social),
        ];

        let gfs = calculate_gfs(
            grandparent_id,
            grandchild_id,
            (0.0, 0.0),
            (4.0, 0.0),
            Some(SettlementId(1)),
            Some(SettlementId(1)),
            &grandparent_social,
            &grandchild_social,
            &values,
            &values,
            0.7,
            0.7,
            1.0,
            &all,
        );

        assert!(gfs > 0.5, "extended kinship should materially boost GFS");
    }

    #[test]
    fn band_formation_creates_provisional_for_three_plus() {
        let (mut world, mut resources) = (World::new(), make_resources());
        let a = spawn_agent(
            &mut world,
            "A",
            0,
            0,
            Some(SettlementId(1)),
            None,
            0.6,
            0.8,
            0.8,
        );
        let b = spawn_agent(
            &mut world,
            "B",
            1,
            0,
            Some(SettlementId(1)),
            None,
            0.6,
            0.8,
            0.8,
        );
        let c = spawn_agent(
            &mut world,
            "C",
            2,
            0,
            Some(SettlementId(1)),
            None,
            0.6,
            0.8,
            0.8,
        );
        resources.settlements.insert(
            SettlementId(1),
            settlement_with_stockpile(
                SettlementId(1),
                vec![
                    EntityId(a.id() as u64),
                    EntityId(b.id() as u64),
                    EntityId(c.id() as u64),
                ],
            ),
        );
        for &(left, right) in &[(a, b), (a, c), (b, a), (b, c), (c, a), (c, b)] {
            set_trust(&mut world, left, right, 0.8);
        }

        BandFormationSystem::new(38, 60).run(&mut world, &mut resources, 60);

        assert_eq!(resources.band_store.len(), 1);
        let band = resources.band_store.all().next().expect("band exists");
        assert!(!band.is_promoted);
        assert_eq!(band.member_count(), 3);
        assert!(world
            .get::<&Identity>(a)
            .expect("identity")
            .band_id
            .is_some());
    }

    #[test]
    fn band_promotion_after_2400_ticks_with_five_members() {
        let (mut world, mut resources) = (World::new(), make_resources());
        let mut members = Vec::new();
        for index in 0..5 {
            members.push(spawn_agent(
                &mut world,
                &format!("M{index}"),
                index,
                0,
                Some(SettlementId(1)),
                None,
                0.5,
                0.9,
                0.8,
            ));
        }
        resources.settlements.insert(
            SettlementId(1),
            settlement_with_stockpile(
                SettlementId(1),
                members
                    .iter()
                    .map(|entity: &Entity| EntityId(entity.id() as u64))
                    .collect(),
            ),
        );
        for &left in &members {
            for &right in &members {
                if left != right {
                    set_trust(&mut world, left, right, 0.8);
                }
            }
        }

        let mut system = BandFormationSystem::new(38, 60);
        system.run(&mut world, &mut resources, 60);
        system.run(&mut world, &mut resources, 60 + config::BAND_PROMOTION_TICKS);

        let band = resources.band_store.all().next().expect("band exists");
        assert!(band.is_promoted);
        assert!(band.promoted_tick.is_some());
        assert!(band.leader.is_some());
    }

    #[test]
    fn band_dissolved_when_below_three_members() {
        let mut resources = make_resources();
        let mut world = World::new();
        let band_id = BandId(1);
        let a = spawn_agent(
            &mut world,
            "A",
            0,
            0,
            Some(SettlementId(1)),
            Some(band_id),
            0.5,
            0.5,
            0.5,
        );
        let b = spawn_agent(
            &mut world,
            "B",
            1,
            0,
            Some(SettlementId(1)),
            Some(band_id),
            0.5,
            0.5,
            0.5,
        );
        resources.band_store = BandStore::new();
        resources.band_store.insert(Band {
            id: band_id,
            name: "Oak".to_string(),
            members: vec![EntityId(a.id() as u64), EntityId(b.id() as u64)],
            leader: None,
            provisional_since: 10,
            promoted_tick: None,
            is_promoted: false,
        });

        BandFormationSystem::new(38, 60).run(&mut world, &mut resources, 60);

        assert!(resources.band_store.is_empty());
        assert!(world
            .get::<&Identity>(a)
            .expect("identity")
            .band_id
            .is_none());
    }

    #[test]
    fn leader_elected_by_extraversion_and_trust() {
        let (mut world, mut resources) = (World::new(), make_resources());
        let a = spawn_agent(
            &mut world,
            "Leader",
            0,
            0,
            Some(SettlementId(1)),
            None,
            0.5,
            1.0,
            0.8,
        );
        let b = spawn_agent(
            &mut world,
            "B",
            1,
            0,
            Some(SettlementId(1)),
            None,
            0.5,
            0.2,
            0.8,
        );
        let c = spawn_agent(
            &mut world,
            "C",
            2,
            0,
            Some(SettlementId(1)),
            None,
            0.5,
            0.2,
            0.8,
        );
        let d = spawn_agent(
            &mut world,
            "D",
            3,
            0,
            Some(SettlementId(1)),
            None,
            0.5,
            0.2,
            0.8,
        );
        let e = spawn_agent(
            &mut world,
            "E",
            4,
            0,
            Some(SettlementId(1)),
            None,
            0.5,
            0.2,
            0.8,
        );
        let members = [a, b, c, d, e];
        resources.settlements.insert(
            SettlementId(1),
            settlement_with_stockpile(
                SettlementId(1),
                members
                    .iter()
                    .map(|entity: &Entity| EntityId(entity.id() as u64))
                    .collect(),
            ),
        );
        for &left in &members {
            for &right in &members {
                if left != right {
                    set_trust(&mut world, left, right, if left == a { 1.0 } else { 0.3 });
                }
            }
        }

        let mut system = BandFormationSystem::new(38, 60);
        system.run(&mut world, &mut resources, 60);
        system.run(&mut world, &mut resources, 60 + config::BAND_PROMOTION_TICKS);

        let band = resources.band_store.all().next().expect("band exists");
        assert_eq!(band.leader, Some(EntityId(a.id() as u64)));
    }

    #[test]
    fn dead_members_do_not_keep_band_alive() {
        let mut resources = make_resources();
        let mut world = World::new();
        let band_id = BandId(1);
        let a = spawn_agent(
            &mut world,
            "A",
            0,
            0,
            Some(SettlementId(1)),
            Some(band_id),
            0.5,
            0.5,
            0.5,
        );
        let b = spawn_agent(
            &mut world,
            "B",
            1,
            0,
            Some(SettlementId(1)),
            Some(band_id),
            0.5,
            0.5,
            0.5,
        );
        let c = spawn_agent(
            &mut world,
            "C",
            2,
            0,
            Some(SettlementId(1)),
            Some(band_id),
            0.5,
            0.5,
            0.5,
        );
        if let Ok(mut age) = world.get::<&mut Age>(c) {
            age.alive = false;
        }
        resources.band_store.insert(Band {
            id: band_id,
            name: "band_1".to_string(),
            members: vec![
                EntityId(a.id() as u64),
                EntityId(b.id() as u64),
                EntityId(c.id() as u64),
            ],
            leader: None,
            provisional_since: 10,
            promoted_tick: None,
            is_promoted: false,
        });

        BandFormationSystem::new(38, 60).run(&mut world, &mut resources, 60);

        assert!(resources.band_store.is_empty());
        assert!(world.get::<&Identity>(a).expect("identity").band_id.is_none());
        assert!(world.get::<&Identity>(b).expect("identity").band_id.is_none());
        assert!(world.get::<&Identity>(c).expect("identity").band_id.is_none());
    }

    #[test]
    fn band_names_are_deterministic_and_locale_neutral() {
        assert_eq!(generate_band_name(BandId(7)), "band_7");
    }
}
