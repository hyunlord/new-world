use hecs::World;
use sim_core::chronicle::DepartCause;
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
///
/// Pass 19: the key set of [`select_departure_explained`] (same signature and
/// result as before).
#[cfg_attr(
    not(test),
    allow(dead_code, reason = "production calls select_departure_explained; the old entry point stays for its tests")
)]
pub(crate) fn select_departure(
    ranked: &[(u64, AgentId, bool)],
    households: &[Vec<AgentId>],
    rule: &FissionRule,
    k: u32,
    elder_partners: Option<&[[AgentId; 2]]>,
) -> Option<BTreeSet<AgentId>> {
    select_departure_explained(ranked, households, rule, k, elder_partners)
        .map(|chosen| chosen.into_keys().collect())
}

/// [`select_departure`] with the phase that first chose each member (Pass 19
/// chronicle): the seed phase's iterated ranked adult is [`DepartCause::Seed`];
/// every other ranked member of a household the seed or the fill added is
/// [`DepartCause::Household`]; an unranked member of such a household (the
/// kept elder partner) is [`DepartCause::ElderPartner`]; a single added by the
/// `group_min` top-up is [`DepartCause::TopUp`]. A member keeps the label of
/// its first insertion. Who is chosen is exactly Pass 18's selection.
pub(crate) fn select_departure_explained(
    ranked: &[(u64, AgentId, bool)],
    households: &[Vec<AgentId>],
    rule: &FissionRule,
    k: u32,
    elder_partners: Option<&[[AgentId; 2]]>,
) -> Option<BTreeMap<AgentId, DepartCause>> {
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
    // A household's members: ranked ones are `Household`, unranked ones are
    // kept elder partners.
    let household_cause = |id: &AgentId| {
        if rank.contains_key(id) {
            DepartCause::Household
        } else {
            DepartCause::ElderPartner
        }
    };
    let mut chosen: BTreeMap<AgentId, DepartCause> = BTreeMap::new();
    for (_, id, _) in ranked.iter().filter(|row| row.2) {
        if chosen
            .keys()
            .filter(|id| rank.get(id).is_some_and(|row| row.1))
            .count()
            >= rule.min_adults as usize
        {
            break;
        }
        let Some(group) = group_of.get(id).map(|index| &groups[*index]) else {
            continue;
        };
        if !chosen.contains_key(id) && chosen.len() + group.len() <= rule.group_max as usize {
            chosen.entry(*id).or_insert(DepartCause::Seed);
            for member in group {
                chosen.entry(*member).or_insert_with(|| household_cause(member));
            }
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
        if !group.iter().any(|id| chosen.contains_key(id)) && chosen.len() + group.len() <= k as usize {
            for member in group {
                chosen.entry(*member).or_insert_with(|| household_cause(member));
            }
        }
    }
    if chosen.len() < rule.group_min as usize {
        for (_, id, _) in ranked {
            if chosen.len() >= rule.group_min as usize {
                break;
            }
            chosen.entry(*id).or_insert(DepartCause::TopUp);
        }
    }
    (chosen.len() >= rule.group_min as usize
        && chosen
            .keys()
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
    use sim_core::chronicle::DepartCause;
    use sim_core::components::AgentId;
    use sim_core::rules::{FissionRule, WorldRules};
    use std::collections::{BTreeMap, BTreeSet};

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
        let households = vec![
            vec![1, 50],
            vec![2, 100],
            vec![3],
            vec![4],
            vec![5],
            vec![6],
        ];
        let rows = ranked(&[1, 2, 3, 4, 5, 6, 100]);
        let couples = [[1, 50]];
        let old = select_departure(&rows, &households, &rule(), 7, None).expect("p17 group");
        // Seed 1, 2+100, 3, 4 (four adults), fill 5 and 6 up to k = 7.
        assert_eq!(old, set(&[1, 2, 3, 4, 5, 6, 100]));
        // The elder takes a slot: seed 1+50, 2+100, 3, 4; fill 5; 6 no longer fits.
        let new =
            select_departure(&rows, &households, &rule(), 7, Some(&couples)).expect("p18 group");
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
        let group =
            select_departure(&rows, &households, &rule(), 7, Some(&couples)).expect("group");
        assert!(group.iter().all(|id| *id < 60 || *id == 101), "{group:?}");
        assert_eq!(
            group,
            select_departure(&rows, &households, &rule(), 7, None).expect("p17")
        );
    }

    #[test]
    fn only_the_group_min_top_up_takes_an_adult_without_their_elder_partner() {
        // Adult 1's household (elder partner 50 and thirteen children) is too
        // big for group_max, so neither the seed nor the fill takes it; the
        // top-up then takes adult 1 alone - the registered G3 exception.
        let mut big = vec![1, 50];
        big.extend(100..=112);
        let households = vec![big, vec![2], vec![3], vec![4]];
        let mut order = vec![1, 2, 3, 4];
        order.extend(100..=112);
        let rows = ranked(&order);
        let rule = rule();
        let group =
            select_departure(&rows, &households, &rule, 7, Some(&[[1, 50]])).expect("group");
        assert_eq!(group, set(&[1, 2, 3, 4, 100, 101]));
        assert_eq!(group.len(), rule.group_min as usize);
        assert!(!group.contains(&50));
    }

    #[test]
    fn no_couples_select_exactly_as_pass17() {
        let households = vec![
            vec![1, 50],
            vec![2, 100],
            vec![3, 51],
            vec![4],
            vec![5],
            vec![6],
            vec![60, 61],
        ];
        let rows = ranked(&[3, 1, 100, 2, 4, 5, 6]);
        for k in 6..=14 {
            assert_eq!(
                select_departure(&rows, &households, &rule(), k, Some(&[])),
                select_departure(&rows, &households, &rule(), k, None),
                "k {k}"
            );
        }
    }

    #[test]
    fn the_seed_skips_a_household_its_elder_pushes_over_group_max() {
        // Adults 2, 3, 4 seed first; adult 1's household (elder partner 50 and
        // ten children) then fits exactly without the elder (3 + 11 = 14 =
        // group_max) but not with it, so the seed and the fill skip it and
        // adults 5, 6, 7 leave instead (no top-up is needed).
        let mut big = vec![1, 50];
        big.extend(100..=109);
        let households = vec![vec![2], vec![3], vec![4], big, vec![5], vec![6], vec![7]];
        let mut order = vec![2, 3, 4, 1, 5, 6, 7];
        order.extend(100..=109);
        let rows = ranked(&order);
        let rule = rule();
        let old = select_departure(&rows, &households, &rule, 14, None).expect("p17 group");
        let mut expected = vec![1, 2, 3, 4];
        expected.extend(100..=109);
        assert_eq!(old, set(&expected));
        let new =
            select_departure(&rows, &households, &rule, 14, Some(&[[1, 50]])).expect("p18 group");
        assert_eq!(new, set(&[2, 3, 4, 5, 6, 7]));
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
            let group =
                select_departure(&rows, &households, &rule, k, Some(&couples)).expect("group");
            assert!(group.len() <= rule.group_max as usize, "k {k}: {group:?}");
            assert!(
                group.len() <= (k as usize).max(2 * rule.min_adults as usize),
                "k {k}: {group:?}"
            );
            let adults = group.iter().filter(|id| **id < 50).count();
            assert!(adults >= rule.min_adults as usize, "k {k}: {group:?}");
            for i in 1..=7 {
                assert_eq!(
                    group.contains(&i),
                    group.contains(&(50 + i)),
                    "k {k}: couple {i} split"
                );
            }
        }
    }

    // Pass 19 (6.3): a frozen copy of Pass 18's `select_departure`, kept
    // verbatim so the explained version can be proven to choose the same set.
    /// Who leaves on a split: whole households (their ranked members) seeded by
    /// the ranked adults, filled in rank order up to `k`, topped up to
    /// `group_min` with single ranked people.
    ///
    /// Pass 18 (`game_v23`): with `elder_partners` (the settlement's mutual
    /// couples) a household also keeps an unranked member (an elder) who is the
    /// mutual partner of one of its ranked members; households without a ranked
    /// member stay behind whole. Kept elders count toward every size limit but
    /// never toward `min_adults`. Without it this is Pass 17's selection.
    fn select_departure_p18(
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

    /// Deterministic xorshift64* stream for the property test.
    struct Stream(u64);

    impl Stream {
        fn next(&mut self) -> u64 {
            self.0 ^= self.0 >> 12;
            self.0 ^= self.0 << 25;
            self.0 ^= self.0 >> 27;
            self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
        }

        fn below(&mut self, n: u64) -> u64 {
            self.next() % n.max(1)
        }
    }

    /// One random settlement: adults (ids < 100), elders (200..), children
    /// (100..200) in households of couples, single parents and singles.
    /// (ranked rows, households, couples) of one random settlement.
    type Case = (Vec<(u64, AgentId, bool)>, Vec<Vec<AgentId>>, Vec<[AgentId; 2]>);

    fn random_case(stream: &mut Stream) -> Case {
        let adults: Vec<AgentId> = (1..=2 + stream.below(24)).collect();
        let elders: Vec<AgentId> = (200..200 + stream.below(8)).collect();
        let mut pool: Vec<AgentId> = adults.iter().chain(elders.iter()).copied().collect();
        // Shuffle the pool.
        for i in (1..pool.len()).rev() {
            let j = stream.below(i as u64 + 1) as usize;
            pool.swap(i, j);
        }
        let mut households: Vec<Vec<AgentId>> = Vec::new();
        let mut couples: Vec<[AgentId; 2]> = Vec::new();
        let mut next_child: AgentId = 100;
        while let Some(first) = pool.pop() {
            let mut household = vec![first];
            if stream.below(2) == 0 {
                if let Some(second) = pool.pop() {
                    household.push(second);
                    couples.push([first.min(second), first.max(second)]);
                }
            }
            for _ in 0..stream.below(4) {
                if stream.below(3) == 0 && next_child < 200 {
                    household.push(next_child);
                    next_child += 1;
                }
            }
            household.sort_unstable();
            households.push(household);
        }
        households.sort_by_key(|group| group[0]);
        let mut ranked_ids: Vec<AgentId> = adults.iter().copied().chain(100..next_child).collect();
        for i in (1..ranked_ids.len()).rev() {
            let j = stream.below(i as u64 + 1) as usize;
            ranked_ids.swap(i, j);
        }
        let rows = ranked_ids
            .iter()
            .enumerate()
            .map(|(rank, &id)| (rank as u64, id, id < 100))
            .collect();
        (rows, households, couples)
    }

    #[test]
    fn explained_selection_chooses_exactly_pass18s_set() {
        let mut stream = Stream(0x5EED_0019_0000_0001);
        let base = rule();
        let mut chosen = 0;
        for _ in 0..2_000 {
            let (rows, households, couples) = random_case(&mut stream);
            let mut rule = base;
            rule.group_min = 2 + stream.below(6) as u32;
            rule.group_max = rule.group_min + stream.below(12) as u32;
            rule.min_adults = stream.below(6) as u32;
            let k = rule.group_min + stream.below(u64::from(rule.group_max - rule.group_min) + 1) as u32;
            let partners = match stream.below(3) {
                0 => None,
                1 => Some(&[][..]),
                _ => Some(couples.as_slice()),
            };
            let frozen = select_departure_p18(&rows, &households, &rule, k, partners);
            let explained = super::select_departure_explained(&rows, &households, &rule, k, partners);
            assert_eq!(
                explained.as_ref().map(|m| m.keys().copied().collect::<BTreeSet<_>>()),
                frozen,
                "rows {rows:?} households {households:?} k {k} rule {rule:?}"
            );
            assert_eq!(select_departure(&rows, &households, &rule, k, partners), frozen);
            if let Some(map) = explained {
                chosen += 1;
                for (id, cause) in map {
                    let ranked = rows.iter().any(|row| row.1 == id);
                    match cause {
                        DepartCause::ElderPartner => assert!(!ranked && partners.is_some()),
                        DepartCause::Seed => assert!(id < 100),
                        DepartCause::Household | DepartCause::TopUp => assert!(ranked),
                        DepartCause::Ranked => panic!("the family path never labels Ranked"),
                    }
                }
            }
        }
        assert!(chosen > 500, "the generator must reach real groups: {chosen}");
    }

    #[test]
    fn explained_labels_each_phase() {
        // Adults 1..=6; 1 partnered with elder 50; 2 with child 100.
        let households = vec![vec![1, 50], vec![2, 100], vec![3], vec![4], vec![5], vec![6]];
        let rows = ranked(&[1, 2, 3, 4, 5, 6, 100]);
        let couples = [[1, 50]];
        let map = super::select_departure_explained(&rows, &households, &rule(), 7, Some(&couples))
            .expect("p18 group");
        // Seed 1 (+ elder 50), 2 (+ child 100), 3, 4; the fill adds 5.
        let expect = [
            (1, DepartCause::Seed),
            (2, DepartCause::Seed),
            (3, DepartCause::Seed),
            (4, DepartCause::Seed),
            (5, DepartCause::Household),
            (50, DepartCause::ElderPartner),
            (100, DepartCause::Household),
        ];
        assert_eq!(map.into_iter().collect::<Vec<_>>(), expect.to_vec());
        // Adult 1's household is too big for group_max, so the top-up takes 1.
        let mut big = vec![1, 50];
        big.extend(100..=112);
        let households = vec![big, vec![2], vec![3], vec![4]];
        let mut order = vec![1, 2, 3, 4];
        order.extend(100..=112);
        let rows = ranked(&order);
        let map = super::select_departure_explained(&rows, &households, &rule(), 7, Some(&[[1, 50]]))
            .expect("topped up");
        let expect = [
            (1, DepartCause::TopUp),
            (2, DepartCause::Seed),
            (3, DepartCause::Seed),
            (4, DepartCause::Seed),
            (100, DepartCause::TopUp),
            (101, DepartCause::TopUp),
        ];
        assert_eq!(map.into_iter().collect::<Vec<_>>(), expect.to_vec());
    }
}
