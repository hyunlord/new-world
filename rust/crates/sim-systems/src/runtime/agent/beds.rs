//! BedAssignmentSystem: assigns live settlement members to beds in owned shelters (and M1 huts), at 20:00 or when the shelters change.
//!
//! Priority: 145 · Interval: 1 tick · Gate: `WorldRules.agent.night_rest` and (`WorldRules.society.shelter_planning` or `WorldRules.colony`)
//! Reads: `Agent`, `Kin`, `Child`, `BodyHealth` (households); `SimResources.building_owner`, `building_kinds`, `settlements`, `rules.society.family`, `rules.colony`
//! Writes: `SimResources.bed_of`
//! Events: none
//!
//! Stable assignments of live settlement members to owned shelter beds.
use hecs::World;
use sim_core::components::{Agent, BuildingKind, SettlementId};
use sim_engine::{RuntimeSystem, SimResources};
use std::collections::{BTreeMap, BTreeSet};

/// Reassigns beds at 20:00 or when the set of owned shelters changes.
#[derive(Debug, Default)]
pub struct BedAssignmentSystem {
    last_shelters: Vec<(SettlementId, u32, u32)>,
}
impl RuntimeSystem for BedAssignmentSystem {
    fn name(&self) -> &str {
        "BedAssignmentSystem"
    }
    fn priority(&self) -> u32 {
        145
    }
    fn tick_interval(&self) -> u64 {
        1
    }
    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        if resources.rules.agent.night_rest.is_none() {
            return;
        }
        let planning = resources.rules.society.shelter_planning;
        // M1: a hut sleeps `hut_beds`; a shelter `residents_per_shelter`.
        let hut_beds = resources.rules.colony.map(|colony| colony.hut_beds);
        if planning.is_none() && hut_beds.is_none() {
            return;
        }
        let capacity_of = |tile: &(u32, u32)| match resources.building_kinds.get(tile) {
            Some(BuildingKind::Shelter) => {
                planning.map_or(0, |rule| rule.residents_per_shelter) as usize
            }
            Some(BuildingKind::Hut) => hut_beds.unwrap_or(0) as usize,
            _ => 0,
        };
        let mut shelters: Vec<_> = resources
            .building_owner
            .iter()
            .filter(|(p, _)| {
                matches!(
                    resources.building_kinds.get(p),
                    Some(BuildingKind::Shelter | BuildingKind::Hut)
                )
            })
            .map(|(&(x, y), &sid)| (sid, y, x))
            .collect();
        shelters.sort();
        let minute = resources
            .current_tick
            .saturating_add(resources.rules.clock_offset_ticks)
            .checked_rem(resources.ticks_per_day)
            .unwrap_or(0);
        if minute != 1200 && shelters == self.last_shelters {
            return;
        }
        let live: BTreeSet<_> = world
            .query::<&Agent>()
            .iter()
            .map(|(_, agent)| agent.id)
            .collect();
        let mut assigned = BTreeMap::new();
        let mut settlements: Vec<_> = resources.settlements.iter().collect();
        settlements.sort_by_key(|&(&sid, _)| sid);
        for (&sid, settlement) in settlements {
            let beds: Vec<_> = shelters
                .iter()
                .filter(|&&(owner, _, _)| owner == sid)
                .map(|&(_, y, x)| (x, y))
                .collect();
            let mut members: Vec<_> = settlement
                .member_agents
                .iter()
                .copied()
                .filter(|id| live.contains(id) && !assigned.contains_key(id))
                .collect();
            members.sort();
            let capacities: Vec<usize> = beds.iter().map(capacity_of).collect();
            if resources.rules.society.family.is_some() {
                if capacities.iter().all(|&c| c == 0) {
                    continue;
                }
                let groups = super::households::households(world, &members.into_iter().collect());
                let (mut shelter, mut used) = (0, 0);
                for group in groups {
                    let capacity = capacities.get(shelter).copied().unwrap_or(0);
                    if used > 0 && group.len() > capacity.saturating_sub(used) {
                        shelter += 1;
                        used = 0;
                    }
                    for id in group {
                        let Some(&bed) = beds.get(shelter) else {
                            break;
                        };
                        assigned.insert(id, bed);
                        used += 1;
                        if used >= capacities.get(shelter).copied().unwrap_or(0) {
                            shelter += 1;
                            used = 0;
                        }
                    }
                }
                continue;
            }
            let mut slots = beds
                .iter()
                .zip(&capacities)
                .flat_map(|(bed, &capacity)| std::iter::repeat_n(*bed, capacity));
            for id in members {
                let Some(bed) = slots.next() else {
                    break;
                };
                assigned.insert(id, bed);
            }
        }
        resources.bed_of = assigned;
        self.last_shelters = shelters;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::{
        components::Settlement,
        rules::{NightRestRule, WorldRules},
        MaterialRegistry,
    };
    fn fixture() -> (World, SimResources) {
        let mut world = World::new();
        let mut r = SimResources::new(32, 32, MaterialRegistry::new());
        r.rules = WorldRules::baseline(0);
        r.rules.agent.night_rest = Some(NightRestRule {
            start_minute: 1260,
            end_minute: 360,
            fatigue_threshold: 0.0,
            recovery_per_tick: 0.12,
            wake_preempt: 75.0,
        });
        r.rules
            .society
            .shelter_planning
            .as_mut()
            .expect("planning")
            .residents_per_shelter = 2;
        let mut settlement = Settlement::new_with_id(1, 0);
        for id in [4, 1, 3, 0, 2] {
            world.spawn((Agent { id },));
            settlement.add_member_agent(id);
        }
        settlement.add_member_agent(99);
        r.settlements.insert(1, settlement);
        for p in [(2, 5), (8, 4)] {
            r.building_owner.insert(p, 1);
            r.building_kinds.insert(p, BuildingKind::Shelter);
        }
        (world, r)
    }
    #[test]
    fn assigns_sorted_live_members_with_per_shelter_capacity() {
        let (mut world, mut r) = fixture();
        let mut system = BedAssignmentSystem::default();
        assert_eq!((system.priority(), system.tick_interval()), (145, 1));
        system.tick(&mut world, &mut r);
        assert_eq!(
            r.bed_of,
            BTreeMap::from([(0, (8, 4)), (1, (8, 4)), (2, (2, 5)), (3, (2, 5))])
        );
        assert!(!r.bed_of.contains_key(&4));
        assert!(!r.bed_of.contains_key(&99));
    }
    #[test]
    fn recomputes_on_shelter_change_and_at_twenty_hundred() {
        let (mut world, mut r) = fixture();
        let mut system = BedAssignmentSystem::default();
        system.tick(&mut world, &mut r);
        r.bed_of.clear();
        r.current_tick = 1;
        system.tick(&mut world, &mut r);
        assert!(r.bed_of.is_empty());
        r.rules.clock_offset_ticks = 480;
        r.current_tick = 720;
        system.tick(&mut world, &mut r);
        assert_eq!(r.bed_of.len(), 4);
        r.current_tick += 1;
        r.building_owner.remove(&(8, 4));
        system.tick(&mut world, &mut r);
        assert_eq!(r.bed_of, BTreeMap::from([(0, (2, 5)), (1, (2, 5))]));
    }
    #[test]
    fn disabled_rules_leave_assignments_untouched() {
        let (mut world, mut r) = fixture();
        r.rules.agent.night_rest = None;
        r.bed_of.insert(88, (1, 1));
        BedAssignmentSystem::default().tick(&mut world, &mut r);
        assert_eq!(r.bed_of, BTreeMap::from([(88, (1, 1))]));
    }
}
