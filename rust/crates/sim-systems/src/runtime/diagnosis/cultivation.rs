//! Read-only field prerequisites, sharing sowing/seed admission with field jobs.
use sim_core::colony::content::installed_or_bundled;
use sim_core::colony::cultivation::{FieldPhase, FieldWork};
use sim_core::colony::knowledge::Knowledge;
use sim_core::components::{Agent, AgentId, Child, Position};
use sim_engine::SimResources;

#[derive(Default)]
pub(super) struct Facts {
    pub no_farmer: i64,
    pub no_seed: i64,
    pub window_closed: i64,
    pub soil_exhausted: i64,
    pub seed_voice: Option<AgentId>,
}

/// Farmer means an adult practitioner in the owning settlement, not an idle worker.
/// Seed shortage counts physical home ground seeds, including reserved stacks.
pub(super) fn collect(world: &hecs::World, res: &SimResources) -> Facts {
    let mut out = Facts::default();
    if res.cultivation.fields.is_empty() {
        return out;
    }
    let Some(rule) = res.rules.colony else {
        return out;
    };
    let adults = world.query::<(&Agent, &Position, Option<&Knowledge>)>();
    let adults: Vec<_> = adults
        .without::<&Child>()
        .iter()
        .map(|(_, (agent, pos, knowledge))| {
            (
                agent.id,
                (pos.x, pos.y),
                if rule.knowledge.is_some() {
                    knowledge.map_or(0, Knowledge::practice)
                } else {
                    u128::MAX
                },
            )
        })
        .collect();
    for (tile, field) in &res.cultivation.fields {
        if !field.active
            || field.work == FieldWork::Paused
            || !crate::runtime::cultivation::uncovered_soil(res, *tile)
        {
            continue;
        }
        let Some(def) = installed_or_bundled().crop(field.crop) else {
            continue;
        };
        let mask = crate::runtime::cultivation::knowledge_mask(field.crop);
        let farmers: Vec<_> = adults
            .iter()
            .filter(|(id, _, practice)| {
                mask & !practice == 0
                    && res
                        .settlements
                        .get(&field.owner)
                        .is_some_and(|s| s.member_agents.contains(id))
            })
            .collect();
        if farmers.is_empty() {
            out.no_farmer += 1;
        }
        if !matches!(
            field.phase,
            FieldPhase::Untilled
                | FieldPhase::Tilled
                | FieldPhase::Harvested { .. }
                | FieldPhase::Fallow
        ) {
            continue;
        }
        let window = crate::runtime::cultivation::sowing_window(res, field.crop);
        let soil = res
            .cultivation
            .fertility
            .get(tile)
            .copied()
            .unwrap_or(def.initial_fertility);
        if !window {
            out.window_closed += 1;
        }
        if soil < def.min_fertility {
            out.soil_exhausted += 1;
        }
        if !window || soil < def.min_fertility {
            continue;
        }
        if crate::runtime::cultivation::ground_seed_count(res, field.owner, field.crop)
            < u64::from(def.seeds_per_tile)
        {
            out.no_seed += 1;
            let voice = adults
                .iter()
                .filter(|(id, _, _)| {
                    res.settlements
                        .get(&field.owner)
                        .is_some_and(|s| s.member_agents.contains(id))
                })
                .map(|(id, _, _)| *id)
                .min();
            out.seed_voice = out.seed_voice.into_iter().chain(voice).min();
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::colony::cultivation::FieldTile;

    fn scene() -> SimResources {
        let mut res = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        res.rules = sim_core::rules::WorldRules::colony(0);
        res.rules.clock_offset_ticks = 0;
        res.cultivation.fields.insert(
            (2, 2),
            FieldTile {
                owner: 1,
                crop: installed_or_bundled().crop_by_name("emmer").expect("emmer"),
                phase: FieldPhase::Tilled,
                active: true,
                work: FieldWork::Auto,
                tended: false,
            },
        );
        res
    }

    #[test]
    fn missing_seeds_and_farmer_are_independent_physical_facts() {
        let res = scene();
        let world = hecs::World::new();
        let facts = collect(&world, &res);
        assert_eq!(
            (
                facts.no_farmer,
                facts.no_seed,
                facts.window_closed,
                facts.soil_exhausted
            ),
            (1, 1, 0, 0)
        );
        assert_eq!(facts.seed_voice, None);
        assert_eq!(res.cultivation.fields[&(2, 2)].phase, FieldPhase::Tilled);
        assert_eq!(res.cultivation.counters.sowings, 0);
    }

    #[test]
    fn closed_window_and_exhausted_soil_do_not_request_seeds() {
        let mut res = scene();
        res.current_tick = res.ticks_per_day * 3;
        res.cultivation.fertility.insert((2, 2), 0);
        let facts = collect(&hecs::World::new(), &res);
        assert_eq!(
            (facts.window_closed, facts.soil_exhausted, facts.no_seed),
            (1, 1, 0)
        );
    }

    #[test]
    fn covered_plot_does_not_request_seed_or_farmer() {
        let mut res = scene();
        let index = res.tile_grid.idx(2, 2);
        res.tile_grid.outdoor[index] = false;
        let facts = collect(&hecs::World::new(), &res);
        assert_eq!((facts.no_farmer, facts.no_seed), (0, 0));
    }

    #[test]
    fn inactive_or_paused_fields_do_not_complain() {
        let mut res = scene();
        for (active, work) in [(false, FieldWork::Auto), (true, FieldWork::Paused)] {
            let field = res.cultivation.fields.get_mut(&(2, 2)).expect("field");
            field.active = active;
            field.work = work;
            let facts = collect(&hecs::World::new(), &res);
            assert_eq!(
                (
                    facts.no_farmer,
                    facts.no_seed,
                    facts.window_closed,
                    facts.soil_exhausted
                ),
                (0, 0, 0, 0)
            );
        }
    }
}
