//! Action1 live job legs through command admission, assignment, movement and execution.

mod colony_support;

use colony_support::{
    adults, carry_of, drop_items, job_of, job_world_m2, plant, run_until, tick_checked, worker,
};
use sim_core::colony::knowledge::{self, Knowledge, Source};
use sim_core::colony::visible_action::{
    CarriedItemView, Phase, VisibleActionKind as Action, VisibleActionRow, VisibleCueKind,
    VisibleTarget, VisibleTargetKind as Target,
};
use sim_core::colony::{
    BillMode, Command, ItemKind, ItemSpec, Job, JobKind, JobStage, RecipeKind, ReserveKey, Tool,
    Workshop,
};
use sim_core::components::{Agent, AgentId, AgentState, Settlement, TargetKind};
use sim_engine::{FireState, RuntimeSystem, SimEngine};
use sim_systems::commands::submit;
use sim_systems::runtime::food::items::{create_ground, FoodSource};
use sim_systems::runtime::wildlife::AnimalSystem;
use sim_systems::visible_action_rows;

fn issue(engine: &mut SimEngine, command: Command) {
    let result = submit(engine, command);
    assert!(result.accepted, "{result:?}");
}

fn row(engine: &SimEngine, id: AgentId) -> VisibleActionRow {
    visible_action_rows(&engine.world, &engine.resources)
        .into_iter()
        .find(|row| row.agent_id == id)
        .expect("a row for the live worker")
}

fn agent_id(engine: &SimEngine, person: hecs::Entity) -> AgentId {
    engine.world.get::<&Agent>(person).expect("worker").id
}

fn stockpile(engine: &mut SimEngine, (x0, y0): (u32, u32)) {
    issue(
        engine,
        Command::AddStockpile {
            x0,
            y0,
            x1: x0 + 1,
            y1: y0 + 1,
        },
    );
}

fn wait_for_job(engine: &mut SimEngine, person: hecs::Entity, stage: impl Fn(Job) -> bool) -> Job {
    assert!(run_until(engine, 2000, |e| job_of(e, person).is_some_and(&stage)) < 2000);
    job_of(engine, person).expect("observed job")
}

fn assert_leg(view: VisibleActionRow, action: Action, phase: Phase, target: VisibleTarget) {
    assert_eq!(
        (view.action, view.phase, view.target),
        (action, phase, Some(target))
    );
    let goal = if phase == Phase::Perform {
        None
    } else {
        target.tile
    };
    assert_eq!(view.goal, goal);
}

#[test]
fn harness_activity_variants_and_legs_project_axe_fetch_and_chopping() {
    let mut engine = job_world_m2(40, 40);
    let person = worker(&mut engine, 5, 5);
    let id = agent_id(&engine, person);
    let (stack, tree) = ((10, 5), (30, 30));
    let axe = ItemSpec {
        quality: 36,
        ..ItemSpec::raw(ItemKind::StoneAxe, None)
    };
    stockpile(&mut engine, stack);
    drop_items(&mut engine, stack, axe, 1);
    plant(&mut engine, tree, 4);
    issue(
        &mut engine,
        Command::DesignateChopTiles { tiles: vec![tree] },
    );
    let chop = |job: Job| job.kind == (JobKind::Chop { tree });
    wait_for_job(&mut engine, person, |job| {
        chop(job) && job.fetch == Some(stack)
    });
    let fetch = row(&engine, id);
    assert_leg(
        fetch,
        Action::Chop,
        Phase::Approach,
        VisibleTarget::at(Target::Stack, stack),
    );
    assert_eq!(fetch.tool, None);

    wait_for_job(&mut engine, person, |job| {
        chop(job) && job.fetch.is_none() && job.stage == JobStage::ToSource
    });
    let approach = row(&engine, id);
    assert_leg(
        approach,
        Action::Chop,
        Phase::Approach,
        VisibleTarget::at(Target::Tree, tree),
    );
    assert_eq!(approach.tool, Some(Tool { item: axe, uses: 0 }));
    let equip = engine
        .resources
        .visible_action_cues
        .since_tick(None)
        .find(|cue| cue.agent_id == id && cue.kind == VisibleCueKind::Equipped)
        .expect("real axe equip cue");
    assert_eq!(equip.item, CarriedItemView::of(axe, 1));
    assert_eq!(equip.target, VisibleTarget::at(Target::Stack, stack));

    wait_for_job(&mut engine, person, |job| {
        chop(job) && matches!(job.stage, JobStage::Working { .. })
    });
    let perform = row(&engine, id);
    assert_leg(
        perform,
        Action::Chop,
        Phase::Perform,
        VisibleTarget::at(Target::Tree, tree),
    );
    assert_eq!(perform.tool, approach.tool);
    assert_eq!(
        perform.started_at_tick, None,
        "claim time is not chop start"
    );

    // Read the boundary while an authoritative need has interrupted the still-held job.
    let working = *engine
        .world
        .get::<&AgentState>(person)
        .expect("working state");
    let at = colony_support::tile_of(&engine, person);
    engine.resources.water_tiles.insert(at, 1);
    engine
        .world
        .insert_one(
            person,
            AgentState::Consuming {
                target: TargetKind::Water,
            },
        )
        .expect("interrupted before job cleanup");
    assert!(job_of(&engine, person).is_some_and(chop));
    assert_leg(
        row(&engine, id),
        Action::Drink,
        Phase::Perform,
        VisibleTarget::at(Target::Water, at),
    );

    engine
        .world
        .insert_one(person, working)
        .expect("restore working state");
    assert_eq!(
        engine
            .resources
            .colony
            .reservations
            .remove(&ReserveKey::Tree(tree)),
        Some(id)
    );
    assert!(job_of(&engine, person).is_some_and(chop));
    let stale = row(&engine, id);
    assert_eq!(
        (stale.action, stale.phase, stale.target),
        (Action::Idle, Phase::Idle, None)
    );
}

