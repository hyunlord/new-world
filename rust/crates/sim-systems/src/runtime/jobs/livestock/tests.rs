//! Physical job admission, moving wildlife, atomic outputs and retained cargo.
use super::*;
use crate::runtime::food::items::{self as food_items, FoodSource};
use crate::runtime::livestock::tests::{add_captured, fixture};
use sim_core::colony::knowledge::Source;
use sim_core::colony::livestock_demographics::capture_draw;
use sim_core::colony::wildlife::{Animal, AnimalKind, AnimalState};
use sim_core::colony::{Carry, ItemSpec, ItemStack};
use sim_engine::RuntimeSystem;

fn worker(world: &mut World, species: AnimalKind, at: Tile) -> Entity {
    let p = species.definition().tameable.as_ref().expect("tameable");
    let mut knows = Knowledge::default();
    for name in std::iter::once(p.capture_knowledge.as_str()).chain(
        p.products
            .milk
            .as_ref()
            .map(|m| m.requires_knowledge.as_str()),
    ) {
        knows.set(
            defs().id_of(name).expect("knowledge"),
            1000,
            Source::Seeded,
            0,
        );
    }
    world.spawn((
        Agent { id: 7 },
        Position { x: at.0, y: at.1 },
        AgentState::Idle,
        knows,
        crate::runtime::agent::movement::MovementRng::new(7),
    ))
}
fn wild(res: &mut SimResources, kind: AnimalKind, at: Tile) -> u32 {
    let id = 1;
    res.colony.wildlife.animals.insert(
        id,
        Animal {
            id,
            kind,
            hp: kind.definition().hp.value,
            state: AnimalState::Wander,
            pos: at,
            pack: None,
            born_tick: res.current_tick,
            path: Vec::new(),
            goal: None,
            home: Some(at),
        },
    );
    res.colony.wildlife.next_id = 2;
    res.colony.wildlife.seeded = true;
    id
}
fn tool(res: &mut SimResources, species: AnimalKind, at: Tile) -> ItemKind {
    let kind = species
        .definition()
        .tameable
        .as_ref()
        .expect("tameable")
        .capture_tool
        .expect("tool");
    let today = day(res);
    food_items::create_ground(
        res,
        at,
        ItemSpec::raw(kind, None),
        1,
        today,
        FoodSource::Debug,
    )
    .expect("physical tool");
    res.colony.zone_of_tile.insert(at, 1);
    kind
}
fn command(world: &mut World, res: &mut SimResources, cmd: Command) {
    let tick = res.current_tick;
    assert!(crate::commands::apply_and_log(world, res, cmd, tick).accepted);
}
fn start_capture(world: &mut World, res: &mut SimResources, entity: Entity) {
    let at = world
        .get::<&Position>(entity)
        .map(|p| (p.x, p.y))
        .expect("position");
    let practice = world
        .get::<&Knowledge>(entity)
        .expect("knowledge")
        .practice();
    let offer =
        pick_capture(res, practice, res.nav.region_of(at), at, u32::MAX).expect("capture offer");
    assert!(start(world, res, entity, Offer::Capture(offer)));
}
fn job(world: &World, entity: Entity) -> Job {
    *world.get::<&Job>(entity).expect("job")
}
fn full_map(res: &mut SimResources) {
    let kind = ItemKind::role("default_placeholder");
    let max = super::super::items::stack_max(res, kind);
    let today = day(res);
    for y in 0..res.tile_grid.height {
        for x in 0..res.tile_grid.width {
            if super::super::items::can_hold_items(res, (x, y))
                && !res.colony.ground_items.contains_key(&(x, y))
            {
                food_items::create_ground(
                    res,
                    (x, y),
                    ItemSpec::raw(kind, None),
                    max,
                    today,
                    FoodSource::Debug,
                )
                .expect("fill map");
            }
        }
    }
}
fn free_tile(res: &mut SimResources, at: Tile) {
    let stack = res.colony.ground_items.remove(&at).expect("filled tile");
    res.colony
        .item_ledger
        .consume(stack.kind, u64::from(stack.count));
}
fn finished_step(world: &mut World, res: &mut SimResources, entity: Entity) {
    let at = world
        .get::<&Position>(entity)
        .map(|p| (p.x, p.y))
        .expect("position");
    let mut held = job(world, entity);
    held.stage = JobStage::Working { ticks_left: 0 };
    world.insert_one(entity, held).expect("working");
    step(world, res, 7, entity, held, at);
}

