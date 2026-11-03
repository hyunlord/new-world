//! Stage B construction identity: retain historical projection and layered completion semantics.
mod colony_support;

use colony_support::job_world_m2;
use sim_core::colony::materials::{FLINT, OAK};
use sim_core::colony::{BlueprintKind, ColonySite, Command, ItemKind, ItemSpec, MaterialNeeds};
use sim_core::rules::ColonyRule;
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::space::{complete_site, deconstruct};

#[test]
fn historical_stone_wall_with_flint_still_returns_logs() {
    let mut engine = job_world_m2(20, 20);
    engine.resources.rules.colony = Some(ColonyRule::M3);
    assert!(
        submit(
            &mut engine,
            Command::PlaceBlueprint {
                kind: BlueprintKind::StoneWall,
                x: 8,
                y: 8
            }
        )
        .accepted
    );
    let site = engine
        .world
        .query::<&ColonySite>()
        .iter()
        .find(|(_, site)| site.top_left == (8, 8))
        .map(|(entity, _)| entity)
        .expect("placed site");
    engine
        .world
        .get::<&mut MaterialNeeds>(site)
        .expect("needs")
        .deliver_spec(ItemSpec::raw(ItemKind::Stone, Some(FLINT)), 3);
    complete_site(&mut engine.world, &mut engine.resources, site);
    assert!(engine.resources.content_state.completed.is_empty());
    assert!(deconstruct(&mut engine.resources, (8, 8)).is_some());
    let stack = engine
        .resources
        .colony
        .ground_items
        .get(&(8, 8))
        .expect("returned material");
    assert_eq!(stack.kind, ItemKind::Log);
    assert_eq!(stack.count, 1);
    assert_eq!(stack.material, Some(FLINT));
}

#[test]
fn historical_bed_removal_preserves_floor_and_empty_identity_map() {
    let mut engine = job_world_m2(20, 20);
    engine.resources.rules.colony = Some(ColonyRule::M3);
    for kind in [BlueprintKind::PackedFloor, BlueprintKind::Bedroll] {
        assert!(submit(&mut engine, Command::PlaceBlueprint { kind, x: 8, y: 8 }).accepted);
        let site = engine
            .world
            .query::<&ColonySite>()
            .iter()
            .find(|(_, site)| site.top_left == (8, 8))
            .map(|(entity, _)| entity)
            .expect("placed site");
        engine
            .world
            .get::<&mut MaterialNeeds>(site)
            .expect("needs")
            .deliver_spec(ItemSpec::raw(ItemKind::Log, Some(OAK)), 1);
        complete_site(&mut engine.world, &mut engine.resources, site);
    }
    let index = engine.resources.tile_grid.idx(8, 8);
    let floor = engine.resources.tile_grid.floor_material[index];
    assert!(floor.is_some());
    assert!(deconstruct(&mut engine.resources, (8, 8)).is_some());
    assert!(!engine.resources.colony.beds.contains_key(&(8, 8)));
    assert_eq!(engine.resources.tile_grid.floor_material[index], floor);
    assert!(engine.resources.content_state.completed.is_empty());
}

