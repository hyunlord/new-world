//! B4 H: ordinary hunter hauling to a live hearth, with real movement and accounting.
#[path = "b4_haul/boundaries.rs"]
mod boundaries;
mod colony_support;
#[path = "b4_haul/continuity.rs"]
mod continuity;

use colony_support::{job_world_m2, tick_checked, worker};
use hecs::Entity;
use sim_core::colony::knowledge::{self, Knowledge, Source};
use sim_core::colony::{
    CancelReason, Carry, Command, ItemKind, Job, JobKind, JobStage, ReserveKey,
};
use sim_core::components::{Agent, AgentId, BuildingKind, Position, ResourceKind, Settlement};
use sim_core::rules::ColonyRule;
use sim_engine::SimEngine;
use sim_systems::commands::submit;
use sim_systems::runtime::jobs::{hearth, invariants};
use sim_systems::runtime::wildlife::{self, AnimalSystem};

const HEARTH: (u32, u32) = (8, 10);
const SOURCE: (u32, u32) = (20, 10);
const STOCKPILE: (u32, u32) = (31, 24);

struct Fixture {
    engine: SimEngine,
    hunter: Entity,
    id: AgentId,
}

fn fixture() -> Fixture {
    fixture_at(HEARTH, SOURCE, true)
}

fn fixture_at(hearth: (u32, u32), source: (u32, u32), mark: bool) -> Fixture {
    let mut engine = job_world_m2(32, 32);
    engine.resources.world_seed = 1;
    let mut rule = ColonyRule::m5_5();
    let mut wildlife = rule.wildlife.expect("M5.5 wildlife");
    wildlife.deer = (0, 0);
    wildlife.deer_hp = 1;
    wildlife.deer_min_hearth_distance = 0;
    wildlife.deer_flee = false;
    wildlife.hunt_hit_permille = (1000, 1000);
    wildlife.hunt_attack_ticks = 1;
    wildlife.hunter_hauls = true;
    wildlife.hunter_meat = 3;
    rule.wildlife = Some(wildlife);
    engine.resources.rules.colony = Some(rule);
    engine
        .resources
        .building_kinds
        .insert(hearth, BuildingKind::Hearth);
    let hunter = worker(&mut engine, source.0.saturating_sub(1), source.1);
    let id = engine.world.get::<&Agent>(hunter).expect("hunter").id;
    let mut knowledge = Knowledge::default();
    knowledge.set(
        knowledge::id("tracking").expect("tracking"),
        1000,
        Source::Seeded,
        0,
    );
    engine
        .world
        .insert_one(hunter, knowledge)
        .expect("tracking component");
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = hearth;
    assert!(settlement.add_member_agent(id));
    engine.resources.settlements.insert(1, settlement);
    engine.resources.colony.wildlife.seeded = true;
    let animal = wildlife::spawn_deer(&mut engine.resources, source).expect("controlled deer");
    if mark {
        assert!(submit(&mut engine, Command::DesignateHunt { animal }).accepted);
    }
    Fixture { engine, hunter, id }
}

fn meat() -> ItemKind {
    ItemKind::role("grazer_meat")
}

fn job(f: &Fixture) -> Option<Job> {
    f.engine.world.get::<&Job>(f.hunter).ok().map(|j| *j)
}

fn food(f: &Fixture) -> u32 {
    f.engine.resources.settlements[&1]
        .stockpile
        .get(&ResourceKind::Food)
        .copied()
        .unwrap_or(0)
}

fn ground(f: &Fixture, kind: ItemKind) -> u64 {
    f.engine
        .resources
        .colony
        .ground_items
        .values()
        .filter(|stack| stack.kind == kind)
        .map(|stack| u64::from(stack.count))
        .sum()
}

fn kill(f: &mut Fixture) {
    let mut observed_hunt = false;
    for _ in 0..240 {
        tick_checked(&mut f.engine);
        observed_hunt |= job(f).is_some_and(|j| matches!(j.kind, JobKind::Hunt { .. }));
        if f.engine.resources.colony.wildlife.tallies.deer_killed == 1 {
            assert!(observed_hunt, "must execute ordinary hunt before kill");
            let ledger = &f.engine.resources.colony.item_ledger;
            assert_eq!(ledger.created_of(meat()), 2);
            assert_eq!(ledger.created_of(ItemKind::Hide), 1);
            assert_eq!(food(f), 0, "kill must not credit food");
            return;
        }
    }
    panic!("controlled guaranteed-hit hunt did not kill within 240 ticks");
}

