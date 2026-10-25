//! B5 pure bridge contract: saved species align with unchanged coordinate reads.
use sim_bridge::ffi::{collect_tree_snapshot, collect_trees};
use sim_core::material::MaterialId;

#[test]
fn tree_snapshot_uses_saved_identity_and_stable_coordinate_order() {
    let mut engine = sim_test::colony_engine(0);
    let coordinates = collect_trees(&engine.resources);
    let tile = coordinates.1[0];
    let material = MaterialId::from_str_hash("pine");
    engine
        .resources
        .content_state
        .record_tree_material(tile, material)
        .expect("initialized default");
    let snapshot = collect_tree_snapshot(&engine.resources);
    assert_eq!(
        (snapshot.version, &snapshot.tiles),
        (coordinates.0, &coordinates.1)
    );
    assert_eq!(snapshot.tiles.len(), snapshot.species.len());
    assert_eq!(snapshot.species[0], "pine");
    for (tile, species) in snapshot.tiles.iter().zip(&snapshot.species) {
        let saved = engine
            .resources
            .content_state
            .tree_material(*tile)
            .expect("saved material");
        assert_eq!(species, engine.resources.colony.materials.name(saved));
    }
}

#[test]
fn missing_saved_tree_identity_is_not_disguised_as_a_species() {
    let mut engine = sim_test::colony_engine(0);
    engine.resources.content_state.tree_material_default = None;
    engine.resources.content_state.tree_materials.clear();
    let snapshot = collect_tree_snapshot(&engine.resources);
    assert!(!snapshot.tiles.is_empty());
    assert!(snapshot.species.iter().all(String::is_empty));
}

#[test]
fn an_override_is_not_replaced_by_the_persisted_default() {
    let mut engine = sim_test::colony_engine(1);
    let tile = *engine.resources.colony.trees.keys().next().expect("tree");
    engine.resources.content_state.tree_material_default = Some(MaterialId::from_str_hash("birch"));
    engine.resources.content_state.tree_materials.clear();
    engine
        .resources
        .content_state
        .record_tree_material(tile, MaterialId::from_str_hash("pine"))
        .expect("default exists");
    let snapshot = collect_tree_snapshot(&engine.resources);
    assert_eq!(snapshot.species[0], "pine");
    assert!(snapshot.species.iter().skip(1).all(|name| name == "birch"));
}
