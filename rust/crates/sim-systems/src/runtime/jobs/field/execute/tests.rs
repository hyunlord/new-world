use super::*;
use sim_core::colony::cultivation::FieldTile;
use sim_core::components::{Agent, Position, Settlement};
fn scene(crop: &str, phase: FieldPhase) -> (World, SimResources, Entity, Job) {
    let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
    res.rules = sim_core::rules::WorldRules::colony(0);
    res.rules.clock_offset_ticks = 0;
    res.nav.install(sim_core::nav::CostMap::new(8, 8));
    let mut home = Settlement::new_with_id(1, 0);
    home.member_agents.insert(7);
    res.settlements.insert(1, home);
    let crop = installed_or_bundled().crop_by_name(crop).expect("crop");
    let def = installed_or_bundled().crop(crop).unwrap();
    res.cultivation.fields.insert(
        (2, 2),
        FieldTile {
            owner: 1,
            crop,
            phase,
            active: true,
            work: FieldWork::Auto,
            tended: true,
        },
    );
    let job = Job {
        kind: JobKind::SowField {
            tile: (2, 2),
            from: (1, 1),
        },
        stage: JobStage::Working { ticks_left: 1 },
        taken_at: 0,
        amount: def.seeds_per_tile,
        item: def.seed_item,
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: 0,
    };
    let mut world = World::new();
    let e = world.spawn((Agent { id: 7 }, Position { x: 2, y: 2 }, AgentState::Idle));
    (world, res, e, job)
}
#[test]
fn sow_consumes_physical_durable_seed_once_without_eating() {
    let (mut w, mut r, e, j) = scene("emmer", FieldPhase::Tilled);
    assert!(complete(&mut w, &mut r, 7, j).is_err());
    items::create_ground(
        &mut r,
        (1, 1),
        ItemSpec::raw(j.item, None),
        j.amount,
        0,
        items::FoodSource::Debug,
    )
    .unwrap();
    items::transfer(
        &mut w,
        &mut r,
        FoodOwner::Ground((1, 1)),
        FoodOwner::Carried(7),
        j.amount,
        0,
    )
    .unwrap();
    complete(&mut w, &mut r, 7, j).unwrap();
    assert!(w.get::<&Carry>(e).is_err());
    assert_eq!(r.cultivation.counters.sowings, 1);
    assert_eq!(r.cultivation.counters.seeded_food_units, 0);
    assert!(complete(&mut w, &mut r, 7, j).is_err());
}
#[test]
fn flax_harvest_commits_all_outputs_once() {
    let (mut w, mut r, _, mut j) = scene("flax", FieldPhase::Ripe { ripe_day: 0 });
    j.kind = JobKind::HarvestField { tile: (2, 2) };
    complete(&mut w, &mut r, 7, j).unwrap();
    let def = installed_or_bundled()
        .crop(r.cultivation.fields[&(2, 2)].crop)
        .unwrap();
    for y in &def.yields {
        assert_eq!(
            r.cultivation.counters.harvested_units[&y.item],
            u64::from(y.count)
        );
    }
    assert_eq!(r.cultivation.counters.harvests, 1);
    assert!(complete(&mut w, &mut r, 7, j).is_err());
}
#[test]
fn full_map_rejects_multiyield_without_partial_output() {
    let (mut w, mut r, _, mut j) = scene("flax", FieldPhase::Ripe { ripe_day: 0 });
    j.kind = JobKind::HarvestField { tile: (2, 2) };
    let stone = sim_core::colony::ItemKind::role("default_placeholder");
    for y in 0..8 {
        for x in 0..8 {
            r.colony.ground_items.insert(
                (x, y),
                sim_core::colony::ItemStack::of(ItemSpec::raw(stone, None), u16::MAX),
            );
        }
    }
    let before = r.colony.ground_items.clone();
    assert!(complete(&mut w, &mut r, 7, j).is_err());
    assert_eq!(r.colony.ground_items, before);
    assert_eq!(r.cultivation.counters.harvests, 0);
    assert!(matches!(
        r.cultivation.fields[&(2, 2)].phase,
        FieldPhase::Ripe { .. }
    ));
}
#[test]
fn cancellation_and_death_remove_pending_origin() {
    for death in [false, true] {
        let (mut w, mut r, e, mut j) = scene("emmer", FieldPhase::Tilled);
        j.kind = JobKind::Craft {
            workshop: 1,
            bill: 1,
            at: (2, 2),
        };
        w.insert_one(e, j).unwrap();
        r.cultivation.craft_origins.insert(7, OriginShare::FIELD);
        if death {
            crate::runtime::jobs::cancel::on_death(&mut w, &mut r, e);
        } else {
            cancel(&mut w, &mut r, e, CancelReason::Interrupted);
        }
        assert!(!r.cultivation.craft_origins.contains_key(&7));
    }
}