fn haul(f: &Fixture) -> ((u32, u32), (u32, u32), u16) {
    let j = job(f).expect("ordinary kill must start a haul");
    let JobKind::Haul { from, to } = j.kind else {
        panic!("expected haul, got {j:?}")
    };
    assert_eq!(j.item, meat());
    assert_eq!(j.stage, JobStage::ToSource);
    assert_eq!(from, SOURCE);
    assert_eq!(j.amount, 2);
    assert_eq!(
        f.engine
            .resources
            .colony
            .reservations
            .get(&ReserveKey::Stack(from)),
        Some(&f.id)
    );
    assert_eq!(
        f.engine
            .resources
            .colony
            .reservations
            .get(&ReserveKey::Cell(to)),
        Some(&f.id)
    );
    (from, to, j.amount)
}

fn pickup(f: &mut Fixture) {
    for _ in 0..60 {
        tick_checked(&mut f.engine);
        if f.engine.world.get::<&Carry>(f.hunter).is_ok() {
            let j = job(f).expect("carried haul");
            assert_eq!(j.stage, JobStage::ToDest);
            assert_eq!(ground(f, meat()), 0);
            assert!(!f
                .engine
                .resources
                .colony
                .reservations
                .contains_key(&ReserveKey::Stack(SOURCE)));
            assert_eq!(food(f), 0);
            return;
        }
    }
    panic!("real haul failed to pick up meat");
}

fn assert_released(f: &Fixture) {
    assert!(job(f).is_none());
    assert!(f.engine.world.get::<&Carry>(f.hunter).is_err());
    assert!(!f
        .engine
        .resources
        .colony
        .reservations
        .values()
        .any(|owner| *owner == f.id));
    invariants::check(&f.engine.world, &f.engine.resources).expect("conservation and reservations");
}

fn remove_hearth(f: &mut Fixture) {
    assert_eq!(
        f.engine.resources.building_kinds.remove(&HEARTH),
        Some(BuildingKind::Hearth)
    );
    assert!(sim_systems::worldgen::colony::hearth(&f.engine.resources).is_none());
    assert_eq!(
        hearth::hearth_tiles(&f.engine.resources),
        vec![HEARTH],
        "formation anchor survives"
    );
}

#[test]
fn hunter_without_stockpile_walks_meat_to_hearth_before_hourly_feed() {
    let mut f = fixture();
    assert!(f.engine.resources.colony.zone_of_tile.is_empty());
    kill(&mut f);
    let (_, to, _) = haul(&f);
    assert_ne!(to, SOURCE);
    assert!(to.0.abs_diff(HEARTH.0).max(to.1.abs_diff(HEARTH.1)) <= hearth::HEARTH_PILE_REACH);
    pickup(&mut f);
    let picked_at = *f.engine.world.get::<&Position>(f.hunter).expect("position");
    for _ in 0..120 {
        tick_checked(&mut f.engine);
        if job(&f).is_none() {
            break;
        }
    }
    assert_released(&f);
    let p = f.engine.world.get::<&Position>(f.hunter).expect("position");
    assert_eq!((p.x, p.y), to);
    assert_ne!(
        (p.x, p.y),
        (picked_at.x, picked_at.y),
        "must walk with the load"
    );
    drop(p);
    assert_eq!(ground(&f, meat()), 2);
    assert_eq!(ground(&f, ItemKind::Hide), 1);
    assert_eq!(
        food(&f),
        0,
        "delivery is ground storage, not immediate feed"
    );
    assert_eq!(f.engine.resources.colony.wildlife.tallies.meat_eaten, 0);
    assert_eq!(
        f.engine
            .resources
            .colony
            .wildlife
            .meat_lots
            .iter()
            .map(|(_, n)| *n)
            .sum::<u32>(),
        2
    );
    f.engine.register_system(Box::new(AnimalSystem));
    for _ in 0..61 {
        tick_checked(&mut f.engine);
        if f.engine.resources.colony.wildlife.tallies.meat_eaten > 0 {
            break;
        }
    }
    assert_eq!(f.engine.resources.colony.wildlife.tallies.meat_eaten, 2);
    let units = f
        .engine
        .resources
        .rules
        .colony
        .expect("colony")
        .wildlife
        .expect("wildlife")
        .meat_food_units;
    assert_eq!(food(&f), 2 * units);
    assert_eq!(ground(&f, meat()), 0);
    assert_eq!(ground(&f, ItemKind::Hide), 1);
    assert!(f.engine.resources.colony.wildlife.meat_lots.is_empty());
    invariants::check(&f.engine.world, &f.engine.resources).expect("fed conservation");
}

