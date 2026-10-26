//! Fresh-world FireSystem samples on the preregistered absolute tick grid.
use super::dto::ColdRow;
use sim_core::colony::people::{Apparel, Top};
use sim_core::components::{Agent, Cold, Founding, Lineage, PorterTrip, ResourceKind, Settlement};
use sim_core::rng::{keyed_below, STREAM_APPAREL};
use sim_core::rules::WorldRules;
use sim_core::MaterialRegistry;
use sim_engine::{RuntimeSystem, SimEngine};
use sim_systems::runtime::agent_actions::clock_in_window;
use sim_systems::runtime::settlement::{firewood, FireSystem};

pub fn sample(seed: u64, tick: u64, top: Option<Top>, rows: &mut Vec<ColdRow>) {
    let mut engine = SimEngine::new(16, 16, MaterialRegistry::new());
    // Reserve allocator ID 0 without creating an extra fixture resident.
    assert_eq!(engine.resources.issue_agent_id(), 0);
    engine.resources.rules = WorldRules::colony(seed);
    engine.resources.world_seed = seed;
    engine.resources.rules.clock_offset_ticks = 0;
    engine.resources.rules.society.exposure = None;
    engine.resources.current_tick = tick;
    assert_eq!(engine.resources.ticks_per_day, 1440);
    let mut settlement = Settlement::new_with_id(1, 0);
    settlement.formation_tile = (8, 8);
    let mut entities = Vec::new();
    for id in 1..=8 {
        let entity = engine.spawn_agent(8, 8);
        assert_eq!(engine.world.get::<&Agent>(entity).expect("agent").id, id);
        engine
            .world
            .insert_one(entity, Lineage { key: id, births: 0 })
            .expect("lineage");
        if let Some(top) = top {
            engine
                .world
                .insert_one(entity, Apparel { top, since_day: 0 })
                .expect("apparel");
        }
        assert!(settlement.add_member_agent(id));
        entities.push((id, entity));
    }
    engine.resources.settlements.insert(1, settlement);
    let rule = engine
        .resources
        .rules
        .society
        .firewood
        .expect("colony fire rule");
    let season = u8::try_from(firewood::night_season(&engine.resources, &rule)).expect("season");
    let in_cold_window = clock_in_window(
        &engine.resources,
        rule.cold_start_minute,
        rule.cold_end_minute,
    );
    let burn_rate = firewood::night_rate(&engine.resources, &rule, tick);
    assert_eq!(season, 3);
    assert!(in_cold_window && burn_rate > 0);
    let (garment_arm, p) = match top {
        None => ("none", 1000),
        Some(Top::HideWrap) => ("hide_wrap", 700),
        Some(Top::FurCloak) => ("fur_cloak", 500),
    };
    assert_eq!(top.map_or(1000, Top::cold_permille), p);
    assert_eq!(engine.resources.settlements[&1].member_agents.len(), 8);
    FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    for (id, entity) in entities {
        let observed_cold = engine.world.get::<&Cold>(entity).is_ok();
        let lineage_key = engine
            .world
            .get::<&Lineage>(entity)
            .expect("lineage retained")
            .key;
        let row = ColdRow {
            seed,
            agent_id: id,
            lineage_key,
            executed_tick: tick,
            garment_arm,
            item_code: top.map(|top| top.item().code()),
            cold_permille: p,
            observed_cold,
            member: engine.resources.settlements[&1].member_agents.contains(&id),
            season,
            in_cold_window,
            burn_rate,
            no_roof: !sim_systems::runtime::rooms::roofed(&engine.resources, (8, 8)),
            wood_zero: engine.resources.settlements[&1]
                .stockpile
                .get(&ResourceKind::Wood)
                .copied()
                .unwrap_or(0)
                == 0,
            excluded_markers_absent: engine.world.get::<&Founding>(entity).is_err()
                && engine.world.get::<&PorterTrip>(entity).is_err(),
            damage_disabled: engine.resources.rules.society.exposure.is_none(),
        };
        let eligible = row.member
            && row.no_roof
            && row.wood_zero
            && row.excluded_markers_absent
            && row.damage_disabled;
        rows.push(row);
        assert!(
            eligible,
            "fixture eligibility seed={seed} tick={tick} id={id}"
        );
        assert_eq!(
            observed_cold,
            keyed_below(seed, STREAM_APPAREL, lineage_key, tick, 0, 0, 1000) < u64::from(p)
        );
    }
}