#[test]
fn automatic_fallow_tilling_waits_for_window_but_now_is_explicit() {
    let (_, mut r, _, _) = scene("emmer", FieldPhase::Fallow);
    r.current_tick = r.ticks_per_day * 6;
    let region = r.nav.region_of((2, 2));
    assert!(
        crate::runtime::jobs::field::pick(&r, 7, u128::MAX, region, (2, 2), 10, false).is_none()
    );
    r.cultivation.fields.get_mut(&(2, 2)).unwrap().work = FieldWork::Now;
    assert!(matches!(
        crate::runtime::jobs::field::pick(&r, 7, u128::MAX, region, (2, 2), 10, true),
        Some((JobKind::TillField { .. }, _, _))
    ));
    r.cultivation.fields.get_mut(&(2, 2)).unwrap().phase = FieldPhase::Tilled;
    assert!(
        crate::runtime::jobs::field::pick(&r, 7, u128::MAX, region, (2, 2), 10, true).is_none()
    );
}

#[test]
fn seed_fact_counts_reserved_home_seed_but_not_other_settlement_seed() {
    let (_, mut r, _, j) = scene("emmer", FieldPhase::Tilled);
    let crop = r.cultivation.fields[&(2, 2)].crop;
    let mut other = Settlement::new_with_id(2, 0);
    other.formation_tile = (7, 7);
    r.settlements.insert(2, other);
    items::create_ground(
        &mut r,
        (1, 1),
        ItemSpec::raw(j.item, None),
        2,
        0,
        items::FoodSource::Debug,
    )
    .unwrap();
    items::create_ground(
        &mut r,
        (7, 7),
        ItemSpec::raw(j.item, None),
        5,
        0,
        items::FoodSource::Debug,
    )
    .unwrap();
    r.colony.reservations.insert(ReserveKey::Stack((1, 1)), 99);
    assert_eq!(
        crate::runtime::cultivation::ground_seed_count(&r, 1, crop),
        2
    );
    assert_eq!(
        crate::runtime::cultivation::ground_seed_count(&r, 2, crop),
        5
    );
    let region = r.nav.region_of((2, 2));
    assert!(
        crate::runtime::jobs::field::pick(&r, 7, u128::MAX, region, (2, 2), 10, false).is_none()
    );
}

#[test]
fn covering_crop_rejects_work_before_daily_failure() {
    let (mut w, mut r, _, mut j) = scene("emmer", FieldPhase::Ripe { ripe_day: 0 });
    j.kind = JobKind::HarvestField { tile: (2, 2) };
    let idx = r.tile_grid.idx(2, 2);
    r.tile_grid.outdoor[idx] = false;
    assert!(!super::super::target_exists(&r, 7, &j));
    assert!(complete(&mut w, &mut r, 7, j).is_err());
    assert_eq!(r.cultivation.counters.harvests, 0);
    assert!(matches!(
        r.cultivation.fields[&(2, 2)].phase,
        FieldPhase::Ripe { .. }
    ));
}

#[test]
fn prior_summer_durable_seed_runs_real_sow_stages_next_spring() {
    use sim_engine::RuntimeSystem;
    let (mut w, mut r, e, mut j) = scene("emmer", FieldPhase::Tilled);
    r.rules.colony.as_mut().unwrap().knowledge = None;
    assert!(j.item.definition().food.is_none());
    items::create_ground(
        &mut r,
        (1, 1),
        ItemSpec::raw(j.item, None),
        j.amount,
        9,
        items::FoodSource::Debug,
    )
    .unwrap();
    r.current_tick = r.ticks_per_day * 24;
    j.stage = JobStage::ToSource;
    j.taken_at = r.current_tick;
    j.progress_tick = r.current_tick;
    for key in j.keys() {
        r.colony.reservations.insert(key, 7);
    }
    w.insert(
        e,
        (
            j,
            Position { x: 1, y: 1 },
            AgentState::Seeking {
                target: TargetKind::Job,
            },
            SeekTarget::new((1, 1)),
        ),
    )
    .unwrap();
    let mut system = crate::runtime::jobs::JobExecuteSystem::new();
    system.tick(&mut w, &mut r);
    assert_eq!(w.get::<&Job>(e).unwrap().stage, JobStage::ToDest);
    assert_eq!(w.get::<&Carry>(e).unwrap().count, j.amount);
    assert!(!r.colony.ground_items.contains_key(&(1, 1)));
    // Supply the arrival coordinate; the real executor owns every transfer/work transition.
    w.insert_one(e, Position { x: 2, y: 2 }).unwrap();
    r.current_tick += 1;
    system.tick(&mut w, &mut r);
    let crop = r.cultivation.fields[&(2, 2)].crop;
    let duration = installed_or_bundled().crop(crop).unwrap().sow_ticks;
    assert_eq!(
        w.get::<&Job>(e).unwrap().stage,
        JobStage::Working {
            ticks_left: duration
        }
    );
    for _ in 0..duration {
        r.current_tick += 1;
        system.tick(&mut w, &mut r);
    }
    assert!(w.get::<&Job>(e).is_err());
    assert!(w.get::<&Carry>(e).is_err());
    assert!(r.colony.reservations.is_empty());
    assert_eq!(
        r.cultivation.fields[&(2, 2)].phase,
        FieldPhase::Sown { planted_day: 24 }
    );
    assert_eq!(r.cultivation.counters.seed_units_sown[&crop], 2);
    assert_eq!(r.cultivation.counters.seeded_food_units, 0);
}

