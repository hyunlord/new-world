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
        let Some(rule) = resources.rules.society.shelter_planning else {
            return;
        };
        let mut shelters: Vec<_> = resources
            .building_owner
            .iter()
            .filter(|(p, _)| resources.building_kinds.get(p) == Some(&BuildingKind::Shelter))
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
            if resources.rules.society.family.is_some() {
                let capacity = rule.residents_per_shelter as usize;
                if capacity == 0 {
                    continue;
                }
                let groups = super::households::households(world, &members.into_iter().collect());
                let (mut shelter, mut used) = (0, 0);
                for group in groups {
                    if used > 0 && group.len() > capacity - used {
                        shelter += 1;
                        used = 0;
                    }
                    for id in group {
                        let Some(&bed) = beds.get(shelter) else {
                            break;
                        };
                        assigned.insert(id, bed);
                        used += 1;
                        if used == capacity {
                            shelter += 1;
                            used = 0;
                        }
                    }
                }
                continue;
            }
            for (index, id) in members.into_iter().enumerate() {
                let Some(bed_index) = index.checked_div(rule.residents_per_shelter as usize) else {
                    break;
                };
                let Some(&bed) = beds.get(bed_index) else {
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
        r.rules = WorldRules::game_v8();
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
