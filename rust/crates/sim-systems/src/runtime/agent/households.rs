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

/// Who leaves on a split: whole households (their ranked members) seeded by
/// the ranked adults, filled in rank order up to `k`, topped up to
/// `group_min` with single ranked people.
///
/// Pass 18 (`game_v23`): with `elder_partners` (the settlement's mutual
/// couples) a household also keeps an unranked member (an elder) who is the
/// mutual partner of one of its ranked members; households without a ranked
/// member stay behind whole. Kept elders count toward every size limit but
/// never toward `min_adults`. Without it this is Pass 17's selection.
pub(crate) fn select_departure(
    ranked: &[(u64, AgentId, bool)],
    households: &[Vec<AgentId>],
    rule: &FissionRule,
    k: u32,
    elder_partners: Option<&[[AgentId; 2]]>,
) -> Option<BTreeSet<AgentId>> {
    let rank: BTreeMap<_, _> = ranked
        .iter()
        .enumerate()
        .map(|(i, row)| (row.1, (i, row.2)))
        .collect();
    let partner_of: BTreeMap<AgentId, AgentId> = elder_partners
        .into_iter()
        .flatten()
        .flat_map(|&[a, b]| [(a, b), (b, a)])
        .collect();
    let groups: Vec<Vec<_>> = households
        .iter()
        .map(|group| {
            group
                .iter()
                .copied()
                .filter(|id| {
                    rank.contains_key(id)
                        || partner_of.get(id).is_some_and(|partner| {
                            rank.contains_key(partner) && group.contains(partner)
                        })
                })
                .collect()
        })
        .filter(|group: &Vec<_>| group.iter().any(|id| rank.contains_key(id)))
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

#[cfg(test)]
mod tests {
    use super::select_departure;
    use sim_core::components::AgentId;
    use sim_core::rules::{FissionRule, WorldRules};
    use std::collections::BTreeSet;

    fn rule() -> FissionRule {
        WorldRules::game_v22(0).society.fission.expect("v22 splits")
    }

    /// Ranked rows (rank, id, adult) in order; ids >= 100 are children.
    fn ranked(ids: &[AgentId]) -> Vec<(u64, AgentId, bool)> {
        ids.iter()
            .enumerate()
            .map(|(rank, &id)| (rank as u64, id, id < 100))
            .collect()
    }

    fn set(ids: &[AgentId]) -> BTreeSet<AgentId> {
        ids.iter().copied().collect()
    }

    #[test]
    fn an_elder_partner_leaves_with_the_departing_adult_only_when_following() {
        // Adults 1..=6; 1 is partnered with elder 50; 2 with child 100.
        let households = vec![vec![1, 50], vec![2, 100], vec![3], vec![4], vec![5], vec![6]];
        let rows = ranked(&[1, 2, 3, 4, 5, 6, 100]);
        let couples = [[1, 50]];
        let old = select_departure(&rows, &households, &rule(), 7, None).expect("p17 group");
        // Seed 1, 2+100, 3, 4 (four adults), fill 5 and 6 up to k = 7.
        assert_eq!(old, set(&[1, 2, 3, 4, 5, 6, 100]));
        // The elder takes a slot: seed 1+50, 2+100, 3, 4; fill 5; 6 no longer fits.
        let new = select_departure(&rows, &households, &rule(), 7, Some(&couples)).expect("p18 group");
        assert_eq!(new, set(&[1, 50, 2, 100, 3, 4, 5]));
    }

    #[test]
    fn elder_only_households_widowed_elders_and_elder_couples_stay() {
        // 60 lone elder; 61-62 elder couple; widowed elder 63 with child 101.
        let households = vec![
            vec![1],
            vec![2],
            vec![3],
            vec![4],
            vec![5],
            vec![6],
            vec![60],
            vec![61, 62],
            vec![63, 101],
        ];
        let rows = ranked(&[101, 1, 2, 3, 4, 5, 6]);
        let couples = [[61, 62]];
        let group = select_departure(&rows, &households, &rule(), 7, Some(&couples)).expect("group");
        assert!(group.iter().all(|id| *id < 60 || *id == 101), "{group:?}");
        assert_eq!(group, select_departure(&rows, &households, &rule(), 7, None).expect("p17"));
    }

    #[test]
    fn kept_elders_count_toward_group_max_but_not_min_adults() {
        let rule = rule();
        // Seven adult-elder couples (adult i, elder 50 + i) and two lone adults.
        let mut households: Vec<Vec<AgentId>> = (1..=7).map(|i| vec![i, 50 + i]).collect();
        households.extend([vec![8], vec![9]]);
        let couples: Vec<[AgentId; 2]> = (1..=7).map(|i| [i, 50 + i]).collect();
        let rows = ranked(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
        for k in rule.group_min..=rule.group_max {
            let group = select_departure(&rows, &households, &rule, k, Some(&couples)).expect("group");
            assert!(group.len() <= rule.group_max as usize, "k {k}: {group:?}");
            assert!(group.len() <= (k as usize).max(2 * rule.min_adults as usize), "k {k}: {group:?}");
            let adults = group.iter().filter(|id| **id < 50).count();
            assert!(adults >= rule.min_adults as usize, "k {k}: {group:?}");
            for i in 1..=7 {
                assert_eq!(group.contains(&i), group.contains(&(50 + i)), "k {k}: couple {i} split");
            }
        }
    }
}
