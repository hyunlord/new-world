//! Read-only evidence search: replay the actual captured jobs, never move an entity by hand.
use super::*;
use sim_core::colony::Job;
use sim_core::components::Position;
use sim_systems::runtime::building_templates::commands;

fn inside(tile: Tile, anchor: Tile, size: Tile) -> bool {
    tile.0 >= anchor.0
        && tile.1 >= anchor.1
        && tile.0 - anchor.0 < size.0
        && tile.1 - anchor.1 < size.1
}

pub fn attach(engine: &mut SimEngine, ui: &mut serde_json::Value) {
    let bytes = sim_systems::save::save(engine).expect("immutable motion search source");
    let mut replay = sim_systems::save::load(&bytes).expect("real registered systems replay");
    replay.tick(); // The native cache test advances once before observing a person.
    let command_revision = u64::try_from(replay.resources.colony.command_log.len())
        .expect("command revision fits u64");
    let mut people: Vec<_> = replay
        .world
        .query::<(&Agent, &Position, &Job)>()
        .iter()
        .map(|(entity, (agent, pos, job))| {
            (agent.id, entity, (pos.x, pos.y), job.kind, job.taken_at)
        })
        .collect();
    people.sort_by_key(|&(id, _, _, _, _)| id);
    let id = ui["placement_template"]
        .as_str()
        .expect("selected actual template")
        .to_owned();
    let catalog = sim_core::colony::content::installed_or_bundled();
    let size = catalog
        .building_templates()
        .get(&id)
        .expect("actual template")
        .rotated_footprint(1)
        .expect("native R rotation");
    // Observation budget only; no simulation constants, jobs, commands or materials are changed.
    for ticks in 1..=16 {
        replay.tick();
        if u64::try_from(replay.resources.colony.command_log.len())
            .expect("command revision fits u64")
            != command_revision
        {
            break;
        }
        for &(_, entity, before, kind, taken_at) in &people {
            let Ok(job) = replay.world.get::<&Job>(entity) else {
                continue;
            };
            if job.kind != kind || job.taken_at != taken_at {
                continue;
            }
            let Ok(position) = replay.world.get::<&Position>(entity) else {
                continue;
            };
            let after = (position.x, position.y);
            if before == after {
                continue;
            }
            for dy in 0..size.1 {
                for dx in 0..size.0 {
                    let Some(x) = after.0.checked_sub(dx) else {
                        continue;
                    };
                    let Some(y) = after.1.checked_sub(dy) else {
                        continue;
                    };
                    let anchor = (x, y);
                    if inside(before, anchor, size) {
                        continue;
                    }
                    if ![0, 1].into_iter().all(|rotation| {
                        commands::preview(
                            &engine.world,
                            &engine.resources,
                            &id,
                            anchor,
                            rotation,
                            None,
                        )
                        .is_ok()
                    }) {
                        continue;
                    }
                    ui["placement_cursor_tile"] =
                        serde_json::json!([x + size.0 / 2, y + size.1 / 2]);
                    ui["moving_person_entity_bits"] =
                        serde_json::json!(entity.to_bits().get().to_string());
                    ui["moving_observe_ticks"] = serde_json::json!(ticks);
                    ui["moving_witness"] = serde_json::json!({"first_tile": before, "last_tile": after,
                        "command_revision": command_revision, "source": "actual_existing_job_replay"});
                    return;
                }
            }
        }
    }
    ui["moving_fixture_gap"] = serde_json::json!(
        "No admitted footprint crossing by an existing job within 16 replay ticks before any command revision. Native harness must report GAP; no artificial movement or version-changing substitute.");
}