#[test]
fn emmer_harvest_separates_ten_food_grain_from_two_durable_seeds() {
    let (mut w, mut r, _, mut j) = scene("emmer", FieldPhase::Ripe { ripe_day: 4 });
    r.current_tick = r.ticks_per_day * 4;
    j.kind = JobKind::HarvestField { tile: (2, 2) };
    complete(&mut w, &mut r, 7, j).unwrap();
    let catalog = installed_or_bundled();
    let grain = catalog.item_by_name("wild_grain").unwrap();
    let seed = catalog.item_by_name("emmer_seed").unwrap();
    assert_eq!(r.cultivation.counters.harvested_units[&grain], 10);
    assert_eq!(r.cultivation.counters.harvested_units[&seed], 2);
    for (&tile, stack) in &r.colony.ground_items {
        if stack.kind == grain {
            assert_eq!(
                r.cultivation.food_origins[&FoodOwner::Ground(tile)],
                OriginShare::FIELD
            );
        }
        if stack.kind == seed {
            assert!(!r
                .cultivation
                .food_origins
                .contains_key(&FoodOwner::Ground(tile)));
        }
    }
}

fn stranded_seed_scene() -> (World, SimResources, Entity, Job) {
    let (mut w, mut r, e, j) = scene("emmer", FieldPhase::Tilled);
    items::create_ground(
        &mut r,
        (1, 1),
        ItemSpec::raw(j.item, None),
        j.amount,
        0,
        items::FoodSource::Debug,
    )
    .unwrap();
    items::transfer(
        &mut w,
        &mut r,
        FoodOwner::Ground((1, 1)),
        FoodOwner::Carried(7),
        j.amount,
        0,
    )
    .unwrap();
    let obstruction = sim_core::colony::ItemKind::role("default_placeholder");
    for y in 0..8 {
        for x in 0..8 {
            r.colony.ground_items.insert(
                (x, y),
                sim_core::colony::ItemStack::of(ItemSpec::raw(obstruction, None), 1),
            );
        }
    }
    w.insert_one(e, j).unwrap();
    for key in j.keys() {
        r.colony.reservations.insert(key, 7);
    }
    (w, r, e, j)
}

#[test]
fn packed_map_seed_cancel_retains_cargo_until_atomic_retry() {
    let (mut w, mut r, e, j) = stranded_seed_scene();
    let ground = r.colony.ground_items.clone();
    cancel(&mut w, &mut r, e, CancelReason::Interrupted);
    assert_eq!(w.get::<&Carry>(e).unwrap().count, j.amount);
    assert!(w.get::<&Job>(e).is_err());
    assert!(r.colony.reservations.is_empty());
    assert_eq!(r.colony.ground_items, ground);
    assert!(crate::runtime::jobs::invariants::ledger_of(&w, &r, j.item).balanced());
    crate::runtime::jobs::food_transfers::retry_stranded(&mut w, &mut r);
    assert_eq!(w.get::<&Carry>(e).unwrap().count, j.amount);
    r.colony.ground_items.remove(&(2, 2));
    crate::runtime::jobs::food_transfers::retry_stranded(&mut w, &mut r);
    assert!(w.get::<&Carry>(e).is_err());
    assert_eq!(r.colony.ground_total(j.item), u64::from(j.amount));
    assert!(crate::runtime::jobs::invariants::ledger_of(&w, &r, j.item).balanced());
    assert_eq!(r.cultivation.counters.seed_lost, 0);
}

#[test]
fn packed_map_seed_death_records_loss_before_entity_despawn() {
    let (mut w, mut r, e, j) = stranded_seed_scene();
    crate::runtime::jobs::cancel::on_death(&mut w, &mut r, e);
    assert!(w.get::<&Carry>(e).is_err());
    assert_eq!(r.cultivation.counters.seed_lost, u64::from(j.amount));
    assert_eq!(
        r.colony.item_ledger.consumed_of(j.item),
        u64::from(j.amount)
    );
    assert_eq!(r.food.tallies.lost_units, 0);
    w.despawn(e).unwrap();
    assert!(crate::runtime::jobs::invariants::ledger_of(&w, &r, j.item).balanced());
}
