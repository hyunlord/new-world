//! OralTransmissionSystem: once each evening at the hearth gathering, each listener hears one keyed teller and may learn one thing.
//!
//! Priority: 160 (`ORAL_PRIORITY`). Interval: 1,440 ticks, at the rule's `oral_minute` of the clock (`tick_offset`). Gate: `ColonyRule::knowledge`.
//! Reads: `Agent`, `Position`, `Child`, `Lineage`, `Life`, `Knowledge`; `SimResources.colony` (rooms), `settlements` (hearths), `rules`, `world_seed`
//! Writes: `Knowledge` (listeners' new entries; fire keeping and storytelling stamped used); `SimResources.colony.tallies`
//! Events: none
//!
//! M4 goal 3.3 (ADR 0025). The gathered are the people within
//! `KnowledgeRule::gather_radius` (Chebyshev) of a hearth or standing in the
//! hall. Tellers are adults; listeners are adults and children of at least
//! `learner_min_years`. In `AgentId` order each listener is paired with a
//! keyed random other gathered teller (`STREAM_KNOWLEDGE`, subject = the
//! listener's lineage key, slot 0x1_0000); the teller's candidates are the
//! nodes it practises that the listener does not (below threshold); each is
//! told, in a keyed order (slot 0x1_0200 + id), and takes with
//! `oral_chance_permille` (+ `storytelling_bonus_permille` when the teller
//! practises storytelling; slot 0x1_0100 + id); the first that takes is
//! learned, at most one per pair per evening (goal 3.3). A listener who learns rises to
//! `oral_gain` ‰ (`Source::Oral`), so a node whose threshold is at most that
//! makes a practitioner at once, a harder one a learner; the teller is
//! stamped as using what it told. Everyone reads the
//! evening's snapshot: what is learned tonight is told from tomorrow. Every
//! pair also rehearses one node the teller practises and the listener has
//! sufficiently learned: at least min(oral_gain, practitioner threshold).
//! Prefer the pair's oldest last use; equal ages use keyed slot 0x1_0003.
//! Both are stamped as using it, without gaining proficiency. This lets an
//! expert revisit difficult knowledge with an oral learner without promoting
//! that learner (M6b S1-CAL-3). Among feasible pairs, prefer the teller's
//! earliest practitioner-loss deadline if unused, including its grace period.
//! Apply that same deadline to rehearsal and telling, retaining keyed ties.
//! Hearth lore and entries protected by their decay floor need no preference.
//! Actual use today removes deadline priority for later pairs,
//! but teller practice and receiving eligibility still use the evening snapshot.
//! This is local conversation policy, not a forecast of colony-wide loss. Every
//! gathered holder of fire keeping is stamped as using it (the hearth lore),
//! and a teller who practises storytelling as using that.
use hecs::World;
use sim_core::colony::knowledge::{defs, ids_of, Knowledge, Lore, Source};
use sim_core::rng::{keyed_below, STREAM_KNOWLEDGE};
use sim_engine::{RuntimeSystem, SimResources};

use super::{knowledge_of, people, rule, store, Person};

/// Priority of [`OralTransmissionSystem`].
pub const ORAL_PRIORITY: u32 = 160;

/// See the module header.
#[derive(Debug, Default)]
pub struct OralTransmissionSystem;

/// Whether `p` is at the evening gathering: near a hearth or in the hall.
pub fn gathered(res: &SimResources, hearths: &[(u32, u32)], radius: u32, p: &Person) -> bool {
    hearths
        .iter()
        .any(|h| crate::runtime::jobs::items::chebyshev(*h, p.pos) <= radius)
        || crate::runtime::rooms::roofed_as(res, p.pos, sim_core::rooms::RoomRole::Hall)
}

/// No-further-use threshold crossing; oral runs before same-tick decay.
fn decay_deadline(
    known: &Knowledge,
    used: &Knowledge,
    node: u8,
    tick: u64,
    rule: &sim_core::rules::KnowledgeRule,
) -> Option<u64> {
    use sim_core::colony::knowledge::{bit, day_of, gates, DAY_TICKS};
    let entry = known.get(node)?;
    let threshold = defs().threshold(node);
    let rate = u64::from(rule.decay_of(entry.source()));
    let last_used = used.get(node).map_or(entry.last_used, |e| e.last_used);
    if entry.proficiency < threshold
        || rate == 0
        || rule.decay_interval == 0
        || rule.floor_of(entry.source()) >= threshold
        || bit(node) & gates().lore(Lore::Hearth) != 0
        || last_used == day_of(tick)
    {
        return None;
    }
    let onset_day = u64::from(last_used)
        .checked_add(rule.unused_days)?
        .checked_add(1)?;
    if onset_day > u64::from(u16::MAX) {
        return None;
    }
    let start = tick.max(onset_day.checked_mul(DAY_TICKS)?);
    let first = start
        .div_ceil(rule.decay_interval)
        .checked_mul(rule.decay_interval)?;
    let steps = u64::from(entry.proficiency - threshold) / rate + 1;
    first.checked_add((steps - 1).checked_mul(rule.decay_interval)?)
}

impl RuntimeSystem for OralTransmissionSystem {
    fn name(&self) -> &str {
        "OralTransmissionSystem"
    }

    fn priority(&self) -> u32 {
        ORAL_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        1_440
    }

    fn tick_offset(&self, resources: &SimResources) -> u64 {
        let day = 1_440;
        let minute = rule(resources).map_or(1230, |r| u64::from(r.oral_minute)) % day;
        (minute + day - resources.rules.clock_offset_ticks % day) % day
    }