#[test]
fn harness_activity_variants_and_legs_project_haul_and_transfer_cues() {
    let mut engine = job_world_m2(30, 30);
    let person = worker(&mut engine, 1, 1);
    let id = agent_id(&engine, person);
    let source = (8, 8);
    drop_items(&mut engine, source, ItemSpec::raw(ItemKind::Log, None), 3);
    stockpile(&mut engine, (20, 20));
    let haul = |job: Job| matches!(job.kind, JobKind::Haul { from, .. } if from == source);
    wait_for_job(&mut engine, person, |job| {
        haul(job) && job.stage == JobStage::ToSource
    });
    let approach = row(&engine, id);
    assert_leg(
        approach,
        Action::Carry,
        Phase::Approach,
        VisibleTarget::at(Target::Stack, source),
    );
    assert_eq!(approach.held, None);

    let job = wait_for_job(&mut engine, person, |job| {
        haul(job) && job.stage == JobStage::ToDest
    });
    let dest = job.kind.dest().expect("haul destination");
    let physical = carry_of(&engine, person).expect("picked up logs");
    let held = CarriedItemView::of(physical.spec(), physical.count);
    let transport = row(&engine, id);
    let zone = engine.resources.colony.zone_of_tile[&dest];
    assert_leg(
        transport,
        Action::Carry,
        Phase::Transport,
        VisibleTarget::identified(Target::Stockpile, dest, u64::from(zone)),
    );
    assert_eq!(transport.held, Some(held));
    let pickup_tick = engine.resources.current_tick;
    assert!(run_until(&mut engine, 1000, |e| carry_of(e, person).is_none()) < 1000);
    let stack = &engine.resources.colony.ground_items[&dest];
    assert_eq!(CarriedItemView::of(stack.spec(), stack.count), held);

    let cues: Vec<_> = engine
        .resources
        .visible_action_cues
        .since_tick(None)
        .filter(|cue| cue.agent_id == id && cue.item.item == ItemKind::Log)
        .collect();
    let [picked_up, deposited] = cues.as_slice() else {
        panic!("expected one pickup and one deposit: {cues:?}");
    };
    assert_eq!(picked_up.kind, VisibleCueKind::PickedUp);
    assert_eq!(picked_up.target.tile, Some(source));
    assert_eq!(picked_up.tick, pickup_tick);
    assert_eq!(deposited.kind, VisibleCueKind::Deposited);
    assert_eq!(deposited.target.tile, Some(dest));
    assert_eq!(deposited.tick, engine.resources.current_tick);
    assert_eq!((picked_up.item, deposited.item), (held, held));
    assert!(picked_up.tick < deposited.tick);
}

