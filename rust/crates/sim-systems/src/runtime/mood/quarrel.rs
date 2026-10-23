//! QuarrelSystem: two unhappy or estranged people standing close may quarrel; a hostile pair in a black mood may come to blows.
//!
//! Priority: 173 (`QUARREL_PRIORITY`). Interval: 30 ticks. Gate: `ColonyRule::mood`.
//! Reads: `Agent`, `Position`, `AgentState`, `Mood`, `Kin`, `Lineage`, `Temperament`, `Drafted`; `SimResources.colony` (bonds, story), `relationships` (hostility), `rules`, `world_seed`
//! Writes: `Mood` (value, quarrel day); `SimResources.colony` (bonds, fights), `relationships` (hostility), `chronicle`
//! Events: chronicle `Quarrel` (19; cause 1 + the M5 cause code; the cause table → the initiator's largest negative modifier), `Fight` (cause → the quarrel)
//!
//! M5 (ADR 0030). The V7 `SocialInteractionSystem` has no quarrel and is left
//! alone; this system checks pairs every 30 ticks. Candidates: people with a
//! mood who are not asleep, not working (`Consuming { Job }`) and not drafted.
//! For each pair within `quarrel_tiles` (lower id first, pairs in id order; each person
//! checks at most the `quarrel_partners_max` nearest of the later ids, DECISIONS D18),
//! not kin-close (kin base ≥ `kin_base_min`), whose relation is below 0 or
//! one of whom is below `low`: the chance per check is
//! `base + max(0, low − min mood) / 25 + max(0, −relation) / 100` ‰, ×3 under
//! tantrum risk, at most `quarrel_cap_permille`, one keyed roll
//! (`STREAM_QUARREL`, subject = the lower lineage key, slot = the higher's).
//! A quarrel: the initiator is the lower mood (ties: lower id); the relation
//! loses `quarrel_relation` × ns / 500 of the initiator; hostility gains
//! `quarrel_hostility_permille`; both lose `quarrel_mood` and mark the day; the
//! cause is the initiator's largest negative modifier (hunger, cold, sleep,
//! grief, pressure; none or a quarrel already today: the relation). At
//! `fight_hostility_permille` with both moods below `low`, adjacent, the pair
//! fight once (`colony.fights`, `fight_damage`, one hit; chronicle `Fight`).
use hecs::{Entity, World};
use sim_core::chronicle::{CauseRef, ChronicleEntry, ChronicleKind};
use sim_core::colony::people::{Bond, Drafted, Mood};
use sim_core::colony::wildlife::{Fight, Fighter};
use sim_core::components::{
    Agent, AgentId, AgentState, Kin, Lineage, Position, RelationshipKey, TargetKind, Temperament,
};
use sim_core::rng::{keyed_below, STREAM_QUARREL};
use sim_core::rules::MoodRule;
use sim_engine::{RuntimeSystem, SimResources};

/// Priority of [`QuarrelSystem`].
pub const QUARREL_PRIORITY: u32 = 173;

/// Whether two people are kin-close: parent and child, siblings (a shared known parent), or partners.
pub fn kin_close(a: AgentId, ka: Option<&Kin>, b: AgentId, kb: Option<&Kin>) -> bool {
    let (Some(ka), Some(kb)) = (ka, kb) else {
        return false;
    };
    let parent = |k: &Kin, id| k.mother == Some(id) || k.father == Some(id);
    let shared = |x: Option<AgentId>, y: Option<AgentId>| x.is_some() && x == y;
    parent(ka, b)
        || parent(kb, a)
        || shared(ka.mother, kb.mother)
        || shared(ka.father, kb.father)
        || ka.partner == Some(b)
        || kb.partner == Some(a)
}

/// The base of a pair: `kin_base` for kin-close pairs; for others a keyed
/// "chemistry" in −`chemistry`..=`chemistry` (DECISIONS D11; `STREAM_QUARREL`,
/// subject = the lower lineage key, slot = the higher's, tick `u64::MAX`, which
/// no quarrel roll uses).
pub fn kin_base(rule: &MoodRule, close: bool, seed: u64, keys: (u64, u64)) -> i16 {
    if close {
        return rule.kin_base;
    }
    let (lo, hi) = if keys.0 <= keys.1 {
        keys
    } else {
        (keys.1, keys.0)
    };
    let span = u64::from(rule.chemistry.unsigned_abs()) * 2 + 1;
    (keyed_below(seed, STREAM_QUARREL, lo, u64::MAX, hi as u32, 0, span) as i64
        - i64::from(rule.chemistry)) as i16
}

/// The pair key `(lower, higher)`.
pub const fn pair(a: AgentId, b: AgentId) -> (AgentId, AgentId) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

/// The relation of `a` and `b` today (their kin base when never changed).
pub fn relation(res: &SimResources, rule: &MoodRule, a: AgentId, b: AgentId, base: i16) -> i16 {
    let day = super::today(res);
    res.colony
        .bonds
        .get(&pair(a, b))
        .map_or(base, |bond| bond.value_on(day, base, rule.relation_drift))
}

