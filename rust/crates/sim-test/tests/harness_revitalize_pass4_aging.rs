//! Pass 4 Part 3: deterministic lifespans, elders and old-age replacement.
use sim_core::components::{Agent, AgentId, AgentState, Elder, Life, Position, TargetKind};
use sim_core::rules::WorldRules;
use sim_systems::runtime::agent::aging::{founder_life, newborn_life};
use sim_test::probe::fingerprint;
use sim_test::scenes::{game_v10_scene, game_v9_scene};
use std::collections::BTreeSet;

const DAY: u64 = 1440;

fn rule() -> sim_core::rules::AgingRule {
    WorldRules::game_v10().society.aging.expect("v10 aging rule")
}

#[test]
fn harness_aging_lifespans_bounded_and_deterministic() {
    let rule = rule();
    let (low, high) = (
        rule.lifespan_mean_ticks - rule.lifespan_spread_ticks,
        rule.lifespan_mean_ticks + rule.lifespan_spread_ticks,
    );
    let mut total = 0u64;
    for id in 0..1000u64 {
        let life = newborn_life(20_261_010, id, 5_000, &rule);
        assert!((low..=high).contains(&life.lifespan_ticks), "id {id}: {}", life.lifespan_ticks);
        assert_eq!(life, newborn_life(20_261_010, id, 5_000, &rule), "twin draw differs for {id}");
        assert_eq!(life.born_tick, 5_000);
        total += life.lifespan_ticks;
    }
    let mean_days = total as f64 / 1000.0 / DAY as f64;
    assert!((57.0..=59.0).contains(&mean_days), "mean lifespan {mean_days} days");
}

#[test]
fn harness_aging_founders() {
    let rule = rule();
    for seed in 0..8u64 {
        let mut engine = game_v10_scene(seed);
        let lives: Vec<Life> = engine.world.query::<(&Agent, &Life)>().iter().map(|(_, (_, l))| *l).collect();
        assert_eq!(lives.len(), 24, "every founder carries Life (seed {seed})");
        for life in &lives {
            let age = (-life.born_tick) as u64;
            assert!((6 * DAY..=40 * DAY).contains(&age), "founder age {age}");
            assert!(life.lifespan_ticks >= age + 3 * DAY, "founder would die before day 3");
        }
        let ids: Vec<AgentId> = engine.world.query::<&Agent>().iter().map(|(_, a)| a.id).collect();
        for (id, life) in ids.iter().zip(&lives) {
            assert_eq!(*life, founder_life(seed, *id, &rule));
        }
        if seed == 0 {
            for _ in 0..3 * DAY {
                engine.tick();
            }
            assert_eq!(engine.resources.tallies.deaths_old_age, 0);
        }
    }
}

#[test]
fn harness_aging_old_age_death_and_replacement() {
    let mut engine = game_v10_scene(1);
    let mut alive: BTreeSet<AgentId> = engine.world.query::<&Agent>().iter().map(|(_, a)| a.id).collect();
    let mut first_old_age = None;
    let mut birth_after = false;
    let mut cleaned = 0;
    for _ in 0..60 * DAY {
        let births = engine.resources.tallies.births;
        let old_age = engine.resources.tallies.deaths_old_age;
        engine.tick();
        let now: BTreeSet<AgentId> = engine.world.query::<&Agent>().iter().map(|(_, a)| a.id).collect();
        if engine.resources.tallies.deaths_old_age > old_age {
            first_old_age.get_or_insert(engine.current_tick());
            for gone in alive.difference(&now) {
                assert!(!engine.resources.bed_of.contains_key(gone), "bed kept for dead {gone}");
                assert!(
                    engine.resources.settlements.values().all(|s| !s.member_agents.contains(gone)),
                    "roster kept dead {gone}"
                );
                cleaned += 1;
            }
        }
        if first_old_age.is_some() && engine.resources.tallies.births > births {
            birth_after = true;
        }
        alive = now;
    }
    assert!(first_old_age.is_some(), "no old-age death in 60 days");
    assert!(cleaned >= 1);
    assert!(birth_after, "no birth after the first old-age death");
    assert_eq!(engine.resources.tallies.deaths_starvation, 0);
    assert_eq!(engine.resources.tallies.deaths_dehydration, 0);
}

#[test]
fn harness_aging_elders_stay_home() {
    let radius = rule().elder_tether_radius;
    let mut engine = game_v10_scene(5);
    while engine.resources.settlements.is_empty() {
        engine.tick();
    }
    let (home, members) = {
        let settlement = engine.resources.settlements.values().next().expect("village");
        let mut members: Vec<_> = settlement.member_agents.iter().copied().collect();
        members.sort_unstable();
        (settlement.formation_tile, members)
    };
    // Make the first six members elders, standing at the hearth.
    let elders: Vec<_> = engine
        .world
        .query::<&Agent>()
        .iter()
        .filter(|(_, a)| members[..6].contains(&a.id))
        .map(|(e, _)| e)
        .collect();
    for entity in &elders {
        engine.world.insert(*entity, (Elder, Position::new(home.0, home.1), AgentState::Idle)).expect("elder");
        let _ = engine.world.remove_one::<sim_core::components::SeekTarget>(*entity);
    }
    let mut last_idle_distance = vec![None; elders.len()];
    let mut idle_observations = 0;
    for _ in 0..3 * DAY {
        engine.tick();
        for (index, entity) in elders.iter().enumerate() {
            let Ok(state) = engine.world.get::<&AgentState>(*entity).map(|s| *s) else { continue };
            assert!(
                !matches!(state.target(), Some(TargetKind::ConstructionSite | TargetKind::GatherFood)),
                "elder entered {state:?}"
            );
            let pos = *engine.world.get::<&Position>(*entity).expect("pos");
            let distance = pos.x.abs_diff(home.0).max(pos.y.abs_diff(home.1));
            if state == AgentState::Idle {
                idle_observations += 1;
                // Inside the tether it stays inside; outside (back from a need trip) it never drifts out.
                if let Some(previous) = last_idle_distance[index] {
                    assert!(distance <= radius.max(previous), "idle elder drifted to {distance} from {previous}");
                }
                last_idle_distance[index] = Some(distance);
            } else {
                last_idle_distance[index] = None;
            }
        }
    }
    assert!(idle_observations > 1000);
    assert_eq!(engine.resources.tallies.deaths_combat, 0);
}

#[test]
fn harness_aging_v9_unchanged() {
    for (seed, expected) in ["c022f6555945fcad", "5ca9b7e597238ee3", "d8afca881c554360", "48cf47e1341f5048"]
        .into_iter()
        .enumerate()
    {
        let mut engine = game_v9_scene(seed as u64);
        assert_eq!(engine.world.query::<&Life>().iter().count(), 0);
        for _ in 0..1440 {
            engine.tick();
        }
        assert_eq!(fingerprint(&engine), expected, "seed {seed}");
        assert_eq!(engine.world.query::<&Life>().iter().count(), 0);
        assert_eq!(engine.world.query::<&Elder>().iter().count(), 0);
        assert_eq!(engine.resources.tallies.deaths_old_age, 0);
    }
}