    fn tick(&mut self, world: &mut World, resources: &mut SimResources) {
        let Some(rule) = rule(resources) else {
            return;
        };
        let tick = resources.current_tick;
        let seed = resources.world_seed;
        let hearths = crate::runtime::jobs::hearth::hearth_tiles(resources);
        let here: Vec<Person> = people(world, resources)
            .into_iter()
            .filter(|p| gathered(resources, &hearths, rule.gather_radius, p))
            .collect();
        resources.colony.tallies.oral_gathered += here.len() as u64;
        if here.len() < 2 {
            return;
        }
        let d = defs();
        let gates = sim_core::colony::knowledge::gates();
        let hearth_lore = gates.lore(Lore::Hearth);
        let story = gates.oral();
        let deadline = |known: &Knowledge, used: &Knowledge, node: u8| {
            decay_deadline(known, used, node, tick, &rule)
        };
        // The evening's snapshot: every gathered person's knowledge before tonight.
        let before: Vec<_> = here.iter().map(|p| knowledge_of(world, p.entity)).collect();
        let mut after = before.clone();
        let mut told = 0u64;
        let mut learned = 0u64;
        let mut listeners: Vec<hecs::Entity> = Vec::new();
        let mut rehearsed = 0u64;
        // The old listener filter is total: p>=gain satisfies its second arm;
        // below gain, either not practising or p>=threshold satisfies an arm.
        // Eligibility stays frozen in before; only last-use changes within tonight.
        let oldest_topic = |i: usize, used: &Knowledge| {
            ids_of(before[i].practice())
                .filter_map(|node| deadline(&before[i], used, node))
                .min()
        };
        let mut ranks: Vec<_> = (0..here.len())
            .map(|i| oldest_topic(i, &after[i]))
            .collect();
        for (li, listener) in here.iter().enumerate() {
            if !listener.adult && listener.years < rule.learner_min_years {
                continue;
            }
            let mut tellers: Vec<_> = (0..here.len())
                .filter(|&i| i != li && here[i].adult)
                .map(|i| (i, ranks[i]))
                .collect();
            if tellers.is_empty() {
                continue;
            }
            if let Some(oldest) = tellers.iter().filter_map(|(_, due)| *due).min() {
                tellers.retain(|(_, due)| *due == Some(oldest));
            }
            let pick = keyed_below(
                seed,
                STREAM_KNOWLEDGE,
                listener.key,
                tick,
                0x1_0000,
                0,
                tellers.len() as u64,
            );
            let ti = tellers[pick as usize].0;
            let teller = &before[ti];
            // Rehearsal uses existing understanding, never grants proficiency.
            let mut shared: Vec<u8> = ids_of(teller.practice())
                .filter(|node| {
                    before[li].proficiency(*node) >= rule.oral_gain.min(d.threshold(*node))
                })
                .collect();
            let last_use = |node: &u8| {
                teller
                    .get(*node)
                    .map_or(u16::MAX, |entry| entry.last_used)
                    .min(
                        before[li]
                            .get(*node)
                            .map_or(u16::MAX, |entry| entry.last_used),
                    )
            };
            if let Some(oldest) = shared
                .iter()
                .filter_map(|&node| deadline(teller, &after[ti], node))
                .min()
            {
                shared.retain(|&node| deadline(teller, &after[ti], node) == Some(oldest));
            } else if let Some(oldest) = shared.iter().map(last_use).min() {
                shared.retain(|node| last_use(node) == oldest);
            }
            if !shared.is_empty() {
                let pick = keyed_below(
                    seed,
                    STREAM_KNOWLEDGE,
                    listener.key,
                    tick,
                    0x1_0003,
                    0,
                    shared.len() as u64,
                );
                let node = shared[pick as usize];
                after[li].touch(node, tick);
                after[ti].touch(node, tick);
                rehearsed += 1;
            }
            // A node the listener already holds at the telling's gain or above
            // cannot rise by it, so it is not a candidate (code review F2).
            let candidates: Vec<u8> = ids_of(teller.practice() & !before[li].practice())
                .filter(|c| after[li].proficiency(*c) < rule.oral_gain)
                .collect();
            if candidates.is_empty() {
                for i in [li, ti] {
                    ranks[i] = oldest_topic(i, &after[i]);
                }
                continue;
            }
            told += 1;
            // Goal 3.3: each candidate is told with its own keyed chance (slot
            // 0x1_0100 + id), tried in a keyed order (slot 0x1_0200 + id); the
            // first that takes is learned — at most one per pair per evening.
            let chance = rule.oral_chance_permille
                + if teller.practice() & story != 0 {
                    rule.storytelling_bonus_permille
                } else {
                    0
                };
            if teller.practice() & story != 0 {
                if let Some(id) = ids_of(story).next() {
                    after[ti].touch(id, tick);
                }
            }
            let mut order: Vec<_> = candidates
                .iter()
                .map(|c| {
                    let rank = sim_core::rng::keyed_u64(
                        seed,
                        STREAM_KNOWLEDGE,
                        listener.key,
                        tick,
                        0x1_0200 + u32::from(*c),
                        0,
                    );
                    let priority = match deadline(teller, &after[ti], *c) {
                        Some(due) => (false, due, rank),
                        None => (true, 0, rank),
                    };
                    (priority, *c)
                })
                .collect();
            order.sort_unstable();
            for (_, node) in order {
                // Telling it is using it, for the teller (DECISIONS D14).
                after[ti].touch(node, tick);
                let roll = keyed_below(
                    seed,
                    STREAM_KNOWLEDGE,
                    listener.key,
                    tick,
                    0x1_0100 + u32::from(node),
                    0,
                    1000,
                );
                if roll < u64::from(chance) && d.get(node).is_some() {
                    after[li].raise_to(node, rule.oral_gain, Source::Oral, tick);
                    learned += 1;
                    listeners.push(here[li].entity);
                    break;
                }
            }
            for i in [li, ti] {
                ranks[i] = oldest_topic(i, &after[i]);
            }
        }
        // The hearth lore: everyone gathered who holds fire keeping uses it tonight.
        for k in after.iter_mut() {
            for id in ids_of(hearth_lore) {
                if k.get(id).is_some() {
                    k.touch(id, tick);
                }
            }
        }
        for ((p, b), a) in here.iter().zip(&before).zip(after) {
            if *b != a {
                store(world, p.entity, a);
            }
        }
        // M5 (ADR 0030): learning at the fire cheers.
        for e in listeners {
            crate::runtime::mood::note(world, resources, e, crate::runtime::mood::Fact::Learned);
        }
        let t = &mut resources.colony.tallies;
        t.oral_tellings += told;
        t.oral_learned += learned;
        t.oral_rehearsed += rehearsed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::colony::knowledge::{Knowledge, KnowledgeEntry};
    use sim_core::components::Position;

    fn pair() -> (World, SimResources, Vec<hecs::Entity>) {
        let mut engine = crate::worldgen::build_world_with(
            &crate::worldgen::WorldGenConfig::colony(1),
            sim_core::rules::WorldRules::colony(1),
        );
        let hearth = crate::runtime::jobs::hearth::hearth_tiles(&engine.resources)[0];
        let people = people(&engine.world, &engine.resources);
        let entities: Vec<_> = people.iter().take(2).map(|p| p.entity).collect();
        for p in people.iter().skip(2) {
            engine
                .world
                .despawn(p.entity)
                .expect("remove unrelated people");
        }
        for &entity in &entities {
            let mut pos = engine.world.get::<&mut Position>(entity).expect("position");
            pos.x = hearth.0;
            pos.y = hearth.1;
        }
        engine.resources.current_tick = 10 * 1440 + 750;
        (engine.world, engine.resources, entities)
    }

    #[test]
    fn an_expert_rehearses_with_an_informed_learner_without_promoting_them() {
        let (mut world, mut res, entities) = pair();
        let node = defs().id_of("pot_firing").expect("pot firing");
        for (entity, proficiency) in entities.iter().zip([400, 350]) {
            store(
                &mut world,
                *entity,
                Knowledge::from_entries(vec![KnowledgeEntry::new(
                    node,
                    Source::Oral,
                    proficiency,
                    0,
                )]),
            );
        }
        OralTransmissionSystem.tick(&mut world, &mut res);
        for (entity, proficiency) in entities.iter().zip([400, 350]) {
            let known = knowledge_of(&world, *entity);
            assert_eq!(known.proficiency(node), proficiency);
            assert_eq!(known.get(node).expect("existing knowledge").last_used, 10);
        }
        assert!(!knowledge_of(&world, entities[1]).practises(node));
        assert_eq!(res.colony.tallies.oral_learned, 0);
        assert!(res.colony.command_log.is_empty());
    }

    #[test]
    fn rehearsal_advances_deadlines_once_per_pair_without_refreshing_every_node() {
        let (mut world, mut res, entities) = pair();
        let old = defs().id_of("pot_firing").expect("pot firing");
        let recent = defs().id_of("hut_building").expect("hut building");
        let later = defs().id_of("pressure_flaking").unwrap();
        for &entity in &entities {
            store(
                &mut world,
                entity,
                Knowledge::from_entries(vec![
                    KnowledgeEntry::new(old, Source::Oral, 500, 0),
                    KnowledgeEntry::new(recent, Source::Oral, 500, 9),
                    KnowledgeEntry::new(later, Source::Oral, 1000, 9),
                ]),
            );
        }
        OralTransmissionSystem.tick(&mut world, &mut res);
        for entity in entities {
            let known = knowledge_of(&world, entity);
            assert_eq!(known.get(old).expect("old").last_used, 10);
            assert_eq!(known.get(recent).expect("recent").last_used, 10);
            assert_eq!(known.get(later).unwrap().last_used, 9);
            assert_eq!(known.proficiency(recent), 500);
            assert_eq!(known.proficiency(later), 1000);
            assert_eq!(known.proficiency(old), 500);
        }
        assert_eq!(res.colony.tallies.oral_rehearsed, 2);
    }

    #[test]
    fn a_lone_expert_does_not_get_free_rehearsal() {
        let (mut world, mut res, entities) = pair();
        world.despawn(entities[1]).expect("remove listener");
        let node = defs().id_of("pot_firing").expect("pot firing");
        store(
            &mut world,
            entities[0],
            Knowledge::from_entries(vec![KnowledgeEntry::new(node, Source::Oral, 400, 0)]),
        );
        OralTransmissionSystem.tick(&mut world, &mut res);
        assert_eq!(
            knowledge_of(&world, entities[0])
                .get(node)
                .expect("knowledge")
                .last_used,
            0
        );
        assert_eq!(res.colony.tallies.oral_rehearsed, 0);
    }

    fn certain_telling(res: &mut SimResources) {
        let rule = res
            .rules
            .colony
            .as_mut()
            .expect("colony")
            .knowledge
            .as_mut()
            .expect("knowledge");
        rule.oral_chance_permille = 1000;
        rule.storytelling_bonus_permille = 0;
    }

    fn recent_topic_first_seed(res: &SimResources, listener: &Person, old: u8, recent: u8) -> u64 {
        (0..1000)
            .find(|&seed| {
                let rank = |node| {
                    sim_core::rng::keyed_u64(
                        seed,
                        STREAM_KNOWLEDGE,
                        listener.key,
                        res.current_tick,
                        0x1_0200 + u32::from(node),
                        0,
                    )
                };
                rank(recent) < rank(old)
            })
            .expect("a seed whose ordinary topic order prefers recent knowledge")
    }

    #[test]
    fn deadline_matches_source_rate_grace_and_pending_decay_cadence() {
        let (_, res, _) = pair();
        let mut rule = rule(&res).unwrap();
        let node = defs().id_of("stone_walling").unwrap();
        let known = |p, source, last| {
            Knowledge::from_entries(vec![KnowledgeEntry::new(node, source, p, last)])
        };
        let k = known(500, Source::Discovered, 24);
        assert_eq!(decay_deadline(&k, &k, node, 41070, &rule), Some(50640));
        let pot_margin = known(514, Source::Discovered, 24);
        assert_eq!(
            decay_deadline(&pot_margin, &pot_margin, node, 41070, &rule),
            Some(51480)
        );
        rule.decay[Source::Oral.table_index()] = 2;
        let faster = known(500, Source::Oral, 24);
        assert_eq!(
            decay_deadline(&faster, &faster, node, 41070, &rule),
            Some(47640)
        );
        let edge = known(400, Source::Discovered, 24);
        assert_eq!(
            decay_deadline(&edge, &edge, node, 44640, &rule),
            Some(44640)
        );
        assert_eq!(
            decay_deadline(&edge, &edge, node, 44641, &rule),
            Some(44700)
        );
        let below = known(399, Source::Discovered, 24);
        assert_eq!(decay_deadline(&below, &below, node, 41070, &rule), None);
        let mut used = k.clone();
        used.touch(node, 41070);
        assert_eq!(decay_deadline(&k, &used, node, 41070, &rule), None);
        rule.decay_interval = 0;
        assert_eq!(decay_deadline(&k, &k, node, 41070, &rule), None);
    }

    #[test]
    fn predicted_threshold_deadline_matches_real_pending_decay() {
        use super::super::decay::KnowledgeDecaySystem;
        let (mut world, mut res, entities) = pair();
        let node = defs().id_of("stone_walling").unwrap();
        let k =
            Knowledge::from_entries(vec![KnowledgeEntry::new(node, Source::Discovered, 400, 24)]);
        let due = decay_deadline(&k, &k, node, 41070, &rule(&res).unwrap()).unwrap();
        store(&mut world, entities[0], k);
        world
            .insert_one(entities[0], sim_core::components::AgentState::Idle)
            .unwrap();
        assert!(OralTransmissionSystem.priority() < KnowledgeDecaySystem.priority());
        res.current_tick = due - KnowledgeDecaySystem.tick_interval();
        KnowledgeDecaySystem.tick(&mut world, &mut res);
        assert_eq!(knowledge_of(&world, entities[0]).proficiency(node), 400);
        res.current_tick = due;
        let before = knowledge_of(&world, entities[0]);
        assert_eq!(
            decay_deadline(&before, &before, node, due, &rule(&res).unwrap()),
            Some(due)
        );
        KnowledgeDecaySystem.tick(&mut world, &mut res);
        let after = knowledge_of(&world, entities[0]);
        assert_eq!(after.proficiency(node), 399);
        assert!(!after.practises(node));
    }

    #[test]
    fn deadline_respects_saturated_days_and_checked_tick_arithmetic() {
        let (_, res, _) = pair();
        let mut rule = rule(&res).unwrap();
        let node = defs().id_of("stone_walling").unwrap();
        let known = |last| {
            Knowledge::from_entries(vec![KnowledgeEntry::new(
                node,
                Source::Discovered,
                400,
                last,
            )])
        };
        let beyond = known(u16::MAX - 6);
        let tick = u64::from(u16::MAX) * 1440;
        assert_eq!(decay_deadline(&beyond, &beyond, node, tick, &rule), None);
        let final_day = known(u16::MAX - 7);
        assert_eq!(
            decay_deadline(&final_day, &final_day, node, tick, &rule),
            Some(tick)
        );
        assert_eq!(
            decay_deadline(&final_day, &final_day, node, u64::MAX, &rule),
            None
        );
        rule.unused_days = u64::MAX;
        assert_eq!(
            decay_deadline(&final_day, &final_day, node, tick, &rule),
            None
        );
    }

    #[test]
    fn pregrace_deadline_precedes_a_successful_competing_topic() {
        let (mut world, mut res, entities) = pair();
        certain_telling(&mut res);
        res.current_tick = 28 * 1440 + 750;
        let old = defs().id_of("stone_walling").unwrap();
        let recent = defs().id_of("food_drying").unwrap();
        store(
            &mut world,
            entities[0],
            Knowledge::from_entries(vec![
                KnowledgeEntry::new(old, Source::Discovered, 500, 24),
                KnowledgeEntry::new(recent, Source::Discovered, 650, 27),
            ]),
        );
        store(&mut world, entities[1], Knowledge::default());
        res.world_seed = recent_topic_first_seed(&res, &people(&world, &res)[1], old, recent);
        OralTransmissionSystem.tick(&mut world, &mut res);
        let listener = knowledge_of(&world, entities[1]);
        assert_eq!(listener.proficiency(old), 350);
        assert_eq!(listener.proficiency(recent), 0);
        assert!(!listener.practises(old));
        assert_eq!(res.colony.tallies.oral_learned, 1);
        assert!(res.colony.command_log.is_empty());
    }

    #[test]
    fn overdue_topic_precedes_a_recent_success_without_extra_learning() {
        // Given: the old topic would lose to a successful recent topic under the original draw.
        let (mut world, mut res, entities) = pair();
        certain_telling(&mut res);
        let old = defs().id_of("pot_firing").expect("pot");
        let recent = defs().id_of("pressure_flaking").expect("pressure");
        store(
            &mut world,
            entities[0],
            Knowledge::from_entries(vec![
                KnowledgeEntry::new(old, Source::Discovered, 400, 0),
                KnowledgeEntry::new(recent, Source::Discovered, 500, 9),
            ]),
        );
        store(&mut world, entities[1], Knowledge::default());
        res.world_seed = recent_topic_first_seed(&res, &people(&world, &res)[1], old, recent);
        // When: the same real pair talks once.
        OralTransmissionSystem.tick(&mut world, &mut res);
        // Then: one ordinary gain, not a new practitioner or command.
        let listener = knowledge_of(&world, entities[1]);
        assert_eq!(listener.proficiency(old), 350);
        assert_eq!(listener.proficiency(recent), 0);
        assert!(!listener.practises(old));
        assert_eq!(res.colony.tallies.oral_learned, 1);
        assert!(res.colony.command_log.is_empty());
    }

    // Frozen pre-optimization two-pass teller selector, independently kept for the oracle.
    fn old_teller_choices(
        here: &[Person],
        before: &[Knowledge],
        after: &[Knowledge],
        li: usize,
        tick: u64,
        rule: &sim_core::rules::KnowledgeRule,
    ) -> Vec<usize> {
        let mut tellers: Vec<_> = (0..here.len())
            .filter(|&i| i != li && here[i].adult)
            .collect();
        let oldest_topic = |i: usize| {
            ids_of(before[i].practice())
                .filter(|&node| {
                    (!before[li].practises(node) && before[li].proficiency(node) < rule.oral_gain)
                        || before[li].proficiency(node)
                            >= rule.oral_gain.min(defs().threshold(node))
                })
                .filter_map(|node| decay_deadline(&before[i], &after[i], node, tick, rule))
                .min()
        };
        if let Some(oldest) = tellers.iter().filter_map(|&i| oldest_topic(i)).min() {
            tellers.retain(|&i| oldest_topic(i) == Some(oldest));
        }
        tellers
    }

    #[test]
    fn listener_topic_filter_is_total_at_gain_and_threshold_boundaries() {
        for node in 0..32 {
            let threshold = defs().threshold(node);
            for gain in [
                0,
                1,
                threshold.saturating_sub(1),
                threshold,
                threshold.saturating_add(1),
                1000,
                u16::MAX,
            ] {
                for proficiency in [
                    0,
                    1,
                    threshold.saturating_sub(1),
                    threshold,
                    threshold.saturating_add(1),
                    gain.saturating_sub(1),
                    gain,
                    1000,
                ] {
                    let mut knowledge = Knowledge::default();
                    knowledge.set(node, proficiency, Source::Oral, 0);
                    let bytes = bincode::serialize(&knowledge).unwrap();
                    let loaded: Knowledge = bincode::deserialize(&bytes).unwrap();
                    for listener in [&knowledge, &loaded, &Knowledge::default()] {
                        let p = listener.proficiency(node);
                        assert!(
                            (!listener.practises(node) && p < gain) || p >= gain.min(threshold),
                            "node={node} p={p} gain={gain} threshold={threshold}"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn rehearsal_empty_telling_refreshes_both_participants_before_next_pair() {
        let (mut world, mut res, entities) = pair();
        certain_telling(&mut res);
        let shared = defs().id_of("pot_firing").unwrap();
        let other = defs().id_of("pressure_flaking").unwrap();
        for &entity in &entities {
            store(
                &mut world,
                entity,
                Knowledge::from_entries(vec![KnowledgeEntry::new(
                    shared,
                    Source::Discovered,
                    400,
                    0,
                )]),
            );
        }
        let pos = *world.get::<&Position>(entities[0]).unwrap();
        world.spawn((
            sim_core::components::Agent { id: 1000 },
            pos,
            Knowledge::from_entries(vec![KnowledgeEntry::new(other, Source::Discovered, 400, 0)]),
        ));
        let here = people(&world, &res);
        let before: Vec<_> = here
            .iter()
            .map(|p| knowledge_of(&world, p.entity))
            .collect();
        let choices = old_teller_choices(
            &here,
            &before,
            &before,
            0,
            res.current_tick,
            &rule(&res).unwrap(),
        );
        assert_eq!(choices, vec![1, 2]);
        res.world_seed = (0..1000)
            .find(|&seed| {
                keyed_below(
                    seed,
                    STREAM_KNOWLEDGE,
                    here[0].key,
                    res.current_tick,
                    0x1_0000,
                    0,
                    2,
                ) == 0
            })
            .unwrap();
        let mut after = before.clone();
        after[0].touch(shared, res.current_tick);
        after[1].touch(shared, res.current_tick);
        assert_eq!(
            old_teller_choices(
                &here,
                &before,
                &after,
                1,
                res.current_tick,
                &rule(&res).unwrap()
            ),
            vec![2]
        );
        OralTransmissionSystem.tick(&mut world, &mut res);
        let first = knowledge_of(&world, entities[0]);
        let second = knowledge_of(&world, entities[1]);
        assert_eq!(first.get(shared).unwrap().last_used, 10);
        assert_eq!(second.get(shared).unwrap().last_used, 10);
        assert_eq!(first.proficiency(other), 0);
        assert_eq!(second.proficiency(other), 350);
        assert!(res.colony.command_log.is_empty());
    }

    #[test]
    fn teller_choice_matches_frozen_two_pass_oracle() {
        let topics = [
            defs().id_of("pot_firing").unwrap(),
            defs().id_of("pressure_flaking").unwrap(),
        ];
        for days in [[0, 9], [0, 0], [10, 10]] {
            for informed in [false, true] {
                for seed in 0..8 {
                    let (mut world, mut res, entities) = pair();
                    certain_telling(&mut res);
                    res.world_seed = seed;
                    world
                        .get::<&mut sim_core::components::Agent>(entities[1])
                        .unwrap()
                        .id = 100;
                    store(
                        &mut world,
                        entities[0],
                        if informed {
                            Knowledge::from_entries(vec![KnowledgeEntry::new(
                                topics[0],
                                Source::Oral,
                                350,
                                0,
                            )])
                        } else {
                            Knowledge::default()
                        },
                    );
                    store(
                        &mut world,
                        entities[1],
                        Knowledge::from_entries(vec![KnowledgeEntry::new(
                            topics[0],
                            Source::Discovered,
                            400,
                            days[0],
                        )]),
                    );
                    let pos = *world.get::<&Position>(entities[0]).unwrap();
                    world.spawn((
                        sim_core::components::Agent { id: 1000 },
                        pos,
                        Knowledge::from_entries(vec![KnowledgeEntry::new(
                            topics[1],
                            Source::Discovered,
                            400,
                            days[1],
                        )]),
                    ));
                    let here = people(&world, &res);
                    assert_eq!(here[0].entity, entities[0]);
                    let before: Vec<_> = here
                        .iter()
                        .map(|p| knowledge_of(&world, p.entity))
                        .collect();
                    let choices = old_teller_choices(
                        &here,
                        &before,
                        &before,
                        0,
                        res.current_tick,
                        &rule(&res).unwrap(),
                    );
                    assert_eq!(choices, if days == [0, 9] { vec![1] } else { vec![1, 2] });
                    let pick = keyed_below(
                        seed,
                        STREAM_KNOWLEDGE,
                        here[0].key,
                        res.current_tick,
                        0x1_0000,
                        0,
                        choices.len() as u64,
                    );
                    let selected = choices[pick as usize] - 1;
                    OralTransmissionSystem.tick(&mut world, &mut res);
                    let known = knowledge_of(&world, entities[0]);
                    for (i, &topic) in topics.iter().enumerate() {
                        assert_eq!(
                            known.proficiency(topic),
                            if i == selected || (informed && i == 0) {
                                350
                            } else {
                                0
                            }
                        );
                        if let Some(entry) = known.get(topic) {
                            assert_eq!(entry.last_used, if i == selected { 10 } else { 0 });
                        }
                    }
                    assert!(!known.practises(topics[0]) && !known.practises(topics[1]));
                    assert!(res.colony.command_log.is_empty());
                }
            }
        }
    }

    #[test]
    fn deadline_teller_is_preferred_but_equal_deadlines_keep_the_original_draw() {
        for last_used in [0, 9] {
            // Given: the empty listener speaks first, before either teller is refreshed.
            let (mut world, mut res, entities) = pair();
            certain_telling(&mut res);
            world
                .get::<&mut sim_core::components::Agent>(entities[0])
                .unwrap()
                .id = 100;
            let old = defs().id_of("pot_firing").expect("pot");
            let recent = defs().id_of("pressure_flaking").expect("pressure");
            store(
                &mut world,
                entities[0],
                Knowledge::from_entries(vec![KnowledgeEntry::new(
                    old,
                    Source::Discovered,
                    400,
                    last_used,
                )]),
            );
            store(&mut world, entities[1], Knowledge::default());
            let pos = *world.get::<&Position>(entities[0]).expect("position");
            world.spawn((
                sim_core::components::Agent { id: 1000 },
                pos,
                Knowledge::from_entries(vec![KnowledgeEntry::new(
                    recent,
                    Source::Discovered,
                    if last_used == 9 { 400 } else { 500 },
                    9,
                )]),
            ));
            let listener = &people(&world, &res)[0];
            res.world_seed = (0..1000)
                .find(|&seed| {
                    keyed_below(
                        seed,
                        STREAM_KNOWLEDGE,
                        listener.key,
                        res.current_tick,
                        0x1_0000,
                        0,
                        2,
                    ) == 1
                })
                .expect("safe teller draw");
            // When: each listener still chooses only one real teller.
            OralTransmissionSystem.tick(&mut world, &mut res);
            // Then: earlier deadline wins; equal deadlines preserve the original keyed draw.
            let known = knowledge_of(&world, entities[1]);
            assert_eq!(known.proficiency(old), if last_used == 0 { 350 } else { 0 });
            assert_eq!(
                known.proficiency(recent),
                if last_used == 0 { 0 } else { 350 }
            );
            assert!(res.colony.command_log.is_empty());
        }
    }

    #[test]
    fn overdue_rehearsal_uses_the_experts_age_without_promoting_a_350_learner() {
        // Given: the listener's older safe memory would distract original pair-min selection.
        let (mut world, mut res, entities) = pair();
        let old = defs().id_of("pot_firing").expect("pot");
        let recent = defs().id_of("pressure_flaking").expect("pressure");
        store(
            &mut world,
            entities[0],
            Knowledge::from_entries(vec![
                KnowledgeEntry::new(old, Source::Discovered, 400, 1),
                KnowledgeEntry::new(recent, Source::Discovered, 500, 9),
            ]),
        );
        store(
            &mut world,
            entities[1],
            Knowledge::from_entries(vec![
                KnowledgeEntry::new(old, Source::Oral, 350, 8),
                KnowledgeEntry::new(recent, Source::Oral, 350, 0),
            ]),
        );
        // When: the informed listener rehearses with the expert.
        OralTransmissionSystem.tick(&mut world, &mut res);
        // Then: only the overdue expert topic is refreshed, with no proficiency grant.
        for &entity in &entities {
            assert_eq!(
                knowledge_of(&world, entity)
                    .get(old)
                    .expect("old")
                    .last_used,
                10
            );
        }
        assert_eq!(
            knowledge_of(&world, entities[0])
                .get(recent)
                .expect("recent")
                .last_used,
            9
        );
        assert_eq!(knowledge_of(&world, entities[1]).proficiency(old), 350);
        assert_eq!(res.colony.tallies.oral_learned, 0);
        assert_eq!(res.colony.tallies.oral_rehearsed, 1);
        assert!(res.colony.command_log.is_empty());
    }

    #[test]
    fn deadline_excludes_protected_lore_but_includes_the_exact_grace_boundary() {
        for case in 0..4 {
            // Given: ordinary ordering prefers recent knowledge over a nonurgent old topic.
            let (mut world, mut res, entities) = pair();
            certain_telling(&mut res);
            let old = defs()
                .id_of(if case == 2 {
                    "fire_keeping"
                } else {
                    "pot_firing"
                })
                .expect("old");
            let recent = defs().id_of("pressure_flaking").expect("pressure");
            let rule = res
                .rules
                .colony
                .as_mut()
                .expect("colony")
                .knowledge
                .as_mut()
                .expect("knowledge");
            if case == 0 {
                rule.decay[Source::Seeded.table_index()] = 0;
            }
            if case == 1 {
                rule.floor[Source::Seeded.table_index()] = defs().threshold(old);
            }
            store(
                &mut world,
                entities[0],
                Knowledge::from_entries(vec![
                    KnowledgeEntry::new(old, Source::Seeded, 500, if case == 3 { 4 } else { 0 }),
                    KnowledgeEntry::new(recent, Source::Discovered, 500, 9),
                ]),
            );
            store(&mut world, entities[1], Knowledge::default());
            res.world_seed = recent_topic_first_seed(&res, &people(&world, &res)[1], old, recent);
            // When: protected cases remain excluded; exact grace now has a finite deadline.
            OralTransmissionSystem.tick(&mut world, &mut res);
            // Then: S1-CAL-35 intentionally prioritizes the exact-grace case only.
            assert_eq!(
                knowledge_of(&world, entities[1]).proficiency(recent),
                if case == 3 { 0 } else { 350 },
                "case={case}"
            );
            assert_eq!(
                knowledge_of(&world, entities[1]).proficiency(old),
                if case == 3 { 350 } else { 0 },
                "case={case}"
            );
        }
    }

    #[test]
    fn overdue_priority_moves_on_after_an_actual_earlier_conversation() {
        // Given: two empty listeners precede two experts with distinct overdue topics.
        let (mut world, mut res, listeners) = pair();
        certain_telling(&mut res);
        for &entity in &listeners {
            store(&mut world, entity, Knowledge::default());
        }
        let first = defs().id_of("pot_firing").expect("pot");
        let second = defs().id_of("pressure_flaking").expect("pressure");
        let pos = *world.get::<&Position>(listeners[0]).expect("position");
        let experts: Vec<_> = [(first, 0), (second, 1)]
            .into_iter()
            .enumerate()
            .map(|(i, (node, day))| {
                world.spawn((
                    sim_core::components::Agent {
                        id: 1000 + i as u64,
                    },
                    pos,
                    Knowledge::from_entries(vec![KnowledgeEntry::new(
                        node,
                        Source::Discovered,
                        400,
                        day,
                    )]),
                ))
            })
            .collect();
        // Frozen selector recomputes against after, while eligibility remains the snapshot.
        let here = people(&world, &res);
        let before: Vec<_> = here
            .iter()
            .map(|p| knowledge_of(&world, p.entity))
            .collect();
        let first_choices = old_teller_choices(
            &here,
            &before,
            &before,
            0,
            res.current_tick,
            &rule(&res).unwrap(),
        );
        assert_eq!(first_choices, vec![2, 3]);
        let first_pick = keyed_below(
            res.world_seed,
            STREAM_KNOWLEDGE,
            here[0].key,
            res.current_tick,
            0x1_0000,
            0,
            first_choices.len() as u64,
        );
        assert_eq!(first_choices[first_pick as usize], 2);
        let mut after_first = before.clone();
        after_first[2].touch(first, res.current_tick);
        assert_eq!(
            old_teller_choices(
                &here,
                &before,
                &after_first,
                1,
                res.current_tick,
                &rule(&res).unwrap()
            ),
            vec![3]
        );
        // When: actual use in the first pair removes its topic's overdue priority.
        OralTransmissionSystem.tick(&mut world, &mut res);
        // Then: the second listener hears the other expert, without extra gains or promotion.
        for (&entity, (learned, absent)) in listeners.iter().zip([(first, second), (second, first)])
        {
            let known = knowledge_of(&world, entity);
            assert_eq!(known.proficiency(learned), 350);
            assert_eq!(known.proficiency(absent), 0);
            assert!(!known.practises(learned));
        }
        for (&entity, node) in experts.iter().zip([first, second]) {
            assert_eq!(
                knowledge_of(&world, entity)
                    .get(node)
                    .expect("expert")
                    .last_used,
                10
            );
        }
        assert!(res.colony.command_log.is_empty());
    }

    #[test]
    fn overdue_priority_use_does_not_make_a_new_learner_a_same_evening_teller() {
        // Given: ordinary fallback chooses the newly informed first listener for the second.
        let (mut world, mut res, listeners) = pair();
        certain_telling(&mut res);
        for &entity in &listeners {
            store(&mut world, entity, Knowledge::default());
        }
        let node = defs().id_of("flint_knapping").expect("flint");
        assert!(defs().threshold(node) <= rule(&res).expect("rule").oral_gain);
        let pos = *world.get::<&Position>(listeners[0]).expect("position");
        world.spawn((
            sim_core::components::Agent { id: 1000 },
            pos,
            Knowledge::from_entries(vec![KnowledgeEntry::new(node, Source::Discovered, 500, 0)]),
        ));
        let listener = &people(&world, &res)[1];
        res.world_seed = (0..1000)
            .find(|&seed| {
                keyed_below(
                    seed,
                    STREAM_KNOWLEDGE,
                    listener.key,
                    res.current_tick,
                    0x1_0000,
                    0,
                    2,
                ) == 0
            })
            .expect("new learner fallback draw");
        // When: the original expert has been used, but teaching eligibility stays in before.
        OralTransmissionSystem.tick(&mut world, &mut res);
        // Then: newly learned practice waits until tomorrow before it can be transmitted.
        assert_eq!(knowledge_of(&world, listeners[0]).proficiency(node), 350);
        assert!(knowledge_of(&world, listeners[0]).practises(node));
        assert_eq!(knowledge_of(&world, listeners[1]).proficiency(node), 0);
        assert_eq!(res.colony.tallies.oral_learned, 1);
        assert!(res.colony.command_log.is_empty());
    }

    #[test]
    fn overdue_failed_telling_uses_real_expertise_without_granting_knowledge() {
        // Given: the real pair can discuss the overdue topic, but no learning roll succeeds.
        let (mut world, mut res, entities) = pair();
        let rule = res
            .rules
            .colony
            .as_mut()
            .expect("colony")
            .knowledge
            .as_mut()
            .expect("knowledge");
        rule.oral_chance_permille = 0;
        rule.storytelling_bonus_permille = 0;
        let node = defs().id_of("pot_firing").expect("pot");
        store(
            &mut world,
            entities[0],
            Knowledge::from_entries(vec![KnowledgeEntry::new(node, Source::Discovered, 400, 0)]),
        );
        store(&mut world, entities[1], Knowledge::default());
        // When: the expert actually attempts the unchanged zero-chance telling.
        OralTransmissionSystem.tick(&mut world, &mut res);
        // Then: use is recorded, but no proficiency or extra learning is fabricated.
        assert_eq!(
            knowledge_of(&world, entities[0])
                .get(node)
                .expect("old")
                .last_used,
            10
        );
        assert_eq!(knowledge_of(&world, entities[1]).proficiency(node), 0);
        assert_eq!(res.colony.tallies.oral_tellings, 1);
        assert_eq!(res.colony.tallies.oral_learned, 0);
        assert_eq!(res.colony.tallies.oral_rehearsed, 0);
        assert!(res.colony.command_log.is_empty());
    }

    #[test]
    fn overdue_remote_expert_does_not_receive_free_use() {
        // Given: the sole expert is outside the actual gathering.
        let (mut world, mut res, entities) = pair();
        let node = defs().id_of("pot_firing").expect("pot");
        store(
            &mut world,
            entities[0],
            Knowledge::from_entries(vec![KnowledgeEntry::new(node, Source::Discovered, 400, 0)]),
        );
        store(&mut world, entities[1], Knowledge::default());
        *world.get::<&mut Position>(entities[0]).expect("position") = Position::new(0, 0);
        let person = people(&world, &res)
            .into_iter()
            .find(|p| p.entity == entities[0])
            .expect("expert");
        assert!(!gathered(
            &res,
            &crate::runtime::jobs::hearth::hearth_tiles(&res),
            rule(&res).expect("rule").gather_radius,
            &person
        ));
        // When: the remaining listener has no real teller.
        OralTransmissionSystem.tick(&mut world, &mut res);
        // Then: remote expertise does not refresh or produce learning.
        assert_eq!(
            knowledge_of(&world, entities[0])
                .get(node)
                .expect("old")
                .last_used,
            0
        );
        assert_eq!(res.colony.tallies.oral_learned, 0);
        assert_eq!(res.colony.tallies.oral_rehearsed, 0);
    }

    #[test]
    fn the_telling_runs_at_the_oral_minute_of_the_clock() {
        let mut res = SimResources::new(4, 4, sim_core::MaterialRegistry::new());
        res.rules = sim_core::rules::WorldRules::colony(0);
        let offset = OralTransmissionSystem.tick_offset(&res);
        // The colony clock starts at 08:00 (offset 480): 20:30 is raw tick 750 of the day.
        assert_eq!((offset + res.rules.clock_offset_ticks) % 1440, 1230);
        assert_eq!(offset, 750);
    }
}
