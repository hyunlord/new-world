//! B5: real generated trees, ordinary material flow, current/legacy save boundaries.
mod colony_support;

use sim_core::colony::{BlueprintKind, Carry, Command, ItemKind, Job, JobKind};
use sim_core::components::{AgentState, Hunger, Position, Sleep, Thirst};
use sim_core::material::MaterialId;
use sim_core::rules::ColonyRule;
use sim_engine::SimEngine;
use sim_systems::commands::{check_blueprint, submit};
use sim_systems::runtime::jobs::invariants;
use sim_systems::{runtime::nav, save};

type Tile = (u32, u32);
const PINE: MaterialId = MaterialId::from_str_hash("pine");
const OAK: MaterialId = MaterialId::from_str_hash("oak");

fn tick(engine: &mut SimEngine) {
    engine.tick();
    invariants::check(&engine.world, &engine.resources)
        .expect("ordinary item/reservation invariants");
}
fn until(engine: &mut SimEngine, done: impl Fn(&SimEngine) -> bool) {
    for _ in 0..1440 {
        if done(engine) {
            return;
        }
        tick(engine);
    }
    assert!(done(engine), "ordinary job did not finish within one day");
}
fn position_workers(engine: &mut SimEngine, tile: Tile) {
    for (_, (position, state)) in engine
        .world
        .query::<(&mut Position, &mut AgentState)>()
        .iter()
    {
        *position = Position::new(tile.0, tile.1);
        *state = AgentState::Idle;
    }
    for (_, value) in engine.world.query::<&mut Hunger>().iter() {
        *value = Hunger::new(0.0, 0.0);
    }
    for (_, value) in engine.world.query::<&mut Thirst>().iter() {
        *value = Thirst::new(0.0, 0.0);
    }
    for (_, value) in engine.world.query::<&mut Sleep>().iter() {
        *value = Sleep::new(0.0, 0.0);
    }
}
fn candidate(engine: &SimEngine, material: MaterialId) -> (Tile, Vec<Tile>, Tile) {
    let r = &engine.resources;
    for (&tree, value) in &r.colony.trees {
        if r.content_state.tree_material(tree) != Some(material) || value.logs % 2 != 0 {
            continue;
        }
        let mut free = Vec::new();
        for y in tree.1.saturating_sub(5)..=(tree.1 + 5).min(r.tile_grid.height - 1) {
            for x in tree.0.saturating_sub(5)..=(tree.0 + 5).min(r.tile_grid.width - 1) {
                let tile = (x, y);
                if !r.colony.trees.contains_key(&tile)
                    && nav::passable(r, tile)
                    && check_blueprint(r, BlueprintKind::LogWall, x, y).is_ok()
                    && (x.abs_diff(tree.0) + y.abs_diff(tree.1)) >= 2
                {
                    free.push(tile);
                }
            }
        }
        let need = usize::from(value.logs / 2);
        if free.len() > need {
            let worker = free[need];
            // A short cardinal route avoids selecting an isolated forest island.
            if worker.0.abs_diff(tree.0) + worker.1.abs_diff(tree.1) <= 8 {
                return (tree, free[..need].to_vec(), worker);
            }
        }
    }
    panic!("no naturally generated material tree with nearby free building sites");
}
fn place(engine: &mut SimEngine, walls: &[Tile]) {
    for &(x, y) in walls {
        assert!(
            submit(
                engine,
                Command::PlaceBlueprint {
                    kind: BlueprintKind::LogWall,
                    x,
                    y
                }
            )
            .accepted
        );
    }
}
fn complete(engine: &SimEngine, walls: &[Tile], material: MaterialId) -> bool {
    walls.iter().all(|&(x, y)| {
        engine.resources.tile_grid.wall_material[engine.resources.tile_grid.idx(x, y)]
            == Some(material)
    })
}
fn save_bytes(engine: &mut SimEngine) -> Vec<u8> {
    save::save(engine).expect("save")
}

