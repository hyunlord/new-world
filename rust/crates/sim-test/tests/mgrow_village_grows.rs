//! Ordinary-steward M-GROW-a acceptance: four 120-day worlds, real growth and replay.
use sim_core::colony::building_site::BuildingStatus;
use sim_core::colony::building_template::BuildingCategory;
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::paths::STAGE_DIRT;
use sim_core::colony::steward::StewardScope;
use sim_core::colony::Command;
use sim_core::components::{Agent, Position};
use sim_engine::SimEngine;
use std::collections::BTreeMap;

type Tile = (u32, u32);

#[derive(Default, Debug)]
struct Witnesses {
    last_moves: BTreeMap<u64, u64>,
    fast_dirt_steps: usize,
    busiest_dirt_used: bool,
    busiest: Option<(Tile, u16)>,
    yard_seen: bool,
    house_seen: bool,
    seen_zones: usize,
}

fn scene(engine: &mut SimEngine, step: &str) {
    let Ok(directory) = std::env::var("MGROW_SCENE_OUT") else {
        return;
    };
    let directory = std::path::Path::new(&directory);
    std::fs::create_dir_all(directory).expect("scene directory");
    let path = directory.join(format!("seed-{}-{step}.wsave", engine.resources.world_seed));
    std::fs::write(path, sim_systems::save::save(engine).expect("scene save"))
        .expect("scene write");
}

fn positions(engine: &SimEngine) -> BTreeMap<u64, Tile> {
    engine
        .world
        .query::<(&Agent, &Position)>()
        .iter()
        .map(|(_, (agent, pos))| (agent.id, (pos.x, pos.y)))
        .collect()
}

impl Witnesses {
    fn tick(&mut self, engine: &mut SimEngine) {
        let had_house = self.house_seen;
        let had_yard = self.yard_seen;
        let had_path = self.fast_dirt_steps > 0;
        let tick = engine.current_tick();
        let before = positions(engine);
        let stages: BTreeMap<_, _> = before
            .iter()
            .map(|(&id, &tile)| {
                let index = engine.resources.tile_grid.idx(tile.0, tile.1);
                (
                    id,
                    engine
                        .resources
                        .colony
                        .path_stage
                        .get(index)
                        .copied()
                        .unwrap_or(0),
                )
            })
            .collect();
        engine.tick();
        for (id, tile) in positions(engine) {
            let Some(&from) = before.get(&id) else {
                continue;
            };
            if from == tile {
                continue;
            }
            let previous = self.last_moves.insert(id, tick);
            if stages.get(&id) == Some(&STAGE_DIRT) {
                if previous.is_some_and(|last| tick - last == 2) {
                    self.fast_dirt_steps += 1;
                }
                if self.busiest.is_some_and(|(busy, _)| busy == from) {
                    self.busiest_dirt_used = true;
                }
            }
        }
        let res = &engine.resources;
        if tick.is_multiple_of(res.ticks_per_day) {
            self.busiest = res
                .colony
                .footfall
                .iter()
                .enumerate()
                .filter(|(index, _)| res.colony.path_stage.get(*index) == Some(&STAGE_DIRT))
                .max_by_key(|(index, count)| (**count, std::cmp::Reverse(*index)))
                .map(|(index, &count)| {
                    (
                        (
                            index as u32 % res.tile_grid.width,
                            index as u32 / res.tile_grid.width,
                        ),
                        count,
                    )
                });
            let catalog = installed_or_bundled();
            self.house_seen |= res.colony.buildings.sites.values().any(|site| {
                site.status == BuildingStatus::Complete
                    && catalog
                        .building_templates()
                        .get(&site.template)
                        .is_some_and(|template| template.category == BuildingCategory::Dwelling)
                    && res.colony.buildings.site_zones.contains_key(&site.id)
            });
            self.yard_seen |= res
                .colony
                .buildings
                .house_growth
                .values()
                .any(|progress| progress.stage >= 2);
        }
        let zones = engine.resources.colony.buildings.zone_causes.len();
        if zones > self.seen_zones && self.seen_zones < 2 {
            scene(engine, &format!("zones-{zones}"));
        }
        self.seen_zones = zones;
        if self.house_seen && !had_house {
            scene(engine, "first-house");
        }
        if self.yard_seen && !had_yard {
            scene(engine, "first-yard");
        }
        if self.fast_dirt_steps > 0 && !had_path {
            scene(engine, "first-dirt-use");
        }
    }
}

fn assert_same(seed: u64, live: &mut SimEngine, other: &mut SimEngine, label: &str) {
    assert_eq!(
        sim_test::probe::colony::colony_fingerprint(live),
        sim_test::probe::colony::colony_fingerprint(other),
        "seed {seed} {label}: fingerprint"
    );
    assert_eq!(
        live.resources.colony.command_log, other.resources.colony.command_log,
        "seed {seed} {label}: commands"
    );
    assert!(
        live.resources.chronicle == other.resources.chronicle,
        "seed {seed} {label}: chronicle"
    );
    assert!(
        sim_systems::save::save(live).expect("live full save")
            == sim_systems::save::save(other).expect("other full save"),
        "seed {seed} {label}: full saved state"
    );
}