#[test]
fn capture_requires_logged_intent_and_reserves_real_sources() {
    let (mut w, mut r, _, species) = fixture(0);
    let e = worker(&mut w, species, (8, 8));
    let id = wild(&mut r, species, (9, 8));
    tool(&mut r, species, (8, 8));
    assert!(
        !crate::commands::apply_command(&mut w, &mut r, Command::DesignateCapture { animal: id })
            .accepted
    );
    assert!(r.livestock.capture_orders.is_empty());
    assert!(r.colony.command_log.is_empty());
    command(&mut w, &mut r, Command::DesignateCapture { animal: id });
    assert_eq!(r.livestock.capture_orders[&id].source_record, 0);
    let practice = w.get::<&Knowledge>(e).expect("knowledge").practice();
    assert!(capture_candidate(
        &r,
        practice,
        r.nav.region_of((8, 8)),
        (8, 8),
        u32::MAX,
        false
    )
    .is_none());
    start_capture(&mut w, &mut r, e);
    assert_eq!(r.colony.reservations.len(), 3);
    assert!(r.colony.reservations.contains_key(&ReserveKey::Animal(id)));
    assert!(r
        .colony
        .reservations
        .contains_key(&ReserveKey::PenCapture(1)));
    assert!(r
        .colony
        .reservations
        .contains_key(&ReserveKey::Stack((8, 8))));
    super::super::invariants::check(&w, &r).expect("exact ledger and reservations");
}

#[test]
fn moving_kid_completes_real_chase_and_an_attempt_without_policy_changes() {
    let (mut w, small, _, species) = fixture(0);
    let mut r = SimResources::new(64, 64, sim_core::MaterialRegistry::new());
    r.rules = small.rules;
    r.rules.clock_offset_ticks = 0;
    r.colony = small.colony;
    r.content_state = small.content_state;
    for site in r.colony.buildings.sites.values() {
        for piece in &site.pieces {
            let index = r.tile_grid.idx(piece.tile.0, piece.tile.1);
            r.tile_grid.wall_material[index] = Some(sim_core::colony::materials::OAK);
            r.tile_grid.set_door(
                index,
                piece.kind.definition().completion
                    == sim_core::colony::content::CompletionCategory::Door,
            );
        }
    }
    r.tile_grid.elevation.fill(200);
    r.tile_grid.terrain_type.fill(sim_core::TerrainType::Plain);
    r.building_kinds
        .insert((1, 1), sim_core::components::BuildingKind::Hearth);
    crate::runtime::nav::ensure(&mut r);
    let e = worker(&mut w, species, (23, 24));
    let id = wild(&mut r, species, (24, 24));
    let kind = tool(&mut r, species, (23, 24));
    command(&mut w, &mut r, Command::DesignateCapture { animal: id });
    start_capture(&mut w, &mut r, e);
    let initial = (24, 24);
    let mut moved = false;
    let mut worked = false;
    let mut movement = crate::runtime::agent::movement::AgentMovementSystem::new();
    let mut animals = crate::runtime::wildlife::AnimalSystem;
    let mut execute = super::super::execute::JobExecuteSystem::new();
    // The unchanged animal FSM flees four 3-tick steps, then rests 40 such steps.
    // The 120-tick natural rest window can admit the authored 60-tick capture.
    for tick in 1..=500 {
        r.current_tick = tick;
        movement.tick(&mut w, &mut r);
        execute.tick(&mut w, &mut r);
        if tick % 3 == 0 {
            animals.tick(&mut w, &mut r);
        }
        moved |= r
            .colony
            .wildlife
            .animals
            .get(&id)
            .is_some_and(|a| a.pos != initial);
        worked |= w
            .get::<&Job>(e)
            .is_ok_and(|j| matches!(j.stage, JobStage::Working { .. }));
        if w.get::<&Job>(e).is_err() {
            break;
        }
    }
    assert!(moved, "real wildlife must move while pursued");
    assert!(worked, "ordinary work stage reached");
    assert!(
        w.get::<&Job>(e).is_err(),
        "chase must complete an attempt within natural rest window"
    );
    let success = capture_draw(r.world_seed, species, id, 1)
        < species
            .definition()
            .tameable
            .as_ref()
            .expect("policy")
            .capture_success_permille;
    assert_eq!(r.livestock.animals.len(), usize::from(success));
    assert_eq!(r.colony.item_ledger.consumed_of(kind), u64::from(success));
    if !success {
        assert_eq!(r.livestock.capture_attempts[&id].completed_attempts, 1);
    }
    assert!(r.colony.reservations.is_empty());
    super::super::invariants::check(&w, &r).expect("moving job closes exact accounting");
}