/// The quarrel chance ‰ of a pair with moods `ma`, `mb` and relation `rel` (goal 3.4).
pub fn chance(rule: &MoodRule, ma: i16, mb: i16, rel: i16) -> u32 {
    let low = ma.min(mb);
    let from_mood = (i32::from(rule.low) - i32::from(low)).max(0) / 25;
    let from_rel = (-i32::from(rel)).max(0) / 100;
    let mut c = rule.quarrel_base_permille as i32 + from_mood + from_rel;
    if low < rule.low {
        c *= 3;
    }
    (c.max(0) as u32).min(rule.quarrel_cap_permille)
}

/// One candidate: id, entity, tile, lineage key, mood value, novelty seeking, kin.
type Candidate = (AgentId, Entity, (u32, u32), u64, i16, u16, Option<Kin>);

/// See the module header.
#[derive(Debug, Default)]
pub struct QuarrelSystem;

impl RuntimeSystem for QuarrelSystem {
    fn name(&self) -> &str {
        "QuarrelSystem"
    }

    fn priority(&self) -> u32 {
        QUARREL_PRIORITY
    }

    fn tick_interval(&self) -> u64 {
        30
    }

    fn tick(&mut self, world: &mut World, res: &mut SimResources) {
        let Some(rule) = super::rule(res) else {
            return;
        };
        let mut folk: Vec<Candidate> = world
            .query::<(
                &Agent,
                &Position,
                &Mood,
                Option<&AgentState>,
                Option<&Lineage>,
                Option<&Temperament>,
                Option<&Kin>,
            )>()
            .without::<&Drafted>()
            .iter()
            .filter(|(_, (_, _, _, s, ..))| {
                !matches!(
                    s,
                    Some(AgentState::Consuming {
                        target: TargetKind::Sleep | TargetKind::Job
                    })
                )
            })
            .map(|(e, (a, p, m, _, l, t, k))| {
                (
                    a.id,
                    e,
                    (p.x, p.y),
                    l.map_or(a.id, |l| l.key),
                    m.value,
                    t.map_or(500, |t| t.ns),
                    k.copied(),
                )
            })
            .collect();
        if folk.len() < 2 {
            return;
        }
        folk.sort_by_key(|row| row.0);
        // Candidates by tile (indices ascending = `AgentId` ascending), for the near pairs.
        let reach = rule.quarrel_tiles;
        let most = rule.quarrel_partners_max as usize;
        let mut tiles: std::collections::BTreeMap<(u32, u32), Vec<usize>> =
            std::collections::BTreeMap::new();
        for (i, row) in folk.iter().enumerate() {
            tiles.entry(row.2).or_default().push(i);
        }
        let day = super::today(res);
        let tick = res.current_tick;
        let mut quarrelled: std::collections::BTreeSet<AgentId> = std::collections::BTreeSet::new();
        for i in 0..folk.len() {
            let a = folk[i];
            // The nearest `quarrel_partners_max` later ids within reach, by (distance, id), ring by ring
            // (D18); with fewer near, all of them. Then checked in id order.
            let mut near: Vec<usize> = Vec::new();
            for d in 0..=reach {
                let mut ring: Vec<usize> = Vec::new();
                let (x0, y0) = (a.2 .0.saturating_sub(d), a.2 .1.saturating_sub(d));
                for y in y0..=a.2 .1 + d {
                    for x in x0..=a.2 .0 + d {
                        if sim_core::nav::chebyshev(a.2, (x, y)) != d {
                            continue;
                        }
                        if let Some(list) = tiles.get(&(x, y)) {
                            let from = list.partition_point(|&j| j <= i);
                            ring.extend(list[from..].iter().take(most).copied());
                        }
                    }
                }
                ring.sort_unstable();
                near.extend(ring.into_iter().take(most - near.len()));
                if near.len() >= most {
                    break;
                }
            }
            near.sort_unstable();
            for j in near {
                let b = folk[j];
                if quarrelled.contains(&a.0) || quarrelled.contains(&b.0) {
                    continue;
                }
                if sim_core::nav::chebyshev(a.2, b.2) > reach {
                    continue;
                }
                let close = kin_close(a.0, a.6.as_ref(), b.0, b.6.as_ref());
                let base = kin_base(&rule, close, res.world_seed, (a.3, b.3));
                if base >= rule.kin_base_min {
                    continue;
                }
                let rel = relation(res, &rule, a.0, b.0, base);
                if rel >= 0 && a.4.min(b.4) >= rule.low {
                    continue;
                }
                let c = chance(&rule, a.4, b.4, rel);
                let (lo, hi) = if a.0 < b.0 { (&a, &b) } else { (&b, &a) };
                let roll = keyed_below(
                    res.world_seed,
                    STREAM_QUARREL,
                    lo.3,
                    tick,
                    hi.3 as u32,
                    0,
                    1000,
                );
                if roll >= u64::from(c) {
                    continue;
                }
                quarrelled.insert(a.0);
                quarrelled.insert(b.0);
                quarrel(world, res, &rule, &a, &b, rel, base, day);
            }
        }
    }
}