#[test]
fn original_pine_chops_delivers_builds_and_continues_across_both_save_boundaries() {
    let mut engine = sim_test::colony_engine(7);
    engine.resources.rules.colony = Some(ColonyRule::M3);
    engine.resources.rules.agent.move_interval_ticks = 1;
    nav::ensure(&mut engine.resources);
    let (tree, walls, worker) = candidate(&engine, PINE);
    let original_logs = engine.resources.colony.trees[&tree].logs;
    let original_identity = engine.resources.content_state.tree_material(tree);
    position_workers(&mut engine, worker);
    let before = save_bytes(&mut engine);
    let mut restored_before =
        save::load(&before).expect("before chop restore with production registry");
    let command = Command::DesignateChopTiles { tiles: vec![tree] };
    assert!(submit(&mut engine, command.clone()).accepted);
    assert!(submit(&mut restored_before, command).accepted);
    until(&mut engine, |e| {
        !e.resources.colony.trees.contains_key(&tree)
    });
    while restored_before.current_tick() < engine.current_tick() {
        tick(&mut restored_before);
    }
    assert_eq!(save_bytes(&mut engine), save_bytes(&mut restored_before));
    assert_eq!(original_identity, Some(PINE));
    assert!(!engine
        .resources
        .content_state
        .tree_materials
        .contains_key(&tree));
    let stack = engine
        .resources
        .colony
        .ground_items
        .get(&tree)
        .expect("actual felled logs");
    assert_eq!(
        (stack.kind, stack.material, stack.count),
        (ItemKind::Log, Some(PINE), u16::from(original_logs))
    );
    place(&mut engine, &walls);
    until(&mut engine, |e| {
        e.world
            .query::<(&Carry, &Job)>()
            .iter()
            .any(|(_, (carry, job))| {
                carry.material == Some(PINE) && matches!(job.kind, JobKind::Deliver { .. })
            })
    });
    let middle = save_bytes(&mut engine);
    let mut restored_middle = save::load(&middle).expect("mid-delivery restore");
    until(&mut engine, |e| complete(e, &walls, PINE));
    while restored_middle.current_tick() < engine.current_tick() {
        tick(&mut restored_middle);
    }
    assert_eq!(save_bytes(&mut engine), save_bytes(&mut restored_middle));
    assert!(complete(&restored_middle, &walls, PINE));
    let terms = invariants::ledger_of(&engine.world, &engine.resources, ItemKind::Log);
    assert!(terms.balanced());
    assert_eq!(terms.consumed, u64::from(original_logs));
    // Exhaust the pine yield through real walls before selecting an oak control.
    assert_eq!(terms.ground + terms.carried + terms.delivered, 0);
    let (oak_tree, oak_walls, oak_worker) = candidate(&engine, OAK);
    position_workers(&mut engine, oak_worker);
    assert!(
        submit(
            &mut engine,
            Command::DesignateChopTiles {
                tiles: vec![oak_tree]
            }
        )
        .accepted
    );
    until(&mut engine, |e| {
        !e.resources.colony.trees.contains_key(&oak_tree)
    });
    place(&mut engine, &oak_walls);
    until(&mut engine, |e| complete(e, &oak_walls, OAK));
    let completed = save_bytes(&mut engine);
    save::load(&completed).expect("completed production save");
    if let Some(dir) = std::env::var_os("WORLDSIM_B5_EXPORT_DIR") {
        let dir = std::path::PathBuf::from(dir);
        assert!(
            dir.is_absolute(),
            "export must use explicit absolute external directory"
        );
        assert!(
            !dir.starts_with(
                std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                    .ancestors()
                    .nth(3)
                    .expect("repo")
            ),
            "export outside repository"
        );
        std::fs::create_dir_all(&dir).expect("external export directory");
        for (name, bytes) in [
            ("before_chop.save", before),
            ("mid_delivery.save", middle),
            ("completed.save", completed),
        ] {
            std::fs::write(dir.join(name), bytes).expect("export save");
        }
        let receipt = serde_json::json!({"tree":tree,"species":"pine","wall":walls[0],"wall_material":PINE,"oak_tree":oak_tree,"oak_wall":oak_walls[0],"oak_wall_material":OAK,"before_chop_tick":0,"completed_tick":engine.current_tick(),"chop_already_designated":false,"population_provenance":"original_worldgen_coordinates","ordinary_commands":true,"world_seed":7,"original_tree_logs":original_logs,"pine_walls":walls,"oak_walls":oak_walls,"setup":"Original colony world and trees; M3 rules, movement interval 1, workers positioned near source; hunger/thirst/sleep growth zero. Full production registry. Every wall consumes ordinary felled logs; no inserted trees/logs/walls."});
        std::fs::write(
            dir.join("receipt.json"),
            serde_json::to_vec_pretty(&receipt).expect("receipt JSON"),
        )
        .expect("export receipt");
    }
}

