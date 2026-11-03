//! Seven-family selected-pack integration through host bootstrap, ordinary jobs and save boundaries.
//! Each selected catalogue runs in a fresh process; the parent never mutates process-global environment.
mod colony_support;

use colony_support::{build_workshop, drop_items, job_world_m2, run_until, tick_checked, worker};
use sim_core::colony::content::{self, AnimalKind, PressureKind};
use sim_core::colony::content_state::CompletedLayer;
use sim_core::colony::knowledge::{self, Knowledge, Source};
use sim_core::colony::{BillMode, BlueprintKind, Command, ItemKind, ItemSpec, RecipeKind, Tool};
use sim_core::components::{Agent, ResourceKind};
use sim_core::rules::ColonyRule;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::{commands::submit, runtime::story, save};
use std::path::{Path, PathBuf};

const CHILD: &str = "WORLDSIM_DATA_ONLY_CASE";
const ITEM: u32 = 65537;
const RECIPE: u32 = 65538;
const WALL: u32 = 65539;
const PRESSURE: u32 = 4294967294;
const ANIMAL: u32 = 4294967293;

fn pack() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data/ext_pack/pack.ron")
}

#[test]
fn selected_pack_runs_all_simulation_families_and_rejects_bad_boundaries() {
    for case in [
        "base_before",
        "craft_build_learn",
        "pressure",
        "animal",
        "duplicate",
        "malformed",
        "missing",
        "overflow",
        "base_after",
    ] {
        let mut child =
            std::process::Command::new(std::env::current_exe().expect("test executable"));
        child
            .args(["--exact", "selected_pack_child", "--nocapture"])
            .env(CHILD, case);
        if matches!(case, "base_before" | "base_after") {
            child.env_remove("WORLDSIM_CONTENT_PACK");
        } else {
            child.env("WORLDSIM_CONTENT_PACK", pack());
        }
        let status = child.status().expect("fresh catalogue child");
        assert!(status.success(), "selected-pack case {case}");
    }
}

#[test]
fn selected_pack_child() {
    let Ok(case) = std::env::var(CHILD) else {
        return;
    };
    match case.as_str() {
        "base_before" | "base_after" => bundled_pack(),
        "duplicate" | "malformed" | "missing" | "overflow" => invalid_pack(&case),
        "craft_build_learn" | "pressure" | "animal" => {
            sim_test::content_boot::from_environment().expect("real host selected-pack bootstrap");
            match case.as_str() {
                "craft_build_learn" => craft_build_learn(),
                "pressure" => pressure(),
                "animal" => animal(),
                _ => unreachable!(),
            }
        }
        _ => panic!("unknown child case"),
    }
}

fn bundled_pack() {
    assert!(std::env::var_os("WORLDSIM_CONTENT_PACK").is_none());
    sim_test::content_boot::from_environment().expect("real host bundled bootstrap");
    assert!(ItemKind::checked(ITEM).is_err());
    assert!(RecipeKind::checked(RECIPE).is_err());
    assert!(BlueprintKind::checked(WALL).is_err());
    assert!(PressureKind::checked(PRESSURE).is_err());
    assert!(AnimalKind::checked(ANIMAL).is_err());
    let catalog = content::installed().expect("bundled catalogue");
    assert_eq!(catalog.item_role("tree_output"), Some(ItemKind::Log));
    assert_eq!(
        catalog.pressure_role("cold_default"),
        Some(PressureKind::ColdSnap)
    );
    assert_eq!(
        catalog.animal_role("default_grazer"),
        Some(AnimalKind::Deer)
    );
}

fn invalid_pack(case: &str) {
    let root = std::env::temp_dir().join(format!("worldsim-ext-{}-{case}", std::process::id()));
    std::fs::create_dir_all(&root).expect("isolated temporary pack");
    for entry in std::fs::read_dir(pack().parent().expect("pack directory")).expect("pack entries")
    {
        let entry = entry.expect("pack file");
        if entry.file_type().expect("file type").is_file() {
            std::fs::copy(entry.path(), root.join(entry.file_name()))
                .expect("copy selected source");
        }
    }
    let item_path = root.join("items.ron");
    let original = std::fs::read_to_string(&item_path).expect("item source");
    match case {
        "duplicate" => std::fs::write(&item_path, original.replace("code:65537", "code:0")),
        "malformed" => std::fs::write(&item_path, original.replace("stack_max:10", "stack_max:0")),
        "overflow" => std::fs::write(
            &item_path,
            original.replace("code:65537", "code:4294967296"),
        ),
        "missing" => std::fs::remove_file(&item_path),
        _ => unreachable!(),
    }
    .expect("prepare invalid source");
    let result = content::host::read_selection(&root.join("pack.ron"))
        .map_err(|e| e.to_string())
        .and_then(|selection| content::bootstrap(selection).map_err(|e| e.to_string()));
    assert!(
        result.is_err(),
        "{case} must fail the actual host/catalogue boundary"
    );
    assert!(
        content::installed().is_err(),
        "failed selection must not install bundled content"
    );
    std::fs::remove_dir_all(root).expect("remove private negative fixture");
}

