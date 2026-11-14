//! D-SIMCHECK-1: a steward-OFF colony with no orders still gets M6a R2-1C's winter-food
//! fallback. With no stockpile zone, the hearth pile is its food store, so the `stock_*`
//! nodes can run and food is put by before the first winter.
//! Release-sized: three seeds × 18 days (`cargo test --release -p sim-test --test m6a_off_winter_fallback`).
use sim_core::colony::food::FoodOwner;
use sim_systems::runtime::{food, jobs, resource_regen};

const WINTER: usize = 3;

struct WinterStart {
    tick: u64,
    stock_choice_ticks: u64,
    pile_units: u64,
    reserve_units: u64,
}

fn run_to_first_winter(seed: u64) -> WinterStart {
    let mut engine = sim_test::colony_engine(seed);
    assert!(
        !engine.resources.steward.enabled,
        "seed {seed}: steward OFF"
    );
    let mut stock_choice_ticks = 0u64;
    while resource_regen::season_at(&engine.resources).map(|(s, _)| s) != Some(WINTER) {
        assert!(engine.current_tick() < 30 * 1440, "seed {seed}: no winter");
        engine.tick();
        let mind = &engine.resources.mind;
        if let Some(dictionary) = mind.history.as_ref().map(|h| h.dictionary()) {
            stock_choice_ticks += mind
                .running
                .values()
                .filter(|run| {
                    !run.finished
                        && dictionary
                            .node_label(run.node.index())
                            .is_some_and(|label| label.name.starts_with("stock_"))
                })
                .count() as u64;
        }
        if engine.current_tick().is_multiple_of(1440) {
            jobs::invariants::check(&engine.world, &engine.resources)
                .unwrap_or_else(|e| panic!("seed {seed} tick {}: {e}", engine.current_tick()));
        }
    }
    let res = &engine.resources;
    assert!(
        res.colony.zone_of_tile.is_empty(),
        "seed {seed}: no orders, no zone"
    );
    let hearths = jobs::hearth::hearth_tiles(res);
    let reach = jobs::hearth::HEARTH_PILE_REACH;
    // Independent physical count: fresh edible ground stacks on the hearth pile.
    let day = food::day(res);
    let pile_units = res
        .colony
        .ground_items
        .iter()
        .filter(|(tile, _)| {
            hearths
                .iter()
                .any(|h| tile.0.abs_diff(h.0).max(tile.1.abs_diff(h.1)) <= reach)
        })
        .filter_map(|(tile, stack)| {
            let edible = stack.kind.definition().food.as_ref()?;
            let left =
                food::storage::remaining_days(&engine.world, res, FoodOwner::Ground(*tile), day)
                    .ok()?;
            (edible.raw_edible && left > 0)
                .then(|| u64::from(edible.units) * u64::from(stack.count))
        })
        .sum();
    let reserve_units = res
        .settlements
        .keys()
        .map(|sid| food::reserves::units(&engine.world, res, *sid, false))
        .sum();
    jobs::invariants::check(&engine.world, res).expect("ledger and reservations at winter");
    WinterStart {
        // The tick just run (the engine's counter is already one ahead).
        tick: res.current_tick,
        stock_choice_ticks,
        pile_units,
        reserve_units,
    }
}

#[test]
fn off_colony_without_orders_stores_food_at_the_hearth_before_winter() {
    for seed in [0, 1, 2] {
        let w = run_to_first_winter(seed);
        // Display day 18 = raw tick 18 × 1440 − clock offset 480.
        assert_eq!(w.tick, 25_440, "seed {seed}: winter start");
        assert!(
            w.stock_choice_ticks > 0,
            "seed {seed}: no R2-1C stock_* node ran"
        );
        assert!(
            w.pile_units > 0,
            "seed {seed}: no fresh food on the hearth pile at winter"
        );
        assert_eq!(
            w.reserve_units, w.pile_units,
            "seed {seed}: the settlement reserve counts the hearth-pile store"
        );
        eprintln!(
            "seed {seed}: stock_* agent-ticks {} hearth-pile units {} reserve units {}",
            w.stock_choice_ticks, w.pile_units, w.reserve_units
        );
    }
}
