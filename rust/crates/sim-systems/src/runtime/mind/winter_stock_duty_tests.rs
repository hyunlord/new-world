//! R2-1A: winter stocking displaces held ordinary work while the winter store is short.
use super::harvest_stock_tests::{fixture, mind};
use crate::runtime::food::items::{self, FoodSource};
use sim_core::colony::{Designation, ItemKind, ItemSpec, Job, JobKind, JobStage, ReserveKey};
use sim_core::components::{AgentState, SeekTarget, TargetKind};

/// First tick of noon on display day 6, the first day of summer (6-day seasons, offset 0).
const SUMMER_NOON: u64 = 6 * 1440 + 720;
const STORE_CELLS: [(u32, u32); 3] = [(6, 5), (5, 6), (5, 5)];

fn holding_chop(
    kind: &str,
    per_cell: u16,
    tick: u64,
) -> (hecs::World, sim_engine::SimResources, hecs::Entity) {
    let (mut world, mut res, entity) = fixture();
    res.current_tick = tick;
    let day = (tick / 1440) as u32;
    let zone = res.colony.zones.get_mut(&1).expect("stockpile");
    for cell in STORE_CELLS {
        zone.tiles.insert(cell);
        res.colony.zone_of_tile.insert(cell, 1);
    }
    if per_cell > 0 {
        let spec = ItemSpec::raw(ItemKind::from_name(kind).expect("food kind"), None);
        for cell in STORE_CELLS {
            items::create_ground(&mut res, cell, spec, per_cell, day, FoodSource::Debug)
                .expect("store");
        }
    }
    world
        .insert(
            entity,
            (
                Job {
                    kind: JobKind::Chop { tree: (4, 4) },
                    stage: JobStage::ToSource,
                    taken_at: tick - 20,
                    amount: 0,
                    item: ItemKind::role("default_placeholder"),
                    material: None,
                    fetch: None,
                    best_distance: 2,
                    progress_tick: tick,
                },
                AgentState::Seeking {
                    target: TargetKind::Job,
                },
                SeekTarget::new((4, 4)),
            ),
        )
        .expect("old work");
    res.colony.designations.insert((4, 4), Designation::Chop);
    res.colony.reservations.insert(ReserveKey::Tree((4, 4)), 7);
    res.colony.touch(false);
    res.spatial.colony.refresh(&res.colony);
    (world, res, entity)
}

fn chooses_harvest(kind: &str, per_cell: u16, tick: u64) -> bool {
    let (mut world, mut res, entity) = holding_chop(kind, per_cell, tick);
    mind().try_tick(&mut world, &mut res).expect("decision");
    let harvest = world
        .get::<&Job>(entity)
        .is_ok_and(|job| matches!(job.kind, JobKind::Harvest { .. }));
    if harvest {
        // The displaced order is released, not leaked.
        assert_eq!(res.colony.reservations.get(&ReserveKey::Tree((4, 4))), None);
    } else {
        assert!(world
            .get::<&Job>(entity)
            .is_ok_and(|job| matches!(job.kind, JobKind::Chop { tree: (4, 4) })));
        assert_eq!(
            res.colony.reservations.get(&ReserveKey::Tree((4, 4))),
            Some(&7)
        );
    }
    crate::runtime::jobs::invariants::check(&world, &res).expect("ledger and reservations");
    harvest
}

#[test]
fn summer_short_store_releases_a_fed_worker_from_held_work() {
    // Store 1 root against one adult's target 2.5 x 2.4 + 6 x 2.4 = 20.4 units.
    assert!(chooses_harvest("roots", 0, SUMMER_NOON));
}

#[test]
fn spring_short_store_keeps_held_work() {
    // Spring has no winter-preparation part; P20's duty protection stands.
    assert!(!chooses_harvest("roots", 0, 720));
}

#[test]
fn a_lasting_winter_store_keeps_held_work() {
    // 30 hazelnuts last 18 days in a stockpile: >= 18 days to the end of winter and >= 20.4.
    assert!(!chooses_harvest("hazelnuts", 10, SUMMER_NOON));
}