#[test]
fn hunter_keeps_stockpile_priority_even_when_hearth_is_closer() {
    let mut f = fixture();
    assert!(
        submit(
            &mut f.engine,
            Command::AddStockpile {
                x0: STOCKPILE.0,
                y0: STOCKPILE.1,
                x1: STOCKPILE.0,
                y1: STOCKPILE.1,
            }
        )
        .accepted
    );
    kill(&mut f);
    assert_eq!(haul(&f).1, STOCKPILE);
}

#[test]
fn hunter_with_no_building_hearth_keeps_meat_at_kill() {
    let mut f = fixture();
    remove_hearth(&mut f);
    kill(&mut f);
    assert_released(&f);
    assert_eq!(ground(&f, meat()), 2);
    assert_eq!(f.engine.resources.colony.wildlife.tallies.meat_hauls, 0);
}

#[test]
fn hunter_with_every_hearth_cell_occupied_keeps_meat_at_kill() {
    let mut f = fixture();
    for y in HEARTH.1 - 2..=HEARTH.1 + 2 {
        for x in HEARTH.0 - 2..=HEARTH.0 + 2 {
            if (x, y) != HEARTH {
                f.engine
                    .resources
                    .building_kinds
                    .insert((x, y), BuildingKind::Shelter);
            }
        }
    }
    kill(&mut f);
    assert_released(&f);
    assert_eq!(ground(&f, meat()), 2);
    assert_eq!(f.engine.resources.colony.wildlife.tallies.meat_hauls, 0);
}

#[test]
fn removing_last_hearth_before_pickup_cancels_without_taking_meat() {
    let mut f = fixture();
    kill(&mut f);
    haul(&f);
    remove_hearth(&mut f);
    tick_checked(&mut f.engine);
    assert_released(&f);
    assert_eq!(ground(&f, meat()), 2);
    assert_eq!(ground(&f, ItemKind::Hide), 1);
    assert_eq!(
        f.engine
            .resources
            .colony
            .tallies
            .cancels
            .get(&CancelReason::TargetGone),
        Some(&1)
    );
    assert_eq!(food(&f), 0);
}

#[test]
fn removing_last_hearth_midcarry_drops_load_and_releases_reservations() {
    let mut f = fixture();
    kill(&mut f);
    haul(&f);
    pickup(&mut f);
    remove_hearth(&mut f);
    tick_checked(&mut f.engine);
    assert_released(&f);
    assert_eq!(ground(&f, meat()), 2);
    assert_eq!(ground(&f, ItemKind::Hide), 1);
    let p = f
        .engine
        .world
        .get::<&Position>(f.hunter)
        .expect("cancel position");
    assert!(
        f.engine
            .resources
            .colony
            .ground_items
            .iter()
            .any(|(tile, stack)| {
                stack.kind == meat() && tile.0.abs_diff(p.x).max(tile.1.abs_diff(p.y)) <= 1
            }),
        "cancel drops near the actual carrier position, including a local merge/spill"
    );
    assert_eq!(
        f.engine
            .resources
            .colony
            .tallies
            .cancels
            .get(&CancelReason::TargetGone),
        Some(&1)
    );
    assert_eq!(food(&f), 0);
    assert_eq!(f.engine.resources.colony.wildlife.tallies.meat_eaten, 0);
    assert_eq!(
        f.engine
            .resources
            .colony
            .wildlife
            .meat_lots
            .iter()
            .map(|(_, n)| *n)
            .sum::<u32>(),
        2
    );
}

#[test]
fn hunter_stockpile_path_does_not_require_a_building_hearth() {
    let mut f = fixture();
    assert!(
        submit(
            &mut f.engine,
            Command::AddStockpile {
                x0: STOCKPILE.0,
                y0: STOCKPILE.1,
                x1: STOCKPILE.0,
                y1: STOCKPILE.1,
            }
        )
        .accepted
    );
    remove_hearth(&mut f);
    kill(&mut f);
    assert_eq!(haul(&f).1, STOCKPILE);
    pickup(&mut f);
    for _ in 0..180 {
        tick_checked(&mut f.engine);
        if job(&f).is_none() {
            break;
        }
    }
    assert_released(&f);
    let stack = f
        .engine
        .resources
        .colony
        .ground_items
        .get(&STOCKPILE)
        .expect("stocked meat");
    assert_eq!((stack.kind, stack.count), (meat(), 2));
    assert_eq!(food(&f), 0);
}