#[test]
fn actual_h_format9_fixture_loads_as_oak_then_resaves_as_format10() {
    let Ok(path) = std::env::var("WORLDSIM_B5_LEGACY9") else {
        return;
    };
    let bytes = std::fs::read(path).expect("required actual H fixture");
    assert_eq!(&bytes[8..12], &9u32.to_le_bytes());
    let mut engine = save::load(&bytes).expect("historical format9 fixture");
    assert!(!engine.resources.colony.trees.is_empty());
    for &tile in engine.resources.colony.trees.keys() {
        assert_eq!(
            engine.resources.content_state.tree_material(tile),
            Some(OAK)
        );
    }
    assert!(engine.resources.content_state.tree_materials.is_empty());
    assert!(engine.resources.colony.materials.stats(PINE).is_some());
    let new = save_bytes(&mut engine);
    assert_eq!(&new[8..12], &10u32.to_le_bytes());
    let mut twin = save::load(&new).expect("migrated format10");
    for _ in 0..24 {
        tick(&mut engine);
        tick(&mut twin);
    }
    assert_eq!(save_bytes(&mut engine), save_bytes(&mut twin));
    // Controlled continuation of the actual migrated H world: retain every
    // original tree coordinate/yield and cancel active jobs through their normal
    // release path before repositioning workers and stabilizing needs.
    let active: Vec<_> = engine
        .world
        .query::<&Job>()
        .iter()
        .map(|(entity, _)| entity)
        .collect();
    for entity in active {
        sim_systems::runtime::jobs::cancel::cancel(
            &mut engine.world,
            &mut engine.resources,
            entity,
            sim_core::colony::CancelReason::Interrupted,
        );
    }
    engine.resources.rules.colony = Some(ColonyRule::M3);
    engine.resources.rules.agent.move_interval_ticks = 1;
    nav::ensure(&mut engine.resources);
    let (tree, _, worker) = candidate(&engine, OAK);
    let original_logs = engine.resources.colony.trees[&tree].logs;
    let created_before =
        invariants::ledger_of(&engine.world, &engine.resources, ItemKind::Log).created;
    position_workers(&mut engine, worker);
    assert!(
        submit(
            &mut engine,
            Command::DesignateChopTiles { tiles: vec![tree] }
        )
        .accepted
    );
    until(&mut engine, |e| {
        !e.resources.colony.trees.contains_key(&tree)
    });
    let logs = engine
        .resources
        .colony
        .ground_items
        .get(&tree)
        .expect("actual migrated tree yielded logs");
    assert_eq!(
        (logs.kind, logs.material, logs.count),
        (ItemKind::Log, Some(OAK), u16::from(original_logs))
    );
    assert_eq!(
        invariants::ledger_of(&engine.world, &engine.resources, ItemKind::Log).created
            - created_before,
        u64::from(original_logs)
    );
    assert_eq!(
        engine.resources.content_state.tree_material_default,
        Some(OAK)
    );
    save::load(&save_bytes(&mut engine)).expect("post-chop migrated world remains valid");
}

