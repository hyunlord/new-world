//! Disabled physical command families reject before template metadata is allocated.
mod spine_support;
use sim_core::colony::{Command, RejectReason};
use sim_systems::commands::submit;
use spine_support::*;

#[test]
fn spine_build_rejects_disabled_physical_capabilities_without_allocation() {
    // Given a valid seed0 house anchor and known prerequisites, but one missing rule.
    for (disabled, template, station) in [
        ("space", "log_house", None),
        ("economy", "log_house", None),
        ("knowledge", "workshop_shed", Some("grinding_stone")),
    ] {
        let mut engine = scene();
        let anchor = anchor(&engine, template);
        let rule = engine
            .resources
            .rules
            .colony
            .as_mut()
            .expect("colony fixture");
        match disabled {
            "space" => rule.space = None,
            "economy" => rule.economy = None,
            "knowledge" => rule.knowledge = None,
            _ => unreachable!("closed test cases"),
        }
        let before = engine.resources.colony.buildings.clone();
        let physical = sim_systems::runtime::jobs::items::colony_sites(&engine.world).len();
        let occupied = engine.resources.colony.occupied.clone();
        assert_eq!(
            sim_systems::runtime::building_templates::commands::preview(
                &engine.world,
                &engine.resources,
                template,
                anchor,
                0,
                station
            ),
            Err(RejectReason::NotYet),
            "preview uses identical admission"
        );
        // When the queued player command is submitted with no inventory requirement.
        let result = submit(
            &mut engine,
            Command::BuildTemplate {
                id: template.to_owned(),
                anchor,
                rotation: 0,
                station: station.map(str::to_owned),
            },
        );
        // Then the same physical NotYet admission applies before allocating any work.
        assert_eq!(result.reason, Some(RejectReason::NotYet), "{disabled}");
        assert_eq!(engine.resources.colony.buildings, before, "{disabled}");
        assert_eq!(engine.resources.colony.occupied, occupied, "{disabled}");
        assert_eq!(
            sim_systems::runtime::jobs::items::colony_sites(&engine.world).len(),
            physical
        );
    }
}
