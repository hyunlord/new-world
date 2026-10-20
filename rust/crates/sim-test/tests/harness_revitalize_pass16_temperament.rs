//! Pass 16: temperament (`game_v20`) — the placebo equals `game_v19`, and the
//! component, the RD rate and the leans are wired where section 6 says.
use sim_core::components::{Agent, Child, Elder, Lineage, Social, Temperament};
use sim_core::rules::WorldRules;
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v19_scene, game_v20_inert_scene, game_v20_scene};

fn run(make: fn(u64) -> sim_engine::SimEngine, seed: u64, ticks: u64) -> sim_engine::SimEngine {
    let mut engine = make(seed);
    for _ in 0..ticks {
        engine.tick();
    }
    engine
}

/// Section 6.3: `game_v20_inert` and `game_v19` of seeds 0–4 reach the same
/// probe fingerprint after 40,320 ticks (28 days, before
/// any fission in any v19 seed 0–31): drawing and storing temperament (and
/// inserting the component next to `Hardiness`) changes nothing by itself.
#[test]
fn harness_v20_inert_equals_v19_at_40320_ticks() {
    let rows: Vec<_> = std::thread::scope(|scope| {
        let handles: Vec<_> = (0..5u64)
            .flat_map(|seed| {
                [
                    game_v19_scene as fn(u64) -> sim_engine::SimEngine,
                    game_v20_inert_scene,
                ]
                .into_iter()
                .map(move |make| (seed, make))
            })
            .map(|(seed, make)| {
                scope.spawn(move || {
                    let engine = run(make, seed, 40_320);
                    let temperaments = engine.world.query::<&Temperament>().iter().count();
                    (seed, fingerprint(&engine), temperaments)
                })
            })
            .collect();
        handles
            .into_iter()
            .map(|h| h.join().expect("thread"))
            .collect()
    });
    for pair in rows.chunks(2) {
        let (v19, inert) = (&pair[0], &pair[1]);
        assert_eq!(v19.0, inert.0);
        assert_eq!(v19.1, inert.1, "seed {} fingerprint", v19.0);
        assert_eq!(v19.2, 0, "v19 never stores temperament");
        assert!(inert.2 > 0, "inert stores temperament");
    }
}

/// Every resident of a v20 world carries a temperament keyed on its lineage
/// (founders exactly `Temperament::founder`), and its loneliness rate is the
/// RD lean of the base 0.04; under the placebo the rate stays 0.04 exactly.
#[test]
fn harness_v20_every_resident_has_temperament_and_rd_rate() {
    let rule = WorldRules::game_v20(3).agent.temperament.expect("v20 rule");
    for (make, effects) in [
        (game_v20_scene as fn(u64) -> sim_engine::SimEngine, true),
        (game_v20_inert_scene, false),
    ] {
        let start = make(3);
        let mut founders = 0;
        for (_, (lineage, t, social)) in start
            .world
            .query::<(&Lineage, &Temperament, &Social)>()
            .iter()
        {
            founders += 1;
            assert_eq!(*t, Temperament::founder(3, lineage.key));
            let expected = if effects {
                rule.social_rate(0.04, t)
            } else {
                0.04
            };
            assert_eq!(social.growth_rate.to_bits(), expected.to_bits());
        }
        assert_eq!(founders, start.world.query::<&Agent>().iter().count());
        // 30 days: births happen; every newborn carries its own temperament.
        let engine = run(make, 3, 43_200);
        let agents = engine.world.query::<&Agent>().iter().count();
        let with = engine
            .world
            .query::<(&Agent, &Temperament, &Social)>()
            .iter()
            .count();
        assert_eq!(agents, with);
        assert!(engine.resources.tallies.births > 0);
        for (_, (t, social)) in engine.world.query::<(&Temperament, &Social)>().iter() {
            assert!(t.axes().iter().all(|v| *v <= 1000));
            let expected = if effects {
                rule.social_rate(0.04, t)
            } else {
                0.04
            };
            assert_eq!(social.growth_rate.to_bits(), expected.to_bits());
        }
    }
}

/// The leans act: v20 diverges from v19 within the first day, and between
/// 06:00 and 18:00 Idle non-elder adults with high NS stand farther from
/// their own hearth than those with low NS (a coarse direction check over two
/// worlds and ten days; the T table measures it properly).
#[test]
fn harness_v20_diverges_and_adults_wander_by_ns() {
    let v19 = run(game_v19_scene, 1, 1_440);
    let v20 = run(game_v20_scene, 1, 1_440);
    assert_ne!(fingerprint(&v19), fingerprint(&v20));
    let (mut low, mut high) = ((0u64, 0u64), (0u64, 0u64));
    for seed in 0..2u64 {
        let mut engine = game_v20_scene(seed);
        for tick in 0..14_400u64 {
            engine.tick();
            let minute =
                (engine.resources.current_tick + engine.resources.rules.clock_offset_ticks) % 1440;
            if tick % 10 != 0 || !(360..1080).contains(&minute) {
                continue;
            }
            let homes: Vec<_> = engine
                .resources
                .settlements
                .values()
                .map(|s| (s.member_agents.clone(), s.formation_tile))
                .collect();
            for (_, (agent, t, pos, state, child, elder)) in engine
                .world
                .query::<(
                    &Agent,
                    &Temperament,
                    &sim_core::components::Position,
                    &sim_core::components::AgentState,
                    Option<&Child>,
                    Option<&Elder>,
                )>()
                .iter()
            {
                if child.is_some()
                    || elder.is_some()
                    || !matches!(state, sim_core::components::AgentState::Idle)
                {
                    continue;
                }
                let Some((_, hearth)) = homes
                    .iter()
                    .find(|(members, _)| members.contains(&agent.id))
                else {
                    continue;
                };
                let d = u64::from(pos.x.abs_diff(hearth.0).max(pos.y.abs_diff(hearth.1)));
                if t.ns < 400 {
                    low = (low.0 + d, low.1 + 1);
                } else if t.ns > 600 {
                    high = (high.0 + d, high.1 + 1);
                }
            }
        }
    }
    assert!(low.1 > 100 && high.1 > 100, "{low:?} {high:?}");
    let (low_mean, high_mean) = (low.0 as f64 / low.1 as f64, high.0 as f64 / high.1 as f64);
    assert!(
        high_mean > low_mean,
        "NS high {high_mean} vs low {low_mean}"
    );
}