fn replay_tick(live: &SimEngine, replay: &mut SimEngine, log_index: &mut usize) {
    for record in &live.resources.colony.command_log[*log_index..] {
        assert!(
            matches!(record.command, Command::Steward { .. }),
            "acceptance must issue only the initial steward command"
        );
        assert_eq!(record.tick, replay.current_tick(), "replay command phase");
        replay
            .resources
            .steward
            .pending
            .push((record.tick, record.command.clone()));
    }
    *log_index = live.resources.colony.command_log.len();
    replay.tick();
}

fn run_seed(seed: u64) -> bool {
    let mut live = sim_test::colony_engine(seed);
    let tpd = live.resources.ticks_per_day;
    let mid = 60 * tpd;
    let end = 120 * tpd;
    assert!(
        sim_systems::commands::submit(
            &mut live,
            Command::SetSteward {
                on: true,
                scope: StewardScope::All
            }
        )
        .accepted
    );
    let mut witnesses = Witnesses::default();
    scene(&mut live, "start");
    let mut replay = sim_test::colony_engine(seed);
    assert!(
        sim_systems::commands::submit(
            &mut replay,
            Command::SetSteward {
                on: true,
                scope: StewardScope::All
            }
        )
        .accepted
    );
    replay.resources.steward.replay = true;
    let mut log_index = live.resources.colony.command_log.len();
    while live.current_tick() < mid {
        witnesses.tick(&mut live);
        replay_tick(&live, &mut replay, &mut log_index);
        if live.current_tick().is_multiple_of(12 * tpd) {
            eprintln!(
                "seed {seed} day {} zones={} sites={} house={} yard={} dirt_steps={}",
                live.current_tick() / tpd,
                live.resources.colony.buildings.zone_causes.len(),
                live.resources.colony.buildings.sites.len(),
                witnesses.house_seen,
                witnesses.yard_seen,
                witnesses.fast_dirt_steps
            );
        }
    }
    let saved = sim_systems::save::save(&mut live).expect("day60 save");
    let mut loaded = sim_systems::save::load(&saved).expect("day60 load");
    while live.current_tick() < end {
        witnesses.tick(&mut live);
        loaded.tick();
        replay_tick(&live, &mut replay, &mut log_index);
        if live.current_tick().is_multiple_of(12 * tpd) {
            eprintln!(
                "seed {seed} day {} zones={} houses={} yard={} dirt_steps={}",
                live.current_tick() / tpd,
                live.resources.colony.buildings.zone_causes.len(),
                witnesses.house_seen,
                witnesses.yard_seen,
                witnesses.fast_dirt_steps
            );
        }
    }
    assert!(
        replay.resources.steward.pending.is_empty(),
        "seed {seed}: replay commands drained"
    );
    replay.resources.steward.replay = false;
    assert_same(seed, &mut live, &mut loaded, "loaded");
    assert_same(seed, &mut live, &mut replay, "replayed");
    let causes = &live.resources.colony.buildings.zone_causes;
    let valid_causes = causes
        .values()
        .all(|cause| !cause.trigger.is_empty() && cause.people > 0);
    let passed = causes.len() >= 2
        && valid_causes
        && witnesses.house_seen
        && witnesses.yard_seen
        && witnesses.fast_dirt_steps > 0
        && witnesses.busiest_dirt_used;
    eprintln!("MGROW seed={seed} zones={} valid_causes={valid_causes} house={} yard={} fast_dirt_steps={} busiest_dirt_used={} PASS={passed} alive_end={}",
        causes.len(), witnesses.house_seen, witnesses.yard_seen,
        witnesses.fast_dirt_steps, witnesses.busiest_dirt_used, positions(&live).len());
    if let Ok(directory) = std::env::var("MGROW_SCENE_OUT") {
        let path = std::path::Path::new(&directory);
        std::fs::create_dir_all(path).expect("acceptance output directory");
        std::fs::write(path.join(format!("seed-{seed}-day60.wsave")), saved).expect("mid scene");
        std::fs::write(
            path.join(format!("seed-{seed}-day120.wsave")),
            sim_systems::save::save(&mut live).expect("final scene save"),
        )
        .expect("final scene");
    }
    passed
}

#[test]
fn mgrow_village_grows() {
    let results: Vec<_> = (0..4).map(|seed| (seed, run_seed(seed))).collect();
    assert!(
        results.iter().all(|(_, passed)| *passed),
        "growth acceptance: {results:?}"
    );
}