#[test]
fn novel_same_art_layers_survive_save_and_upper_layer_demolition() {
    const CHILD: &str = "WORLDSIM_BLUEPRINT_LAYER_TEST_CHILD";
    if std::env::var_os(CHILD).is_none() {
        let status = std::process::Command::new(std::env::current_exe().expect("test executable"))
            .args([
                "--exact",
                "novel_same_art_layers_survive_save_and_upper_layer_demolition",
                "--nocapture",
            ])
            .env(CHILD, "1")
            .status()
            .expect("fresh catalogue subprocess");
        assert!(status.success());
        return;
    }
    use sim_core::colony::content::{bootstrap, BootSelection, NamedSource, RawSources};
    use sim_core::colony::content_state::CompletedLayer;
    let record = |code: u32,
                  id: &str,
                  completion: &str,
                  placement: &str,
                  material: &str,
                  style: &str| {
        let needs = if completion == "Bed" {
            r#"[(item:"log",count:4,count_rule:None)]"#
        } else {
            "[]"
        };
        format!(
            r#"(
            identity:(code:{code},id:"{id}",name_key:"TEST_{id}",serde_name:"{id}",serde_aliases:[],resolver_aliases:[]),
            placeable:true,menu_order:-1,shortcut_slot:-1,footprint:(1,1),footprint_rule:None,
            needs:{needs},work:1,work_rule:None,placement:{placement},completion:{completion},workshop_tags:[],
            requires_economy:false,requires_knowledge:false,requires_space:true,planned_nav:Unchanged,
            material:Fixed(material:"{material}"),style_id:"{style}",completion_policy:Authoritative,initial_bill:None
        )"#
        )
    };
    let text = format!(
        "(schema_version:1,roles:{{}},material_roles:{{}},historical_stone_wall_materials:[],blueprints:[{},{}])",
        record(65537, "test_same_art_floor", "Floor", "Floor", "packed_earth", "PackedFloor"),
        record(u32::MAX, "test_same_art_bed", "Bed", "Furnishing", "oak", "Bedroll"),
    );
    bootstrap(BootSelection::Explicit {
        request_id: "construction-layer-regression".into(),
        sources: RawSources {
            sources: vec![NamedSource {
                name: "blueprints.ron".into(),
                bytes: text.into_bytes(),
            }],
        },
    })
    .expect("install extension before world");
    let floor = BlueprintKind::checked(65537).expect("extension floor");
    let bed = BlueprintKind::checked(u32::MAX).expect("full-width extension bed");
    let mut engine = job_world_m2(20, 20);
    engine.resources.rules.colony = Some(ColonyRule::M3);
    for kind in [floor, bed] {
        assert!(submit(&mut engine, Command::PlaceBlueprint { kind, x: 8, y: 8 }).accepted);
        let site = engine
            .world
            .query::<&ColonySite>()
            .iter()
            .find(|(_, site)| site.top_left == (8, 8))
            .map(|(entity, _)| entity)
            .expect("placed extension site");
        engine
            .world
            .get::<&mut MaterialNeeds>(site)
            .expect("extension needs")
            .deliver_spec(ItemSpec::raw(ItemKind::Log, Some(OAK)), 4);
        complete_site(&mut engine.world, &mut engine.resources, site);
    }
    let saved = sim_systems::save::save(&mut engine).expect("save both layers");
    let mut engine =
        sim_systems::save::load_with(&saved, register_fixture_systems).expect("load both layers");
    let key = ((8, 8), CompletedLayer::Floor);
    let original = engine
        .resources
        .content_state
        .completed
        .get(&key)
        .expect("floor identity")
        .clone();
    assert_eq!(original.blueprint, floor);
    assert_eq!(original.style_id, "PackedFloor");
    assert_eq!(
        engine.resources.content_state.completed[&((8, 8), CompletedLayer::Bed)].blueprint,
        bed
    );
    assert_eq!(deconstruct(&mut engine.resources, (8, 8)), Some(bed));
    assert_eq!(engine.resources.colony.ground_items[&(8, 8)].count, 2);
    assert_eq!(
        engine.resources.colony.ground_items[&(8, 8)].kind,
        ItemKind::Log
    );
    assert!(!engine
        .resources
        .content_state
        .completed
        .contains_key(&((8, 8), CompletedLayer::Bed)));
    assert_eq!(
        engine.resources.content_state.completed.get(&key),
        Some(&original)
    );
    let saved = sim_systems::save::save(&mut engine).expect("save remaining floor");
    let mut engine = sim_systems::save::load_with(&saved, register_fixture_systems)
        .expect("load remaining floor");
    assert_eq!(
        engine.resources.content_state.completed.get(&key),
        Some(&original)
    );
    assert_eq!(
        sim_systems::runtime::jobs::space::built_at(&engine.resources, (8, 8)),
        Some(floor)
    );
    let index = engine.resources.tile_grid.idx(8, 8);
    engine.resources.tile_grid.terrain_type[index] = sim_core::TerrainType::Hill;
    assert!(sim_systems::runtime::jobs::space::dig(
        &mut engine.resources,
        (8, 8)
    ));
    assert!(!engine.resources.content_state.completed.contains_key(&key));
    let saved = sim_systems::save::save(&mut engine).expect("save dug former floor");
    let restored = sim_systems::save::load_with(&saved, register_fixture_systems)
        .expect("load dug former floor");
    assert!(restored.resources.tile_grid.is_dug(index));
    assert!(!restored
        .resources
        .content_state
        .completed
        .contains_key(&key));
}

fn register_fixture_systems(engine: &mut sim_engine::SimEngine) {
    use sim_systems::runtime::agent::AgentMovementSystem;
    use sim_systems::runtime::construction::ConstructionSystem;
    use sim_systems::runtime::jobs::{JobAssignSystem, JobExecuteSystem};
    engine.register_system(Box::new(AgentMovementSystem::new()));
    engine.register_system(Box::new(JobAssignSystem::new()));
    engine.register_system(Box::new(JobExecuteSystem::new()));
    engine.register_system(Box::new(ConstructionSystem::new()));
    engine.register_system(Box::new(sim_systems::runtime::jobs::BillSystem::new()));
}
