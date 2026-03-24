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
use sim_engine::{
    ChronicleEvent, ChronicleEventCause, ChronicleEventMagnitude, ChronicleEventType, SimEvent,
    SimEventType, SimResources, SimSystem,
};

use super::band_behavior::refresh_band_behavior_state;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BandSplitCause {
    TrustCollapse,
    ValueClash,
    Overpopulation,
}

#[derive(Debug, Clone)]
struct BandFissionPlan {
    retained_band: Band,
    split_band: Option<Band>,
    loners: Vec<EntityId>,
    cause: BandSplitCause,
    original_members: Vec<EntityId>,
}

#[derive(Debug, Clone, Copy)]
struct PairMetric {
    left: EntityId,
    right: EntityId,
    trust: f64,
    value_alignment: f64,
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
        let social_refs: Vec<(EntityId, &Social)> =
            all_agents.iter().map(|agent| (agent.id, &agent.social)).collect();
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
        let mut dissolved_events: Vec<(BandId, String, Vec<EntityId>)> = Vec::new();
        let mut leader_events: Vec<(BandId, EntityId, Vec<EntityId>)> = Vec::new();
        let mut split_events: Vec<(BandId, Vec<EntityId>, BandSplitCause)> = Vec::new();
        let mut loner_join_events: Vec<(BandId, EntityId)> = Vec::new();
        let mut final_bands: BTreeMap<BandId, Band> = BTreeMap::new();

        if candidates.len() >= config::BAND_MIN_SIZE_PROVISIONAL {
            let candidate_by_id: BTreeMap<EntityId, AgentSnapshot> = candidates
                .iter()
                .cloned()
                .map(|agent| (agent.id, agent))
                .collect();
            let adjacency = build_adjacency_graph(&candidates, resources, &social_refs);
            let candidate_ids: Vec<EntityId> = candidates.iter().map(|agent| agent.id).collect();
            let components = find_connected_components(&candidate_ids, &adjacency);

            for proposed in components
                .into_iter()
                .filter_map(|component| {
                    build_proposed_band(&component, &candidate_by_id, existing_bands.as_slice())
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
            final_bands.insert(kept.id, kept);
        }

        let candidate_band_ids: Vec<BandId> = final_bands.keys().copied().collect();
        for band_id in candidate_band_ids {
            apply_band_fission(
                band_id,
                &mut final_bands,
                &agent_by_id,
                tick,
                resources,
                &mut split_events,
                &mut formed_events,
                &mut promoted_events,
            );
        }

        recruit_loners_into_bands(
            &mut final_bands,
            &agent_by_id,
            resources,
            &social_refs,
            &mut loner_join_events,
        );

        for band in final_bands.values_mut() {
            if !band.is_promoted
                && band.member_count() >= config::BAND_MIN_SIZE_PROMOTED
                && tick.saturating_sub(band.provisional_since) >= config::BAND_PROMOTION_TICKS
            {
                band.is_promoted = true;
                band.promoted_tick = Some(tick);
                promoted_events.push((band.id, band.members.clone()));
            }

            if band.is_promoted {
                let leader = elect_leader(&band.members, &agent_by_id);
                if leader != band.leader {
                    if let Some(leader_id) = leader {
                        leader_events.push((band.id, leader_id, band.members.clone()));
                    }
                    band.leader = leader;
                }
            } else {
                band.leader = None;
            }
        }

        for band_id in &dissolved_band_ids {
            if let Some(old_band) = existing_bands.iter().find(|band| band.id == *band_id) {
                dissolved_events.push((old_band.id, old_band.name.clone(), old_band.members.clone()));
            }
        }

        apply_identity_band_ids(world, &final_bands);

        for band_id in &dissolved_band_ids {
            resources.band_store.remove(*band_id);
        }
        for band in final_bands.values().cloned() {
            resources.band_store.insert(band);
        }
        refresh_band_behavior_state(world, resources);

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
            emit_band_formed_narrative(world, resources, tick, band_id, &members);
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
            emit_band_promoted_narrative(world, resources, tick, band_id, &members);
        }
        for (band_id, members, cause) in split_events {
            push_band_causal(
                resources,
                tick,
                band_id,
                &members,
                cause.as_kind(),
                "BAND_SPLIT",
                members.len() as f64,
            );
            emit_band_split_narrative(world, resources, tick, band_id, &members, cause);
        }
        for (band_id, band_name, members) in dissolved_events {
            push_band_causal(
                resources,
                tick,
                band_id,
                &members,
                "band_dissolved",
                "BAND_DISSOLVED",
                members.len() as f64,
            );
            emit_band_dissolved_narrative(world, resources, tick, band_id, band_name.as_str(), &members);
        }
        for (band_id, loner_id) in loner_join_events {
            push_band_causal(
                resources,
                tick,
                band_id,
                &[loner_id],
                "loner_joined_band",
                "LONER_JOINED",
                1.0,
            );
            emit_loner_join_narrative(world, resources, tick, band_id, loner_id);
        }
        for (band_id, leader_id, members) in leader_events {
            push_band_leader_causal(resources, tick, band_id, leader_id, &members);
            emit_band_leader_narrative(world, resources, tick, band_id, leader_id, &members);
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
            dissolved.push((band.id, band.name.clone(), band.members.clone()));
        }
    }
    if dissolved.is_empty() {
        return;
    }

    let dissolved_ids: BTreeSet<BandId> =
        dissolved.iter().map(|(band_id, _, _)| *band_id).collect();
    let retained_bands: BTreeMap<BandId, Band> = resources
        .band_store
        .all()
        .filter(|band| !dissolved_ids.contains(&band.id))
        .cloned()
        .map(|band| (band.id, band))
        .collect();

    apply_identity_band_ids(world, &retained_bands);

