//! Deterministic capacity matching for one defence batch; no inventory creation.
//! Reads: weapon stacks, navigation, people and draft ownership. Writes: none.
use crate::runtime::jobs::{hunt, items::chebyshev};
use hecs::World;
use sim_core::colony::people::Drafted;
use sim_core::colony::{ReserveKey, Tool};
use sim_core::components::{Agent, BodyHealth, Position};
use sim_engine::SimResources;
use std::cmp::Reverse;
use std::collections::{BTreeMap, BTreeSet};

type Tile = (u32, u32);

/// Match the candidate and already selected unarmed defenders together. Existing
/// manual arming stops are fixed claims; automatic stops may be reassigned so a
/// flexible person's preferred stack cannot strand a constrained person.
pub(super) fn allocation(
    world: &World,
    res: &SimResources,
    candidate: u64,
) -> Option<BTreeMap<u64, Tile>> {
    let reach = crate::runtime::wildlife::rule(res)?.arm_reach;
    let hearth = crate::worldgen::colony::hearth(res)?;
    let mut people = Vec::new();
    for (_, (a, p, hp, tool, draft)) in world
        .query::<(
            &Agent,
            &Position,
            &BodyHealth,
            Option<&Tool>,
            Option<&Drafted>,
        )>()
        .iter()
    {
        let automatic = res.steward.defence.owned_ids.contains(&a.id) && draft.is_some();
        let armed = tool.is_some_and(|t| {
            t.item
                .kind
                .definition()
                .weapon
                .as_ref()
                .is_some_and(|w| w.draft_rank.is_some())
        });
        if (a.id == candidate || automatic) && !armed {
            people.push((a.id, (p.x, p.y), hp.hp.to_bits()));
        }
    }
    if !people.iter().any(|(id, _, _)| *id == candidate) {
        return None;
    }
    let participants: BTreeSet<_> = people.iter().map(|(id, _, _)| *id).collect();
    let mut fixed = BTreeMap::<Tile, u32>::new();
    for (_, (a, d)) in world.query::<(&Agent, &Drafted)>().iter() {
        if !participants.contains(&a.id) {
            if let Some(tile) = d.arming {
                *fixed.entry(tile).or_default() += 1;
            }
        }
    }
    let capacities: BTreeMap<_, _> = res
        .colony
        .ground_items
        .iter()
        .map(|(tile, stack)| {
            (
                *tile,
                u32::from(stack.count).saturating_sub(fixed.get(tile).copied().unwrap_or(0)),
            )
        })
        .collect();
    let kinds = hunt::draft_weapons();
    let mut choices = BTreeMap::new();
    let mut priority = Vec::new();
    for (id, pos, hp) in people {
        let region = res.nav.region_of(pos);
        let mut options = Vec::new();
        for kind in &kinds {
            let rank = kind.definition().weapon.as_ref()?.draft_rank?;
            for (tile, stack) in &res.colony.ground_items {
                if stack.kind == *kind
                    && capacities.get(tile).copied().unwrap_or(0) > 0
                    && chebyshev(pos, *tile) <= reach
                    && !res
                        .colony
                        .reservations
                        .contains_key(&ReserveKey::Stack(*tile))
                    && res.nav.reaches(region, *tile)
                {
                    options.push((Reverse(rank), chebyshev(pos, *tile), tile.1, tile.0));
                }
            }
        }
        options.sort_unstable();
        let best = options.first()?.0 .0;
        choices.insert(
            id,
            options
                .into_iter()
                .map(|(_, _, y, x)| (x, y))
                .collect::<Vec<_>>(),
        );
        // Higher-priority people choose last and retain the better weapon when
        // both assignments are feasible. Reach constraints still win over taste.
        priority.push((best, hp, Reverse(chebyshev(pos, hearth)), Reverse(id), id));
    }
    priority.sort_unstable();
    let mut assigned = BTreeMap::new();
    for (_, _, _, _, id) in priority {
        if !augment(
            id,
            &choices,
            &capacities,
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

/// Compare the whole proposed team after reassignment, rather than hiding a
/// previously selected person's downgrade behind the new candidate's weapon.
pub(super) fn team_priority(
    world: &World,
    res: &SimResources,
    candidate: u64,
    proposed: &BTreeMap<u64, Tile>,
    hearth: Tile,
) -> Vec<(Reverse<u16>, Reverse<u64>, u32, u64)> {
    let mut priority = Vec::new();
    for (_, (a, p, hp, tool, draft)) in world
        .query::<(
            &Agent,
            &Position,
            &BodyHealth,
            Option<&Tool>,
            Option<&Drafted>,
        )>()
        .iter()
    {
        if a.id != candidate && !res.steward.defence.owned_ids.contains(&a.id) {
            continue;
        }
        let held = tool.and_then(|tool| {
            tool.item
                .kind
                .definition()
                .weapon
                .as_ref()
                .and_then(|weapon| weapon.draft_rank)
        });
        let destination = proposed
            .get(&a.id)
            .copied()
            .or_else(|| draft.and_then(|draft| draft.arming));
        let rank = held
            .or_else(|| {
                destination
                    .and_then(|tile| res.colony.ground_items.get(&tile))
                    .and_then(|stack| {
                        stack
                            .kind
                            .definition()
                            .weapon
                            .as_ref()
                            .and_then(|weapon| weapon.draft_rank)
                    })
            })
            .unwrap_or(0);
        priority.push((
            Reverse(rank),
            Reverse(hp.hp.to_bits()),
            chebyshev((p.x, p.y), hearth),
            a.id,
        ));
    }
    priority.sort_unstable();
    priority
}
