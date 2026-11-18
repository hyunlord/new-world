//! M-GROW 3.6: the settlement reputation is exactly base + its weighted causes, survives save/load, and drives
//! the migrant waves (refused when low, arriving when high).
use sim_core::chronicle::ChronicleKind;
use sim_core::colony::reputation;
use sim_systems::save;

fn run_days(engine: &mut sim_engine::SimEngine, days: u64) {
    for _ in 0..days * engine.resources.ticks_per_day {
        engine.tick();
    }
}

#[test]
fn reputation_cause_sum() {
    let data = reputation::data().expect("reputation.ron");
    let mut engine = sim_test::colony_engine(0);
    run_days(&mut engine, 5);
    let state = engine.resources.colony.reputation.clone();
    let value = state.value.expect("a value after the first months");
    assert_eq!(state.contributions.len(), data.causes.len());
    let weighted: i32 = state.contributions.iter().sum();
    assert_eq!(i32::from(value), i32::from(data.base) + weighted);
    for (part, cause) in state.contributions.iter().zip(&data.causes) {
        assert!(
            part.unsigned_abs() <= u32::from(cause.weight),
            "{} {part}",
            cause.id
        );
    }
    assert!(engine
        .resources
        .chronicle
        .entries()
        .iter()
        .any(|e| e.kind == ChronicleKind::ReputationChanged));
    let bytes = save::save(&mut engine).expect("save");
    let loaded = save::load(&bytes).expect("load");
    assert_eq!(loaded.resources.colony.reputation, state);
}

/// Same world, reputation pinned low or high (period pinned so the monthly update leaves it): the first fixed
/// wave (days 8–12) is refused when low and arrives when high.
#[test]
fn reputation_migration_rate() {
    let data = reputation::data().expect("reputation.ron");
    let waves = |value: u8| {
        let mut engine = sim_test::colony_engine(0);
        let rep = &mut engine.resources.colony.reputation;
        rep.value = Some(value);
        rep.contributions = vec![0; data.causes.len()];
        rep.period = u32::MAX;
        run_days(&mut engine, 14);
        (
            engine.resources.colony.neighbours.tallies.waves,
            engine.resources.colony.reputation.refused,
        )
    };
    let (low_waves, low_refused) = waves(data.refuse_below - 1);
    let (high_waves, high_refused) = waves(data.base);
    assert_eq!(low_waves, 0);
    assert!(low_refused >= 1, "the first wave is refused");
    assert!(high_waves >= 1, "the first wave arrives");
    assert_eq!(high_refused, 0);
}