    let tick = resources.calendar.tick;
    for (band_id, band_name, members) in dissolved {
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
        emit_band_dissolved_narrative(world, resources, tick, band_id, band_name.as_str(), &members);
    }
    refresh_band_behavior_state(world, resources);
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

impl BandSplitCause {
    fn as_kind(self) -> &'static str {
        match self {
            Self::TrustCollapse => "band_split_trust_collapse",
            Self::ValueClash => "band_split_value_clash",
            Self::Overpopulation => "band_split_overpopulation",
        }
    }
}

fn apply_band_fission(
    band_id: BandId,
    final_bands: &mut BTreeMap<BandId, Band>,
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
    tick: u64,
    resources: &mut SimResources,
    split_events: &mut Vec<(BandId, Vec<EntityId>, BandSplitCause)>,
    formed_events: &mut Vec<(BandId, Vec<EntityId>)>,
    promoted_events: &mut Vec<(BandId, Vec<EntityId>)>,
) {
    let mut pending = vec![band_id];
    while let Some(current_band_id) = pending.pop() {
        let Some(current_band) = final_bands.get(&current_band_id).cloned() else {
            continue;
        };
        let Some(plan) = plan_band_fission(&current_band, agent_by_id, tick, resources) else {
            continue;
        };

        if plan.retained_band.members == current_band.members
            && plan.split_band.is_none()
            && plan.loners.is_empty()
        {
            continue;
        }

        final_bands.insert(current_band_id, plan.retained_band.clone());
        pending.push(current_band_id);

        if let Some(split_band) = plan.split_band.clone() {
            let split_band_id = split_band.id;
            formed_events.push((split_band_id, split_band.members.clone()));
            if split_band.is_promoted {
                promoted_events.push((split_band_id, split_band.members.clone()));
            }
            final_bands.insert(split_band_id, split_band);
            pending.push(split_band_id);
        }

        split_events.push((current_band_id, plan.original_members, plan.cause));
    }
}

fn plan_band_fission(
    band: &Band,
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
    tick: u64,
    resources: &mut SimResources,
) -> Option<BandFissionPlan> {
    if band.members.len() < config::BAND_MIN_SIZE_PROVISIONAL {
        return None;
    }

    let pair_metrics = pair_metrics_for_band(&band.members, agent_by_id);
    let avg_trust = average_metric(&pair_metrics, |metric| metric.trust);
    let avg_values = average_metric(&pair_metrics, |metric| metric.value_alignment);
    let cause = determine_split_cause(band, avg_trust, avg_values)?;

    let (group_a, group_b) = if cause == BandSplitCause::Overpopulation {
        split_members_by_position(&band.members, agent_by_id)?
    } else {
        split_members_by_social_graph(&band.members, &pair_metrics).or_else(|| {
            split_members_by_seed_preference(&band.members, &pair_metrics, agent_by_id)
        })?
    };

    if group_a.is_empty() || group_b.is_empty() {
        return None;
    }

    let (mut retained_members, mut split_members) = if group_a.len() >= group_b.len() {
        (group_a, group_b)
    } else {
        (group_b, group_a)
    };
    retained_members.sort_by_key(|member| member.0);
    split_members.sort_by_key(|member| member.0);

    if retained_members.len() < config::BAND_MIN_SIZE_PROVISIONAL {
        return None;
    }

    let mut retained_band = band.clone();
    retained_band.members = retained_members;
    retained_band.leader = None;

    let (split_band, loners) = if split_members.len() >= config::BAND_MIN_SIZE_PROVISIONAL {
        let new_band_id = resources.band_store.allocate_id();
        let mut new_band = Band::new(
            new_band_id,
            generate_band_name(new_band_id),
            split_members,
            tick,
        );
        if band.is_promoted {
            new_band.is_promoted = true;
            new_band.promoted_tick = Some(tick);
        }
        (Some(new_band), Vec::new())
    } else {
        (None, split_members)
    };

    Some(BandFissionPlan {
        retained_band,
        split_band,
        loners,
        cause,
        original_members: band.members.clone(),
    })
}

fn determine_split_cause(
    band: &Band,
    avg_trust: f64,
    avg_values: f64,
) -> Option<BandSplitCause> {
    if band.member_count() > config::BAND_MAX_SIZE {
        Some(BandSplitCause::Overpopulation)
    } else if band.is_promoted && avg_trust < config::BAND_FISSION_TRUST_THRESHOLD {
        Some(BandSplitCause::TrustCollapse)
    } else if band.is_promoted && avg_values < config::BAND_FISSION_VALUES_THRESHOLD {
        Some(BandSplitCause::ValueClash)
    } else {
        None
    }
}

fn pair_metrics_for_band(
    members: &[EntityId],
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
) -> Vec<PairMetric> {
    let mut metrics = Vec::new();
    for left_index in 0..members.len() {
        for right_index in (left_index + 1)..members.len() {
            let Some(left) = agent_by_id.get(&members[left_index]) else {
                continue;
            };
            let Some(right) = agent_by_id.get(&members[right_index]) else {
                continue;
            };
            metrics.push(PairMetric {
                left: left.id,
                right: right.id,
                trust: reciprocal_trust(left, right),
                value_alignment: left.values.alignment_with(&right.values),
            });
        }
    }
    metrics
}

fn average_metric(pair_metrics: &[PairMetric], select: impl Fn(&PairMetric) -> f64) -> f64 {
    if pair_metrics.is_empty() {
        1.0
    } else {
        pair_metrics.iter().map(select).sum::<f64>() / pair_metrics.len() as f64
    }
}

fn reciprocal_trust(left: &AgentSnapshot, right: &AgentSnapshot) -> f64 {
    let left_trust = left
        .social
        .find_edge(right.id)
        .map(|edge| edge.trust)
        .unwrap_or(0.0);
    let right_trust = right
        .social
        .find_edge(left.id)
        .map(|edge| edge.trust)
        .unwrap_or(0.0);
    (left_trust + right_trust) * 0.5
}

