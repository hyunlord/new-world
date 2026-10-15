use hecs::World;
use sim_core::components::{Agent, AgentId, BodyHealth, Child, Kin};
use sim_core::rules::FissionRule;
use std::collections::{BTreeMap, BTreeSet};

/// Living settlement households, each sorted, ordered by their smallest member.
pub fn households(world: &World, members: &BTreeSet<AgentId>) -> Vec<Vec<AgentId>> {
    let people: BTreeMap<_, _> = world
        .query::<(&Agent, Option<&Kin>, Option<&Child>, Option<&BodyHealth>)>()
        .iter()
        .filter(|(_, (agent, _, _, health))| {
            members.contains(&agent.id) && health.is_none_or(|health| !health.is_dead())
        })
        .map(|(_, (agent, kin, child, _))| {
            (
                agent.id,
                (kin.copied().unwrap_or_default(), child.is_some()),
            )
        })
        .collect();
    let alive: BTreeSet<_> = world
        .query::<(&Agent, Option<&BodyHealth>)>()
        .iter()
        .filter(|(_, (_, health))| health.is_none_or(|health| !health.is_dead()))
        .map(|(_, (agent, _))| agent.id)
        .collect();
    let mut group_of = BTreeMap::new();
    for (&id, (kin, _)) in &people {
        if let Some(partner) = kin.partner.filter(|partner| {
            *partner != id
                && people
                    .get(partner)
                    .is_some_and(|(other, _)| other.partner == Some(id))
        }) {
            let key = id.min(partner);
            group_of.insert(id, key);
            group_of.insert(partner, key);
        }
    }
    for (kin, child) in people.values() {
        if *child && kin.father.is_none_or(|father| !alive.contains(&father)) {
            if let Some(mother) = kin.mother.filter(|mother| people.contains_key(mother)) {
                group_of.entry(mother).or_insert(mother);
            }
        }
    }
    let parents = group_of.clone();
    for (&id, (kin, child)) in &people {
        if *child && !group_of.contains_key(&id) {
            if let Some(key) = [kin.mother, kin.father]
                .into_iter()
                .flatten()
                .find_map(|parent| parents.get(&parent).copied())
            {
                group_of.insert(id, key);
            }
        }
        group_of.entry(id).or_insert(id);
    }
    let mut groups: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for (id, key) in group_of {
        groups.entry(key).or_default().push(id);
    }
    let mut result: Vec<_> = groups.into_values().collect();
    result.sort_by_key(|group| group[0]);
    result
}

pub(crate) fn select_departure(
    ranked: &[(u64, AgentId, bool)],
    households: &[Vec<AgentId>],
    rule: &FissionRule,
    k: u32,
) -> Option<BTreeSet<AgentId>> {
    let rank: BTreeMap<_, _> = ranked
        .iter()
        .enumerate()
        .map(|(i, row)| (row.1, (i, row.2)))
        .collect();
    let groups: Vec<Vec<_>> = households
        .iter()
        .map(|group| {
            group
                .iter()
                .copied()
                .filter(|id| rank.contains_key(id))
                .collect()
        })
        .filter(|group: &Vec<_>| !group.is_empty())
        .collect();
    let group_of: BTreeMap<_, _> = groups
        .iter()
        .enumerate()
        .flat_map(|(index, group)| group.iter().map(move |id| (*id, index)))
        .collect();
    let mut chosen = BTreeSet::new();
    for (_, id, _) in ranked.iter().filter(|row| row.2) {
        if chosen
            .iter()
            .filter(|id| rank.get(id).is_some_and(|row| row.1))
            .count()
            >= rule.min_adults as usize
        {
            break;
        }
        let Some(group) = group_of.get(id).map(|index| &groups[*index]) else {
            continue;
        };
        if !chosen.contains(id) && chosen.len() + group.len() <= rule.group_max as usize {
            chosen.extend(group);
        }
    }
    let mut fill: Vec<_> = groups.iter().collect();
    fill.sort_by_key(|group| {
        group
            .iter()
            .filter_map(|id| rank.get(id).map(|row| row.0))
            .min()
    });
    for group in fill {
        if !group.iter().any(|id| chosen.contains(id)) && chosen.len() + group.len() <= k as usize {
            chosen.extend(group);
        }
    }
    if chosen.len() < rule.group_min as usize {
        for (_, id, _) in ranked {
            if chosen.len() >= rule.group_min as usize {
                break;
            }
            chosen.insert(*id);
        }
    }
    (chosen.len() >= rule.group_min as usize
        && chosen
            .iter()
            .filter(|id| rank.get(id).is_some_and(|row| row.1))
            .count()
            >= rule.min_adults as usize)
        .then_some(chosen)
}

pub(crate) fn couples(world: &World, members: &BTreeSet<AgentId>) -> Vec<[AgentId; 2]> {
    let kin: BTreeMap<_, _> = world
        .query::<(&Agent, &Kin, Option<&BodyHealth>)>()
        .iter()
        .filter(|(_, (agent, _, health))| {
            members.contains(&agent.id) && health.is_none_or(|health| !health.is_dead())
        })
        .map(|(_, (agent, kin, _))| (agent.id, *kin))
        .collect();
    kin.iter()
        .filter_map(|(&id, person)| {
            let partner = person.partner?;
            (id < partner
                && kin
                    .get(&partner)
                    .is_some_and(|other| other.partner == Some(id)))
            .then_some([id, partner])
        })
        .collect()
}