fn know(engine: &mut SimEngine, entity: hecs::Entity, names: &[&str]) {
    let mut value = engine
        .world
        .get::<&Knowledge>(entity)
        .map(|k| (*k).clone())
        .unwrap_or_default();
    for name in names {
        value.set(
            knowledge::id(name).expect("registered knowledge"),
            800,
            Source::Seeded,
            0,
        );
    }
    engine
        .world
        .insert_one(entity, value)
        .expect("fixture knowledge");
}

fn agent_id(engine: &SimEngine, entity: hecs::Entity) -> u64 {
    engine.world.get::<&Agent>(entity).expect("agent").id
}

fn register_jobs(engine: &mut SimEngine) {
    use sim_systems::runtime::agent::AgentMovementSystem;
    use sim_systems::runtime::construction::ConstructionSystem;
    use sim_systems::runtime::jobs::{BillSystem, JobAssignSystem, JobExecuteSystem};
    engine.register_system(Box::new(AgentMovementSystem::new()));
    engine.register_system(Box::new(JobAssignSystem::new()));
    engine.register_system(Box::new(JobExecuteSystem::new()));
    engine.register_system(Box::new(ConstructionSystem::new()));
    engine.register_system(Box::new(BillSystem::new()));
}

fn craft_build_learn() {
    let recipe = RecipeKind::checked(RECIPE).expect("new recipe");
    let item = ItemKind::checked(ITEM).expect("new item");
    let wall = BlueprintKind::checked(WALL).expect("new blueprint");
    let node = knowledge::id("test_hide_craft").expect("new knowledge node");
    let mut engine = job_world_m2(32, 32);
    let learner = worker(&mut engine, 6, 6);
    let shop = build_workshop(&mut engine, BlueprintKind::DryingRack, (12, 12), (4, 4));
    engine.resources.rules.colony = Some(ColonyRule::M4);
    engine
        .resources
        .building_kinds
        .insert((10, 10), sim_core::components::BuildingKind::Hearth);
    drop_items(
        &mut engine,
        (20, 20),
        ItemSpec::raw(ItemKind::Hide, None),
        1,
    );
    assert!(
        submit(
            &mut engine,
            Command::AddBill {
                workshop: shop,
                recipe,
                mode: BillMode::Make(1)
            }
        )
        .accepted
    );
    for _ in 0..180 {
        tick_checked(&mut engine);
    }
    assert_eq!(
        engine.resources.colony.item_ledger.crafted_of(item),
        0,
        "unlearned recipe must stay gated"
    );
    assert_eq!(knowledge::gates().recipe(recipe), knowledge::bit(node));
    let bill = engine.resources.colony.workshops[&shop].bills[0].id;
    assert!(
        submit(
            &mut engine,
            Command::SetBillSuspended {
                workshop: shop,
                bill,
                suspended: true
            }
        )
        .accepted
    );
    let teacher = worker(&mut engine, 12, 10);
    know(&mut engine, teacher, &["test_hide_craft", "instruction"]);
    let command = Command::OrderTeach {
        teacher: agent_id(&engine, teacher),
        learner: agent_id(&engine, learner),
        knowledge: node,
    };
    assert!(submit(&mut engine, command).accepted);
    assert!(
        run_until(&mut engine, 4 * 1440, |e| e
            .resources
            .colony
            .tallies
            .teach_done
            == 1)
            < 4 * 1440
    );
    assert_eq!(
        engine
            .world
            .get::<&Knowledge>(learner)
            .expect("learned node")
            .get(node)
            .expect("entry")
            .source(),
        Source::Apprenticed
    );
    // Remove teacher's new-node proficiency so the observed craft must be performed by the learner.
    engine
        .world
        .get::<&mut Knowledge>(teacher)
        .expect("teacher")
        .set(node, 0, Source::Seeded, 0);
    assert!(
        submit(
            &mut engine,
            Command::SetBillSuspended {
                workshop: shop,
                bill,
                suspended: false
            }
        )
        .accepted
    );
    assert!(
        run_until(&mut engine, 3000, |e| e
            .resources
            .colony
            .item_ledger
            .crafted_of(item)
            == 2)
            < 3000
    );
    drop_items(
        &mut engine,
        (19, 20),
        ItemSpec::raw(ItemKind::Clay, None),
        2,
    );
    assert!(
        submit(
            &mut engine,
            Command::PlaceBlueprint {
                kind: wall,
                x: 17,
                y: 17
            }
        )
        .accepted
    );
    let key = ((17, 17), CompletedLayer::Wall);
    assert!(
        run_until(&mut engine, 3000, |e| e
            .resources
            .content_state
            .completed
            .contains_key(&key))
            < 3000
    );
    let learner_id = agent_id(&engine, learner);
    let bytes = save::save(&mut engine).expect("save selected data jobs");
    let mut restored = save::load_with(&bytes, register_jobs).expect("selected data save/load");
    assert_eq!(
        restored.resources.content_state.completed[&key].blueprint,
        wall
    );
    assert_eq!(
        restored.resources.content_state.completed[&key].style_id,
        "test_mudbrick"
    );
    assert_eq!(restored.resources.colony.item_ledger.crafted_of(item), 2);
    assert!(
        submit(
            &mut restored,
            Command::DesignateDeconstruct {
                x0: 17,
                y0: 17,
                x1: 17,
                y1: 17
            }
        )
        .accepted
    );
    assert!(
        run_until(&mut restored, 3000, |e| !e
            .resources
            .content_state
            .completed
            .contains_key(&key))
            < 3000
    );
    let index = restored.resources.tile_grid.idx(22, 22);
    restored.resources.tile_grid.terrain_type[index] = sim_core::TerrainType::Hill;
    let restored_learner = restored
        .world
        .query::<&Agent>()
        .iter()
        .find(|(_, a)| a.id == learner_id)
        .map(|(e, _)| e)
        .expect("restored learner");
    know(&mut restored, restored_learner, &["digging"]);
    sim_systems::runtime::nav::refresh(&mut restored.resources, (22, 22));
    assert!(
        submit(
            &mut restored,
            Command::DesignateDig {
                x0: 22,
                y0: 22,
                x1: 22,
                y1: 22
            }
        )
        .accepted
    );
    assert!(run_until(&mut restored, 3000, |e| e.resources.tile_grid.is_dug(index)) < 3000);
    let revision = content::installed()
        .expect("catalogue")
        .revision()
        .as_bytes();
    let mut foreign = bytes;
    let offset = foreign
        .windows(revision.len())
        .position(|w| w == revision)
        .expect("save revision envelope");
    foreign[offset] ^= 1;
    let error = save::load_with(&foreign, register_jobs)
        .err()
        .expect("foreign revision rejected")
        .to_string();
    assert!(error.contains("revision"), "{error}");
}