#[test]
fn milk_waits_atomically_on_full_map_then_commits_once() {
    let (mut w, mut r, site, species) = fixture(0);
    let id = add_captured(&mut r, site, species, true, 30);
    r.current_tick = r.ticks_per_day;
    r.livestock.animals.get_mut(&id).expect("animal").milk_ready = true;
    let gate = pen_layout(&r, site).expect("physical pen").gates[0];
    let e = worker(&mut w, species, gate);
    let practice = w.get::<&Knowledge>(e).expect("knowledge").practice();
    let offer = pick_milk(&r, practice, r.nav.region_of(gate), gate, u32::MAX).expect("milk offer");
    assert!(start(&mut w, &mut r, e, Offer::Milk(offer)));
    full_map(&mut r);
    let before = r.colony.ground_items.clone();
    let livestock_before = r.livestock.clone();
    let food_before = r.food.clone();
    let events = r.livestock.witnesses.len();
    let milk = species
        .definition()
        .tameable
        .as_ref()
        .expect("policy")
        .products
        .milk
        .as_ref()
        .expect("milk")
        .item;
    for _ in 0..3 {
        finished_step(&mut w, &mut r, e);
    }
    assert_eq!(r.colony.ground_items, before);
    assert_eq!(r.livestock, livestock_before);
    assert_eq!(r.food, food_before);
    assert!(r.livestock.animals[&id].milk_ready);
    assert_eq!(r.livestock.witnesses.len(), events);
    assert_eq!(r.colony.item_ledger.created_of(milk), 0);
    assert!(w.get::<&Job>(e).is_ok());
    assert_eq!(r.colony.reservations.len(), 1);
    free_tile(&mut r, (8, 8));
    finished_step(&mut w, &mut r, e);
    assert!(w.get::<&Job>(e).is_err());
    assert!(!r.livestock.animals[&id].milk_ready);
    assert_eq!(r.colony.item_ledger.created_of(milk), 1);
    assert_eq!(r.livestock.witnesses.len(), events + 1);
    super::super::invariants::check(&w, &r).expect("milk accounting");
}

#[test]
fn slaughter_requires_command_and_waits_for_both_products() {
    let (mut w, mut r, site, species) = fixture(0);
    let id = add_captured(&mut r, site, species, true, 30);
    let gate = pen_layout(&r, site).expect("pen").gates[0];
    let e = worker(&mut w, species, gate);
    assert!(pick_slaughter(&r, u128::MAX, r.nav.region_of(gate), gate, u32::MAX).is_none());
    assert!(
        !crate::commands::apply_command(&mut w, &mut r, Command::DesignateSlaughter { animal: id })
            .accepted
    );
    command(&mut w, &mut r, Command::DesignateSlaughter { animal: id });
    let offer = pick_slaughter(&r, u128::MAX, r.nav.region_of(gate), gate, u32::MAX)
        .expect("explicit offer");
    assert!(start(&mut w, &mut r, e, Offer::Slaughter(offer)));
    full_map(&mut r);
    free_tile(&mut r, (8, 8));
    let before = r.colony.ground_items.clone();
    let livestock_before = r.livestock.clone();
    let food_before = r.food.clone();
    let events = r.livestock.witnesses.len();
    finished_step(&mut w, &mut r, e);
    assert_eq!(r.colony.ground_items, before);
    assert_eq!(r.livestock, livestock_before);
    assert_eq!(r.food, food_before);
    assert!(r.livestock.animals.contains_key(&id));
    assert!(r.livestock.slaughter_orders.contains_key(&id));
    assert_eq!(r.livestock.witnesses.len(), events);
    free_tile(&mut r, (9, 8));
    finished_step(&mut w, &mut r, e);
    assert!(!r.livestock.animals.contains_key(&id));
    assert!(r.livestock.slaughter_orders.is_empty());
    assert_eq!(r.livestock.witnesses.len(), events + 1);
    assert!(r.colony.reservations.is_empty());
    let p = species.definition().tameable.as_ref().expect("policy");
    assert_eq!(
        r.colony.item_ledger.created_of(p.products.meat.0),
        u64::from(p.products.meat.1)
    );
    assert_eq!(
        r.colony.item_ledger.created_of(p.products.hide.0),
        u64::from(p.products.hide.1)
    );
    super::super::invariants::check(&w, &r).expect("atomic mixed output accounting");
}

