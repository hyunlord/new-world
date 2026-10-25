//! Rank-constrained matching for a bounded defence batch.
//! Reads: reachable weapon choices, capacities, ranks and person priority.
//! Writes: no simulation state; returns a complete deterministic assignment.
use super::Tile;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

// Each trial locks the best remaining person/weapon rank only if a complete
// assignment still exists. This lexicographically optimizes the actual team's
// weapon/health/distance/id vector without enumerating combinations of stacks.
pub(super) fn rank_optimal(
    people: &[(u64, u64, u32)],
    choices: &BTreeMap<u64, Vec<Tile>>,
    capacities: &BTreeMap<Tile, u32>,
    ranks: &BTreeMap<Tile, u16>,
) -> Option<BTreeMap<u64, Tile>> {
    let mut trials = BTreeSet::new();
    for &(id, health, distance) in people {
        for tile in choices.get(&id)? {
            trials.insert((Reverse(*ranks.get(tile)?), Reverse(health), distance, id));
        }
    }
    let mut locked = BTreeMap::new();
    let mut assigned = match_with_ranks(choices, capacities, ranks, &locked)?;
    for (Reverse(rank), _, _, id) in trials {
        if locked.contains_key(&id) {
            continue;
        }
        locked.insert(id, rank);
        if let Some(feasible) = match_with_ranks(choices, capacities, ranks, &locked) {
            assigned = feasible;
            if locked.len() == people.len() {
                break;
            }
        } else {
            locked.remove(&id);
        }
    }
    (locked.len() == people.len()).then_some(assigned)
}

fn match_with_ranks(
    choices: &BTreeMap<u64, Vec<Tile>>,
    capacities: &BTreeMap<Tile, u32>,
    ranks: &BTreeMap<Tile, u16>,
    locked: &BTreeMap<u64, u16>,
) -> Option<BTreeMap<u64, Tile>> {
    let permitted: BTreeMap<_, _> = choices
        .iter()
        .map(|(id, tiles)| {
            (
                *id,
                tiles
                    .iter()
                    .copied()
                    .filter(|tile| {
                        locked
                            .get(id)
                            .is_none_or(|rank| ranks.get(tile) == Some(rank))
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();
    let mut assigned = BTreeMap::new();
    for id in permitted.keys() {
        if !augment(
            *id,
            &permitted,
            capacities,
            &mut assigned,
            &mut BTreeSet::new(),
        ) {
            return None;
        }
    }
    Some(assigned)
}

fn augment(
    id: u64,
    choices: &BTreeMap<u64, Vec<Tile>>,
    capacities: &BTreeMap<Tile, u32>,
    assigned: &mut BTreeMap<u64, Tile>,
    seen: &mut BTreeSet<u64>,
) -> bool {
    if !seen.insert(id) {
        return false;
    }
    let Some(options) = choices.get(&id) else {
        return false;
    };
    for tile in options {
        let occupants: Vec<_> = assigned
            .iter()
            .filter(|(_, at)| *at == tile)
            .map(|(person, _)| *person)
            .collect();
        if occupants.len() < capacities.get(tile).copied().unwrap_or(0) as usize {
            assigned.insert(id, *tile);
            return true;
        }
        for other in occupants {
            if augment(other, choices, capacities, assigned, seen) {
                assigned.insert(id, *tile);
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    type Score = Vec<(Reverse<u16>, Reverse<u64>, u32, u64)>;

    fn score(
        assignment: &BTreeMap<u64, Tile>,
        people: &[(u64, u64, u32)],
        ranks: &BTreeMap<Tile, u16>,
    ) -> Score {
        let mut score: Vec<_> = people
            .iter()
            .map(|(id, health, distance)| {
                (
                    Reverse(ranks[&assignment[id]]),
                    Reverse(*health),
                    *distance,
                    *id,
                )
            })
            .collect();
        score.sort_unstable();
        score
    }

    #[test]
    fn rank_constraints_match_exhaustive_small_graph_optimum() {
        let tiles = [(0, 0), (1, 0), (2, 0)];
        let capacities = tiles.into_iter().map(|tile| (tile, 1)).collect();
        for weapon_ranks in [[3, 1, 3], [3, 2, 1], [1, 1, 1]] {
            let ranks = tiles.into_iter().zip(weapon_ranks).collect();
            for people in [
                [(1, 100, 3), (2, 90, 2), (3, 80, 1)],
                [(1, 100, 3), (2, 100, 2), (3, 100, 2)],
            ] {
                for first in 1..8 {
                    for second in 1..8 {
                        for third in 1..8 {
                            let choices: BTreeMap<_, _> = [first, second, third]
                                .into_iter()
                                .enumerate()
                                .map(|(index, mask)| {
                                    (
                                        index as u64 + 1,
                                        tiles
                                            .iter()
                                            .enumerate()
                                            .filter(|(bit, _)| mask & (1 << *bit) != 0)
                                            .map(|(_, tile)| *tile)
                                            .collect::<Vec<_>>(),
                                    )
                                })
                                .collect();
                            // Independent exhaustive oracle, only in the test: all
                            // legal injective assignments of three physical weapons.
                            let mut expected: Option<Score> = None;
                            for a in &choices[&1] {
                                for b in &choices[&2] {
                                    for c in &choices[&3] {
                                        if a == b || a == c || b == c {
                                            continue;
                                        }
                                        let assignment =
                                            BTreeMap::from([(1, *a), (2, *b), (3, *c)]);
                                        let candidate = score(&assignment, &people, &ranks);
                                        if expected.as_ref().is_none_or(|best| candidate < *best) {
                                            expected = Some(candidate);
                                        }
                                    }
                                }
                            }
                            let result = rank_optimal(&people, &choices, &capacities, &ranks);
                            assert_eq!(
                                result
                                    .as_ref()
                                    .map(|assignment| score(assignment, &people, &ranks)),
                                expected,
                                "rank={weapon_ranks:?} reach={:?}",
                                [first, second, third]
                            );
                            let mut reordered = people;
                            reordered.reverse();
                            assert_eq!(
                                rank_optimal(&reordered, &choices, &capacities, &ranks),
                                result,
                                "person iteration order must not affect matching"
                            );
                        }
                    }
                }
            }
        }
    }
}