fn pressure() {
    let kind = PressureKind::checked(PRESSURE).expect("full-width pressure");
    assert_eq!(
        ron::from_str::<PressureKind>(&ron::to_string(&kind).expect("serialize pressure"))
            .expect("decode pressure"),
        kind
    );
    let mut engine = sim_test::colony_engine(2);
    for _ in 0..10 {
        engine.tick();
    }
    let fire = engine.resources.rules.society.firewood.expect("fire rule");
    let night = (0..64u64)
        .map(|d| 960 + d * 1440)
        .find(|tick| {
            engine.resources.current_tick = *tick;
            sim_systems::runtime::settlement::firewood::night_season(&engine.resources, &fire) == 3
        })
        .expect("winter night");
    engine.resources.current_tick = night;
    story::try_force(&mut engine.world, &mut engine.resources, kind, 2)
        .expect("generic pressure force");
    let active = story::active_mechanic(&engine.resources, content::PressureMechanic::Cold)
        .expect("cold mechanic");
    assert_eq!(active.kind, kind);
    assert_eq!(active.rule.snap_outdoor_work_permille, 1750);
    let view = sim_bridge::ffi::collect_story(&engine.world, &engine.resources);
    let row = view
        .active
        .iter()
        .find(|row| row.kind == PRESSURE)
        .expect("forced pressure DTO");
    assert_eq!(row.name, "test_cold");
    assert_eq!(row.name_key, "EXT_COLD");
    assert_eq!(row.category, "cold");
    assert_eq!(row.mechanic, "cold");
    let burn_minutes = (fire.burn_end_minute + 1440 - fire.burn_start_minute) % 1440;
    let night_rate =
        sim_systems::runtime::settlement::firewood::night_rate(&engine.resources, &fire, night);
    let expected_need = u64::from(burn_minutes / 60)
        * u64::from(night_rate)
        * engine.resources.settlements.len().max(1) as u64
        * 1700
        / 1000;
    assert!(expected_need > 0);
    assert_eq!(
        view.firewood_day_need, expected_need,
        "banner uses selected cold burn factor"
    );
    assert_eq!(
        sim_systems::runtime::settlement::fire::effective_burn_per_hour(&engine.resources, 10),
        17
    );
    for settlement in engine.resources.settlements.values_mut() {
        settlement.withdraw(ResourceKind::Wood, u32::MAX);
    }
    sim_systems::runtime::settlement::FireSystem::new()
        .tick(&mut engine.world, &mut engine.resources);
    assert!(engine.resources.causal_log.iter().any(|(_, log)| log.as_slice().iter().any(|event| matches!(event, sim_core::causal::CausalEvent::PressureCause { kind: cause, effect: 0, .. } if *cause == PRESSURE))), "ordinary fire exposure records actual full-width pressure");
    assert!(
        story::try_force(
            &mut engine.world,
            &mut engine.resources,
            PressureKind::ColdSnap,
            2
        )
        .is_err(),
        "conflicting same-mechanic identity rejected"
    );
    let bytes = save::save(&mut engine).expect("save novel pressure");
    let restored = save::load(&bytes).expect("load novel pressure");
    assert!(restored.resources.colony.story.is_active(kind));
}

