//! Provenance distinguishes a player command from zone-directed steward work.
mod spine_support;
use sim_core::colony::building_site::BuildCause;
use sim_core::colony::steward::{rules, Action, StewardScope};
use sim_core::colony::Command;
use spine_support::*;

#[test]
fn spine_direct_inside_zone_stays_command_and_steward_retains_actual_origin() {
    let mut engine = scene();
    let anchor = anchor(&engine, "log_house");
    let zone = engine.resources.colony.next_zone_id;
    command(
        &mut engine,
        Command::AddHousingZone {
            x0: anchor.0,
            y0: anchor.1,
            x1: anchor.0 + 3,
            y1: anchor.1 + 3,
        },
    );
    let direct = engine.resources.colony.buildings.next_site_id;
    // No materials are supplied: a normal player construction order can queue.
    command(
        &mut engine,
        Command::BuildTemplate {
            id: "log_house".to_owned(),
            anchor,
            rotation: 0,
            station: None,
        },
    );
    assert_eq!(
        engine.resources.colony.buildings.sites[&direct].cause,
        BuildCause::Command
    );
    cancel_remaining(&mut engine, direct);
    let rule = rules()
        .expect("validated steward rules")
        .rules
        .iter()
        .find(|rule| {
            rule.scope == StewardScope::Housing && matches!(rule.action, Action::House { .. })
        })
        .expect("real housing rule")
        .id;
    let zoned = engine.resources.colony.buildings.next_site_id;
    command(
        &mut engine,
        Command::Steward {
            rule,
            command: Box::new(Command::BuildTemplate {
                id: "log_house".to_owned(),
                anchor,
                rotation: 0,
                station: None,
            }),
        },
    );
    assert_eq!(
        engine.resources.colony.buildings.sites[&zoned].cause,
        BuildCause::Zone(zone)
    );
    cancel_remaining(&mut engine, zoned);
    command(
        &mut engine,
        Command::RemoveZoneArea {
            x0: anchor.0,
            y0: anchor.1,
            x1: anchor.0 + 3,
            y1: anchor.1 + 3,
        },
    );
    let outside = engine.resources.colony.buildings.next_site_id;
    command(
        &mut engine,
        Command::Steward {
            rule,
            command: Box::new(Command::BuildTemplate {
                id: "log_house".to_owned(),
                anchor,
                rotation: 0,
                station: None,
            }),
        },
    );
    assert_eq!(
        engine.resources.colony.buildings.sites[&outside].cause,
        BuildCause::Steward(u32::from(rule))
    );
    sim_systems::runtime::jobs::invariants::check(&engine.world, &engine.resources)
        .expect("queued/cancelled material ledger");
}
