//! Passive production ownership, wet-time and capacity regressions.
use super::{craft, processing};
use hecs::{Entity, World};
use sim_core::colony::workshop::{recipes, Recipe};
use sim_core::colony::{
    Bill, BillMode, CancelReason, ItemKind, ItemSpec, ItemStack, Job, JobKind, JobStage, Workshop,
};
use sim_core::components::{Agent, Position};
use sim_engine::SimResources;

fn scene(recipe: &'static Recipe) -> (World, SimResources, Entity, Job) {
    let mut world = World::new();
    let mut res = SimResources::new(6, 6, sim_core::MaterialRegistry::new());
    res.rules = sim_core::rules::WorldRules::colony(0);
    res.rules.colony.as_mut().expect("colony").knowledge = None;
    let mut shelf = Vec::new();
    for &(kind, count) in &recipe.inputs {
        shelf.push(ItemStack::of(ItemSpec::raw(kind, None), count));
        res.colony.item_ledger.create(kind, u64::from(count));
    }
    res.colony.workshops.insert(
        1,
        Workshop {
            id: 1,
            kind: recipe.workshop,
            top_left: (2, 2),
            size: (1, 1),
            bills: vec![Bill {
                id: 1,
                recipe: recipe.id,
                mode: BillMode::Make(2),
                done: 0,
                suspended: false,
                auto: false,
            }],
            shelf,
            active: Some(1),
        },
    );
    let index = res.tile_grid.idx(1, 2);
    res.tile_grid.terrain_type[index] = sim_core::TerrainType::River;
    let job = Job {
        kind: JobKind::Craft {
            workshop: 1,
            bill: 1,
            at: (2, 3),
        },
        stage: JobStage::Working { ticks_left: 0 },
        taken_at: 0,
        amount: 0,
        item: recipe.output.0,
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: 0,
    };
    let person = world.spawn((Agent { id: 1 }, Position::new(2, 3)));
    (world, res, person, job)
}

fn passive_recipe() -> &'static Recipe {
    recipes()
        .iter()
        .find(|recipe| recipe.process.is_some())
        .expect("passive recipe")
}

fn start_and_finish(world: &mut World, res: &mut SimResources, person: Entity, job: &mut Job) {
    assert!(craft::start(world, res, person, job, 1, 1).is_some());
    assert!(craft::finish_craft(world, res, 1, person, job));
}

#[test]
fn wet_time_pauses_without_double_consumption_and_owns_its_slot() {
    let recipe = passive_recipe();
    let (mut world, mut res, person, mut job) = scene(recipe);
    start_and_finish(&mut world, &mut res, person, &mut job);
    assert_eq!(res.processing.pending.len(), 1);
    assert_eq!(res.colony.item_ledger.created_of(recipe.output.0), 0);
    let input = recipe.inputs[0];
    assert_eq!(
        res.colony.item_ledger.consumed_of(input.0),
        u64::from(input.1)
    );
    assert!(!processing::permitted(
        &res,
        &res.colony.workshops[&1],
        recipe
    ));
    let remaining = res.processing.pending[0].remaining_ticks;
    processing::advance(&mut res);
    assert_eq!(res.processing.pending[0].remaining_ticks, remaining - 1);
    let index = res.tile_grid.idx(1, 2);
    res.tile_grid.terrain_type[index] = sim_core::TerrainType::Plain;
    res.current_tick += 1_000_000;
    processing::advance(&mut res);
    assert_eq!(res.processing.pending[0].remaining_ticks, remaining - 1);
    res.tile_grid.terrain_type[index] = sim_core::TerrainType::River;
    for _ in 1..remaining {
        processing::advance(&mut res);
    }
    assert!(res.processing.pending.is_empty());
    assert_eq!(
        res.colony.ground_total(recipe.output.0),
        u64::from(recipe.output.1)
    );
    assert_eq!(
        res.colony.item_ledger.created_of(recipe.output.0),
        u64::from(recipe.output.1)
    );
    assert_eq!(
        res.colony.item_ledger.consumed_of(input.0),
        u64::from(input.1)
    );
    super::invariants::check(&world, &res).expect("exact ledger");
}

