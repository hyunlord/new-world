//! Actual loose-haul regression and frozen pre-repair output oracle.
use super::*;
use sim_core::colony::{ItemStack, Zone, ZoneKind};
use sim_core::nav::CostMap;
use sim_core::rules::WorldRules;
use sim_core::MaterialRegistry;
use std::collections::BTreeSet;

fn scene() -> SimResources {
    let mut res = SimResources::new(40, 12, MaterialRegistry::new());
    res.rules = WorldRules::colony(0);
    res.rules.colony.as_mut().unwrap().job_scan_radius = 4;
    res.nav.install(CostMap::new(40, 12));
    res
}
fn spec(name: &str) -> ItemSpec {
    ItemSpec::raw(ItemKind::from_name(name).expect("bundled item"), None)
}
fn stack(res: &mut SimResources, tile: (u32, u32), name: &str) {
    res.colony
        .ground_items
        .insert(tile, ItemStack::of(spec(name), 2));
}
fn cell(res: &mut SimResources, tile: (u32, u32)) {
    let id = res.colony.zones.len() as u32 + 1;
    res.colony.zones.insert(
        id,
        Zone {
            id,
            kind: ZoneKind::Stockpile,
            tiles: BTreeSet::from([tile]),
        },
    );
    res.colony.zone_of_tile.insert(tile, id);
}
fn refresh(res: &mut SimResources) {
    res.colony.touch(false);
    res.spatial.colony.refresh(&res.colony);
}
fn run(
    res: &SimResources,
    pos: (u32, u32),
    ex: &mut Exhausted,
    fuel: bool,
    frozen: bool,
) -> Option<Pick> {
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
    if frozen {
        frozen_loose_haul(res, &ctx, pos, ex, fuel)
    } else {
        loose_haul(res, &ctx, pos, ex, fuel)
    }
}

#[test]
fn exhausted_destination_invalidates_prior_source_sightings_in_both_modes() {
    let mut res = scene();
    stack(&mut res, (3, 2), "firewood");
    refresh(&mut res);
    let mut old = Exhausted::default();
    old.loose.seen = true;
    old.loose_fuel.seen = true;
    let mut actual = old.clone();
    assert_eq!(run(&res, (2, 2), &mut old, false, true), None);
    assert!(
        !old.loose.empty,
        "frozen path retains the stale positive memo"
    );
    assert_eq!(run(&res, (2, 2), &mut actual, false, false), None);
    assert!(actual.open_cell[&spec("firewood")].empty);
    assert!(
        actual.loose.empty,
        "a destination-exhausted spec must permit a fresh global absence proof"
    );
    assert!(
        !actual.loose_fuel.seen,
        "the shared blocked-spec set narrowed fuel search too"
    );
    assert_eq!(run(&res, (2, 2), &mut actual, true, false), None);
    assert!(actual.loose_fuel.empty);
    for x in 0..16 {
        assert_eq!(run(&res, (x, 2), &mut actual, false, false), None);
    }
}

#[test]
fn bounded_miss_keeps_far_nonfuel_source_after_fuel_spec_is_blocked() {
    let mut res = scene();
    stack(&mut res, (3, 2), "firewood");
    stack(&mut res, (24, 2), "roots");
    stack(&mut res, (26, 2), "roots");
    cell(&mut res, (26, 2));
    refresh(&mut res);
    let mut actual = Exhausted::default();
    let mut old = Exhausted::default();
    for (pos, fuel) in [((2, 2), true), ((2, 2), false), ((23, 2), false)] {
        let expected = run(&res, pos, &mut old, fuel, true);
        let got = run(&res, pos, &mut actual, fuel, false);
        assert_eq!(got, expected, "position={pos:?} fuel={fuel}");
        if pos == (2, 2) && !fuel {
            assert!(!actual.loose.empty);
        }
        if pos == (23, 2) {
            assert!(matches!(
                got,
                Some((
                    JobKind::Haul {
                        from: (24, 2),
                        to: (26, 2)
                    },
                    2,
                    _
                ))
            ));
        }
    }
}