#[test]
fn harness_activity_variants_and_legs_project_a_moving_hunt_target() {
    let mut engine = sim_test::colony_engine(1);
    tick_checked(&mut engine);
    let folk = adults(&engine);
    let (id, hunter) = folk[0];
    let tracking = knowledge::id("tracking").expect("tracking");
    for (_, person) in folk {
        let mut know = engine
            .world
            .get::<&Knowledge>(person)
            .map(|k| (*k).clone())
            .unwrap_or_default();
        know.set(
            tracking,
            if person == hunter { 800 } else { 0 },
            Source::Seeded,
            0,
        );
        engine
            .world
            .insert_one(person, know)
            .expect("fixture knowledge");
    }
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
    AnimalSystem.tick(&mut engine.world, &mut engine.resources);
    let wild = &mut engine.resources.colony.wildlife;
    let deer = wild.deer().next().expect("generated deer").id;
    wild.animals.retain(|&id, _| id == deer);
    issue(&mut engine, Command::DesignateHunt { animal: deer });

    let mut first_tile = None;
    for _ in 0..4320 {
        tick_checked(&mut engine);
        if !job_of(&engine, hunter).is_some_and(|job| job.kind == (JobKind::Hunt { animal: deer }))
        {
            continue;
        }
        let at = engine.resources.colony.wildlife.animals[&deer].pos;
        let view = row(&engine, id);
        assert_eq!(view.action, Action::Hunt);
        assert_eq!(
            view.target,
            Some(VisibleTarget::identified(
                Target::WildAnimal,
                at,
                u64::from(deer)
            ))
        );
        if view.phase == Phase::Approach {
            assert_eq!(view.goal, Some(at));
            if *first_tile.get_or_insert(at) != at {
                return;
            }
        }
    }
    panic!("hunt never approached the same deer at two different tiles");
}

fn craft(name: &str, clip: &str, passive: bool) {
    let mut engine = job_world_m2(24, 24);
    let person = worker(&mut engine, 2, 2);
    let id = agent_id(&engine, person);
    let recipe = RecipeKind::from_name(name).expect("recipe");
    let def = recipe.recipe().expect("recipe definition");
    let (workshop, at) = (1, (12, 12));
    let shop = Workshop {
        id: workshop,
        kind: def.workshop,
        top_left: at,
        size: (1, 1),
        bills: Vec::new(),
        shelf: Vec::new(),
        active: None,
    };
    let work_tile = shop.work_tile();
    engine.resources.colony.workshops.insert(workshop, shop);
    if def.food_requirements.requires_lit_hearth {
        let mut camp = Settlement::new_with_id(1, 0);
        camp.formation_tile = at;
        engine.resources.settlements.insert(1, camp);
        engine.resources.fires.insert(1, FireState { lit: true });
    }
    for (&(kind, count), x) in def.inputs.iter().zip(7..) {
        create_ground(
            &mut engine.resources,
            (x, 7),
            ItemSpec::raw(kind, None),
            count,
            0,
            FoodSource::Debug,
        )
        .expect("fixture input");
    }
    issue(
        &mut engine,
        Command::AddBill {
            workshop,
            recipe,
            mode: BillMode::Make(1),
        },
    );
    let mut performing = false;
    for _ in 0..2000 {
        tick_checked(&mut engine);
        if job_of(&engine, person).is_some_and(|job| {
            matches!(job.kind, JobKind::Craft { workshop: w, .. } if w == workshop)
                && matches!(job.stage, JobStage::Working { .. })
        }) {
            let view = row(&engine, id);
            assert_eq!(
                (view.action, view.phase, view.recipe),
                (Action::Craft, Phase::Perform, Some(recipe))
            );
            assert_eq!(
                view.target,
                Some(VisibleTarget::identified(
                    Target::Workshop,
                    work_tile,
                    u64::from(workshop)
                ))
            );
            let catalogue = sim_core::colony::content::installed_or_bundled();
            let visual = catalogue
                .action_visuals()
                .resolve(view.action, view.phase, view.recipe);
            assert_eq!(
                (visual.key.as_str(), visual.fallback.as_str()),
                (clip, "work")
            );
            performing = true;
        }
        if engine.resources.colony.workshops[&workshop].bills[0].done == 1 {
            assert!(performing, "{name} completed without observing work");
            if passive {
                let [batch] = engine.resources.food.pending_drying.as_slice() else {
                    panic!("expected one pending batch");
                };
                assert_eq!(
                    (batch.workshop, batch.output.kind),
                    (workshop, def.output.0)
                );
                assert!(batch.due_tick > engine.resources.current_tick);
                assert_eq!(
                    (row(&engine, id).action, row(&engine, id).recipe),
                    (Action::Idle, None)
                );
            }
            return;
        }
    }
    panic!("{name} bill did not finish");
}

#[test]
fn harness_activity_variants_and_legs_project_cooking_crafting_and_passive_drying() {
    craft("roast_meat", "cook", false);
    craft("knap_blade", "craft", false);
    craft("dry_meat", "craft", true);
}