#[test]
fn capture_tool_survives_full_map_cancel_retry_and_death_is_booked_once() {
    for death in [false, true] {
        let (mut w, mut r, _, species) = fixture(0);
        let e = worker(&mut w, species, (8, 8));
        let id = wild(&mut r, species, (9, 8));
        let kind = tool(&mut r, species, (8, 8));
        command(&mut w, &mut r, Command::DesignateCapture { animal: id });
        start_capture(&mut w, &mut r, e);
        let held = job(&w, e);
        step(&mut w, &mut r, 7, e, held, (8, 8));
        assert_eq!(
            w.get::<&Carry>(e).expect("physical tool picked up").count,
            1
        );
        full_map(&mut r);
        super::super::cancel::cancel(&mut w, &mut r, e, CancelReason::Command);
        assert!(w.get::<&Job>(e).is_err());
        assert!(r.colony.reservations.is_empty());
        assert_eq!(
            w.get::<&Carry>(e).expect("full map retains tool").kind,
            kind
        );
        assert_eq!(r.colony.item_ledger.consumed_of(kind), 0);
        if death {
            super::super::cancel::on_death(&mut w, &mut r, e);
            super::super::cancel::on_death(&mut w, &mut r, e);
            assert_eq!(r.colony.item_ledger.consumed_of(kind), 1);
            assert_eq!(r.livestock.lost_capture_tools[&kind], 1);
        } else {
            free_tile(&mut r, (8, 8));
            super::super::food_transfers::retry_stranded(&mut w, &mut r);
            assert_eq!(
                r.colony.ground_items[&(8, 8)],
                ItemStack::of(ItemSpec::raw(kind, None), 1)
            );
            assert_eq!(r.colony.item_ledger.consumed_of(kind), 0);
        }
        assert!(w.get::<&Carry>(e).is_err());
        super::super::invariants::check(&w, &r).expect("retained tool exact accounting");
    }
}

#[test]
fn broken_pen_does_not_prevent_cancel_area_withdrawing_retained_intent() {
    let (mut w, mut r, site, species) = fixture(0);
    let id = add_captured(&mut r, site, species, false, 30);
    let gate = pen_layout(&r, site).expect("pen").gates[0];
    let e = worker(&mut w, species, gate);
    command(&mut w, &mut r, Command::DesignateSlaughter { animal: id });
    let offer =
        pick_slaughter(&r, u128::MAX, r.nav.region_of(gate), gate, u32::MAX).expect("offer");
    assert!(start(&mut w, &mut r, e, Offer::Slaughter(offer)));
    let piece = r.colony.buildings.sites[&site]
        .pieces
        .iter()
        .find(|p| p.kind.definition().blocks_livestock)
        .expect("fence");
    let index = r.tile_grid.idx(piece.tile.0, piece.tile.1);
    r.tile_grid.wall_material[index] = None;
    assert!(pen_layout(&r, site).is_err());
    command(
        &mut w,
        &mut r,
        Command::CancelArea {
            x0: gate.0,
            y0: gate.1,
            x1: gate.0,
            y1: gate.1,
        },
    );
    assert!(r.livestock.slaughter_orders.is_empty());
    assert!(w.get::<&Job>(e).is_err());
    assert!(r.colony.reservations.is_empty());
    assert!(r.livestock.animals.contains_key(&id));
}

#[test]
fn completed_capture_roll_consumes_only_success_and_failure_has_stable_cooldown() {
    for success in [false, true] {
        let (mut w, mut r, _, species) = fixture(0);
        let e = worker(&mut w, species, (8, 8));
        let initial = wild(&mut r, species, (9, 8));
        let threshold = species
            .definition()
            .tameable
            .as_ref()
            .expect("policy")
            .capture_success_permille;
        // Test both keyed transaction branches; this selects no gameplay outcome or authored value.
        let id = (1..100)
            .find(|id| (capture_draw(r.world_seed, species, *id, 1) < threshold) == success)
            .expect("both keyed branches");
        let mut source = r.colony.wildlife.animals.remove(&initial).expect("source");
        source.id = id;
        r.colony.wildlife.animals.insert(id, source);
        r.colony.wildlife.next_id = id + 1;
        let kind = tool(&mut r, species, (8, 8));
        command(&mut w, &mut r, Command::DesignateCapture { animal: id });
        start_capture(&mut w, &mut r, e);
        let held = job(&w, e);
        step(&mut w, &mut r, 7, e, held, (8, 8));
        assert!(w.get::<&Carry>(e).is_ok());
        finished_step(&mut w, &mut r, e);
        assert!(w.get::<&Job>(e).is_err());
        assert_eq!(r.colony.item_ledger.consumed_of(kind), u64::from(success));
        assert_eq!(r.livestock.animals.len(), usize::from(success));
        assert_eq!(r.colony.wildlife.animals.contains_key(&id), !success);
        let completed = w
            .get::<&sim_core::colony::Skills>(e)
            .map_or(0, |s| s.completed[23]);
        assert_eq!(completed, u32::from(success));
        if success {
            assert!(r.livestock.capture_orders.is_empty());
            assert_eq!(r.livestock.witnesses.len(), 1);
        } else {
            assert!(r.livestock.capture_orders.contains_key(&id));
            assert_eq!(r.livestock.capture_attempts[&id].completed_attempts, 1);
            assert!(!r.livestock.capture_ready(id, day(&r)));
            assert!(r.livestock.witnesses.is_empty());
            let practice = w.get::<&Knowledge>(e).expect("knowledge").practice();
            for _ in 0..3 {
                assert!(
                    pick_capture(&r, practice, r.nav.region_of((8, 8)), (8, 8), u32::MAX).is_none()
                );
            }
            assert_eq!(r.livestock.capture_attempts[&id].completed_attempts, 1);
        }
        assert!(r.colony.reservations.is_empty());
        super::super::invariants::check(&w, &r).expect("both capture branches conserve tool");
    }
}