fn split_members_by_position(
    members: &[EntityId],
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
) -> Option<(Vec<EntityId>, Vec<EntityId>)> {
    if members.len() < config::BAND_MIN_SIZE_PROVISIONAL * 2 {
        return None;
    }

    let mut ordered = members.to_vec();
    ordered.sort_by(|left, right| {
        let left_agent = agent_by_id.get(left);
        let right_agent = agent_by_id.get(right);
        match (left_agent, right_agent) {
            (Some(left_agent), Some(right_agent)) => left_agent
                .x
                .partial_cmp(&right_agent.x)
                .unwrap_or(Ordering::Equal)
                .then_with(|| {
                    left_agent
                        .y
                        .partial_cmp(&right_agent.y)
                        .unwrap_or(Ordering::Equal)
                })
                .then_with(|| left.0.cmp(&right.0)),
            _ => left.0.cmp(&right.0),
        }
    });

    let split_at = ordered.len() / 2;
    let right = ordered.split_off(split_at);
    Some((ordered, right))
}

fn split_members_by_social_graph(
    members: &[EntityId],
    pair_metrics: &[PairMetric],
) -> Option<(Vec<EntityId>, Vec<EntityId>)> {
    let mut adjacency: BTreeMap<EntityId, Vec<EntityId>> =
        members.iter().copied().map(|member| (member, Vec::new())).collect();

    for metric in pair_metrics {
        if metric.trust >= config::BAND_FISSION_TRUST_THRESHOLD
            && metric.value_alignment >= config::BAND_FISSION_VALUES_THRESHOLD
        {
            adjacency.entry(metric.left).or_default().push(metric.right);
            adjacency.entry(metric.right).or_default().push(metric.left);
        }
    }

    for neighbors in adjacency.values_mut() {
        neighbors.sort_by_key(|neighbor| neighbor.0);
        neighbors.dedup();
    }

    let mut components = find_connected_components(members, &adjacency);
    if components.len() < 2 {
        return None;
    }

    components.sort_by(|left, right| {
        right
            .len()
            .cmp(&left.len())
            .then_with(|| left.first().cmp(&right.first()))
    });

    let mut primary = components.remove(0);
    let mut secondary = components.remove(0);
    for component in components {
        if component_connection_score(&component, &primary, pair_metrics)
            >= component_connection_score(&component, &secondary, pair_metrics)
        {
            primary.extend(component);
        } else {
            secondary.extend(component);
        }
    }
    primary.sort_by_key(|member| member.0);
    primary.dedup();
    secondary.sort_by_key(|member| member.0);
    secondary.dedup();

    if primary.is_empty() || secondary.is_empty() {
        None
    } else {
        Some((primary, secondary))
    }
}

fn component_connection_score(
    component: &[EntityId],
    target: &[EntityId],
    pair_metrics: &[PairMetric],
) -> f64 {
    let mut score = 0.0;
    let mut count = 0_u32;
    for member in component {
        for other in target {
            if let Some(metric) = pair_metrics.iter().find(|metric| {
                (metric.left == *member && metric.right == *other)
                    || (metric.left == *other && metric.right == *member)
            }) {
                score += metric.trust + metric.value_alignment;
                count = count.saturating_add(1);
            }
        }
    }
    if count == 0 {
        0.0
    } else {
        score / count as f64
    }
}

fn split_members_by_seed_preference(
    members: &[EntityId],
    pair_metrics: &[PairMetric],
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
) -> Option<(Vec<EntityId>, Vec<EntityId>)> {
    if members.len() < config::BAND_MIN_SIZE_PROVISIONAL * 2 {
        return None;
    }

    let seed_pair = pair_metrics.iter().min_by(|left, right| {
        left.trust
            .partial_cmp(&right.trust)
            .unwrap_or(Ordering::Equal)
            .then_with(|| {
                left.value_alignment
                    .partial_cmp(&right.value_alignment)
                    .unwrap_or(Ordering::Equal)
            })
            .then_with(|| left.left.0.cmp(&right.left.0))
            .then_with(|| left.right.0.cmp(&right.right.0))
    })?;

    let seed_a = seed_pair.left;
    let seed_b = seed_pair.right;
    let mut group_a = vec![seed_a];
    let mut group_b = vec![seed_b];
    let mut assignments = Vec::new();

    for member in members.iter().copied() {
        if member == seed_a || member == seed_b {
            continue;
        }
        let score_a = seed_preference_score(member, seed_a, agent_by_id);
        let score_b = seed_preference_score(member, seed_b, agent_by_id);
        assignments.push((member, score_a, score_b, (score_a - score_b).abs()));
    }

    assignments.sort_by(|left, right| {
        right
            .3
            .partial_cmp(&left.3)
            .unwrap_or(Ordering::Equal)
            .then_with(|| left.0 .0.cmp(&right.0 .0))
    });

    for (member, score_a, score_b, _) in assignments {
        let assign_to_a = score_a
            .partial_cmp(&score_b)
            .unwrap_or(Ordering::Equal)
            .then_with(|| group_b.len().cmp(&group_a.len()))
            .is_ge();
        if assign_to_a {
            group_a.push(member);
        } else {
            group_b.push(member);
        }
    }

    rebalance_groups(&mut group_a, &mut group_b, seed_a, seed_b, agent_by_id);
    if group_a.len() < config::BAND_MIN_SIZE_PROVISIONAL
        || group_b.len() < config::BAND_MIN_SIZE_PROVISIONAL
    {
        return None;
    }

    group_a.sort_by_key(|member| member.0);
    group_b.sort_by_key(|member| member.0);
    Some((group_a, group_b))
}

fn seed_preference_score(
    member: EntityId,
    seed: EntityId,
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
) -> f64 {
    let Some(member_agent) = agent_by_id.get(&member) else {
        return 0.0;
    };
    let Some(seed_agent) = agent_by_id.get(&seed) else {
        return 0.0;
    };
    (reciprocal_trust(member_agent, seed_agent) * 0.7)
        + (member_agent.values.alignment_with(&seed_agent.values) * 0.3)
}