#[test]
fn actual_host_distribution_replacement_runs_in_fresh_process() {
    let root = std::env::temp_dir().join(format!("worldsim-b5-pine-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("isolated pack directory");
    std::fs::write(
        root.join("pack.ron"),
        r#"(schema_version:1,id:"pine_distribution",revision:"1",sources:{"flora":"flora.ron"})"#,
    )
    .expect("descriptor");
    std::fs::write(root.join("flora.ron"), r#"(schema_version:1,species:[],replace_distributions:Some([(terrain:forest,elevation:(0,255),weights:[(material:"pine",weight:1)])]))"#).expect("replacement");
    let status = std::process::Command::new(std::env::current_exe().expect("test executable"))
        .args(["--exact", "host_pine_child", "--nocapture"])
        .env("WORLDSIM_B5_HOST_CHILD", "1")
        .env("WORLDSIM_CONTENT_PACK", root.join("pack.ron"))
        .env_remove("WORLDSIM_B5_EXPORT_DIR")
        .status()
        .expect("fresh real host");
    assert!(status.success());
    std::fs::remove_dir_all(root).expect("temporary pack cleanup");
}

#[test]
fn host_pine_child() {
    if std::env::var_os("WORLDSIM_B5_HOST_CHILD").is_none() {
        return;
    }
    sim_test::content_boot::from_environment().expect("real host pack bootstrap");
    let mut engine = sim_test::colony_engine(7);
    assert!(!engine.resources.colony.trees.is_empty());
    for &tile in engine.resources.colony.trees.keys() {
        assert_eq!(
            engine.resources.content_state.tree_material(tile),
            Some(PINE)
        );
    }
    engine.resources.rules.colony = Some(ColonyRule::M3);
    engine.resources.rules.agent.move_interval_ticks = 1;
    nav::ensure(&mut engine.resources);
    let (tree, walls, worker) = candidate(&engine, PINE);
    position_workers(&mut engine, worker);
    let before = save_bytes(&mut engine);
    engine = save::load(&before).expect("selected revision save");
    assert!(
        submit(
            &mut engine,
            Command::DesignateChopTiles { tiles: vec![tree] }
        )
        .accepted
    );
    until(&mut engine, |e| {
        !e.resources.colony.trees.contains_key(&tree)
    });
    assert_eq!(
        engine.resources.colony.ground_items[&tree].material,
        Some(PINE)
    );
    place(&mut engine, &walls);
    until(&mut engine, |e| complete(e, &walls, PINE));
    save::load(&save_bytes(&mut engine)).expect("selected host completed walls");
}

#[test]
fn current_codec_rejects_missing_default_and_noncanonical_overrides() {
    let mut engine = sim_test::colony_engine(7);
    let tile = *engine.resources.colony.trees.keys().next().expect("tree");
    let pristine = save_bytes(&mut engine);
    engine.resources.content_state.tree_material_default = None;
    assert!(save::load(&save_bytes(&mut engine)).is_err());
    for (entry, material) in [
        (tile, OAK),
        ((u32::MAX, 0), PINE),
        (tile, MaterialId::from_str_hash("flint")),
    ] {
        engine = save::load(&pristine).expect("valid fixture");
        engine
            .resources
            .content_state
            .tree_materials
            .insert(entry, material);
        assert!(save::load(&save_bytes(&mut engine)).is_err());
    }
}

#[test]
fn persisted_default_survives_catalog_default_difference() {
    let mut engine = sim_test::colony_engine(7);
    assert_eq!(
        sim_core::colony::content::installed_or_bundled()
            .flora()
            .default_material(),
        OAK
    );
    let identities: Vec<_> = engine
        .resources
        .colony
        .trees
        .keys()
        .map(|&tile| {
            (
                tile,
                engine
                    .resources
                    .content_state
                    .tree_material(tile)
                    .expect("generated identity"),
            )
        })
        .collect();
    engine.resources.content_state.tree_material_default = Some(PINE);
    engine.resources.content_state.tree_materials.clear();
    for (tile, material) in &identities {
        engine
            .resources
            .content_state
            .record_tree_material(*tile, *material)
            .expect("recompress same material");
    }
    let restored = save::load(&save_bytes(&mut engine)).expect("persisted different default");
    assert_eq!(
        restored.resources.content_state.tree_material_default,
        Some(PINE)
    );
    for (tile, material) in identities {
        assert_eq!(
            restored.resources.content_state.tree_material(tile),
            Some(material)
        );
    }
}
