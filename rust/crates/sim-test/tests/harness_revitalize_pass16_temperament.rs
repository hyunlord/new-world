//! Pass 16: temperament — every resident of the baseline world carries a
//! temperament keyed on its lineage, and its loneliness rate is the RD lean.
//!
//! V8-0 port (R3): the `game_v20` / `game_v20_inert` / `game_v19` scenes are
//! gone; the test runs `sim_test::baseline_engine`. Deleted:
//! `harness_v20_inert_equals_v19_at_40320_ticks` (placebo-vs-v19 identity, a
//! version comparison) and `harness_v20_diverges_and_adults_wander_by_ns`
//! (v19-vs-v20 contrast).
use sim_core::components::{Agent, Lineage, Social, Temperament};
use sim_core::rules::WorldRules;

/// Every resident of the baseline world carries a temperament keyed on its
/// lineage (founders exactly `Temperament::founder`), and its loneliness rate
/// is the RD lean of the base 0.04 — at tick zero and after 30 days, when
/// newborns carry their own.
#[test]
fn harness_v20_every_resident_has_temperament_and_rd_rate() {
    let rule = WorldRules::baseline(3)
        .agent
        .temperament
        .expect("live temperament rule");
    assert!(rule.effects);
    let mut engine = sim_test::baseline_engine(3);
    let mut founders = 0;
    for (_, (lineage, t, social)) in engine
        .world
        .query::<(&Lineage, &Temperament, &Social)>()
        .iter()
    {
        founders += 1;
        assert_eq!(*t, Temperament::founder(3, lineage.key));
        assert_eq!(
            social.growth_rate.to_bits(),
            rule.social_rate(0.04, t).to_bits()
        );
    }
    assert_eq!(founders, engine.world.query::<&Agent>().iter().count());
    // The RD lean is not constant across founders (it really leans).
    let rates: std::collections::BTreeSet<u64> = engine
        .world
        .query::<&Social>()
        .iter()
        .map(|(_, s)| s.growth_rate.to_bits())
        .collect();
    assert!(rates.len() > 1, "one loneliness rate for every founder");
    // 30 days: births happen; every newborn carries its own temperament.
    for _ in 0..43_200 {
        engine.tick();
    }
    let agents = engine.world.query::<&Agent>().iter().count();
    let with = engine
        .world
        .query::<(&Agent, &Temperament, &Social)>()
        .iter()
        .count();
    assert_eq!(agents, with);
    // Re-measured on the baseline (seed 3, 30 days, V8-0): 26 founders,
    // 8 births, 27 alive. The bar is half the measured births, so the
    // per-newborn checks below cover several newborns.
    let births = engine.resources.tallies.births;
    assert!(births >= 4, "births {births}");
    for (_, (t, social)) in engine.world.query::<(&Temperament, &Social)>().iter() {
        assert!(t.axes().iter().all(|v| *v <= 1000));
        assert_eq!(
            social.growth_rate.to_bits(),
            rule.social_rate(0.04, t).to_bits()
        );
    }
}