fn rebalance_groups(
    group_a: &mut Vec<EntityId>,
    group_b: &mut Vec<EntityId>,
    seed_a: EntityId,
    seed_b: EntityId,
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
) {
    rebalance_one_side(group_a, group_b, seed_a, seed_b, agent_by_id);
    rebalance_one_side(group_b, group_a, seed_b, seed_a, agent_by_id);
}

fn rebalance_one_side(
    source: &mut Vec<EntityId>,
    target: &mut Vec<EntityId>,
    source_seed: EntityId,
    target_seed: EntityId,
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
) {
    while target.len() < config::BAND_MIN_SIZE_PROVISIONAL
        && source.len() > config::BAND_MIN_SIZE_PROVISIONAL
    {
        let Some((move_index, _)) = source
            .iter()
            .enumerate()
            .filter(|(_, member)| **member != source_seed)
            .map(|(index, member)| {
                let stay_score = seed_preference_score(*member, source_seed, agent_by_id);
                let switch_score = seed_preference_score(*member, target_seed, agent_by_id);
                (index, stay_score - switch_score)
            })
            .min_by(|left, right| {
                left.1
                    .partial_cmp(&right.1)
                    .unwrap_or(Ordering::Equal)
                    .then_with(|| source[left.0].0.cmp(&source[right.0].0))
            })
        else {
            break;
        };
        let moved = source.remove(move_index);
        target.push(moved);
    }
}

fn recruit_loners_into_bands(
    final_bands: &mut BTreeMap<BandId, Band>,
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
    resources: &SimResources,
    all_socials: &[(EntityId, &Social)],
    loner_join_events: &mut Vec<(BandId, EntityId)>,
) {
    let mut assigned_members: BTreeSet<EntityId> = final_bands
        .values()
        .flat_map(|band| band.members.iter().copied())
        .collect();
    let mut loners: Vec<EntityId> = agent_by_id
        .keys()
        .copied()
        .filter(|entity_id| !assigned_members.contains(entity_id))
        .collect();
    loners.sort_by_key(|entity_id| entity_id.0);

    for loner_id in loners {
        let Some(loner) = agent_by_id.get(&loner_id) else {
            continue;
        };
        let nearby_count = nearby_agent_count(loner, agent_by_id.values()) as f64;
        let mut best_band: Option<(BandId, f64)> = None;

        let band_ids: Vec<BandId> = final_bands.keys().copied().collect();
        for band_id in band_ids {
            let Some(band) = final_bands.get(&band_id) else {
                continue;
            };
            if !band.is_promoted || band.member_count() >= config::BAND_MAX_SIZE {
                continue;
            }
            if !band_has_nearby_member(loner, band, agent_by_id, config::LONER_SEARCH_RADIUS) {
                continue;
            }
            let joined_ratio = (band.member_count() as f64 / nearby_count.max(1.0)).clamp(0.0, 1.0);
            if joined_ratio < granovetter_threshold(loner) {
                continue;
            }
            let avg_gfs = average_gfs_to_band(loner, band, agent_by_id, resources, all_socials);
            if avg_gfs < config::LONER_JOIN_GFS_THRESHOLD {
                continue;
            }

            let should_replace = match best_band {
                Some((best_id, best_score)) => {
                    avg_gfs > best_score
                        || ((avg_gfs - best_score).abs() < 1e-9 && band_id.0 < best_id.0)
                }
                None => true,
            };
            if should_replace {
                best_band = Some((band_id, avg_gfs));
            }
        }

        if let Some((band_id, _)) = best_band {
            if let Some(band) = final_bands.get_mut(&band_id) {
                band.add_member(loner_id);
                assigned_members.insert(loner_id);
                loner_join_events.push((band_id, loner_id));
            }
        }
    }
}

fn band_has_nearby_member(
    loner: &AgentSnapshot,
    band: &Band,
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
    radius: f64,
) -> bool {
    let radius_sq = radius * radius;
    band.members.iter().any(|member_id| {
        agent_by_id.get(member_id).is_some_and(|member| {
            let dx = loner.x - member.x;
            let dy = loner.y - member.y;
            dx * dx + dy * dy <= radius_sq
        })
    })
}

