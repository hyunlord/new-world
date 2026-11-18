//! D-MGROW-6: the step-41 stall scene (live game world, seed 0, steward on) must not
//! spend more than 50 ms on any sim tick over three days, day boundaries included.
use sim_bridge::ffi::build_game_engine;

const DAYS: u64 = 3;
const LIMIT_US: u32 = 50_000;

#[test]
#[cfg_attr(debug_assertions, ignore = "timing budget is for release builds")]
fn mgrow_worst_tick_under_50ms_for_three_days() {
    let mut engine = build_game_engine(0);
    let mut profile = engine.new_tick_profile();
    let mut worst = (0u32, 0u64);
    for _ in 0..DAYS * 1440 {
        let tick = engine.current_tick();
        engine.tick_profiled(&mut profile);
        let us = *profile.tick_us.last().expect("profiled tick");
        if us > worst.0 {
            worst = (us, tick);
        }
        if us > LIMIT_US || tick.is_multiple_of(1440) {
            let mut slow: Vec<(u64, &str)> = profile
                .last_tick_ns
                .iter()
                .zip(&profile.systems)
                .filter(|(ns, _)| **ns > 1_000_000)
                .map(|(ns, s)| (*ns / 1000, s.name.as_str()))
                .collect();
            slow.sort_unstable_by(|a, b| b.cmp(a));
            println!("tick {tick}: {us} us; systems >1ms (us): {slow:?}");
        }
    }
    println!("worst tick {}: {} us", worst.1, worst.0);
    assert!(
        worst.0 < LIMIT_US,
        "worst tick {} took {} us",
        worst.1,
        worst.0
    );
}
