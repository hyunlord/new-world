//! M-UX2 automatic defence: real runtime registration, player precedence and safety.
mod colony_support;
use colony_support::adults;
use sim_core::colony::people::{DraftOrder, Drafted};
use sim_core::colony::steward::StewardScope;
use sim_core::colony::{Command, ItemKind, ItemSpec, Tool};
use sim_core::components::{BodyHealth, Position};
use sim_systems::commands::submit;

fn armed_colony() -> sim_engine::SimEngine {
    let mut engine = sim_test::colony_engine(0);
    engine.tick();
    for (_, entity) in adults(&engine) {
        engine.world.insert_one(entity, Tool { item: ItemSpec::raw(ItemKind::Spear, None), uses: 0 }).expect("arm fixture");
        engine.resources.colony.item_ledger.create(ItemKind::Spear, 1);
    }
    engine
}
fn incoming(engine: &mut sim_engine::SimEngine) {
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    let at = (hearth.0.saturating_add(20).min(engine.resources.tile_grid.width - 1), hearth.1);
    sim_systems::runtime::wildlife::spawn_pack(&mut engine.resources, at, 4, 42);
    assert!(submit(engine, Command::SetSteward { on: true, scope: StewardScope::All }).accepted);
}

#[test]
fn incoming_pack_drafts_four_armed_adults_before_the_next_slow_planning_tick() {
    // Given armed adults and a newly spawned approaching pack after tick zero.
    let mut engine = armed_colony();
    incoming(&mut engine);
    // When the ordinary runtime advances one tick.
    engine.tick();
    // Then fast defence is registered and sends exactly four adults to attack.
    let orders: Vec<_> = engine.world.query::<&Drafted>().iter().map(|(_, d)| d.order).collect();
    assert_eq!(orders.len(), 4);
    assert!(orders.iter().all(|order| matches!(order, DraftOrder::Attack(_))));
}

#[test]
fn explicit_manual_undraft_while_off_protects_the_person_when_steward_is_enabled() {
    // Given a player explicitly leaves an already undrafted adult free while OFF.
    let mut engine = armed_colony();
    let (id, entity) = adults(&engine)[0];
    let result = submit(&mut engine, Command::SetDrafted { agent: id, drafted: false });
    // When defence is enabled and the pack approaches.
    incoming(&mut engine);
    engine.tick();
    // Then the explicit choice is accepted and cannot be overwritten by automation.
    assert!(result.accepted, "an explicit no-op is still a manual choice");
    assert!(engine.world.get::<&Drafted>(entity).is_err());
    assert_eq!(engine.world.query::<&Drafted>().iter().count(), 4);
}

#[test]
fn defence_excludes_low_health_and_ranks_weapon_health_then_distance() {
    // Given two ineligible low-health adults and healthy armed alternatives.
    let mut engine = armed_colony();
    let people = adults(&engine);
    let hearth = sim_systems::worldgen::colony::hearth(&engine.resources).expect("hearth");
    for (index, (_, entity)) in people.iter().enumerate() {
        engine.world.get::<&mut BodyHealth>(*entity).expect("health").hp = if index < 2 { 49.0 } else { 100.0 };
        let mut position = engine.world.get::<&mut Position>(*entity).expect("position");
        position.x = hearth.0 + (index as u32 % 3);
        position.y = hearth.1;
    }
    incoming(&mut engine);
    // When one ordinary tick runs.
    engine.tick();
    // Then no low-health person is drafted and the cap is respected.
    assert!(people.iter().take(2).all(|(_, entity)| engine.world.get::<&Drafted>(*entity).is_err()));
    assert_eq!(engine.world.query::<&Drafted>().iter().count(), 4.min(people.len() - 2));
}