#[test]
fn cancel_area_accepts_automatic_milk_job_with_no_order_to_remove() {
    let (mut w, mut r, site, species) = fixture(0);
    let id = add_captured(&mut r, site, species, true, 30);
    r.current_tick = r.ticks_per_day;
    r.livestock.animals.get_mut(&id).expect("animal").milk_ready = true;
    let gate = pen_layout(&r, site).expect("pen").gates[0];
    let e = worker(&mut w, species, gate);
    let practice = w.get::<&Knowledge>(e).expect("knowledge").practice();
    let offer = pick_milk(&r, practice, r.nav.region_of(gate), gate, u32::MAX).expect("offer");
    assert!(start(&mut w, &mut r, e, Offer::Milk(offer)));
    command(
        &mut w,
        &mut r,
        Command::CancelArea {
            x0: gate.0,
            y0: gate.1,
            x1: gate.0,
            y1: gate.1,
        },
    );
    assert!(w.get::<&Job>(e).is_err());
    assert!(r.colony.reservations.is_empty());
    assert!(r.livestock.animals[&id].milk_ready);
}

#[test]
fn capture_admission_rejects_missing_practice_tool_maturity_damage_and_duplicate_owners() {
    let (mut w, mut r, site, species) = fixture(0);
    let e = worker(&mut w, species, (8, 8));
    let id = wild(&mut r, species, (9, 8));
    let at = (8, 8);
    let practice = w.get::<&Knowledge>(e).expect("knowledge").practice();
    let region = r.nav.region_of(at);
    command(&mut w, &mut r, Command::DesignateCapture { animal: id });
    assert!(
        pick_capture(&r, practice, region, at, u32::MAX).is_none(),
        "physical tool is required"
    );
    tool(&mut r, species, at);
    assert!(
        pick_capture(&r, 0, region, at, u32::MAX).is_none(),
        "actual practice is required"
    );
    r.current_tick = r.ticks_per_day * 30;
    assert!(
        pick_capture(&r, practice, region, at, u32::MAX).is_none(),
        "mature wild source is not capturable"
    );
    r.current_tick = 0;
    let piece = r.colony.buildings.sites[&site]
        .pieces
        .iter()
        .find(|p| p.kind.definition().blocks_livestock)
        .expect("fence");
    let index = r.tile_grid.idx(piece.tile.0, piece.tile.1);
    let saved = r.tile_grid.wall_material[index];
    r.tile_grid.wall_material[index] = None;
    assert!(
        pick_capture(&r, practice, region, at, u32::MAX).is_none(),
        "completed site without its fence is unavailable"
    );
    r.tile_grid.wall_material[index] = saved;
    start_capture(&mut w, &mut r, e);
    assert!(
        pick_capture(&r, practice, region, at, u32::MAX).is_none(),
        "one canonical source/pen owner"
    );
    let offered = match job(&w, e).kind {
        JobKind::Capture { animal, pen } => CaptureOffer {
            wild: animal,
            pen,
            gate: pen_layout(&r, pen).expect("pen").gates[0],
            from_tool: Some(at),
        },
        _ => unreachable!("capture"),
    };
    let knowledge = (*w.get::<&Knowledge>(e).expect("knowledge")).clone();
    let other = w.spawn((
        Agent { id: 8 },
        Position { x: at.0, y: at.1 },
        AgentState::Idle,
        knowledge,
    ));
    assert!(
        !start(&mut w, &mut r, other, Offer::Capture(offered)),
        "racing worker cannot steal an admitted owner"
    );
    assert!(w.get::<&Job>(other).is_err());
    super::super::invariants::check(&w, &r).expect("race preserves exact reservations");
}