#[test]
fn dry_pit_and_missing_spindle_cannot_consume_inputs() {
    let recipe = passive_recipe();
    let (mut world, mut res, person, mut job) = scene(recipe);
    let index = res.tile_grid.idx(1, 2);
    res.tile_grid.terrain_type[index] = sim_core::TerrainType::Plain;
    assert!(craft::start(&mut world, &mut res, person, &mut job, 1, 1).is_none());
    assert_eq!(res.colony.item_ledger.transforms, 0);
    let recipe = recipes()
        .iter()
        .find(|recipe| {
            recipe
                .tool
                .as_ref()
                .is_some_and(|tool| tool.capability != "cutter")
        })
        .expect("spinning recipe");
    let (mut world, mut res, person, mut job) = scene(recipe);
    assert!(craft::start(&mut world, &mut res, person, &mut job, 1, 1).is_none());
    let tool = recipe.tool.as_ref().expect("tool").preferred_item;
    res.colony
        .workshops
        .get_mut(&1)
        .expect("shop")
        .shelf
        .push(ItemStack::of(ItemSpec::raw(tool, None), 1));
    res.colony.item_ledger.create(tool, 1);
    start_and_finish(&mut world, &mut res, person, &mut job);
    assert_eq!(res.colony.workshops[&1].shelved(tool), 1);
    assert_eq!(res.colony.item_ledger.consumed_of(tool), 0);
    assert_eq!(res.colony.wildlife.tallies.knife_uses, 0);
    super::invariants::check(&world, &res).expect("exact ledger");
}

#[test]
fn ready_output_retries_atomically_and_removed_workshop_loses_inputs_once() {
    let recipe = passive_recipe();
    let (mut world, mut res, person, mut job) = scene(recipe);
    start_and_finish(&mut world, &mut res, person, &mut job);
    res.processing.pending[0].remaining_ticks = 0;
    for y in 0..6 {
        for x in 0..6 {
            if !super::items::can_hold_items(&res, (x, y)) {
                continue;
            }
            res.colony
                .ground_items
                .insert((x, y), ItemStack::of(ItemSpec::raw(ItemKind::Log, None), 1));
            res.colony.item_ledger.create(ItemKind::Log, 1);
        }
    }
    for _ in 0..2 {
        processing::advance(&mut res);
    }
    assert_eq!(res.processing.pending.len(), 1);
    assert_eq!(res.colony.item_ledger.created_of(recipe.output.0), 0);
    res.colony.ground_items.remove(&(5, 5));
    res.colony.item_ledger.consume(ItemKind::Log, 1);
    processing::advance(&mut res);
    assert!(res.processing.pending.is_empty());
    assert_eq!(
        res.colony.item_ledger.created_of(recipe.output.0),
        u64::from(recipe.output.1)
    );
    super::invariants::check(&world, &res).expect("exact ledger");

    let (mut world, mut res, person, mut job) = scene(recipe);
    start_and_finish(&mut world, &mut res, person, &mut job);
    res.colony.workshops.remove(&1);
    processing::advance(&mut res);
    processing::advance(&mut res);
    assert!(res.processing.pending.is_empty());
    assert_eq!(res.colony.tallies.crafts_lost, 1);
    assert_eq!(res.colony.item_ledger.created_of(recipe.output.0), 0);
    super::invariants::check(&world, &res).expect("exact ledger");
}

#[test]
fn cancelling_active_retting_loses_consumed_inputs_once_without_starting_passive_work() {
    let recipe = passive_recipe();
    let (mut world, mut res, person, mut job) = scene(recipe);
    let ticks = craft::start(&mut world, &mut res, person, &mut job, 1, 1).expect("start");
    job.stage = JobStage::Working { ticks_left: ticks };
    world.insert_one(person, job).expect("active craft");
    super::cancel::cancel(&mut world, &mut res, person, CancelReason::Interrupted);
    super::cancel::cancel(&mut world, &mut res, person, CancelReason::Interrupted);
    assert_eq!(res.colony.tallies.crafts_lost, 1);
    assert!(res.processing.pending.is_empty());
    assert_eq!(res.colony.item_ledger.created_of(recipe.output.0), 0);
    let input = recipe.inputs[0];
    assert_eq!(
        res.colony.item_ledger.consumed_of(input.0),
        u64::from(input.1)
    );
    super::invariants::check(&world, &res).expect("exact ledger");
}

#[test]
fn authored_garment_condition_is_created_once_and_wrap_is_not_first_tunic() {
    let recipe = recipes()
        .iter()
        .find(|recipe| {
            recipe
                .output
                .0
                .definition()
                .apparel
                .as_ref()
                .is_some_and(|apparel| apparel.wear_days == Some(24))
        })
        .expect("wrap recipe");
    let (mut world, mut res, person, mut job) = scene(recipe);
    if let Some(tool) = &recipe.tool {
        res.colony
            .workshops
            .get_mut(&1)
            .expect("shop")
            .shelf
            .push(ItemStack::of(ItemSpec::raw(tool.preferred_item, None), 1));
        res.colony.item_ledger.create(tool.preferred_item, 1);
    }
    start_and_finish(&mut world, &mut res, person, &mut job);
    let output = res
        .colony
        .ground_items
        .values()
        .find(|stack| stack.kind == recipe.output.0)
        .expect("physical wrap");
    assert_eq!(output.wear_remaining_hours, Some(576));
    assert!(!res
        .chronicle
        .entries()
        .iter()
        .any(|entry| entry.kind == sim_core::chronicle::ChronicleKind::FirstTunic));
    super::invariants::check(&world, &res).expect("exact ledger");
}
