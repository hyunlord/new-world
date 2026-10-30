//! Real haul selection and arrival must not age young food when separate capacity exists.
use super::*;
use crate::runtime::food::{items as food, storage};
use sim_core::colony::Carry;
use sim_core::colony::{
    food::{FoodOwner, FoodStackKey},
    Zone, ZoneKind,
};
use sim_core::components::Agent;

pub(super) fn scene() -> (World, SimResources, ItemSpec) {
    let mut res = SimResources::new(12, 12, sim_core::MaterialRegistry::new());
    res.rules = sim_core::rules::WorldRules::colony(0);
    res.current_tick = res.rules.calendar.ticks_per_day;
    res.nav.install(sim_core::nav::CostMap::new(12, 12));
    let spec = ItemSpec::raw(ItemKind::from_name("berries").expect("berries"), None);
    (World::new(), res, spec)
}
pub(super) fn cell(res: &mut SimResources, tile: (u32, u32)) {
    let id = res.colony.zones.len() as u32 + 1;
    res.colony.zones.insert(
        id,
        Zone {
            id,
            kind: ZoneKind::Stockpile,
            tiles: std::collections::BTreeSet::from([tile]),
        },
    );
    res.colony.zone_of_tile.insert(tile, id);
    res.colony.touch(false);
}
pub(super) fn put(res: &mut SimResources, tile: (u32, u32), spec: ItemSpec, born: u32) {
    food::create_ground(res, tile, spec, 2, born, food::FoodSource::Debug).expect("food");
}
fn pick(res: &SimResources, pos: (u32, u32), ex: &mut Exhausted) -> Option<Pick> {
    let ctx = Ctx {
        rule: res.rules.colony.as_ref().unwrap(),
        sites: &BTreeMap::new(),
        site_index: &ChunkIndex::new(),
        transit: &BTreeMap::new(),
        supply: &BTreeMap::new(),
        crafting: &BTreeMap::new(),
        made: &BTreeMap::new(),
        deadwood: &std::cell::OnceCell::new(),
        pot_sources: &std::cell::OnceCell::new(),
        world: &World::new(),
        by_jobs: false,
        gather_short: &std::cell::Cell::new(None),
        hearth: &std::cell::OnceCell::new(),
        region: res.nav.region_of(pos),
        practice: u32::MAX,
    };
    loose_haul(res, &ctx, pos, ex, false)
}
#[test]
fn ordinary_haul_prefers_empty_storage_to_older_nearest_stack() {
    let (mut world, mut res, spec) = scene();
    put(&mut res, (2, 2), spec, 1);
    put(&mut res, (3, 2), spec, 0);
    cell(&mut res, (3, 2));
    cell(&mut res, (7, 2));
    res.spatial.colony.refresh(&res.colony);
    let picked = pick(&res, (2, 2), &mut Exhausted::default()).expect("haul");
    assert!(
        matches!(
            picked.0,
            JobKind::Haul {
                from: (2, 2),
                to: (7, 2)
            }
        ),
        "{picked:?}"
    );
    let job = Job {
        kind: picked.0,
        stage: JobStage::ToSource,
        taken_at: res.current_tick,
        amount: picked.1,
        item: picked.2,
        material: None,
        fetch: None,
        best_distance: 0,
        progress_tick: res.current_tick,
    };
    let entity = world.spawn((
        Agent { id: 7 },
        Position::new(2, 2),
        job,
        AgentState::Seeking {
            target: TargetKind::Job,
        },
        SeekTarget::new((2, 2)),
        crate::runtime::agent::MovementRng::new(7),
    ));
    for key in job.keys() {
        res.colony.reservations.insert(key, 7);
    }
    let mut movement = crate::runtime::agent::AgentMovementSystem::new();
    let mut execute = super::super::JobExecuteSystem::new();
    for _ in 0..240 {
        res.current_tick += 1;
        movement.tick(&mut world, &mut res);
        execute.tick(&mut world, &mut res);
        if world.get::<&Job>(entity).is_err() {
            break;
        }
    }
    assert!(world.get::<&Job>(entity).is_err());
    assert_eq!(res.food.ages[&FoodOwner::Ground((7, 2))].born_day, 1);
    let expiry = spec
        .kind
        .definition()
        .food
        .as_ref()
        .unwrap()
        .lifetime_days(sim_core::colony::content::FoodStorageClass::Stockpile);
    assert_eq!(storage::spoil_day(&mut world, &mut res, expiry).unwrap(), 2);
    assert_eq!(res.colony.ground_items[&(7, 2)].count, 2);
    super::super::invariants::check(&world, &res).expect("young cohort survives old expiry");
}
#[test]
fn arrival_uses_empty_ground_before_older_pot() {
    let (mut world, mut res, spec) = scene();
    let tile = (5, 5);
    cell(&mut res, tile);
    let vessel = ItemSpec::raw(ItemKind::from_name("clay_pot").expect("pot"), None);
    food::create_ground(&mut res, tile, vessel, 1, 0, food::FoodSource::Debug).expect("vessel");
    let pot =
        storage::deploy_pot(&mut world, &mut res, FoodOwner::Ground(tile), tile, 1).expect("pot");
    put(&mut res, (1, 1), spec, 0);
    let pot_owner = FoodOwner::Pot {
        id: pot,
        key: FoodStackKey::from(spec),
    };
    food::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((1, 1)),
        pot_owner,
        2,
        1,
    )
    .expect("old store");
    put(&mut res, (2, 2), spec, 1);
    let entity = world.spawn((Agent { id: 7 },));
    food::transfer(
        &mut world,
        &mut res,
        FoodOwner::Ground((2, 2)),
        FoodOwner::Carried(7),
        2,
        1,
    )
    .expect("carry");
    let carry = *world.get::<&Carry>(entity).unwrap();
    super::super::food_transfers::arrive(
        &mut world,
        &mut res,
        entity,
        JobKind::Haul {
            from: (2, 2),
            to: tile,
        },
        tile,
        carry,
    );
    assert_eq!(food::stack(&world, &res, pot_owner).unwrap().count, 2);
    assert_eq!(res.food.ages[&FoodOwner::Ground(tile)].born_day, 1);
    super::super::invariants::check(&world, &res).expect("conservation");
}