fn animal() {
    let kind = AnimalKind::checked(ANIMAL).expect("full-width animal");
    let item = ItemKind::checked(ITEM).expect("declared loot");
    let mut engine = job_world_m2(32, 32);
    engine.resources.rules.colony = Some(ColonyRule::M5);
    let hunter = worker(&mut engine, 5, 5);
    know(&mut engine, hunter, &["tracking"]);
    engine
        .world
        .insert_one(
            hunter,
            Tool {
                item: ItemSpec::raw(ItemKind::Spear, None),
                uses: 0,
            },
        )
        .expect("fixture spear");
    engine
        .resources
        .colony
        .item_ledger
        .create(ItemKind::Spear, 1);
    sim_systems::runtime::nav::ensure(&mut engine.resources);
    let before = engine.resources.colony.wildlife.animals.len();
    assert!(sim_systems::runtime::wildlife::spawn_animal(
        &mut engine.resources,
        kind,
        (33, 5),
        None
    )
    .is_err());
    assert!(sim_systems::runtime::wildlife::spawn_animal(
        &mut engine.resources,
        kind,
        (7, 5),
        Some(1)
    )
    .is_err());
    assert_eq!(engine.resources.colony.wildlife.animals.len(), before);
    let id =
        sim_systems::runtime::wildlife::spawn_animal(&mut engine.resources, kind, (7, 5), None)
            .expect("generic registered spawn");
    assert_eq!(engine.resources.colony.wildlife.animals[&id].kind, kind);
    let bytes = save::save(&mut engine).expect("save novel animal");
    let mut engine = save::load_with(&bytes, register_jobs).expect("load novel animal");
    assert_eq!(engine.resources.colony.wildlife.animals[&id].kind, kind);
    assert!(submit(&mut engine, Command::DesignateHunt { animal: id }).accepted);
    let rows = sim_bridge::ffi::collect_animals(&engine.resources);
    let index = rows
        .id
        .iter()
        .position(|value| *value == i64::from(id))
        .expect("restored marked animal DTO");
    assert_eq!(rows.kind[index], i64::from(ANIMAL));
    assert_eq!(rows.names[index], "test_grazer");
    assert_eq!(rows.name_keys[index], "EXT_ANIMAL");
    assert_eq!(rows.categories[index], "grazer");
    assert_eq!(rows.mechanics[index], "grazer");
    assert_eq!(rows.max_hp[index], 1);
    assert_eq!(rows.hp[index], 1);
    assert_eq!(rows.huntable[index], 1);
    assert_eq!(rows.marked[index], 1);
    assert!(
        run_until(&mut engine, 4000, |e| !e
            .resources
            .colony
            .wildlife
            .animals
            .contains_key(&id))
            < 4000
    );
    assert_eq!(
        engine.resources.colony.item_ledger.created_of(item),
        2,
        "declared custom nonperishable loot"
    );
    assert!(
        engine
            .resources
            .chronicle
            .entries()
            .iter()
            .any(
                |entry| entry.kind == sim_core::chronicle::ChronicleKind::Hunted
                    && entry.cause == ANIMAL
            ),
        "ordinary hunt retains species identity"
    );
}
