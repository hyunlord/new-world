//! Shared housing accounting for reporting, planning, and birth gates.

use sim_core::components::{BuildingKind, SettlementId, SETTLEMENT_PROXIMITY_RADIUS};
use sim_core::rules::{HousingCount, ShelterPlanningRule};
use sim_engine::SimResources;

/// Count completed shelters under the selected settlement accounting policy.
pub fn shelter_count(
    resources: &SimResources,
    sid: SettlementId,
    rule: &ShelterPlanningRule,
) -> u32 {
    match rule.housing {
        HousingCount::NearFormation => {
            let Some(settlement) = resources.settlements.get(&sid) else {
                return 0;
            };
            let center = settlement.formation_tile;
            resources
                .building_kinds
                .iter()
                .filter(|(tile, kind)| {
                    **kind == BuildingKind::Shelter
                        && tile.0.abs_diff(center.0).max(tile.1.abs_diff(center.1))
                            <= SETTLEMENT_PROXIMITY_RADIUS + 3
                })
                .count() as u32
        }
        HousingCount::Owned => resources
            .building_owner
            .iter()
            .filter(|(tile, owner)| {
                **owner == sid && resources.building_kinds.get(tile) == Some(&BuildingKind::Shelter)
            })
            .count() as u32,
    }
}

/// Number of residents housed by this settlement's completed shelters.
pub fn housing_capacity(
    resources: &SimResources,
    sid: SettlementId,
    rule: &ShelterPlanningRule,
) -> u32 {
    shelter_count(resources, sid, rule).saturating_mul(rule.residents_per_shelter)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::{components::Settlement, material::MaterialRegistry, rules::WorldRules};

    #[test]
    fn overlapping_radius_and_owned_capacity_are_distinct() {
        let mut resources = SimResources::new(32, 32, MaterialRegistry::new());
        for id in [1, 2] {
            let mut settlement = Settlement::new_with_id(id, 0);
            settlement.formation_tile = (10, 10);
            resources.settlements.insert(id, settlement);
        }
        for tile in [(9, 9), (10, 9), (11, 9), (12, 9)] {
            resources.building_kinds.insert(tile, BuildingKind::Shelter);
            resources.building_owner.insert(tile, 1);
        }
        let mut rule = WorldRules::baseline(0)
            .society
            .shelter_planning
            .expect("planning");
        rule.residents_per_shelter = 8;
        // Count every shelter near the formation first (the baseline counts owned ones).
        rule.housing = HousingCount::NearFormation;
        assert_eq!(housing_capacity(&resources, 1, &rule), 32);
        assert_eq!(housing_capacity(&resources, 2, &rule), 32);
        rule.housing = HousingCount::Owned;
        assert_eq!(housing_capacity(&resources, 1, &rule), 32);
        assert_eq!(housing_capacity(&resources, 2, &rule), 0);
    }
}