#[test]
fn clean_miss_for_one_age_does_not_poison_other_age_or_nonfood() {
    let (_, mut res, spec) = scene();
    put(&mut res, (2, 2), spec, 1);
    put(&mut res, (2, 4), spec, 0);
    put(&mut res, (4, 2), spec, 0);
    cell(&mut res, (4, 2));
    res.spatial.colony.refresh(&res.colony);
    let mut memo = Exhausted::default();
    assert!(matches!(
        pick(&res, (2, 2), &mut memo).unwrap().0,
        JobKind::Haul {
            from: (2, 2),
            to: (4, 2)
        }
    ));
    res.colony.reservations.insert(ReserveKey::Stack((2, 2)), 7);
    assert!(matches!(
        pick(&res, (2, 4), &mut memo).unwrap().0,
        JobKind::Haul {
            from: (2, 4),
            to: (4, 2)
        }
    ));
    let (_, mut res, _) = scene();
    let log = ItemSpec::raw(ItemKind::from_name("log").unwrap(), None);
    put(&mut res, (2, 2), log, 1);
    put(&mut res, (3, 2), log, 0);
    cell(&mut res, (3, 2));
    cell(&mut res, (7, 2));
    res.spatial.colony.refresh(&res.colony);
    assert!(matches!(
        pick(&res, (2, 2), &mut Exhausted::default()).unwrap().0,
        JobKind::Haul { to: (3, 2), .. }
    ));
}

#[test]
fn harvest_handoff_uses_same_clean_destination() {
    let (mut world, mut res, spec) = scene();
    put(&mut res, (2, 2), spec, 1);
    put(&mut res, (3, 2), spec, 0);
    cell(&mut res, (3, 2));
    cell(&mut res, (7, 2));
    let entity = world.spawn((Agent { id: 7 }, Position::new(2, 2), AgentState::Idle));
    assert!(super::super::harvest_haul::start(
        &mut world,
        &mut res,
        entity,
        &[((2, 2), 2)]
    ));
    assert!(matches!(
        world.get::<&Job>(entity).unwrap().kind,
        JobKind::Haul { to: (7, 2), .. }
    ));
    super::super::invariants::check(&world, &res).expect("reserved handoff conservation");
}