/// Book one quarrel between `a` and `b` (relation `rel` before it).
#[allow(clippy::too_many_arguments)]
fn quarrel(
    world: &mut World,
    res: &mut SimResources,
    rule: &MoodRule,
    a: &Candidate,
    b: &Candidate,
    rel: i16,
    base: i16,
    day: u16,
) {
    let (init, other) = if (a.4, a.0) <= (b.4, b.0) {
        (a, b)
    } else {
        (b, a)
    };
    // The cause: the initiator's largest negative modifier (the relation when none).
    let facts = super::facts_of(world, res, init.1);
    let mut mods = super::modifiers(&facts);
    mods.retain(|(m, w)| *w < 0 && *m != sim_core::colony::people::Modifier::Quarrelled);
    mods.sort_by_key(|(m, w)| (*w, *m));
    let previous = res
        .colony
        .bonds
        .get(&pair(a.0, b.0))
        .map(|b| b.last_quarrel)
        .filter(|e| *e != u32::MAX);
    let (code, cause_ref) = match mods.first() {
        Some((m, _)) => (
            m.quarrel_cause(),
            Some(super::cause_of(res, *m, facts.mood)),
        ),
        // Bad blood: the relation itself, pointing at the pair's previous quarrel.
        None => (
            5,
            Some(CauseRef::Modifier {
                code: sim_core::colony::people::RELATION_CAUSE,
                entry: previous,
            }),
        ),
    };
    let loss = i32::from(rule.quarrel_relation) * i32::from(init.5) / 500;
    let after = (i32::from(rel) - loss).clamp(-1000, 1000) as i16;
    let next_entry = res.chronicle.len() as u32;
    res.colony.bonds.insert(
        pair(a.0, b.0),
        Bond {
            value: after,
            day,
            last_quarrel: next_entry,
        },
    );
    let _ = base;
    let key = RelationshipKey::new(a.0, b.0);
    let hostility = {
        let r = res.relationships.entry(key).or_default();
        r.bump_hostility(f64::from(rule.quarrel_hostility_permille) / 1000.0);
        r.hostility
    };
    for e in [a.1, b.1] {
        if let Ok(mut m) = world.get::<&mut Mood>(e) {
            m.value = (m.value - rule.quarrel_mood).max(0);
            m.quarrel_day = day;
        }
    }
    let index = res.chronicle.push_caused(
        ChronicleEntry::new(res.current_tick, ChronicleKind::Quarrel)
            .subject(init.0)
            .other(Some(other.0))
            .cause(1 + code)
            .value(i64::from(after)),
        cause_ref,
    );
    res.colony.mood_tallies.quarrels += 1;
    let low = |e: Entity| world.get::<&Mood>(e).is_ok_and(|m| m.value < rule.low);
    if hostility * 1000.0 >= f64::from(rule.fight_hostility_permille)
        && low(a.1)
        && low(b.1)
        && sim_core::nav::chebyshev(a.2, b.2) <= 1
    {
        res.colony.fights.insert(
            (Fighter::Person(init.0), Fighter::Person(other.0)),
            Fight {
                progress: 0,
                ticks: 1,
                damage: rule.fight_damage,
                hits_left: 1,
                quarrel: Some(code),
            },
        );
        res.colony.mood_tallies.fights += 1;
        res.chronicle.push_caused(
            ChronicleEntry::new(res.current_tick, ChronicleKind::Fight)
                .subject(init.0)
                .other(Some(other.0))
                .cause(1 + code)
                .value(i64::from(rule.fight_damage)),
            Some(CauseRef::Entry(index)),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chance_follows_the_goal_formula() {
        let rule = MoodRule::M5;
        // Content and on good terms: only the base (the system skips such pairs anyway).
        assert_eq!(chance(&rule, 600, 600, 0), 2);
        // Relation −300: 2 + 3.
        assert_eq!(chance(&rule, 600, 600, -300), 5);
        // Mood 100 (tantrum): (2 + 150/25) × 3 = 24.
        assert_eq!(chance(&rule, 100, 700, 0), 24);
        // Capped at 40.
        assert_eq!(chance(&rule, 0, 0, -1000), 40);
    }

    #[test]
    fn kin_close_pairs_are_parents_children_siblings_and_partners() {
        let k = |m: Option<u64>, f: Option<u64>, p: Option<u64>| Kin {
            mother: m,
            father: f,
            partner: p,
            last_birth_tick: None,
        };
        assert!(kin_close(
            1,
            Some(&k(Some(2), None, None)),
            2,
            Some(&k(None, None, None))
        ));
        assert!(kin_close(
            1,
            Some(&k(Some(9), None, None)),
            3,
            Some(&k(Some(9), None, None))
        ));
        assert!(kin_close(
            1,
            Some(&k(None, None, Some(4))),
            4,
            Some(&k(None, None, None))
        ));
        assert!(!kin_close(
            1,
            Some(&k(None, None, None)),
            5,
            Some(&k(None, None, None))
        ));
        assert!(!kin_close(1, None, 2, None));
    }
}