#[test]
fn mixed_specs_reserved_cells_and_regions_preserve_exact_pick_sequence() {
    let mut res = scene();
    for y in 0..12 {
        res.nav.set_cost((20, y), 0);
    }
    for (tile, name) in [
        ((3, 2), "firewood"),
        ((2, 3), "roots"),
        ((30, 2), "roots"),
        ((32, 2), "firewood"),
    ] {
        stack(&mut res, tile, name);
    }
    for tile in [(4, 2), (4, 3), (31, 3)] {
        cell(&mut res, tile);
    }
    res.colony.reservations.insert(ReserveKey::Cell((4, 2)), 99);
    refresh(&mut res);
    let mut actual = BTreeMap::<u32, Exhausted>::new();
    let mut old = BTreeMap::<u32, Exhausted>::new();
    let mut successful = 0;
    for (id, pos, fuel) in [
        (1, (2, 2), true),
        (2, (2, 2), false),
        (3, (29, 2), false),
        (4, (31, 2), true),
        (5, (2, 2), false),
    ] {
        let region = res.nav.region_of(pos);
        let expected = run(&res, pos, old.entry(region).or_default(), fuel, true);
        let got = run(&res, pos, actual.entry(region).or_default(), fuel, false);
        assert_eq!(got, expected, "worker={id} position={pos:?} fuel={fuel}");
        if let Some((JobKind::Haul { from, to }, _, _)) = got {
            successful += 1;
            res.colony.reservations.insert(ReserveKey::Stack(from), id);
            res.colony.reservations.insert(ReserveKey::Cell(to), id);
        }
    }
    assert_eq!(
        successful, 2,
        "both navigation regions must retain a real haul"
    );
}

// Frozen P5 loose_haul; generic search is intentionally unchanged by this repair.
fn frozen_loose_haul(
    res: &SimResources,
    ctx: &Ctx,
    pos: (u32, u32),
    ex: &mut Exhausted,
    fuel_only: bool,
) -> Option<Pick> {
    let colony = &res.colony;
    let index = &res.spatial.colony;
    loop {
        let blocked: Vec<ItemSpec> = ex
            .open_cell
            .iter()
            .filter(|(_, memo)| memo.empty)
            .map(|(spec, _)| *spec)
            .collect();
        let source = if fuel_only {
            Search::LooseFuel
        } else {
            Search::Loose
        };
        let (from, ()) = search(
            &index.ground,
            pos,
            ctx.rule.job_scan_radius,
            ex.memo(source),
            |tile, _| {
                !colony.zone_of_tile.contains_key(&tile)
                    && !colony.reservations.contains_key(&ReserveKey::Stack(tile))
                    && res.nav.reaches(ctx.region, tile)
                    && colony.ground_items.get(&tile).is_some_and(|stack| {
                        stack.count > 0
                            && (!fuel_only || stack.kind.definition().fuel_units == Some(1))
                            && !blocked.contains(&stack.spec())
                    })
            },
        )?;
        let stack = colony.ground_items.get(&from)?;
        let spec = stack.spec();
        let open = |cell: (u32, u32)| {
            room_on(res, cell, spec) > 0
                && !colony.reservations.contains_key(&ReserveKey::Cell(cell))
                && res.nav.reaches(ctx.region, cell)
        };
        let cell = search(
            &index.cells,
            from,
            u32::MAX,
            ex.memo(Search::OpenCell(spec)),
            |cell, _| open(cell),
        )
        .map(|(to, ())| prefer_covered(res, ctx, from, to, &open));
        if let Some(to) = cell {
            let amount = ctx
                .rule
                .carry(spec.kind)
                .min(stack.count)
                .min(room_on(res, to, spec));
            return Some((JobKind::Haul { from, to }, amount, spec.kind));
        }
        // A full-map miss remains valid as this tick only adds reservations.
        ex.memo(Search::OpenCell(spec)).empty = true;
    }
}
