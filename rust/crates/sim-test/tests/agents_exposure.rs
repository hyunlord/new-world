//! Merged in V8-0 (8.4) from `harness_revitalize_pass10_exposure.rs`,
//! `harness_revitalize_pass11_exposure.rs`; each source is one module below,
//! unchanged.

// From `harness_revitalize_pass10_exposure.rs`.
mod by_life_stage {
    //! Pass 10 Part 1: exposure — a cold night wounds, by life stage, and health
    //! can reach zero (`SocietyRules::exposure`, on the baseline rules: exposure
    //! v2 with hardiness and the hunger weight).
    use hecs::Entity;
    use sim_core::causal::event::{CausalEvent, DeathReason};
    use sim_core::components::{
        Agent, AgentId, BodyHealth, Child, Cold, Elder, ExposureClock, Hardiness, Hunger, Position,
        Settlement, Thirst,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rules::WorldRules;
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::settlement::FireSystem;
    use sim_systems::runtime::survival::StarvationSystem;
    use sim_test::probe::fingerprint;

    /// World seed of the fixture rules (`WorldRules::baseline`).
    const SEED: u64 = 7;

    const HOME: (u32, u32) = (20, 20);

    /// Tick at which the display clock (480-tick offset) reads `minute` on the
    /// zero-based display day `day` (day index 21 = the first winter day).
    fn tick_at(day: u64, minute: u64) -> u64 {
        day * 1440 + minute - 480
    }

    /// Life stage of a fixture resident.
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum Stage {
        Child,
        Adult,
        Elder,
    }

    /// A bare world under the baseline rules at `tick`: one settlement at `HOME`
    /// with an empty wood store (the fire goes out) and one member per
    /// `(id, stage, hp)` row, spawned in the given order with its explicit
    /// AgentId, needs met (hunger factor 1) and hardiness 1000 permille
    /// (hardiness factor 1), so a cold hour costs exactly the rule's damage.
    fn exposure_fixture(
        tick: u64,
        residents: &[(AgentId, Stage, f64)],
    ) -> (SimEngine, Vec<(AgentId, Entity)>) {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        engine.resources.rules = WorldRules::baseline(SEED);
        engine.resources.world_seed = SEED;
        engine.resources.current_tick = tick;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = HOME;
        let mut entities = Vec::new();
        for &(id, stage, hp) in residents {
            let entity = engine.world.spawn((
                Position::new(HOME.0, HOME.1),
                Agent { id },
                Hunger::new(0.0, 0.0),
                Thirst::new(0.0, 0.0),
                BodyHealth { hp, max_hp: 100.0 },
                Hardiness { permille: 1000 },
            ));
            match stage {
                Stage::Child => engine
                    .world
                    .insert_one(entity, Child { born_tick: 0 })
                    .expect("child"),
                Stage::Elder => engine.world.insert_one(entity, Elder).expect("elder"),
                Stage::Adult => {}
            }
            settlement.add_member_agent(id);
            entities.push((id, entity));
        }
        engine.resources.settlements.insert(1, settlement);
        (engine, entities)
    }

    /// Run `FireSystem` once at `tick`.
    fn fire_at(engine: &mut SimEngine, tick: u64) {
        engine.resources.current_tick = tick;
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
    }

    fn hp(engine: &SimEngine, entity: Entity) -> f64 {
        engine.world.get::<&BodyHealth>(entity).expect("health").hp
    }

    /// One booked death: (event id, agent, reason, tick).
    type Death = (u64, AgentId, DeathReason, u64);

    /// Every `AgentDied` in the causal log at `HOME`, in push order: (event id, agent, reason, tick).
    fn deaths_at_home(engine: &SimEngine) -> Vec<Death> {
        let tile = HOME.1 * engine.resources.tile_grid.width + HOME.0;
        engine
            .resources
            .causal_log
            .get(tile)
            .map(|log| {
                log.as_slice()
                    .iter()
                    .filter_map(|event| match event {
                        CausalEvent::AgentDied {
                            id,
                            agent,
                            reason,
                            tick,
                            ..
                        } => Some((*id, *agent, *reason, *tick)),
                        _ => None,
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// The rule's hourly damage by stage: (child, adult, elder).
    fn damage(engine: &SimEngine) -> (f64, f64, f64) {
        let d = engine
            .resources
            .rules
            .society
            .exposure
            .expect("exposure rule")
            .damage_per_cold_hour;
        (d[0], d[1], d[2])
    }

    /// An elder with `start` hp and `hardiness` permille beside a fresh adult and
    /// child: the fire is out from 22:00; returns the cold hour (1-based) in which
    /// the elder died.
    fn elder_death_hour(start: f64, hardiness: u16) -> u64 {
        let night = tick_at(21, 1320); // 22:00 of the first winter day, fire out
        let (mut engine, residents) = exposure_fixture(
            night,
            &[
                (1, Stage::Elder, start),
                (2, Stage::Adult, 100.0),
                (3, Stage::Child, 100.0),
            ],
        );
        let (elder, adult, child) = (residents[0].1, residents[1].1, residents[2].1);
        engine
            .world
            .insert_one(
                elder,
                Hardiness {
                    permille: hardiness,
                },
            )
            .expect("elder hardiness");
        let (dc, da, de) = damage(&engine);
        let de = de * f64::from(hardiness) / 1000.0;
        for hour in 1..=8u64 {
            fire_at(&mut engine, night + 60 * (hour - 1));
            assert!(!engine.resources.fires[&1].lit);
            assert_eq!(hp(&engine, adult), 100.0 - da * hour as f64);
            assert_eq!(hp(&engine, child), 100.0 - dc * hour as f64);
            if !engine.world.contains(elder) {
                assert_eq!(engine.resources.tallies.deaths_exposure, 1);
                assert_eq!(engine.resources.tallies.cold_member_hours, 3 * hour);
                let deaths = deaths_at_home(&engine);
                assert_eq!(deaths.len(), 1);
                assert_eq!(
                    (deaths[0].1, deaths[0].2, deaths[0].3),
                    (1, DeathReason::Exposure, night + 60 * (hour - 1))
                );
                assert!(!engine.resources.settlements[&1].member_agents.contains(&1));
                // Tallied once: the next hour wounds the others but kills no one else.
                fire_at(&mut engine, night + 60 * hour);
                assert_eq!(engine.resources.tallies.deaths_exposure, 1);
                assert_eq!(
                    engine.resources.tallies.deaths_starvation
                        + engine.resources.tallies.deaths_old_age,
                    0
                );
                return hour;
            }
            assert!(engine.world.get::<&Cold>(elder).is_ok());
            assert!((hp(&engine, elder) - (start - de * hour as f64)).abs() < 1e-9);
        }
        panic!("the elder survived the night");
    }

    #[test]
    fn harness_exposure_kills_a_cold_elder() {
        // Baseline rule ([7.5, 5, 5] an hour): an elder at three hours' damage
        // dies in the third cold hour, half a point more lasts into the fourth.
        let rule = WorldRules::baseline(SEED).society.exposure.expect("rule");
        let tuned = rule.damage_per_cold_hour[2];
        assert_eq!(tuned, 5.0);
        assert!(rule.use_hardiness);
        assert_eq!(elder_death_hour(3.0 * tuned, 1000), 3);
        assert_eq!(elder_death_hour(3.0 * tuned + 0.5, 1000), 4);
        // Hardiness scales the damage: at 1300 permille (the frailest draw) the
        // same 15.5 hp lasts only into the third hour (6.5 an hour: 15.5 - 13 =
        // 2.5 > 0 after two), at 700 (the hardiest) into the fifth (3.5 an hour).
        assert_eq!(elder_death_hour(3.0 * tuned + 0.5, 1300), 3);
        assert_eq!(elder_death_hour(3.0 * tuned + 0.5, 700), 5);
    }

    #[test]
    fn harness_exposure_blocks_healing() {
        let night = tick_at(21, 1320);
        let (mut engine, residents) = exposure_fixture(night, &[(1, Stage::Adult, 100.0)]);
        let adult = residents[0].1;
        // A whole dark night: eight cold hours, 22:00 .. 05:00.
        for hour in 0..8 {
            fire_at(&mut engine, night + 60 * hour);
        }
        let (_, da, _) = damage(&engine);
        let after_night = 100.0 - 8.0 * da;
        assert_eq!(hp(&engine, adult), after_night);
        let last_cold = night + 60 * 7;
        assert_eq!(
            engine
                .world
                .get::<&ExposureClock>(adult)
                .expect("clock")
                .last_cold_tick,
            last_cold
        );
        // While Cold: no healing even though both needs are met.
        let mut survival = StarvationSystem::new();
        engine.resources.current_tick = last_cold + 30;
        survival.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(hp(&engine, adult), after_night);
        // 06:00: the cold window ends, the marker leaves.
        fire_at(&mut engine, last_cold + 60);
        assert!(engine.world.get::<&Cold>(adult).is_err());
        assert_eq!(
            engine
                .world
                .get::<&ExposureClock>(adult)
                .expect("clock")
                .last_cold_tick,
            last_cold
        );
        // Not Cold, but inside the block (the marked hour plus two hours): no healing.
        for tick in [last_cold + 60, last_cold + 120, last_cold + 179] {
            engine.resources.current_tick = tick;
            survival.tick(&mut engine.world, &mut engine.resources);
            assert_eq!(hp(&engine, adult), after_night, "tick {tick}");
        }
        // The block ends at last_cold + 60 + 60 x heal_block_hours (2): healing
        // resumes at 3 hp an hour.
        assert_eq!(
            engine
                .resources
                .rules
                .society
                .exposure
                .expect("rule")
                .heal_block_hours,
            2
        );
        engine.resources.current_tick = last_cold + 180;
        survival.tick(&mut engine.world, &mut engine.resources);
        assert_eq!(hp(&engine, adult), after_night + 0.05);
        // A resident whose last cold hour lies a day back is outside every
        // block and heals on the first run.
        let (mut plain, residents) = exposure_fixture(night, &[(1, Stage::Adult, 52.0)]);
        let entity = residents[0].1;
        plain
            .world
            .insert_one(
                entity,
                ExposureClock {
                    last_cold_tick: last_cold - 1440,
                },
            )
            .expect("clock");
        plain.resources.current_tick = last_cold + 30;
        StarvationSystem::new().tick(&mut plain.world, &mut plain.resources);
        assert_eq!(hp(&plain, entity), 52.05);
    }

    /// Run a full dark night (22:00..05:00) on the fixture `residents` spawned in
    /// the given order; return the death log and the fingerprint.
    fn night_outcome(residents: &[(AgentId, Stage, f64)]) -> (Vec<Death>, String, u64) {
        let night = tick_at(21, 1320);
        let (mut engine, _) = exposure_fixture(night, residents);
        for hour in 0..8 {
            fire_at(&mut engine, night + 60 * hour);
        }
        (
            deaths_at_home(&engine),
            fingerprint(&engine),
            engine.resources.tallies.deaths_exposure,
        )
    }

    #[test]
    fn harness_exposure_order_independent() {
        // Several residents die in the same hour (same tick): deaths are booked,
        // event ids issued and agents despawned in AgentId order whatever the
        // spawn (hecs storage) order.
        let rows = [
            (4, Stage::Elder, 30.0),
            (2, Stage::Child, 25.0),
            (7, Stage::Adult, 20.0),
            (1, Stage::Elder, 26.0),
            (5, Stage::Adult, 100.0),
            (3, Stage::Child, 60.0),
            (6, Stage::Elder, 9.0),
        ];
        let forward = night_outcome(&rows);
        let mut reversed = rows;
        reversed.reverse();
        let backward = night_outcome(&reversed);
        let mut by_stage = rows;
        by_stage.sort_by_key(|row| (row.1 as u8, std::cmp::Reverse(row.0)));
        let grouped = night_outcome(&by_stage);
        assert!(
            forward.0.len() >= 4,
            "several exposure deaths: {:?}",
            forward.0
        );
        assert!(
            forward.0.windows(2).any(|w| w[0].3 == w[1].3),
            "at least two deaths share an hour"
        );
        for outcome in [&backward, &grouped] {
            assert_eq!(outcome, &forward);
        }
        // Within an hour the ids are ascending.
        for w in forward.0.windows(2) {
            if w[0].3 == w[1].3 {
                assert!(w[0].1 < w[1].1);
            }
        }
    }
}

// From `harness_revitalize_pass11_exposure.rs`.
mod hardiness {
    //! Pass 11 Part 1: a cold that can take anyone (exposure v2: hardiness,
    //! hunger and per-stage damage).
    //!
    //! V8-0: rebased from `game_v17_nofission` (damage [9, 5, 6] an hour) and the
    //! goal's start values ([8, 6, 8]) on `WorldRules::baseline` (damage [7.5, 5,
    //! 5] an hour, hunger weight 0.5, hardiness on); the numbers below are the
    //! baseline's.
    use hecs::Entity;
    use sim_core::causal::event::{CausalEvent, DeathReason};
    use sim_core::components::hardiness::{HARDINESS_MAX_PERMILLE, HARDINESS_MIN_PERMILLE};
    use sim_core::components::{
        Agent, AgentId, BodyHealth, Child, Elder, Hardiness, Hunger, Lineage, Position, Settlement,
        Thirst,
    };
    use sim_core::material::MaterialRegistry;
    use sim_core::rng::NoiseFork;
    use sim_core::rules::{ExposureRule, WorldRules};
    use sim_engine::{RuntimeSystem, SimEngine};
    use sim_systems::runtime::settlement::fire::exposure_damage;
    use sim_systems::runtime::settlement::FireSystem;
    use sim_systems::runtime::survival::StarvationSystem;
    use sim_test::baseline_engine;

    const HOME: (u32, u32) = (20, 20);

    /// Tick at which the display clock (480-tick offset) reads `minute` on the
    /// zero-based display day `day` (day index 21 = the first winter day).
    fn tick_at(day: u64, minute: u64) -> u64 {
        day * 1440 + minute - 480
    }

    /// Life stage of a fixture resident.
    #[derive(Clone, Copy, PartialEq, Debug)]
    enum Stage {
        Child,
        Adult,
        Elder,
    }

    /// One fixture resident: id, stage, hardiness (None = no component), hunger, hp.
    type Resident = (AgentId, Stage, Option<u16>, f32, f64);

    /// A bare world under `rules` at `tick`: one settlement at `HOME` with an
    /// empty wood store (the fire goes out) and the given residents, needs fixed
    /// (growth rate 0), thirst 0.
    fn fixture(rules: WorldRules, tick: u64, residents: &[Resident]) -> (SimEngine, Vec<Entity>) {
        let mut engine = SimEngine::new(64, 64, MaterialRegistry::new());
        engine.resources.rules = rules;
        engine.resources.current_tick = tick;
        let mut settlement = Settlement::new_with_id(1, 0);
        settlement.formation_tile = HOME;
        let mut entities = Vec::new();
        for &(id, stage, hardiness, hunger, hp) in residents {
            let entity = engine.world.spawn((
                Position::new(HOME.0, HOME.1),
                Agent { id },
                Hunger::new(hunger, 0.0),
                Thirst::new(0.0, 0.0),
                BodyHealth { hp, max_hp: 100.0 },
            ));
            match stage {
                Stage::Child => engine
                    .world
                    .insert_one(entity, Child { born_tick: 0 })
                    .expect("child"),
                Stage::Elder => engine.world.insert_one(entity, Elder).expect("elder"),
                Stage::Adult => {}
            }
            if let Some(permille) = hardiness {
                engine
                    .world
                    .insert_one(entity, Hardiness { permille })
                    .expect("hardiness");
            }
            settlement.add_member_agent(id);
            entities.push(entity);
        }
        engine.resources.settlements.insert(1, settlement);
        (engine, entities)
    }

    fn hp(engine: &SimEngine, entity: Entity) -> f64 {
        engine.world.get::<&BodyHealth>(entity).expect("health").hp
    }

    /// World seed of the fixture rules; the exposure rule does not depend on it.
    const SEED: u64 = 0;

    fn rules() -> WorldRules {
        WorldRules::baseline(SEED)
    }

    fn baseline_exposure() -> ExposureRule {
        rules().society.exposure.expect("baseline exposure rule")
    }

    #[test]
    fn harness_hardiness_keyed_and_ranged() {
        let mut total_births = 0;
        for seed in [3u64, 41, 900_017] {
            // Unforked and forked (replicate 3 from tick 1) runs of the same world.
            let mut plain = baseline_engine(seed);
            let mut forked = baseline_engine(seed);
            forked.resources.noise_fork = Some(NoiseFork {
                tick: 1,
                replicate: 3,
            });
            let founders = |engine: &SimEngine| {
                let mut rows: Vec<(u64, u16)> = engine
                    .world
                    .query::<(&Lineage, &Hardiness)>()
                    .iter()
                    .map(|(_, (lineage, h))| (lineage.key, h.permille))
                    .collect();
                rows.sort_unstable();
                rows
            };
            let at_start = founders(&plain);
            assert_eq!(
                at_start.len(),
                plain.world.query::<&Agent>().iter().count(),
                "every founder carries hardiness"
            );
            assert_eq!(at_start, founders(&forked));
            for _ in 0..6 * 1440 {
                plain.tick();
                forked.tick();
            }
            // Newborns must be checked too: measured births 1 / 11 / 5 (both runs)
            // on seeds 3 / 41 / 900017 over these six days.
            let births = plain.resources.tallies.births;
            assert!(
                births >= 1 && forked.resources.tallies.births >= 1,
                "seed {seed}: births happened in both runs"
            );
            total_births += births;
            for engine in [&plain, &forked] {
                let mut count = 0;
                for (_, (agent, lineage, hardiness)) in engine
                    .world
                    .query::<(&Agent, &Lineage, &Hardiness)>()
                    .iter()
                {
                    // A pure function of (seed, lineage key): independent of the
                    // replicate, of the AgentId and of the order of births.
                    assert_eq!(
                        *hardiness,
                        Hardiness::draw(seed, lineage.key),
                        "seed {seed} agent {}",
                        agent.id
                    );
                    assert!((HARDINESS_MIN_PERMILLE..=HARDINESS_MAX_PERMILLE)
                        .contains(&hardiness.permille));
                    count += 1;
                }
                assert_eq!(
                    count,
                    engine.world.query::<&Agent>().iter().count(),
                    "every resident carries hardiness"
                );
            }
        }
        assert!(total_births >= 10, "{total_births} births (measured 17)");
        // The draw ignores the replicate: same key, same value; spread over the range.
        let values: std::collections::BTreeSet<u16> = (0..3_000u64)
            .map(|k| Hardiness::draw(5, k).permille)
            .collect();
        assert!(
            values.contains(&HARDINESS_MIN_PERMILLE)
                || values.iter().next().is_some_and(|v| *v < 710)
        );
        assert!(values.iter().next_back().is_some_and(|v| *v > 1290));
    }

    #[test]
    fn harness_exposure_v2_formula() {
        let rule = baseline_exposure();
        assert_eq!(rule.damage_per_cold_hour, [7.5, 5.0, 5.0]);
        assert_eq!((rule.hunger_weight, rule.use_hardiness), (0.5, true));
        // 22:00 of the first winter day, fire out. Three hardiness x hunger cases
        // (adults), plus a child and an elder.
        let night = tick_at(21, 1320);
        let cases: [Resident; 5] = [
            (1, Stage::Adult, Some(700), 0.0, 100.0),
            (2, Stage::Adult, Some(1000), 75.0, 100.0),
            (3, Stage::Adult, Some(1300), 99.0, 100.0),
            (4, Stage::Child, Some(1100), 60.0, 100.0),
            (5, Stage::Elder, Some(900), 40.0, 100.0),
        ];
        let (mut engine, entities) = fixture(rules(), night, &cases);
        FireSystem::new().tick(&mut engine.world, &mut engine.resources);
        for (&(id, stage, hardiness, hunger, start), entity) in cases.iter().zip(&entities) {
            let index = match stage {
                Stage::Child => 0,
                Stage::Adult => 1,
                Stage::Elder => 2,
            };
            let factor = f64::from(hardiness.expect("hardiness")) / 1000.0;
            let excess = ((f64::from(hunger) - 50.0) / 50.0).clamp(0.0, 1.0);
            let expected =
                rule.damage_per_cold_hour[index] * factor * (1.0 + rule.hunger_weight * excess);
            let lost = start - hp(&engine, *entity);
            assert!(
                (lost - expected).abs() < 1e-9,
                "agent {id}: lost {lost}, formula {expected}"
            );
            let helper = exposure_damage(
                &rule,
                index,
                hardiness.map(|permille| Hardiness { permille }),
                f64::from(hunger),
            );
            assert!((helper - expected).abs() < 1e-12);
        }
        // Spot values: (700, 0) = 5 x 0.7 = 3.5; (1000, 75) = 5 x 1.25 = 6.25;
        // (1300, 99) = 5 x 1.3 x 1.49 = 9.685; child (1100, 60) = 7.5 x 1.1 x 1.1
        // = 9.075; elder (900, 40) = 5 x 0.9 = 4.5.
        assert!((100.0 - hp(&engine, entities[0]) - 3.5).abs() < 1e-9);
        assert!((100.0 - hp(&engine, entities[1]) - 6.25).abs() < 1e-9);
        assert!((100.0 - hp(&engine, entities[2]) - 5.0 * 1.3 * 1.49).abs() < 1e-9);
        assert!((100.0 - hp(&engine, entities[3]) - 7.5 * 1.1 * 1.1).abs() < 1e-9);
        assert!((100.0 - hp(&engine, entities[4]) - 4.5).abs() < 1e-9);
    }

    /// Run one full dark night (22:00..06:00, eight hourly `FireSystem` runs)
    /// starting on display day `day`, and `StarvationSystem` every tick from the
    /// first cold hour to the next night's 22:00 run (healing as the rules allow).
    /// Returns whether `entity` is still alive after the night.
    fn night_and_day(engine: &mut SimEngine, day: u64, entity: Entity) -> bool {
        let start = tick_at(day, 1320);
        let mut fire = FireSystem::new();
        let mut starvation = StarvationSystem::new();
        for tick in start..start + 1440 {
            engine.resources.current_tick = tick;
            if (tick - start).is_multiple_of(60) && tick < start + 8 * 60 {
                fire.tick(&mut engine.world, &mut engine.resources);
            }
            if !engine.world.contains(entity) {
                return false;
            }
            starvation.tick(&mut engine.world, &mut engine.resources);
        }
        engine.world.contains(entity)
    }

    #[test]
    fn harness_exposure_adult_can_die() {
        let rules = rules();
        // Frail and hungry: hardiness 1300, hunger fixed at 99 (below
        // `Hunger::SATURATION` 100, so no starvation damage), thirst 0.
        let (mut engine, entities) = fixture(
            rules,
            tick_at(21, 1320),
            &[(1, Stage::Adult, Some(1300), 99.0, 100.0)],
        );
        let frail = entities[0];
        let per_hour = 5.0 * 1.3 * (1.0 + 0.5 * 0.98);
        assert!(
            night_and_day(&mut engine, 21, frail),
            "survives the first night"
        );
        // 8 x 9.685 = 77.48 hp lost on the first full dark night; no healing
        // (hunger > 50), so the 22.5 left last under three hours of the next.
        let after_first = hp(&engine, frail);
        assert!(
            (100.0 - after_first - 8.0 * per_hour).abs() < 1e-6,
            "lost {}",
            100.0 - after_first
        );
        assert!((77.0..78.0).contains(&(100.0 - after_first)));
        assert!(
            !night_and_day(&mut engine, 22, frail),
            "dies on the second night"
        );
        assert_eq!(engine.resources.tallies.deaths_exposure, 1);
        assert_eq!(
            engine.resources.tallies.deaths_starvation
                + engine.resources.tallies.deaths_dehydration,
            0
        );
        let width = engine.resources.tile_grid.width;
        let deaths: Vec<_> = engine
            .resources
            .causal_log
            .get(HOME.1 * width + HOME.0)
            .map(|log| {
                log.as_slice()
                    .iter()
                    .filter_map(|event| match event {
                        CausalEvent::AgentDied { agent, reason, .. } => Some((*agent, *reason)),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        assert_eq!(deaths, vec![(1, DeathReason::Exposure)]);

        // Hardy and fed: hardiness 700, hunger 0 survives two dark nights (3.5 an
        // hour, 28 a night).
        let (mut engine, entities) = fixture(
            rules,
            tick_at(21, 1320),
            &[(1, Stage::Adult, Some(700), 0.0, 100.0)],
        );
        assert!(night_and_day(&mut engine, 21, entities[0]));
        assert!(night_and_day(&mut engine, 22, entities[0]));
        // Measured 44.0 = 100 - 2 x 28: no more than the two nights' damage.
        assert!(
            hp(&engine, entities[0]) >= 100.0 - 2.0 * 28.0 - 1e-9,
            "hp {}",
            hp(&engine, entities[0])
        );
        assert_eq!(engine.resources.tallies.deaths_exposure, 0);
    }
}