fn average_gfs_to_band(
    loner: &AgentSnapshot,
    band: &Band,
    agent_by_id: &BTreeMap<EntityId, AgentSnapshot>,
    resources: &SimResources,
    all_socials: &[(EntityId, &Social)],
) -> f64 {
    let mut total = 0.0;
    let mut count = 0_u32;
    for member_id in &band.members {
        let Some(member) = agent_by_id.get(member_id) else {
            continue;
        };
        let gfs = calculate_gfs(
            loner.id,
            member.id,
            (loner.x, loner.y),
            (member.x, member.y),
            loner.settlement_id,
            member.settlement_id,
            &loner.social,
            &member.social,
            &loner.values,
            &member.values,
            loner.safety,
            member.safety,
            settlement_resource_score(resources, loner.settlement_id, member.settlement_id),
            all_socials,
        );
        total += gfs;
        count = count.saturating_add(1);
    }

    if count == 0 {
        0.0
    } else {
        total / count as f64
    }
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

/// Generates a Korean nature-themed band name from the band ID.
/// 15 × 15 = 225 unique combinations before repeating with a numeric suffix.
fn generate_band_name(band_id: BandId) -> String {
    const PREFIXES: &[&str] = &[
        "붉은", "푸른", "검은", "흰", "금빛", "은빛", "높은", "깊은", "넓은", "밝은", "어둔",
        "거센", "고요한", "빠른", "강한",
    ];
    const SUFFIXES: &[&str] = &[
        "바위", "산", "강", "숲", "바람", "불꽃", "이슬", "달빛", "별빛", "여울", "구름",
        "뿌리", "이끼", "매", "늑대",
    ];
    let idx = band_id.0 as usize;
    let prefix = PREFIXES[idx % PREFIXES.len()];
    let suffix = SUFFIXES[(idx / PREFIXES.len()) % SUFFIXES.len()];
    format!("{} {}", prefix, suffix)
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

fn emit_band_formed_narrative(
    world: &World,
    resources: &mut SimResources,
    tick: u64,
    band_id: BandId,
    members: &[EntityId],
) {
    let Some(actor_id) = band_actor_entity_id(resources, band_id, members) else {
        return;
    };
    let band_name = band_name_for(resources, band_id);
    push_band_sim_event(
        resources,
        tick,
        SimEventType::BandFormed,
        actor_id,
        None,
        &["band", "social", "formation"],
        format!("provisional_formed_{}", band_id.0),
        members.len() as f64,
    );

    let mut params = BTreeMap::new();
    params.insert("count".to_string(), members.len().to_string());
    params.insert("name".to_string(), band_name.clone());
    append_band_chronicle_event(
        world,
        resources,
        tick,
        actor_id,
        members,
        "CHRONICLE_BAND_FORMED",
        params,
        format!("band_formed:{}", band_id.0),
        members.len() as f64,
    );
}

fn emit_band_promoted_narrative(
    world: &World,
    resources: &mut SimResources,
    tick: u64,
    band_id: BandId,
    members: &[EntityId],
) {
    let Some(actor_id) = band_actor_entity_id(resources, band_id, members) else {
        return;
    };
    let band_name = band_name_for(resources, band_id);
    push_band_sim_event(
        resources,
        tick,
        SimEventType::BandPromoted,
        actor_id,
        None,
        &["band", "social", "promotion"],
        format!("band_promoted_{}", band_id.0),
        members.len() as f64,
    );

    let mut params = BTreeMap::new();
    params.insert("name".to_string(), band_name);
    append_band_chronicle_event(
        world,
        resources,
        tick,
        actor_id,
        members,
        "CHRONICLE_BAND_PROMOTED",
        params,
        format!("band_promoted:{}", band_id.0),
        members.len() as f64,
    );
}

fn emit_band_split_narrative(
    world: &World,
    resources: &mut SimResources,
    tick: u64,
    band_id: BandId,
    members: &[EntityId],
    cause: BandSplitCause,
) {
    let Some(actor_id) = band_actor_entity_id(resources, band_id, members) else {
        return;
    };
    let band_name = band_name_for(resources, band_id);
    push_band_sim_event(
        resources,
        tick,
        SimEventType::BandSplit,
        actor_id,
        None,
        &["band", "social", "split"],
        cause.as_kind().to_string(),
        members.len() as f64,
    );

    let mut params = BTreeMap::new();
    params.insert("name".to_string(), band_name);
    params.insert("cause".to_string(), cause.as_kind().to_string());
    append_band_chronicle_event(
        world,
        resources,
        tick,
        actor_id,
        members,
        "CHRONICLE_BAND_SPLIT",
        params,
        format!("band_split:{}:{}", band_id.0, cause.as_kind()),
        members.len() as f64,
    );
}

fn emit_band_dissolved_narrative(
    world: &World,
    resources: &mut SimResources,
    tick: u64,
    band_id: BandId,
    band_name: &str,
    members: &[EntityId],
) {
    let Some(actor_id) = members.first().copied() else {
        return;
    };
    push_band_sim_event(
        resources,
        tick,
        SimEventType::BandDissolved,
        actor_id,
        None,
        &["band", "social", "dissolution"],
        format!("band_dissolved_{}", band_id.0),
        members.len() as f64,
    );

    let mut params = BTreeMap::new();
    params.insert("name".to_string(), band_name.to_string());
    append_band_chronicle_event(
        world,
        resources,
        tick,
        actor_id,
        members,
        "CHRONICLE_BAND_DISSOLVED",
        params,
        format!("band_dissolved:{}", band_id.0),
        members.len() as f64,
    );
}

fn emit_band_leader_narrative(
    world: &World,
    resources: &mut SimResources,
    tick: u64,
    band_id: BandId,
    leader_id: EntityId,
    members: &[EntityId],
) {
    let band_name = band_name_for(resources, band_id);
    let leader_name = entity_label(world, leader_id);
    push_band_sim_event(
        resources,
        tick,
        SimEventType::BandLeaderElected,
        leader_id,
        None,
        &["band", "social", "leadership"],
        format!("band_leader_elected_{}", band_id.0),
        1.0,
    );

    let mut params = BTreeMap::new();
    params.insert("name".to_string(), band_name);
    params.insert("leader".to_string(), leader_name);
    append_band_chronicle_event(
        world,
        resources,
        tick,
        leader_id,
        members,
        "CHRONICLE_BAND_LEADER",
        params,
        format!("band_leader:{}:{}", band_id.0, leader_id.0),
        3.0,
    );
}

fn emit_loner_join_narrative(
    world: &World,
    resources: &mut SimResources,
    tick: u64,
    band_id: BandId,
    loner_id: EntityId,
) {
    let band_name = band_name_for(resources, band_id);
    let agent_name = entity_label(world, loner_id);
    push_band_sim_event(
        resources,
        tick,
        SimEventType::LonerJoinedBand,
        loner_id,
        None,
        &["band", "social", "recruitment"],
        format!("loner_joined_band_{}", band_id.0),
        1.0,
    );

    let mut params = BTreeMap::new();
    params.insert("name".to_string(), band_name);
    params.insert("agent".to_string(), agent_name);
    append_band_chronicle_event(
        world,
        resources,
        tick,
        loner_id,
        &[loner_id],
        "CHRONICLE_LONER_JOINED",
        params,
        format!("band_join:{}", band_id.0),
        3.0,
    );
}

fn push_band_sim_event(
    resources: &mut SimResources,
    tick: u64,
    event_type: SimEventType,
    actor_id: EntityId,
    target_id: Option<EntityId>,
    tags: &[&str],
    cause: String,
    value: f64,
) {
    let Some(actor) = raw_entity_id(actor_id) else {
        return;
    };
    resources.event_store.push(SimEvent {
        tick,
        event_type,
        actor,
        target: target_id.and_then(raw_entity_id),
        tags: tags.iter().map(|tag| (*tag).to_string()).collect(),
        cause,
        value,
    });
}

fn append_band_chronicle_event(
    world: &World,
    resources: &mut SimResources,
    tick: u64,
    entity_id: EntityId,
    members: &[EntityId],
    summary_key: &str,
    summary_params: BTreeMap<String, String>,
    effect_key: String,
    magnitude: f64,
) {
    let (tile_x, tile_y) = band_event_tile(world, members);
    resources.chronicle_log.append_event(ChronicleEvent {
        tick,
        entity_id,
        event_type: ChronicleEventType::BandLifecycle,
        cause: ChronicleEventCause::SocialGroup,
        magnitude: ChronicleEventMagnitude {
            influence: magnitude,
            steering: 0.0,
            significance: magnitude.max(3.0),
        },
        tile_x,
        tile_y,
        summary_key: summary_key.to_string(),
        summary_params,
        effect_key,
    });
}

fn band_name_for(resources: &SimResources, band_id: BandId) -> String {
    resources
        .band_store
        .get(band_id)
        .map(|band| band.name.clone())
        .unwrap_or_else(|| generate_band_name(band_id))
}

fn band_actor_entity_id(
    resources: &SimResources,
    band_id: BandId,
    members: &[EntityId],
) -> Option<EntityId> {
    resources
        .band_store
        .get(band_id)
        .and_then(|band| band.leader.or_else(|| band.members.first().copied()))
        .or_else(|| members.first().copied())
}

fn raw_entity_id(entity_id: EntityId) -> Option<u32> {
    u32::try_from(entity_id.0).ok()
}

fn entity_label(world: &World, entity_id: EntityId) -> String {
    let mut query = world.query::<&Identity>();
    query
        .iter()
        .find_map(|(entity, identity)| {
            (entity.id() as u64 == entity_id.0).then(|| identity.name.clone())
        })
        .unwrap_or_else(|| format!("#{}", entity_id.0))
}

fn band_event_tile(world: &World, members: &[EntityId]) -> (i32, i32) {
    let member_ids: BTreeSet<EntityId> = members.iter().copied().collect();
    if member_ids.is_empty() {
        return (0, 0);
    }

    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut count = 0usize;
    let mut query = world.query::<&Position>();
    for (entity, position) in &mut query {
        if member_ids.contains(&EntityId(entity.id() as u64)) {
            sum_x += position.x;
            sum_y += position.y;
            count += 1;
        }
    }

    if count == 0 {
        (0, 0)
    } else {
        (
            (sum_x / count as f64).round() as i32,
            (sum_y / count as f64).round() as i32,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hecs::Entity;
    use sim_core::band::BandStore;
    use sim_core::{config::GameConfig, GameCalendar, WorldMap};
    use sim_engine::{SimEventType, SimResources};

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

    fn set_mutual_trust(world: &mut World, left: Entity, right: Entity, trust: f64) {
        set_trust(world, left, right, trust);
        set_trust(world, right, left, trust);
    }

    fn seed_band(
        resources: &mut SimResources,
        band_id: BandId,
        members: &[Entity],
        promoted: bool,
        provisional_since: u64,
    ) {
        resources.band_store.insert(Band {
            id: band_id,
            name: generate_band_name(band_id),
            members: members
                .iter()
                .map(|entity| EntityId(entity.id() as u64))
                .collect(),
            leader: None,
            provisional_since,
            promoted_tick: promoted.then_some(provisional_since),
            is_promoted: promoted,
        });
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
        assert_eq!(
            resources
                .event_store
                .by_type(&SimEventType::BandFormed, 0)
                .len(),
            1
        );
        let chronicle = resources.chronicle_log.recent_events(4);
        assert!(chronicle
            .iter()
            .any(|event| event.summary_key == "CHRONICLE_BAND_FORMED"));
        assert!(chronicle.iter().any(|event| {
            event.summary_key == "CHRONICLE_BAND_FORMED"
                && event.summary_params.get("count").map(String::as_str) == Some("3")
        }));
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
    fn dissolving_small_band_preserves_other_band_memberships() {
        let mut resources = make_resources();
        let mut world = World::new();
        let small_band_id = BandId(1);
        let kept_band_id = BandId(2);

        let a = spawn_agent(
            &mut world,
            "A",
            0,
            0,
            Some(SettlementId(1)),
            Some(small_band_id),
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
            None,
            0.5,
            0.5,
            0.5,
        );
        let c = spawn_agent(
            &mut world,
            "C",
            4,
            0,
            Some(SettlementId(1)),
            Some(kept_band_id),
            0.5,
            0.5,
            0.5,
        );
        let d = spawn_agent(
            &mut world,
            "D",
            5,
            0,
            Some(SettlementId(1)),
            Some(kept_band_id),
            0.5,
            0.5,
            0.5,
        );
        let e = spawn_agent(
            &mut world,
            "E",
            6,
            0,
            Some(SettlementId(1)),
            Some(kept_band_id),
            0.5,
            0.5,
            0.5,
        );

        resources.band_store = BandStore::new();
        resources.band_store.insert(Band {
            id: small_band_id,
            name: "Small".to_string(),
            members: vec![EntityId(a.id() as u64)],
            leader: None,
            provisional_since: 10,
            promoted_tick: None,
            is_promoted: false,
        });
        resources.band_store.insert(Band {
            id: kept_band_id,
            name: "Kept".to_string(),
            members: vec![
                EntityId(c.id() as u64),
                EntityId(d.id() as u64),
                EntityId(e.id() as u64),
            ],
            leader: None,
            provisional_since: 10,
            promoted_tick: None,
            is_promoted: false,
        });

        cleanup_small_bands(&mut world, &mut resources);

        assert!(resources.band_store.get(small_band_id).is_none());
        assert!(resources.band_store.get(kept_band_id).is_some());
        assert_eq!(
            world
                .get::<&Identity>(c)
                .expect("identity")
                .band_id,
            Some(kept_band_id)
        );
        assert_eq!(
            world
                .get::<&Identity>(d)
                .expect("identity")
                .band_id,
            Some(kept_band_id)
        );
        assert_eq!(
            world
                .get::<&Identity>(e)
                .expect("identity")
                .band_id,
            Some(kept_band_id)
        );
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
    fn fission_triggers_when_trust_below_threshold() {
        let (mut world, mut resources) = (World::new(), make_resources());
        let band_id = resources.band_store.allocate_id();
        let mut members = Vec::new();
        for index in 0..6 {
            members.push(spawn_agent(
                &mut world,
                &format!("M{index}"),
                index,
                0,
                Some(SettlementId(1)),
                Some(band_id),
                0.6,
                0.7,
                0.7,
            ));
        }
        seed_band(&mut resources, band_id, &members, true, 10);

        for left in 0..3 {
            for right in (left + 1)..3 {
                set_mutual_trust(&mut world, members[left], members[right], 0.7);
            }
        }
        for left in 3..6 {
            for right in (left + 1)..6 {
                set_mutual_trust(&mut world, members[left], members[right], 0.7);
            }
        }

        BandFormationSystem::new(38, 60).run(&mut world, &mut resources, 60);

        assert_eq!(resources.band_store.len(), 2);
        let sizes: Vec<usize> = resources.band_store.all().map(Band::member_count).collect();
        assert!(sizes.contains(&3));
        let split_band = resources
            .band_store
            .all()
            .find(|band| band.id != band_id)
            .expect("split band");
        assert_eq!(split_band.provisional_since, 60);
        assert_eq!(split_band.promoted_tick, Some(60));
        let split_member = split_band.members[0];
        let split_member_events = resources.causal_log.recent(split_member, 8);
        assert!(split_member_events
            .iter()
            .any(|event| event.summary_key == "BAND_FORMED"));
        assert!(split_member_events
            .iter()
            .any(|event| event.summary_key == "BAND_PROMOTED"));
        let recent = resources
            .causal_log
            .recent(EntityId(members[0].id() as u64), 8)
            .into_iter()
            .map(|event| event.summary_key.as_str())
            .collect::<Vec<_>>();
        assert!(recent.contains(&"BAND_SPLIT"));
        assert_eq!(
            resources.event_store.by_type(&SimEventType::BandSplit, 0).len(),
            1
        );
        let chronicle = resources.chronicle_log.recent_events(8);
        assert!(chronicle
            .iter()
            .any(|event| event.summary_key == "CHRONICLE_BAND_SPLIT"));
        assert!(chronicle.iter().any(|event| {
            event.summary_key == "CHRONICLE_BAND_SPLIT"
                && event.summary_params.get("cause").map(String::as_str)
                    == Some(BandSplitCause::TrustCollapse.as_kind())
        }));
    }

    #[test]
    fn fission_triggers_when_over_max_size() {
        let (mut world, mut resources) = (World::new(), make_resources());
        let band_id = resources.band_store.allocate_id();
        let mut members = Vec::new();
        for index in 0..31 {
            members.push(spawn_agent(
                &mut world,
                &format!("M{index}"),
                index,
                0,
                Some(SettlementId(1)),
                Some(band_id),
                0.8,
                0.6,
                0.6,
            ));
        }
        seed_band(&mut resources, band_id, &members, true, 10);
        for left in 0..members.len() {
            for right in (left + 1)..members.len() {
                set_mutual_trust(&mut world, members[left], members[right], 0.8);
            }
        }

        BandFormationSystem::new(38, 60).run(&mut world, &mut resources, 60);

        assert_eq!(resources.band_store.len(), 2);
        let total_members: usize = resources.band_store.all().map(Band::member_count).sum();
        assert_eq!(total_members, 31);
        assert!(resources
            .band_store
            .all()
            .all(|band| band.member_count() <= config::BAND_MAX_SIZE));
    }

    #[test]
    fn no_fission_when_trust_healthy() {
        let (mut world, mut resources) = (World::new(), make_resources());
        let band_id = resources.band_store.allocate_id();
        let mut members = Vec::new();
        for index in 0..6 {
            members.push(spawn_agent(
                &mut world,
                &format!("M{index}"),
                index,
                0,
                Some(SettlementId(1)),
                Some(band_id),
                0.7,
                0.6,
                0.6,
            ));
        }
        seed_band(&mut resources, band_id, &members, true, 10);

        for left in 0..members.len() {
            for right in (left + 1)..members.len() {
                set_mutual_trust(&mut world, members[left], members[right], 0.8);
            }
        }

        BandFormationSystem::new(38, 60).run(&mut world, &mut resources, 60);

        assert_eq!(resources.band_store.len(), 1);
        let band = resources.band_store.get(band_id).expect("band");
        assert_eq!(band.member_count(), 6);
    }

    #[test]
    fn fission_creates_two_valid_sub_bands() {
        let (mut world, mut resources) = (World::new(), make_resources());
        let band_id = resources.band_store.allocate_id();
        let mut members = Vec::new();
        for index in 0..6 {
            members.push(spawn_agent(
                &mut world,
                &format!("M{index}"),
                index,
                0,
                Some(SettlementId(1)),
                Some(band_id),
                0.6,
                if index < 3 { 0.9 } else { 0.4 },
                0.7,
            ));
        }
        seed_band(&mut resources, band_id, &members, true, 10);

        for left in 0..3 {
            for right in (left + 1)..3 {
                set_mutual_trust(&mut world, members[left], members[right], 0.7);
            }
        }
        for left in 3..6 {
            for right in (left + 1)..6 {
                set_mutual_trust(&mut world, members[left], members[right], 0.7);
            }
        }

        BandFormationSystem::new(38, 60).run(&mut world, &mut resources, 60);

        let bands: Vec<&Band> = resources.band_store.all().collect();
        assert_eq!(bands.len(), 2);
        assert!(bands.iter().all(|band| band.member_count() >= 3));
        assert!(bands.iter().all(|band| band.leader.is_some()));
    }

    #[test]
    fn loner_joins_nearby_band_when_gfs_sufficient() {
        let (mut world, mut resources) = (World::new(), make_resources());
        let band_id = resources.band_store.allocate_id();
        let mut members = Vec::new();
        for index in 0..5 {
            members.push(spawn_agent(
                &mut world,
                &format!("B{index}"),
                index,
                0,
                Some(SettlementId(1)),
                Some(band_id),
                0.7,
                0.7,
                0.8,
            ));
        }
        let loner = spawn_agent(
            &mut world,
            "Loner",
            2,
            1,
            Some(SettlementId(1)),
            None,
            0.7,
            0.8,
            0.9,
        );
        seed_band(&mut resources, band_id, &members, true, 10);
        resources.settlements.insert(
            SettlementId(1),
            settlement_with_stockpile(
                SettlementId(1),
                members
                    .iter()
                    .map(|entity| EntityId(entity.id() as u64))
                    .chain(std::iter::once(EntityId(loner.id() as u64)))
                    .collect(),
            ),
        );

        for left in 0..members.len() {
            for right in (left + 1)..members.len() {
                set_mutual_trust(&mut world, members[left], members[right], 0.8);
            }
        }
        for &member in &members {
            set_mutual_trust(&mut world, loner, member, 0.8);
        }

        BandFormationSystem::new(38, 60).run(&mut world, &mut resources, 60);

        let band = resources.band_store.get(band_id).expect("band");
        assert_eq!(band.member_count(), 6);
        assert_eq!(
            world.get::<&Identity>(loner).expect("identity").band_id,
            Some(band_id)
        );
        let loner_events = resources.causal_log.recent(EntityId(loner.id() as u64), 8);
        assert!(loner_events
            .iter()
            .any(|event| event.summary_key == "LONER_JOINED"));
        let member_events = resources
            .causal_log
            .recent(EntityId(members[0].id() as u64), 8);
        assert!(!member_events
            .iter()
            .any(|event| event.summary_key == "LONER_JOINED"));
        assert_eq!(
            resources
                .event_store
                .by_type(&SimEventType::LonerJoinedBand, 0)
                .len(),
            1
        );
        let chronicle = resources.chronicle_log.recent_events(8);
        assert!(chronicle
            .iter()
            .any(|event| event.summary_key == "CHRONICLE_LONER_JOINED"));
        assert!(chronicle.iter().any(|event| {
            event.summary_key == "CHRONICLE_LONER_JOINED"
                && event.summary_params.get("agent").map(String::as_str) == Some("Loner")
        }));
    }

    #[test]
    fn oversized_provisional_band_splits_before_promotion() {
        let (mut world, mut resources) = (World::new(), make_resources());
        let band_id = resources.band_store.allocate_id();
        let mut members = Vec::new();
        for index in 0..31 {
            members.push(spawn_agent(
                &mut world,
                &format!("M{index}"),
                index,
                0,
                Some(SettlementId(1)),
                Some(band_id),
                0.7,
                0.7,
                0.7,
            ));
        }
        for left in 0..members.len() {
            for right in (left + 1)..members.len() {
                set_mutual_trust(&mut world, members[left], members[right], 0.8);
            }
        }
        let agent_by_id: BTreeMap<EntityId, AgentSnapshot> = collect_agent_snapshots(&world)
            .into_iter()
            .map(|agent| (agent.id, agent))
            .collect();
        let band = Band {
            id: band_id,
            name: generate_band_name(band_id),
            members: members
                .iter()
                .map(|entity| EntityId(entity.id() as u64))
                .collect(),
            leader: None,
            provisional_since: 10,
            promoted_tick: None,
            is_promoted: false,
        };

        let plan =
            plan_band_fission(&band, &agent_by_id, 60, &mut resources).expect("fission plan");
        let split_band = plan.split_band.expect("split band");
        assert!(!plan.retained_band.is_promoted);
        assert!(!split_band.is_promoted);
        assert_eq!(split_band.provisional_since, 60);
        assert_eq!(split_band.promoted_tick, None);
        assert_eq!(
            plan.retained_band.member_count() + split_band.member_count() + plan.loners.len(),
            31
        );
        assert!(plan.retained_band.member_count() <= config::BAND_MAX_SIZE);
        assert!(split_band.member_count() <= config::BAND_MAX_SIZE);
    }

    #[test]
    fn determine_split_cause_prefers_value_clash_when_values_are_low() {
        let band = Band {
            id: BandId(7),
            name: "band_7".to_string(),
            members: vec![
                EntityId(1),
                EntityId(2),
                EntityId(3),
                EntityId(4),
                EntityId(5),
                EntityId(6),
            ],
            leader: None,
            provisional_since: 10,
            promoted_tick: Some(20),
            is_promoted: true,
        };

        assert_eq!(
            determine_split_cause(&band, 0.8, 0.2),
            Some(BandSplitCause::ValueClash)
        );
    }

    #[test]
    fn band_names_are_deterministic_and_unique() {
        // BandId(7): idx=7, prefix=PREFIXES[7%15]="깊은", suffix=SUFFIXES[0%15]="바위"
        assert_eq!(generate_band_name(BandId(7)), "깊은 바위");
        // BandId(0): idx=0, prefix="붉은", suffix="바위"
        assert_eq!(generate_band_name(BandId(0)), "붉은 바위");
        // BandId(15): idx=15, prefix="붉은", suffix="산" (wraps prefix, advances suffix)
        assert_eq!(generate_band_name(BandId(15)), "붉은 산");
        // Same id always produces same name
        assert_eq!(generate_band_name(BandId(7)), generate_band_name(BandId(7)));
    }
}
