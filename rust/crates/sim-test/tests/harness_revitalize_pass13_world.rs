//! Pass 13 Part 2: the settled-area floor keeps its own accumulator
//! (PASS13_GOAL.md 7.2) and the J10 fork properties for both v19 scenes.
use sim_core::components::Settlement;
use sim_engine::{apply_noise_fork, NoiseFork, RuntimeSystem, SimEngine};
use sim_systems::runtime::resource_regen::{ResourceRegenSystem, REGEN_INTERVAL};
use sim_test::scenes::{game_v18_nofission_scene, game_v19_nofission_scene, game_v19_scene};
use sim_test::worldmodel::digest::state_digest;

fn tick_at(day: u64, minute: u64) -> u64 {
    day * 1440 + minute - 480
}

/// Wood on the spots of area `area`.
fn area_wood(engine: &SimEngine, area: u8) -> u32 {
    engine
        .resources
        .wood_spot_area
        .iter()
        .filter(|(_, a)| **a == area)
        .map(|(tile, _)| u32::from(engine.resources.wood_tiles.get(tile).copied().unwrap_or(0)))
        .sum()
}

#[test]
fn harness_area_floor_not_lost_to_the_cap() {
    // A settlement (one member) on region 0's centre; every spot starts the
    // winter empty and is never gathered, so the snow cap of 1 per spot binds
    // after the first units. Six winter days of regrowth.
    let spots = |e: &SimEngine, area: u8| e.resources.wood_spot_area.values().filter(|a| **a == area).count() as u64;
    let seed = (0..400u64)
        .find(|&s| {
            let e = game_v19_nofission_scene(s);
            !e.resources.wild_regions.is_empty() && spots(&e, 1) * 1000 < 800 * spots(&e, 0)
        })
        .expect("a world whose region 0 has fewer spots than the camp");
    let day = tick_at(21, 600);
    let regrow = |mut engine: SimEngine| {
        let centre = engine.resources.wild_regions[0].centre;
        let mut daughter = Settlement::new_with_id(9, 0);
        daughter.formation_tile = centre;
        daughter.add_member_agent(9_999);
        engine.resources.settlements.insert(9, daughter);
        let all: Vec<_> = engine.resources.wood_source_max.keys().copied().collect();
        for tile in &all {
            engine.resources.wood_tiles.insert(*tile, 0);
        }
        let mut system = ResourceRegenSystem;
        let mut tick = day;
        while tick < day + 6 * 1440 {
            engine.resources.current_tick = tick;
            system.tick(&mut engine.world, &mut engine.resources);
            tick += REGEN_INTERVAL;
        }
        engine
    };
    let v18 = regrow(game_v18_nofission_scene(seed));
    let v19 = regrow(game_v19_nofission_scene(seed));
    let (t18, t19) = (v18.resources.tallies, v19.resources.tallies);
    // v18: the floor's extra shares the capped accumulator and is lost at the cap.
    assert!(t18.area_floor_wood > 0);
    assert!(t18.winter_wood_lost_settled > 0, "v18 loses settled-area units to the cap");
    assert_eq!((t18.area_floor_placed, t18.area_floor_lost_at_max), (0, 0), "v18's tallies unchanged");
    // v19: every floor unit lands (spots stay below their max), none is lost.
    assert_eq!(t19.area_floor_wood, t18.area_floor_wood, "the same extra is accrued");
    assert!(t19.area_floor_placed > 0);
    assert_eq!(t19.area_floor_lost_at_max, 0, "no floor unit lost");
    assert!(area_wood(&v19, 1) > area_wood(&v18, 1), "the settled region keeps more wood");
    // The camp and the unsettled regions are untouched by the separate accumulator.
    assert_eq!(area_wood(&v19, 0), area_wood(&v18, 0));
    for area in 2..4u8 {
        assert_eq!(area_wood(&v19, area), area_wood(&v18, area));
    }
    // Without a settled region the two rules are identical.
    let empty18 = {
        let mut e = game_v18_nofission_scene(seed);
        e.resources.current_tick = day;
        ResourceRegenSystem.tick(&mut e.world, &mut e.resources);
        e.resources.wood_tiles.clone()
    };
    let empty19 = {
        let mut e = game_v19_nofission_scene(seed);
        e.resources.current_tick = day;
        ResourceRegenSystem.tick(&mut e.world, &mut e.resources);
        e.resources.wood_tiles.clone()
    };
    assert_eq!(empty18, empty19);
}

/// `state_digest` every 60 ticks over `0..=ticks`, with an optional fork
/// `(tick, replicate, set directly)` applied before that tick.
fn digests(mut engine: SimEngine, ticks: u64, fork: Option<(u64, u32, bool)>) -> Vec<u64> {
    let mut out = Vec::new();
    for t in 0..=ticks {
        if t % 60 == 0 {
            out.push(state_digest(&engine));
        }
        if t == ticks {
            break;
        }
        if let Some((tick, replicate, direct)) = fork {
            if t == tick {
                if direct {
                    engine.resources.noise_fork = Some(NoiseFork { tick, replicate });
                } else {
                    apply_noise_fork(&mut engine, replicate);
                }
            }
        }
        engine.tick();
    }
    out
}

/// Pass 13 J10: replicate 0 is the unforked run and a fork leaves the prefix
/// untouched and changes the run after it, for both v19 scenes.
#[test]
fn harness_v19_noise_fork_replicate0_and_prefix_identity() {
    for make in [game_v19_scene as fn(u64) -> SimEngine, game_v19_nofission_scene] {
        let base = digests(make(12_002), 4 * 1440, None);
        assert_eq!(base, digests(make(12_002), 4 * 1440, Some((1440, 0, true))));
        let fork = digests(make(12_002), 4 * 1440, Some((1440, 3, false)));
        let prefix = 1440 / 60;
        assert_eq!(base[..=prefix], fork[..=prefix]);
        assert!(base[prefix + 1..].iter().zip(&fork[prefix + 1..]).any(|(a, b)| a != b));
    }
}
