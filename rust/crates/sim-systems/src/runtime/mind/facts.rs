//! Registered factual readers. No provider chooses a behaviour or mutates ECS.
//! Numeric needs are millipoints (round to nearest); breach booleans use the
//! original floating comparison, so rounding cannot move a threshold edge.

use super::context::{ContextError, Domain, PersonSnapshot, TargetCandidate, TickContext};
use crate::runtime::agent_actions::{forage::ForagePlan, needs, targets};
use sim_core::colony::mind::{
    CompiledMind, FactId, FactKind, FactSpec, LifeStage, MindEvalError, PersonFacts,
};
use sim_core::colony::{JobKind, JobStage};
use sim_core::components::{
    ActivityKind, AgentId, AgentState, SettlementId, TargetKind, INVENTORY_CAPACITY,
};
use sim_engine::SimResources;
use std::collections::BTreeMap;
use std::fmt;

#[cfg(test)]
#[path = "fact_binding_tests.rs"]
mod fact_binding_tests;

#[cfg(test)]
#[path = "food_index_tests.rs"]
mod food_index_tests;

/// Failure to bind or populate factual evidence; missing values never become zero.
#[derive(Debug)]
pub enum FactError {
    /// Required snapshot evidence was absent or invalidated.
    Context(ContextError),
    /// The typed fact table rejected a row, column, or value.
    Table(MindEvalError),
    /// A compiled fact has no registered reader.
    UnknownProvider(String),
    /// Two readers attempted to register the same fact name.
    DuplicateProvider(String),
    /// Compiled and registered fact kinds disagree.
    KindMismatch(String),
    /// Resources, snapshot, and table do not represent the same tick.
    ContextTickMismatch,
    /// This bound provider plan belongs to another policy revision.
    PolicyMismatch,
    /// A node-keyed random fact must belong to exactly one authored node.
    RandomFactNodeCount {
        /// Registered random-input fact name.
        fact: String,
        /// Number of distinct authored nodes referencing it.
        users: usize,
    },
    /// A required optional component or prepared observation is absent.
    MissingValue {
        /// Person whose factual input is absent.
        agent: AgentId,
        /// Registered input that could not be populated.
        fact: String,
    },
    /// A numeric value is non-finite or cannot fit the declared integer unit.
    InvalidNumber {
        /// Person whose numeric input was rejected.
        agent: AgentId,
        /// Registered input with an invalid numeric representation.
        fact: String,
    },
}

impl fmt::Display for FactError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for FactError {}
impl From<ContextError> for FactError {
    fn from(value: ContextError) -> Self {
        Self::Context(value)
    }
}
impl From<MindEvalError> for FactError {
    fn from(value: MindEvalError) -> Self {
        Self::Table(value)
    }
}

/// Pure factual reader; the final argument identifies the registered input in errors.
pub type FactReader =
    fn(&PersonSnapshot, &TickContext, &SimResources, &str) -> Result<i64, FactError>;
/// Registered factual implementation and its declared scalar kind.
#[derive(Clone, Copy)]
pub struct FactProvider {
    /// Stable name resolved against policy facts during binding.
    pub name: &'static str,
    /// Boolean or signed scalar type enforced by the fact table.
    pub kind: FactKind,
    /// Read-only implementation; never selects a policy node or mutates ECS.
    pub read: FactReader,
    /// Node-bound random input, not a global or content-name random stream.
    pub node_random: bool,
}

fn missing(agent: AgentId, name: &str) -> FactError {
    FactError::MissingValue {
        agent,
        fact: name.to_owned(),
    }
}
fn milli(value: Option<f64>, agent: AgentId, name: &str) -> Result<i64, FactError> {
    let value = value.ok_or_else(|| missing(agent, name))? * 1000.0;
    if !value.is_finite() || value < i64::MIN as f64 || value >= i64::MAX as f64 {
        return Err(FactError::InvalidNumber {
            agent,
            fact: name.to_owned(),
        });
    }
    Ok(value.round() as i64)
}
fn tenths(value: Option<f64>, agent: AgentId, name: &str) -> Result<i64, FactError> {
    let value = value.ok_or_else(|| missing(agent, name))? * 10.0;
    if !value.is_finite() || value < i64::MIN as f64 || value >= i64::MAX as f64 {
        return Err(FactError::InvalidNumber {
            agent,
            fact: name.to_owned(),
        });
    }
    Ok(value as i64)
}
fn fatigue_threshold(p: &PersonSnapshot, r: &SimResources) -> f64 {
    let day_tick = (r.ticks_per_day != 0)
        .then(|| r.current_tick.saturating_add(r.rules.clock_offset_ticks) % r.ticks_per_day);
    needs::person_night_rest(
        r.rules.agent.night_rest,
        r.rules.agent.temperament,
        p.temperament.as_ref(),
        day_tick,
    )
    .map_or(needs::FATIGUE_THRESHOLD, |rule| rule.fatigue_threshold)
}
fn derived(
    p: &PersonSnapshot,
    c: &TickContext,
    _: &SimResources,
    name: &str,
) -> Result<i64, FactError> {
    Ok(c.derived(p.agent, name)?)
}

fn unbound_draw(
    p: &PersonSnapshot,
    _: &TickContext,
    _: &SimResources,
    name: &str,
) -> Result<i64, FactError> {
    Err(missing(p.agent, name))
}
/// Exact ordering against the configured threshold, retaining sub-millipoint edges.
fn critical_order(
    value: Option<f64>,
    p: &PersonSnapshot,
    r: &SimResources,
    name: &str,
) -> Result<i64, FactError> {
    let value = value.ok_or_else(|| missing(p.agent, name))?;
    let limit = r
        .rules
        .agent
        .critical_preemption
        .unwrap_or(needs::CRITICAL_NEED);
    if !value.is_finite() || !limit.is_finite() {
        return Err(FactError::InvalidNumber {
            agent: p.agent,
            fact: name.to_owned(),
        });
    }
    Ok(if value < limit {
        -1
    } else if value > limit {
        1
    } else {
        0
    })
}

fn routine_preference(
    p: &PersonSnapshot,
    r: &SimResources,
    kind: usize,
    name: &str,
) -> Result<i64, FactError> {
    let Some(rule) = r.rules.agent.routine else {
        return Ok(0);
    };
    let temperament = p.temperament.ok_or_else(|| missing(p.agent, name))?;
    let weights = rule.adjusted_weights(
        p.life_stage == LifeStage::Child,
        p.life_stage == LifeStage::Elder,
        &temperament,
    );
    let total: u64 = weights.iter().map(|v| u64::from(*v)).sum();
    Ok(if total == 0 {
        0
    } else {
        (u64::from(weights[kind]) * 1000 / total) as i64
    })
}
macro_rules! fact {
    ($name:literal,$kind:ident,|$p:ident,$c:ident,$r:ident,$n:ident| $body:expr) => {
        FactProvider {
            name: $name,
            kind: FactKind::$kind,
            read: |$p, $c, $r, $n| $body,
            node_random: false,
        }
    };
}
macro_rules! overlay {
    ($name:literal,$kind:ident) => {
        FactProvider {
            name: $name,
            kind: FactKind::$kind,
            read: derived,
            node_random: false,
        }
    };
}

/// Names are registration data, not a selector's content-name dispatch.
pub const BUILTIN_PROVIDERS: &[FactProvider] = &[
    FactProvider {
        name: "routine_tend_fire_draw",
        kind: FactKind::Signed,
        read: unbound_draw,
        node_random: true,
    },
    fact!("routine_tend_fire_preference", Signed, |p, _c, r, n| {
        routine_preference(p, r, 0, n)
    }),
    FactProvider {
        name: "routine_craft_draw",
        kind: FactKind::Signed,
        read: unbound_draw,
        node_random: true,
    },
    fact!("routine_craft_preference", Signed, |p, _c, r, n| {
        routine_preference(p, r, 1, n)
    }),
    FactProvider {
        name: "routine_repair_draw",
        kind: FactKind::Signed,
        read: unbound_draw,
        node_random: true,
    },
    fact!("routine_repair_preference", Signed, |p, _c, r, n| {
        routine_preference(p, r, 2, n)
    }),
    FactProvider {
        name: "routine_hearth_circle_draw",
        kind: FactKind::Signed,
        read: unbound_draw,
        node_random: true,
    },
    fact!("routine_hearth_circle_preference", Signed, |p, _c, r, n| {
        routine_preference(p, r, 3, n)
    }),
    FactProvider {
        name: "routine_explore_draw",
        kind: FactKind::Signed,
        read: unbound_draw,
        node_random: true,
    },
    fact!("routine_explore_preference", Signed, |p, _c, r, n| {
        routine_preference(p, r, 4, n)
    }),
    FactProvider {
        name: "routine_play_draw",
        kind: FactKind::Signed,
        read: unbound_draw,
        node_random: true,
    },
    fact!("routine_play_preference", Signed, |p, _c, r, n| {
        routine_preference(p, r, 5, n)
    }),
    FactProvider {
        name: "routine_sit_by_fire_draw",
        kind: FactKind::Signed,
        read: unbound_draw,
        node_random: true,
    },
    fact!("routine_sit_by_fire_preference", Signed, |p, _c, r, n| {
        routine_preference(p, r, 6, n)
    }),
    fact!("hunger_tenths", Signed, |p, _c, _r, n| tenths(
        p.hunger, p.agent, n
    )),
    fact!("thirst_tenths", Signed, |p, _c, _r, n| tenths(
        p.thirst, p.agent, n
    )),
    fact!("fatigue_tenths", Signed, |p, _c, _r, n| tenths(
        p.fatigue, p.agent, n
    )),
    fact!("hunger_threshold_tenths", Signed, |p, _c, _r, n| tenths(
        Some(f64::from(needs::HUNGER_THRESHOLD)),
        p.agent,
        n
    )),
    fact!("thirst_threshold_tenths", Signed, |p, _c, _r, n| tenths(
        Some(needs::THIRST_THRESHOLD),
        p.agent,
        n
    )),
    fact!("fatigue_threshold_tenths", Signed, |p, _c, r, n| tenths(
        Some(fatigue_threshold(p, r)),
        p.agent,
        n
    )),
    fact!("critical_threshold_tenths", Signed, |p, _c, r, n| tenths(
        Some(
            r.rules
                .agent
                .critical_preemption
                .unwrap_or(needs::CRITICAL_NEED)
        ),
        p.agent,
        n
    )),
    fact!("yield_margin_tenths", Signed, |p, _c, r, n| Ok(i64::from(
        r.rules
            .colony
            .and_then(|rule| rule.yield_margin_permille)
            .ok_or_else(|| missing(p.agent, n))?
    ))),
    fact!("finish_leg_tiles", Signed, |p, _c, r, n| Ok(i64::from(
        r.rules
            .colony
            .and_then(|rule| rule.finish_leg_tiles)
            .ok_or_else(|| missing(p.agent, n))?
    ))),
    fact!("job_harvesting", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.job
            .is_some_and(|job| matches!(job.kind, JobKind::Harvest { .. }))
    ))),
    fact!("craft_working", Bool, |p, _c, r, _n| Ok(i64::from(
        r.rules.colony.is_some_and(|rule| rule.economy.is_some())
            && p.job
                .is_some_and(|job| matches!(job.kind, JobKind::Craft { .. })
                    && matches!(job.stage, JobStage::Working { .. }))
    ))),
    // MAX is a documented no-destination distance, not an unknown measurement.
    fact!("job_destination_distance", Signed, |p, _c, _r, _n| Ok(p
        .job
        .and_then(|job| job.kind.dest())
        .map_or(i64::MAX, |to| i64::from(
            p.position.x.abs_diff(to.0).max(p.position.y.abs_diff(to.1))
        )))),
    fact!("hunger_excess_tenths", Signed, |p, _c, _r, n| Ok(tenths(
        p.hunger, p.agent, n
    )? - 500)),
    fact!("thirst_excess_tenths", Signed, |p, _c, _r, n| Ok(tenths(
        p.thirst, p.agent, n
    )? - 500)),
    fact!("fatigue_excess_tenths", Signed, |p, _c, r, n| Ok(tenths(
        p.fatigue, p.agent, n
    )?
        - tenths(Some(fatigue_threshold(p, r)), p.agent, n)?)),
    fact!("worker", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.life_stage == LifeStage::Adult
    ))),
    fact!("breakdown", Bool, |p, _c, r, _n| Ok(i64::from(
        crate::runtime::mood::rule(r).is_some()
            && p.mood
                .is_some_and(|mood| mood.broken(crate::runtime::mood::today(r)))
    ))),
    fact!("edible_plant_available", Bool, |p, c, r, _n| Ok(i64::from(
        edible_plant(p, c, r).is_some()
    ))),
    fact!("hunger_actionable", Bool, |p, c, r, _n| Ok(i64::from(
        p.hunger
            .is_some_and(|v| v > f64::from(needs::HUNGER_THRESHOLD))
            && physical_food_available(p, c, r, "food")?
    ))),
    fact!("thirst_actionable", Bool, |p, c, _r, _n| Ok(i64::from(
        p.thirst.is_some_and(|v| v > needs::THIRST_THRESHOLD) && c.availability.water
    ))),
    fact!("fatigue_actionable", Bool, |p, c, r, _n| Ok(i64::from(
        p.fatigue.is_some_and(|v| v > fatigue_threshold(p, r)) && c.availability.sleep
    ))),
    fact!("stocked_camp_in_radius", Bool, |p, c, r, _n| {
        let scope = c.target_scope(p.agent)?;
        Ok(i64::from(
            scope.camp_stocked
                && r.rules
                    .society
                    .meals
                    .zip(scope.home)
                    .is_some_and(|(rule, home)| {
                        p.position
                            .x
                            .abs_diff(home.0)
                            .max(p.position.y.abs_diff(home.1))
                            <= rule.camp_radius
                    }),
        ))
    }),
    fact!("camp_stocked", Bool, |p, c, _r, _n| Ok(i64::from(
        c.target_scope(p.agent)?.camp_stocked
    ))),
    fact!("within_camp_radius", Bool, |p, c, r, _n| Ok(i64::from(
        r.rules
            .society
            .meals
            .zip(c.target_scope(p.agent)?.home)
            .is_some_and(|(rule, home)| p
                .position
                .x
                .abs_diff(home.0)
                .max(p.position.y.abs_diff(home.1))
                <= rule.camp_radius)
    ))),
    fact!("stock_in_hand", Signed, |p, c, _r, n| i64::try_from(
        c.target_scope(p.agent)?.stock_in_hand
    )
    .map_err(|_| FactError::InvalidNumber {
        agent: p.agent,
        fact: n.into()
    })),
    // An absent plan has no cap: -1 denotes below an unbounded limit.
    fact!("stock_hoard_order", Signed, |p, c, _r, n| {
        let scope = c.target_scope(p.agent)?;
        let Some(limit) = scope.stock_hoard_limit else {
            return Ok(-1);
        };
        if !limit.is_finite() {
            return Err(FactError::InvalidNumber {
                agent: p.agent,
                fact: n.into(),
            });
        }
        let amount = scope.stock_in_hand as f64;
        Ok(if amount < limit {
            -1
        } else if amount > limit {
            1
        } else {
            0
        })
    }),
    fact!("social_seek_enabled", Bool, |_p, _c, r, _n| Ok(i64::from(
        r.rules.agent.social_seek.is_some()
    ))),
    fact!("stock_shortfall", Bool, |p, c, _r, _n| Ok(i64::from(
        c.target_scope(p.agent)?.stock_shortfall
    ))),
    fact!("forage_window", Bool, |_p, _c, r, _n| Ok(i64::from(
        r.rules.colony.is_none()
            && r.rules
                .agent
                .forage
                .filter(|_| r.rules.society.stock_target.is_some())
                .is_some_and(|rule| needs::clock_in_window(r, rule.start_minute, rule.end_minute))
    ))),
    fact!("food_carry_loaded", Bool, |p, _c, r, n| {
        if r.rules.colony.is_some() {
            return Ok(i64::from(p.physical_food_units(r) > 0));
        }
        let Some(rule) = r
            .rules
            .agent
            .forage
            .filter(|_| r.rules.society.stock_target.is_some())
        else {
            return Ok(0);
        };
        let load = if p.life_stage == LifeStage::Adult {
            r.rules
                .agent
                .temperament
                .zip(p.temperament)
                .map_or(rule.haul_threshold, |(lean, t)| {
                    lean.food_haul(rule.haul_threshold, &t)
                })
        } else {
            rule.haul_threshold
        };
        Ok(i64::from(
            p.inventory_food.ok_or_else(|| missing(p.agent, n))? >= load,
        ))
    }),
    fact!("hearth_available", Bool, |p, c, r, _n| Ok(i64::from(
        c.target_scope(p.agent)?
            .home
            .is_some_and(|home| r.nav.reachable((p.position.x, p.position.y), home))
    ))),
    fact!("target_social", Bool, |p, _c, _r, _n| Ok(i64::from(
        matches!(p.state.target(), Some(TargetKind::Agent(_)))
    ))),
    fact!("social_partner_present", Bool, |p, c, _r, _n| Ok(
        i64::from(match p.state.target() {
            Some(TargetKind::Agent(peer)) => c.row(peer).is_ok(),
            _ => false,
        })
    )),
    fact!("social_chase_expired", Bool, |p, _c, r, _n| Ok(i64::from(
        r.rules.agent.social_seek.is_some_and(|rule| r
            .social_chase_since
            .get(&p.agent)
            .is_some_and(|since| r.current_tick.saturating_sub(*since) >= rule.max_chase_ticks))
    ))),
    fact!("social_retry_ready", Bool, |p, _c, r, _n| Ok(i64::from(
        r.rules.agent.social_seek.is_none()
            || r.social_retry_after
                .get(&p.agent)
                .is_none_or(|after| r.current_tick >= *after)
    ))),
    fact!("activity_kind", Signed, |p, _c, _r, _n| Ok(p
        .activity
        .map_or(-1, |a| a.kind.index() as i64))),
    fact!("own_site", Bool, |p, _c, r, _n| Ok(i64::from(
        p.state.target() == Some(TargetKind::ConstructionSite)
            && r.colony
                .occupied
                .contains_key(&(p.position.x, p.position.y))
    ))),
    fact!("hunger", Signed, |p, _c, _r, n| milli(p.hunger, p.agent, n)),
    fact!("thirst", Signed, |p, _c, _r, n| milli(p.thirst, p.agent, n)),
    fact!("fatigue", Signed, |p, _c, _r, n| milli(
        p.fatigue, p.agent, n
    )),
    fact!("social", Signed, |p, _c, _r, n| milli(p.social, p.agent, n)),
    fact!("hunger_threshold", Signed, |p, _c, _r, n| milli(
        Some(f64::from(needs::HUNGER_THRESHOLD)),
        p.agent,
        n
    )),
    fact!("thirst_threshold", Signed, |p, _c, _r, n| milli(
        Some(needs::THIRST_THRESHOLD),
        p.agent,
        n
    )),
    fact!("fatigue_threshold", Signed, |p, _c, r, n| milli(
        Some(fatigue_threshold(p, r)),
        p.agent,
        n
    )),
    fact!("social_threshold", Signed, |p, _c, _r, n| milli(
        Some(needs::SOCIAL_THRESHOLD),
        p.agent,
        n
    )),
    fact!("critical_preemption_enabled", Bool, |_p, _c, r, _n| Ok(
        i64::from(r.rules.agent.critical_preemption.is_some())
    )),
    fact!("preempt_gathering_enabled", Bool, |_p, _c, r, _n| Ok(
        i64::from(r.rules.agent.preempt_gathering)
    )),
    fact!("hunger_critical_order", Signed, |p, _c, r, n| {
        critical_order(p.hunger, p, r, n)
    }),
    fact!("thirst_critical_order", Signed, |p, _c, r, n| {
        critical_order(p.thirst, p, r, n)
    }),
    fact!("fatigue_critical_order", Signed, |p, _c, r, n| {
        critical_order(p.fatigue, p, r, n)
    }),
    fact!("critical_threshold", Signed, |p, _c, r, n| milli(
        Some(
            r.rules
                .agent
                .critical_preemption
                .unwrap_or(needs::CRITICAL_NEED)
        ),
        p.agent,
        n
    )),
    fact!("hungry", Bool, |p, _c, _r, n| Ok(i64::from(
        p.hunger.ok_or_else(|| missing(p.agent, n))? > f64::from(needs::HUNGER_THRESHOLD)
    ))),
    fact!("thirsty", Bool, |p, _c, _r, n| Ok(i64::from(
        p.thirst.ok_or_else(|| missing(p.agent, n))? > needs::THIRST_THRESHOLD
    ))),
    fact!("tired", Bool, |p, _c, r, n| Ok(i64::from(
        p.fatigue.ok_or_else(|| missing(p.agent, n))? > fatigue_threshold(p, r)
    ))),
    fact!("lonely", Bool, |p, _c, _r, n| Ok(i64::from(
        p.social.ok_or_else(|| missing(p.agent, n))? > needs::SOCIAL_THRESHOLD
    ))),
    fact!("night_window", Bool, |p, _c, r, _n| {
        let day = (r.ticks_per_day != 0)
            .then(|| r.current_tick.saturating_add(r.rules.clock_offset_ticks) % r.ticks_per_day);
        Ok(i64::from(
            needs::person_night_rest(
                r.rules.agent.night_rest,
                r.rules.agent.temperament,
                p.temperament.as_ref(),
                day,
            )
            .is_some(),
        ))
    }),
    fact!("temperament_ns", Signed, |p, _c, _r, n| Ok(i64::from(
        p.temperament.ok_or_else(|| missing(p.agent, n))?.ns
    ))),
    fact!("temperament_ha", Signed, |p, _c, _r, n| Ok(i64::from(
        p.temperament.ok_or_else(|| missing(p.agent, n))?.ha
    ))),
    fact!("temperament_rd", Signed, |p, _c, _r, n| Ok(i64::from(
        p.temperament.ok_or_else(|| missing(p.agent, n))?.rd
    ))),
    fact!("temperament_p", Signed, |p, _c, _r, n| Ok(i64::from(
        p.temperament.ok_or_else(|| missing(p.agent, n))?.p
    ))),
    fact!("idle", Bool, |p, _c, _r, _n| Ok(i64::from(matches!(
        p.state,
        AgentState::Idle
    )))),
    fact!("seeking", Bool, |p, _c, _r, _n| Ok(i64::from(matches!(
        p.state,
        AgentState::Seeking { .. }
    )))),
    fact!("consuming", Bool, |p, _c, _r, _n| Ok(i64::from(matches!(
        p.state,
        AgentState::Consuming { .. }
    )))),
    fact!("drafted", Bool, |p, _c, _r, _n| Ok(i64::from(p.drafted))),
    fact!("fleeing", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.fleeing.is_some()
    ))),
    fact!("cold", Bool, |p, _c, _r, _n| Ok(i64::from(p.cold))),
    fact!("sick", Bool, |p, _c, _r, _n| Ok(i64::from(p.sick))),
    fact!("hp", Signed, |p, _c, _r, n| milli(p.hp, p.agent, n)),
    fact!("mood", Signed, |p, _c, _r, n| Ok(i64::from(
        p.mood.ok_or_else(|| missing(p.agent, n))?.value
    ))),
    fact!("job_owned_state", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.job.is_some()
            && match p.state {
                AgentState::Seeking {
                    target: TargetKind::Job,
                }
                | AgentState::Consuming {
                    target: TargetKind::Job,
                } => true,
                AgentState::Consuming {
                    target: TargetKind::ConstructionSite,
                } => true,
                AgentState::Idle | AgentState::Seeking { .. } | AgentState::Consuming { .. } =>
                    false,
            }
    ))),
    fact!("has_job", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.job.is_some()
    ))),
    fact!("job_working", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.job
            .is_some_and(|j| matches!(j.stage, JobStage::Working { .. }))
    ))),
    fact!("job_to_source", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.job.is_some_and(|j| matches!(j.stage, JobStage::ToSource))
    ))),
    fact!("job_to_dest", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.job.is_some_and(|j| matches!(j.stage, JobStage::ToDest))
    ))),
    fact!("job_distance", Signed, |p, _c, _r, n| {
        let Some(job) = p.job else {
            return Ok(i64::MAX);
        };
        let goal = match job.stage {
            JobStage::ToSource => job.fetch.unwrap_or_else(|| job.kind.source()),
            JobStage::Working { .. } => job.kind.source(),
            JobStage::ToDest => job.kind.dest().ok_or_else(|| missing(p.agent, n))?,
        };
        Ok(i64::from(
            p.position
                .x
                .abs_diff(goal.0)
                .max(p.position.y.abs_diff(goal.1)),
        ))
    }),
    fact!("carrying", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.carry.is_some()
    ))),
    fact!("carry_count", Signed, |p, _c, _r, _n| Ok(p
        .carry
        .map_or(0, |v| i64::from(v.count)))),
    fact!("inventory_total", Signed, |p, _c, r, n| Ok(i64::from(
        inventory_total(p, r, n)?
    ))),
    fact!("inventory_food", Signed, |p, _c, r, n| {
        if r.rules.colony.is_some() {
            return i64::try_from(p.physical_food_units(r)).map_err(|_| FactError::InvalidNumber {
                agent: p.agent,
                fact: n.into(),
            });
        }
        Ok(i64::from(
            p.inventory_food.ok_or_else(|| missing(p.agent, n))?,
        ))
    }),
    fact!("inventory_room", Signed, |p, _c, r, n| Ok(i64::from(
        INVENTORY_CAPACITY.saturating_sub(inventory_total(p, r, n)?)
    ))),
    fact!("activity_active", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.activity.is_some()
    ))),
    fact!("activity_arrived", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.activity.is_some_and(|v| v.arrived_tick.is_some())
    ))),
    fact!("activity_resting", Bool, |p, c, _r, _n| Ok(i64::from(
        p.activity_rest.is_some_and(|v| v.until_tick > c.tick)
    ))),
    fact!("has_memory", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.memory_entries.is_some()
    ))),
    fact!("memory_entries", Signed, |p, _c, _r, n| i64::try_from(
        p.memory_entries.ok_or_else(|| missing(p.agent, n))?
    )
    .map_err(|_| FactError::InvalidNumber {
        agent: p.agent,
        fact: n.to_owned()
    })),
    fact!("has_home", Bool, |p, c, _r, _n| Ok(i64::from(
        c.target_scope(p.agent)?.home.is_some()
    ))),
    // Availability is independent of priority; target helpers retain their own tie order.
    fact!("food_available", Bool, |p, c, r, _n| Ok(i64::from(
        physical_food_available(p, c, r, "food")?
    ))),
    fact!("food_cooked_available", Bool, |p, c, r, _n| Ok(i64::from(
        physical_food_available(p, c, r, "food_cooked")?
    ))),
    fact!("food_novel_available", Bool, |p, c, r, _n| Ok(i64::from(
        physical_food_available(p, c, r, "food_novel")?
    ))),
    fact!("food_cooked_novel_available", Bool, |p, c, r, _n| Ok(
        i64::from(physical_food_available(p, c, r, "food_cooked_novel")?)
    )),
    fact!("water_available", Bool, |_p, c, _r, _n| Ok(i64::from(
        c.availability.water
    ))),
    fact!("sleep_available", Bool, |_p, c, _r, _n| Ok(i64::from(
        c.availability.sleep
    ))),
    fact!("gather_available", Bool, |p, c, r, _n| Ok(i64::from(
        c.gather_available(p.agent, r)?
    ))),
    fact!("at_food", Bool, |p, c, r, _n| {
        if r.rules.colony.is_some() {
            Ok(i64::from(
                super::food_targets::candidate(
                    &c.physical_food.borrow(),
                    p,
                    r,
                    super::food_targets::FoodFilter::Any,
                )
                .is_some_and(|target| target.tile == (p.position.x, p.position.y)),
            ))
        } else {
            Ok(i64::from(
                r.food_tiles.contains_key(&(p.position.x, p.position.y)),
            ))
        }
    }),
    fact!("at_water", Bool, |p, _c, r, _n| Ok(i64::from(
        r.water_tiles.contains_key(&(p.position.x, p.position.y))
    ))),
    fact!("at_sleep", Bool, |p, _c, r, _n| Ok(i64::from(
        r.sleep_tiles.contains_key(&(p.position.x, p.position.y))
    ))),
    fact!("target_food", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.state.target() == Some(TargetKind::Food)
    ))),
    fact!("target_water", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.state.target() == Some(TargetKind::Water)
    ))),
    fact!("target_sleep", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.state.target() == Some(TargetKind::Sleep)
    ))),
    fact!("target_gather", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.state.target() == Some(TargetKind::GatherFood)
    ))),
    // These require the coordinator's factual lifecycle/claim view, not an old chooser.
    fact!("hearth_lit", Bool, |p, c, r, _n| Ok(i64::from(
        c.target_scope(p.agent)?
            .settlement
            .and_then(|id| r.fires.get(&id))
            .is_some_and(|fire| fire.lit)
    ))),
    fact!("reachable_shelter", Bool, |p, _c, r, _n| Ok(i64::from(
        r.colony
            .rooms
            .values()
            .filter(|room| room.roofed)
            .any(|room| room
                .tiles
                .iter()
                .any(|tile| r.nav.reachable((p.position.x, p.position.y), *tile)))
    ))),
    fact!("threat_distance", Signed, |p, c, _r, _n| Ok(c
        .threats
        .iter()
        .map(|(_, tile)| i64::from(
            p.position
                .x
                .abs_diff(tile.0)
                .max(p.position.y.abs_diff(tile.1))
        ))
        .min()
        .unwrap_or(i64::MAX))),
    overlay!("social_candidate", Bool),
    overlay!("social_memory_candidate", Bool),
    overlay!("social_handshake", Bool),
    fact!("running_foraging", Bool, |p, c, _r, _n| Ok(i64::from(
        c.running(p.agent)?
            .is_some_and(|running| !running.finished && running.behaviour == "forage")
    ))),
    overlay!("routine_admission", Bool),
    overlay!("routine_weighted_place_available", Bool),
    overlay!("routine_available", Bool),
    overlay!("routine_deadline", Bool),
    overlay!("activity_deadline_reached", Bool),
    overlay!("routine_tend_fire_available", Bool),
    overlay!("routine_craft_available", Bool),
    overlay!("routine_repair_available", Bool),
    overlay!("routine_hearth_circle_available", Bool),
    overlay!("routine_explore_available", Bool),
    overlay!("routine_play_available", Bool),
    overlay!("routine_sit_by_fire_available", Bool),
    overlay!("invited", Bool),
    fact!("forage_available", Bool, |p, c, r, _n| Ok(i64::from(
        forage_candidate(p, c, r)?.is_some()
    ))),
    fact!("social_memory_salient", Bool, |p, _c, _r, _n| Ok(
        i64::from(p.memory_causes[4].is_some())
    )),
    overlay!("construction_available", Bool),
    fact!("memory_hunger", Signed, |p, _c, _r, n| milli(
        Some(p.memory_weights[0]),
        p.agent,
        n
    )),
    fact!("recallable_hunger", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.memory_causes[0].is_some()
    ))),
    fact!("memory_thirst", Signed, |p, _c, _r, n| milli(
        Some(p.memory_weights[1]),
        p.agent,
        n
    )),
    fact!("recallable_thirst", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.memory_causes[1].is_some()
    ))),
    fact!("memory_fatigue", Signed, |p, _c, _r, n| milli(
        Some(p.memory_weights[2]),
        p.agent,
        n
    )),
    fact!("recallable_fatigue", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.memory_causes[2].is_some()
    ))),
    fact!("memory_social", Signed, |p, _c, _r, n| milli(
        Some(p.memory_weights[4]),
        p.agent,
        n
    )),
    fact!("recallable_social", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.memory_causes[4].is_some()
    ))),
    fact!("memory_construction", Signed, |p, _c, _r, n| milli(
        Some(p.memory_weights[3]),
        p.agent,
        n
    )),
    fact!("recallable_construction", Bool, |p, _c, _r, _n| Ok(
        i64::from(p.memory_causes[3].is_some())
    )),
    fact!("memory_combat", Signed, |p, _c, _r, n| milli(
        Some(p.memory_weights[5]),
        p.agent,
        n
    )),
    fact!("recallable_combat", Bool, |p, _c, _r, _n| Ok(i64::from(
        p.memory_causes[5].is_some()
    ))),
];

/// Validated function registry; binding resolves names once, outside the tick loop.
pub struct ProviderRegistry {
    providers: BTreeMap<&'static str, FactProvider>,
}
impl ProviderRegistry {
    /// Create a registry, rejecting duplicate names before any policy is bound.
    pub fn new(providers: &[FactProvider]) -> Result<Self, FactError> {
        let mut registry = Self {
            providers: BTreeMap::new(),
        };
        for provider in providers {
            registry.register(*provider)?;
        }
        Ok(registry)
    }
    /// Add one named factual reader; existing registrations cannot be overwritten.
    pub fn register(&mut self, provider: FactProvider) -> Result<(), FactError> {
        if self.providers.contains_key(provider.name) {
            return Err(FactError::DuplicateProvider(provider.name.to_owned()));
        }
        self.providers.insert(provider.name, provider);
        Ok(())
    }
    /// Export the complete fact schema in stable name order for policy compilation.
    pub fn schema(&self) -> Vec<FactSpec> {
        self.providers
            .values()
            .map(|p| FactSpec {
                name: p.name.to_owned(),
                kind: p.kind,
            })
            .collect()
    }
    /// Resolve every compiled fact to its reader and validate unique node ownership for random facts.
    pub fn bind(&self, policy: &CompiledMind) -> Result<ProviderPlan, FactError> {
        let mut columns = Vec::with_capacity(policy.facts().len());
        for spec in policy.facts() {
            let provider = self
                .providers
                .get(spec.name.as_str())
                .ok_or_else(|| FactError::UnknownProvider(spec.name.clone()))?;
            if provider.kind != spec.kind {
                return Err(FactError::KindMismatch(spec.name.clone()));
            }
            let id = policy
                .fact_id(&spec.name)
                .ok_or_else(|| FactError::UnknownProvider(spec.name.clone()))?;
            let random_key = if provider.node_random {
                let users: Vec<_> = policy.fact_node_users(id).collect();
                if users.len() != 1 {
                    return Err(FactError::RandomFactNodeCount {
                        fact: spec.name.clone(),
                        users: users.len(),
                    });
                }
                Some(
                    policy
                        .node(users[0])
                        .ok_or_else(|| FactError::UnknownProvider(spec.name.clone()))?
                        .random_key,
                )
            } else {
                None
            };
            columns.push((id, *provider, random_key));
        }
        let preparations = columns
            .iter()
            .map(|(_, provider, _)| match provider.name {
                "forage_available" => FactPreparation::Forage,
                "gather_available" => FactPreparation::Gather,
                name if name == "routine_available"
                    || name == "routine_weighted_place_available"
                    || (name.starts_with("routine_") && name.ends_with("_available")) =>
                {
                    FactPreparation::Routine(RoutinePreparation::bind(name))
                }
                _ => FactPreparation::Direct,
            })
            .collect();
        let continuation = columns
            .iter()
            .map(|(_, provider, _)| continuation_supported(provider.name))
            .collect();
        Ok(ProviderPlan {
            continuation,
            preparations,
            columns,
            revision: policy.revision(),
        })
    }
}
#[derive(Clone, Copy)]
enum FactPreparation {
    Direct,
    Routine(RoutinePreparation),
    Forage,
    Gather,
}

/// Policy-bound column readers and unique node keys, resolved outside evaluation.
pub struct ProviderPlan {
    revision: sim_core::colony::mind::MindRevision,
    columns: Vec<(FactId, FactProvider, Option<u64>)>,
    preparations: Vec<FactPreparation>,
    continuation: Vec<bool>,
}
impl ProviderPlan {
    pub(super) fn continuation_read(
        &self,
        fact: FactId,
        person: &PersonSnapshot,
        context: &TickContext,
        resources: &SimResources,
    ) -> Result<sim_core::colony::mind::GuardRead, FactError> {
        let (id, provider, _) = self
            .columns
            .get(fact.index())
            .ok_or(MindEvalError::InvalidIndex("continuation fact"))?;
        if *id != fact {
            return Err(FactError::PolicyMismatch);
        }
        if !self.continuation[fact.index()] {
            return Ok(sim_core::colony::mind::GuardRead::Unsupported);
        }
        (provider.read)(person, context, resources, provider.name)
            .map(sim_core::colony::mind::GuardRead::Value)
    }
    /// Populate all sorted rows after their required overlays are prepared, rejecting revision or tick mismatch.
    pub fn build(
        &self,
        policy: &CompiledMind,
        context: &TickContext,
        resources: &SimResources,
    ) -> Result<PersonFacts, FactError> {
        if self.revision != policy.revision() {
            return Err(FactError::PolicyMismatch);
        }
        if context.tick != resources.current_tick {
            return Err(FactError::ContextTickMismatch);
        }
        let ids = context.rows().iter().map(|p| p.agent).collect();
        let mut table = PersonFacts::new(policy, ids, context.tick)?;
        for person in context.rows() {
            self.refresh_row(&mut table, context, resources, person.agent)?;
        }
        Ok(table)
    }
    /// Refresh immediately before selection after earlier effects/replay dirty facts.
    pub fn refresh_row(
        &self,
        table: &mut PersonFacts,
        context: &TickContext,
        resources: &SimResources,
        agent: AgentId,
    ) -> Result<(), FactError> {
        if context.tick != resources.current_tick || table.tick() != context.tick {
            return Err(FactError::ContextTickMismatch);
        }
        context.require_rich_row(agent)?;
        let person = context.row(agent)?;
        let index = table
            .row_index(agent)
            .ok_or(ContextError::UnknownAgent(agent))?;
        table.set_life_stage(index, person.life_stage)?;
        for (id, provider, random_key) in &self.columns {
            let value = match random_key {
                Some(key) => sim_core::rng::keyed_below(
                    resources.world_seed,
                    sim_core::rng::STREAM_ROUTINE ^ key,
                    agent,
                    context.tick,
                    0,
                    resources.noise_replicate_for(context.tick),
                    1000,
                ) as i64,
                None => (provider.read)(person, context, resources, provider.name)?,
            };
            table.set(*id, index, value)?;
        }
        Ok(())
    }
}

/// Per-evaluation memo: never reused across effects or routine replay.
pub(in crate::runtime) struct LazyRow<'a> {
    plan: &'a ProviderPlan,
    context: &'a mut TickContext,
    resources: &'a SimResources,
    routine: Option<&'a crate::runtime::agent_actions::routine::RoutineTick>,
    agent: AgentId,
    person_index: usize,
    stage: LifeStage,
    values: &'a mut Vec<Option<i64>>,
}
impl ProviderPlan {
    /// Bind a fresh lazy row after lifecycle overlays have been synchronized.
    pub(in crate::runtime) fn lazy_row<'a>(
        &'a self,
        context: &'a mut TickContext,
        resources: &'a SimResources,
        routine: Option<&'a crate::runtime::agent_actions::routine::RoutineTick>,
        agent: AgentId,
        values: &'a mut Vec<Option<i64>>,
    ) -> Result<LazyRow<'a>, FactError> {
        if context.tick != resources.current_tick {
            return Err(FactError::ContextTickMismatch);
        }
        context.require_rich_row(agent)?;
        let person_index = context.row_index(agent)?;
        let stage = context.row_at(person_index, agent)?.life_stage;
        values.resize(self.columns.len(), None);
        values.fill(None);
        Ok(LazyRow {
            plan: self,
            context,
            resources,
            routine,
            agent,
            person_index,
            stage,
            values,
        })
    }
}
impl sim_core::colony::mind::FactReader for LazyRow<'_> {
    const STABLE_READS: bool = true;
    type Error = FactError;
    fn agent(&self) -> AgentId {
        self.agent
    }
    fn tick(&self) -> u64 {
        self.context.tick
    }
    fn life_stage(&self) -> LifeStage {
        self.stage
    }
    fn revision(&self) -> sim_core::colony::mind::MindRevision {
        self.plan.revision
    }
    #[inline]
    fn get(&mut self, fact: FactId) -> Result<i64, FactError> {
        let slot = self
            .values
            .get(fact.index())
            .ok_or(MindEvalError::InvalidIndex("fact"))?;
        if let Some(value) = slot {
            return Ok(*value);
        }
        self.read_uncached(fact)
    }
}
impl LazyRow<'_> {
    #[inline(never)]
    fn read_uncached(&mut self, fact: FactId) -> Result<i64, FactError> {
        let (id, provider, random_key) = self
            .plan
            .columns
            .get(fact.index())
            .ok_or(MindEvalError::InvalidIndex("fact"))?;
        if *id != fact {
            return Err(FactError::PolicyMismatch);
        }
        let name = provider.name;
        let preparation = self.plan.preparations[fact.index()];
        if let FactPreparation::Routine(binding) = preparation {
            prepare_bound_routine_fact(self.context, self.routine, self.agent, name, binding)?;
        }
        let target_binding = match preparation {
            FactPreparation::Forage => Some("forage_bush"),
            FactPreparation::Gather => Some("gather_food"),
            FactPreparation::Direct | FactPreparation::Routine(_) => None,
        };
        let value = if let Some(binding) = target_binding {
            i64::from(resolve_target(self.context, self.resources, self.agent, binding)?.is_some())
        } else if let Some(key) = random_key {
            sim_core::rng::keyed_below(
                self.resources.world_seed,
                sim_core::rng::STREAM_ROUTINE ^ key,
                self.agent,
                self.context.tick,
                0,
                self.resources.noise_replicate_for(self.context.tick),
                1000,
            ) as i64
        } else {
            (provider.read)(
                self.context.row_at(self.person_index, self.agent)?,
                self.context,
                self.resources,
                name,
            )?
        };
        if provider.kind == FactKind::Bool && !matches!(value, 0 | 1) {
            return Err(MindEvalError::InvalidBoolean(fact).into());
        }
        self.values[fact.index()] = Some(value);
        Ok(value)
    }
}

fn edible_plant(p: &PersonSnapshot, c: &TickContext, r: &SimResources) -> Option<TargetCandidate> {
    r.rules.colony?;
    let mut index = c.harvest_food.borrow_mut();
    let (candidate, no_ripe) = index
        .get_or_insert_with(|| {
            std::sync::Arc::new(super::harvest_food::PlantCandidates::capture(r))
        })
        .nearest_observed(p, r);
    *c.plant_query_observation.borrow_mut() = Some((p.agent, no_ripe));
    candidate
}

fn inventory_total(p: &PersonSnapshot, r: &SimResources, name: &str) -> Result<u32, FactError> {
    if r.rules.colony.is_some() {
        // Preserve other shared Inventory resources, retire only its abstract Food.
        return Ok(p
            .inventory_total
            .unwrap_or(0)
            .saturating_sub(p.inventory_food.unwrap_or(0))
            .saturating_add(p.carry.map_or(0, |carry| u32::from(carry.count))));
    }
    p.inventory_total.ok_or_else(|| missing(p.agent, name))
}

fn physical_food_available(
    p: &PersonSnapshot,
    c: &TickContext,
    r: &SimResources,
    binding: &str,
) -> Result<bool, FactError> {
    if r.rules.colony.is_none() {
        return Ok(c.target_scope(p.agent)?.food_available);
    }
    if binding == "food" {
        return Ok(c.physical_food.borrow().available_to(
            r,
            (p.position.x, p.position.y),
            p.agent,
            p.carry,
        ));
    }
    let filter = super::food_targets::FoodFilter::binding(binding)
        .ok_or_else(|| FactError::UnknownProvider(binding.to_owned()))?;
    Ok(super::food_targets::candidate(&c.physical_food.borrow(), p, r, filter).is_some())
}

/// Resolve only the selected capability, after the data evaluator chooses.
/// Other registered providers can populate context targets without core changes.
pub fn resolve_target(
    context: &mut TickContext,
    resources: &SimResources,
    agent: AgentId,
    name: &str,
) -> Result<Option<TargetCandidate>, FactError> {
    if resources.rules.colony.is_some() {
        if let Some(filter) = super::food_targets::FoodFilter::binding(name) {
            return Ok(super::food_targets::candidate(
                &context.physical_food.borrow(),
                context.row(agent)?,
                resources,
                filter,
            ));
        }
    }
    if name == "edible_plant" {
        return Ok(edible_plant(context.row(agent)?, context, resources));
    }
    if name == "social_current_peer" {
        let person = context.row(agent)?;
        return Ok(match person.state.target() {
            Some(TargetKind::Agent(peer)) => context.row(peer).ok().map(|other| TargetCandidate {
                food: None,
                tile: (other.position.x, other.position.y),
                agent: Some(peer),
            }),
            _ => None,
        });
    }
    if let Ok(value) = context.target(agent, name) {
        return Ok(value);
    }
    let person = context.row(agent)?;
    let scope = context.target_scope(agent)?;
    let (tile, domains): (Option<(u32, u32)>, &[Domain]) = match name {
        "food" => (
            match scope.local_food {
                Some((home, radius)) => targets::nearest_local_food(
                    &person.position,
                    &resources.food_tiles,
                    home,
                    radius,
                ),
                None => targets::nearest_resource_tile(&person.position, &resources.food_tiles),
            },
            &[Domain::Person, Domain::FoodSources],
        ),
        "water" => (
            targets::nearest_resource_tile(&person.position, &resources.water_tiles),
            &[Domain::Person, Domain::WaterSources],
        ),
        "sleep" => (
            targets::nearest_resource_tile(&person.position, &resources.sleep_tiles),
            &[Domain::Person, Domain::SleepSources],
        ),
        "gather_food" => (
            context.gather_candidate(agent, resources)?,
            &[Domain::Person, Domain::FoodSources],
        ),
        "forage_bush" => (
            forage_candidate(person, context, resources)?,
            &[Domain::Person, Domain::FoodSources, Domain::Claims],
        ),
        "refuge" => (
            person.fleeing.map(|flight| flight.to),
            &[Domain::Person, Domain::Threats],
        ),
        "job" => (
            person.job.and_then(|job| match job.stage {
                JobStage::ToSource => Some(job.fetch.unwrap_or_else(|| job.kind.source())),
                JobStage::Working { .. } => Some(job.kind.source()),
                JobStage::ToDest => job.kind.dest(),
            }),
            &[Domain::Person, Domain::Jobs],
        ),
        "hearth" => (
            scope.home.filter(|home| {
                resources
                    .nav
                    .reachable((person.position.x, person.position.y), *home)
            }),
            &[Domain::Person],
        ),
        _ => return Ok(context.target(agent, name)?),
    };
    let target = tile.map(|tile| TargetCandidate {
        tile,
        agent: None,
        food: None,
    });
    context.set_target(agent, name, target, domains)?;
    Ok(target)
}
fn forage_candidate(
    p: &PersonSnapshot,
    c: &TickContext,
    r: &SimResources,
) -> Result<Option<(u32, u32)>, FactError> {
    if r.rules.colony.is_some() {
        return Ok(None);
    }
    let scope = c.target_scope(p.agent)?;
    Ok(r.rules
        .agent
        .forage
        .filter(|_| r.rules.society.stock_target.is_some())
        .zip(scope.home)
        .and_then(|(rule, home)| {
            let mut best = None;
            c.visit_food_in_square(r, home, rule.forage_radius, |tile, amount| {
                if !crate::runtime::agent_actions::forage::bush_eligible(
                    &rule,
                    home,
                    c.claims(),
                    None,
                    &scope.avoid_hearths,
                    tile,
                    amount,
                ) {
                    return;
                }
                let key = (
                    c.claims().get(&tile).copied().unwrap_or(0),
                    sim_core::spatial::chebyshev((p.position.x, p.position.y), tile),
                    tile.1,
                    tile.0,
                );
                if best.is_none_or(|old| key < old) {
                    best = Some(key);
                }
            });
            best.map(|(_, _, y, x)| (x, y))
        }))
}

/// Pure observations of the one real lifecycle transaction. Parent prepares its
/// places cache once before this call; shadow never constructs or mutates it.
pub(in crate::runtime) fn refresh_routine_lifecycle(
    context: &mut TickContext,
    routine: Option<&crate::runtime::agent_actions::routine::RoutineTick>,
    agent: AgentId,
) -> Result<(), FactError> {
    if routine.is_some_and(|r| !r.preview_ready()) {
        return Err(missing(agent, "routine_preview_prepared"));
    }
    context.set_activity_snapshot(agent, routine.and_then(|r| r.activity(agent)))?;
    let p = context.row(agent)?;
    let member = context.target_scope(agent)?.settlement;
    let deadline = routine
        .and_then(|r| {
            r.deadline(
                agent,
                member,
                matches!(p.state, AgentState::Seeking { .. })
                    && p.activity
                        .is_some_and(|activity| (p.position.x, p.position.y) != activity.place),
                p.lineage,
                p.temperament.as_ref(),
            )
        })
        .is_some();
    let admission =
        routine.is_some_and(|r| r.can_enter(agent, p.lineage, member, p.temperament.as_ref()));
    let invited = routine.is_some_and(|r| r.invitation_pending(agent));
    context.set_routine_lifecycle(agent, admission, invited, deadline)?;
    Ok(())
}

#[derive(Clone, Copy)]
struct RoutineBinding {
    target: &'static str,
    fact: &'static str,
    kind: ActivityKind,
}

static ROUTINE_BINDINGS: [RoutineBinding; 7] = [
    RoutineBinding {
        target: "routine_tend_fire",
        fact: "routine_tend_fire_available",
        kind: ActivityKind::TendFire,
    },
    RoutineBinding {
        target: "routine_craft",
        fact: "routine_craft_available",
        kind: ActivityKind::Craft,
    },
    RoutineBinding {
        target: "routine_repair",
        fact: "routine_repair_available",
        kind: ActivityKind::Repair,
    },
    RoutineBinding {
        target: "routine_hearth_circle",
        fact: "routine_hearth_circle_available",
        kind: ActivityKind::HearthCircle,
    },
    RoutineBinding {
        target: "routine_explore",
        fact: "routine_explore_available",
        kind: ActivityKind::Explore,
    },
    RoutineBinding {
        target: "routine_play",
        fact: "routine_play_available",
        kind: ActivityKind::Play,
    },
    RoutineBinding {
        target: "routine_sit_by_fire",
        fact: "routine_sit_by_fire_available",
        kind: ActivityKind::SitByFire,
    },
];

#[derive(Clone, Copy)]
enum RoutinePreparation {
    All,
    Weighted,
    Kind(&'static RoutineBinding),
    Unrecognized,
}

impl RoutinePreparation {
    fn bind(name: &str) -> Self {
        match name {
            "routine_available" => Self::All,
            "routine_weighted_place_available" => Self::Weighted,
            _ => ROUTINE_BINDINGS
                .iter()
                .find(|binding| binding.fact == name)
                .map_or(Self::Unrecognized, Self::Kind),
        }
    }
}

pub(in crate::runtime) fn prepare_routine_fact(
    context: &mut TickContext,
    routine: Option<&crate::runtime::agent_actions::routine::RoutineTick>,
    agent: AgentId,
    name: &str,
) -> Result<(), FactError> {
    prepare_bound_routine_fact(
        context,
        routine,
        agent,
        name,
        RoutinePreparation::bind(name),
    )
}

fn prepare_bound_routine_fact(
    context: &mut TickContext,
    routine: Option<&crate::runtime::agent_actions::routine::RoutineTick>,
    agent: AgentId,
    name: &str,
    preparation: RoutinePreparation,
) -> Result<(), FactError> {
    let p = context.row(agent)?;
    let member = context.target_scope(agent)?.settlement;
    let domains = &[
        Domain::Person,
        Domain::RoutineOccupancy,
        Domain::Invitations,
    ];
    if matches!(preparation, RoutinePreparation::Weighted) {
        let value = routine.is_some_and(|r| {
            r.weighted_place_available(
                agent,
                p.lineage,
                member,
                p.life_stage == LifeStage::Child,
                p.life_stage == LifeStage::Elder,
                p.temperament.as_ref(),
            )
        });
        return Ok(context.set_derived(agent, name, i64::from(value), domains)?);
    }
    let bindings: &[RoutineBinding] = match preparation {
        RoutinePreparation::All => &ROUTINE_BINDINGS,
        RoutinePreparation::Kind(binding) => std::slice::from_ref(binding),
        RoutinePreparation::Weighted | RoutinePreparation::Unrecognized => &[],
    };
    let mut available = false;
    for binding in bindings {
        let p = context.row(agent)?;
        let tile = routine.and_then(|r| {
            r.preview_kind(
                agent,
                p.lineage,
                member,
                p.temperament.as_ref(),
                binding.kind,
            )
        });
        available |= tile.is_some();
        context.set_derived(agent, binding.fact, i64::from(tile.is_some()), domains)?;
        context.set_target(
            agent,
            binding.target,
            tile.map(|tile| TargetCandidate {
                tile,
                agent: None,
                food: None,
            }),
            domains,
        )?;
    }
    if matches!(preparation, RoutinePreparation::All) {
        context.set_derived(agent, name, i64::from(available), domains)?;
    }
    Ok(())
}

#[cfg(any(test, feature = "mind_shadow"))]
pub(in crate::runtime) fn refresh_routine(
    context: &mut TickContext,
    routine: Option<&crate::runtime::agent_actions::routine::RoutineTick>,
    agent: AgentId,
) -> Result<(), FactError> {
    refresh_routine_lifecycle(context, routine, agent)?;
    prepare_routine_fact(context, routine, agent, "routine_available")?;
    prepare_routine_fact(context, routine, agent, "routine_weighted_place_available")
}

// Captured rows are unique and ascending: first peer other than self needs only two IDs.
#[derive(Clone, Copy, Default)]
struct SocialPeers([Option<AgentId>; 2]);
impl SocialPeers {
    fn push(&mut self, agent: AgentId) {
        if self.0[0].is_none() {
            self.0[0] = Some(agent);
        } else if self.0[1].is_none() {
            self.0[1] = Some(agent);
        }
    }
    fn other(self, agent: AgentId) -> Option<AgentId> {
        self.0.into_iter().flatten().find(|id| *id != agent)
    }
}

/// Freeze social compatibility and mutual-seeking observations at the same
/// pre-effect boundary as the old handshake. Later state changes must not
/// recompute these maps: both sides of a mutual pair must see the same start.
pub fn prepare_social_snapshot(
    context: &mut TickContext,
    world: &hecs::World,
    resources: &SimResources,
) -> Result<(), FactError> {
    use sim_core::components::ConstructionSite;
    use std::collections::BTreeSet;
    let sites: BTreeSet<_> = world
        .query::<&ConstructionSite>()
        .iter()
        .filter(|(entity, site)| {
            !site.is_complete()
                && crate::runtime::knowledge::open_to_any_builder(world, resources, *entity)
        })
        .map(|(_, site)| (site.position.x, site.position.y))
        .collect();
    let mut peers: BTreeMap<(u32, u32), [SocialPeers; 3]> = BTreeMap::new();
    let routine = resources.rules.agent.routine.is_some();
    for person in context.rows() {
        let available = person.life_stage != LifeStage::Child
            && crate::runtime::agent_actions::routine::idle_equivalent(person.state, routine)
            && (resources.rules.agent.social_seek.is_none()
                || resources
                    .social_retry_after
                    .get(&person.agent)
                    .is_none_or(|until| *until <= resources.current_tick));
        if !available {
            continue;
        }
        let tile = (person.position.x, person.position.y);
        let threshold = fatigue_threshold(person, resources);
        let lonely = person.social.is_some_and(|v| v > needs::SOCIAL_THRESHOLD);
        let hungry = person
            .hunger
            .is_some_and(|v| v > f64::from(needs::HUNGER_THRESHOLD));
        let thirsty = person.thirst.is_some_and(|v| v > needs::THIRST_THRESHOLD);
        let tired = person.fatigue.is_some_and(|v| v > threshold);
        if lonely && !hungry && !thirsty && !tired && !sites.contains(&tile) {
            peers.entry(tile).or_default()[0].push(person.agent);
        }
        let day = (resources.ticks_per_day != 0).then(|| {
            resources
                .current_tick
                .saturating_add(resources.rules.clock_offset_ticks)
                % resources.ticks_per_day
        });
        let night = needs::person_night_rest(
            resources.rules.agent.night_rest,
            resources.rules.agent.temperament,
            person.temperament.as_ref(),
            day,
        )
        .is_some();
        if !night || !tired {
            peers.entry(tile).or_default()[2].push(person.agent);
            if lonely {
                peers.entry(tile).or_default()[1].push(person.agent);
            }
        }
    }
    let rows: Vec<_> = context
        .rows()
        .iter()
        .map(|person| {
            let tile = (person.position.x, person.position.y);
            let buckets = peers.get(&tile);
            let peer = |kind: usize| buckets.and_then(|ids| ids[kind].other(person.agent));
            let natural_peer = peer(0);
            let memory_peer = if person.social.is_some_and(|v| v > needs::SOCIAL_THRESHOLD) {
                peer(1)
            } else {
                None
            }
            .or_else(|| person.memory_causes[4].and_then(|_| peer(2)));
            let handshake = match person.state {
                AgentState::Seeking {
                    target: TargetKind::Agent(partner),
                } => context.row(partner).is_ok_and(|other| {
                    other.position == person.position
                        && other.state
                            == AgentState::Seeking {
                                target: TargetKind::Agent(person.agent),
                            }
                }),
                AgentState::Idle | AgentState::Consuming { .. } | AgentState::Seeking { .. } => {
                    false
                }
            };
            (person.agent, tile, natural_peer, memory_peer, handshake)
        })
        .collect();
    context.install_social_snapshot(rows)?;
    Ok(())
}

/// Full read-only preparation, excluding the real owner's routine transaction.
pub fn prepare_context(
    world: &hecs::World,
    resources: &SimResources,
) -> Result<TickContext, FactError> {
    let mut context = TickContext::capture(world, resources)?;
    prepare_social_snapshot(&mut context, world, resources)?;
    Ok(context)
}

// Only cheap, read-only observations; unlisted/custom preparations force full evaluation.
pub(super) fn continuation_supported(name: &str) -> bool {
    matches!(
        name,
        "stock_hoard_order"
            | "social_partner_present"
            | "social_chase_expired"
            | "idle"
            | "seeking"
            | "consuming"
            | "drafted"
            | "fleeing"
            | "target_food"
            | "target_water"
            | "target_sleep"
            | "target_gather"
            | "target_social"
            | "activity_active"
            | "activity_kind"
            | "activity_arrived"
            | "activity_resting"
            | "activity_deadline_reached"
            | "running_foraging"
            | "has_job"
            | "job_owned_state"
            | "job_working"
            | "job_to_source"
            | "job_to_dest"
            | "job_distance"
            | "job_destination_distance"
            | "craft_working"
            | "job_harvesting"
            | "worker"
            | "hunger"
            | "thirst"
            | "fatigue"
            | "hunger_tenths"
            | "thirst_tenths"
            | "fatigue_tenths"
            | "hunger_threshold"
            | "thirst_threshold"
            | "fatigue_threshold"
            | "hunger_threshold_tenths"
            | "thirst_threshold_tenths"
            | "fatigue_threshold_tenths"
            | "critical_threshold"
            | "critical_threshold_tenths"
            | "hunger_excess_tenths"
            | "thirst_excess_tenths"
            | "fatigue_excess_tenths"
            | "hunger_actionable"
            | "thirst_actionable"
            | "fatigue_actionable"
            | "hungry"
            | "thirsty"
            | "tired"
            | "critical_preemption_enabled"
            | "preempt_gathering_enabled"
            | "hunger_critical_order"
            | "thirst_critical_order"
            | "fatigue_critical_order"
            | "food_available"
            | "water_available"
            | "sleep_available"
            | "camp_stocked"
            | "within_camp_radius"
            | "stocked_camp_in_radius"
            | "social_retry_ready"
            | "social_seek_enabled"
            | "food_carry_loaded"
            | "carry_count"
            | "carrying"
            | "inventory_total"
            | "inventory_food"
            | "inventory_room"
            | "night_window"
    )
}

/// Prepare the original lightweight snapshot for independent host adapters.
pub fn prepare_continuation_context(
    world: &hecs::World,
    resources: &SimResources,
) -> Result<TickContext, FactError> {
    let mut context = TickContext::capture_continuation(world, resources)?;
    prepare_social_snapshot(&mut context, world, resources)?;
    Ok(context)
}

pub(super) fn prepare_continuation_context_with_claims(
    world: &hecs::World,
    resources: &SimResources,
    claims: BTreeMap<(u32, u32), u32>,
    initial_stock: Option<&BTreeMap<SettlementId, ForagePlan>>,
) -> Result<TickContext, FactError> {
    let mut context =
        TickContext::capture_continuation_with_claims(world, resources, claims, initial_stock)?;
    prepare_social_snapshot(&mut context, world, resources)?;
    // Match the original capture -> social snapshot -> replace_claims boundary.
    context.invalidate(Domain::Claims)?;
    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use sim_core::components::{Agent, Hunger, Position};

    #[test]
    fn lazy_runtime_matches_eager_witnesses_and_skips_unrequested_missing_need() {
        use sim_core::colony::mind::{choose, choose_with_reader, EvalKey, EvalScratch};
        use sim_core::components::Sleep;
        let program = crate::runtime::mind::MindProgram::load(r#"(
            schema_version: 1, bands: ["critical", "duty", "need", "work", "leisure", "idle"],
            nodes: [
                (id: "need", band: "need", behaviour: "rest", when: [(fact: "idle", op: "has")],
                 score: (constant: 1, terms: [(fact: "hunger", weight_permille: 1000)]),
                 hysteresis: 0, life_stages: ["adult", "child", "elder"], tags: [], name_key: "NEED"),
                (id: "idle", band: "idle", behaviour: "wander", when: [(fact: "fatigue", op: ">=", value: Some(0))],
                 score: (constant: 0, terms: []), hysteresis: 0,
                 life_stages: ["adult", "child", "elder"], tags: [], name_key: "IDLE")
            ])"#).expect("policy");
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Agent { id: 7 },
            Position::new(1, 1),
            AgentState::Idle,
            Hunger::new(60.0, 0.0),
            Sleep::new(10.0, 0.0),
        ));
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut context = prepare_context(&world, &resources).expect("context");
        let table = program
            .providers
            .build(&program.policy, &context, &resources)
            .expect("eager facts");
        let key = EvalKey {
            world_seed: 0,
            agent: 7,
            tick: resources.current_tick,
            replicate: 0,
        };
        let mut eager = EvalScratch::default();
        let expected = choose(
            &program.policy,
            table.row(7).expect("row"),
            None,
            key,
            &mut eager,
        )
        .expect("eager");
        let mut lazy = EvalScratch::default();
        let mut memo = Vec::new();
        let mut row = program
            .providers
            .lazy_row(&mut context, &resources, None, 7, &mut memo)
            .expect("lazy row");
        let actual =
            choose_with_reader(&program.policy, &mut row, None, key, &mut lazy).expect("lazy");
        assert_eq!(actual, expected);
        assert_eq!(lazy.witnesses(), eager.witnesses());
        assert!(row.values[program.policy.fact_id("fatigue").expect("fact").index()].is_none());
        world
            .remove_one::<Sleep>(entity)
            .expect("remove unused need");
        let mut context = prepare_context(&world, &resources).expect("context");
        let mut row = program
            .providers
            .lazy_row(&mut context, &resources, None, 7, &mut memo)
            .expect("row");
        assert_eq!(
            choose_with_reader(&program.policy, &mut row, None, key, &mut lazy)
                .expect("unused missing fact is not read"),
            expected
        );
        assert!(sim_core::colony::mind::FactReader::get(
            &mut row,
            program.policy.fact_id("fatigue").expect("fact")
        )
        .is_err());
    }

    #[test]
    fn fixed_point_units_preserve_seam_truncation_and_reject_missing_values() {
        assert_eq!(tenths(Some(64.999), 7, "hunger").expect("tenths"), 649);
        assert_eq!(tenths(Some(65.0), 7, "hunger").expect("tenths"), 650);
        assert_eq!(
            milli(Some(64.999), 7, "hunger").expect("millipoints"),
            64999
        );
        assert!(matches!(
            milli(None, 7, "hunger"),
            Err(FactError::MissingValue { .. })
        ));
        assert!(matches!(
            milli(Some(f64::NAN), 7, "hunger"),
            Err(FactError::InvalidNumber { .. })
        ));
    }

    #[test]
    fn factual_hunger_is_independent_of_other_needs_and_resource_target_resolution() {
        let mut world = hecs::World::new();
        let entity = world.spawn((
            Agent { id: 7 },
            Position::new(1, 1),
            AgentState::Idle,
            Hunger::new(50.0001, 0.0),
        ));
        let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        resources.rules.agent.need_fallthrough = true;
        resources.food_tiles.insert((2, 1), 1);
        let mut context = prepare_context(&world, &resources).expect("facts");
        let provider = BUILTIN_PROVIDERS
            .iter()
            .find(|p| p.name == "hunger_actionable")
            .expect("provider");
        assert_eq!(
            (provider.read)(
                context.row(7).expect("row"),
                &context,
                &resources,
                provider.name
            )
            .expect("read"),
            1
        );
        assert_eq!(
            resolve_target(&mut context, &resources, 7, "food")
                .expect("target")
                .expect("food")
                .tile,
            (2, 1)
        );
        resources.food_tiles.clear();
        context.invalidate(Domain::FoodSources).expect("invalidate");
        assert!(resolve_target(&mut context, &resources, 7, "food")
            .expect("refresh")
            .is_none());
        assert_eq!(
            *world.get::<&AgentState>(entity).expect("state"),
            AgentState::Idle
        );
        // Eligibility is the start-of-tick snapshot; target resolution reads live sources.
        assert_eq!(
            (provider.read)(
                context.row(7).expect("row"),
                &context,
                &resources,
                provider.name
            )
            .expect("read"),
            1
        );
    }
    fn selected_node(stage: LifeStage, values: &[(&str, i64)]) -> String {
        selected_node_from(
            include_str!("../../../../sim-core/data/colony/mind.ron"),
            stage,
            values,
        )
    }

    fn selected_node_from(source: &str, stage: LifeStage, values: &[(&str, i64)]) -> String {
        use sim_core::colony::mind::{choose, EvalKey, EvalScratch};
        let program = crate::runtime::mind::MindProgram::load(source).expect("policy");
        let policy = &program.policy;
        let mut table = PersonFacts::new(policy, vec![7], 0).expect("table");
        table.set_life_stage(0, stage).expect("stage");
        for spec in policy.facts() {
            table
                .set(policy.fact_id(&spec.name).expect("id"), 0, 0)
                .expect("initial value");
        }
        for (name, value) in values {
            table
                .set(policy.fact_id(name).expect("referenced fact"), 0, *value)
                .expect("fixture value");
        }
        let selected = choose(
            policy,
            table.row(7).expect("row"),
            None,
            EvalKey {
                world_seed: 0,
                agent: 7,
                tick: 0,
                replicate: 0,
            },
            &mut EvalScratch::default(),
        )
        .expect("selection");
        policy
            .node(selected.selected.node)
            .expect("node")
            .name
            .clone()
    }

    #[test]
    fn social_continuation_keeps_current_peer_instead_of_new_candidate() {
        use sim_core::components::Social;
        let mut world = hecs::World::new();
        world.spawn((
            Agent { id: 1 },
            Position::new(1, 1),
            AgentState::Seeking {
                target: TargetKind::Agent(2),
            },
        ));
        world.spawn((
            Agent { id: 2 },
            Position::new(3, 1),
            AgentState::Seeking {
                target: TargetKind::Agent(1),
            },
        ));
        world.spawn((
            Agent { id: 3 },
            Position::new(1, 1),
            AgentState::Idle,
            Social::new(90.0, 0.0),
        ));
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut context = prepare_context(&world, &resources).expect("context");
        assert_eq!(
            context
                .target(1, "social_peer")
                .expect("candidate")
                .expect("fresh peer")
                .agent,
            Some(3)
        );
        let continued = resolve_target(&mut context, &resources, 1, "social_current_peer")
            .expect("current target")
            .expect("current peer");
        assert_eq!(continued.agent, Some(2));
        assert_eq!(continued.tile, (3, 1));
    }

    #[test]
    fn social_entry_rejects_child_and_retry_cooldown_even_with_a_peer() {
        for (stage, ready, expected) in [
            (LifeStage::Adult, 1, true),
            (LifeStage::Elder, 1, true),
            (LifeStage::Child, 1, false),
            (LifeStage::Adult, 0, false),
        ] {
            let selected = selected_node(
                stage,
                &[
                    ("idle", 1),
                    ("lonely", 1),
                    ("social_candidate", 1),
                    ("social_retry_ready", ready),
                ],
            );
            assert_eq!(
                selected == "need_idle_social",
                expected,
                "{stage:?}, retry={ready}: {selected}"
            );
        }
    }

    #[test]
    fn shipped_colony_policy_has_no_abstract_food_production_nodes() {
        let program = crate::runtime::mind::MindProgram::load(include_str!(
            "../../../../sim-core/data/colony/mind.ron"
        ))
        .expect("shipped policy");
        for name in ["gather", "forage"] {
            let behaviour = program.policy.behaviour_id(name).expect("legacy binding");
            assert!(program
                .policy
                .nodes()
                .iter()
                .all(|node| node.behaviour != behaviour));
        }
    }

    #[test]
    fn legacy_gather_stops_at_hoard_cap_but_unbounded_nonmember_can_gather() {
        // Baseline/legacy providers retain their contract; colony food production
        // now uses physical harvest jobs instead of these abstract work nodes.
        const LEGACY_GATHER: &str = r#"(schema_version:1,bands:["critical","duty","need","work","leisure","idle"],nodes:[
          (id:"gather_idle",band:"work",behaviour:"gather",target:Some("gather_food"),when:[(fact:"idle",op:"has"),(fact:"worker",op:"has"),(fact:"inventory_room",op:">",value:Some(0)),(fact:"gather_available",op:"has"),(fact:"stock_hoard_order",op:"<",value:Some(0))],score:(constant:0,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"GATHER"),
          (id:"idle",band:"idle",behaviour:"wander",when:[(fact:"idle",op:"has")],score:(constant:0,terms:[]),hysteresis:0,life_stages:["adult","child","elder"],tags:[],name_key:"IDLE")])"#;
        let mut world = hecs::World::new();
        world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let mut context = prepare_context(&world, &resources).expect("context");
        let reader = BUILTIN_PROVIDERS
            .iter()
            .find(|p| p.name == "stock_hoard_order")
            .expect("reader");
        for (amount, limit, expected) in [
            (9, Some(10.0), -1),
            (10, Some(10.0), 0),
            (11, Some(10.0), 1),
            (11, None, -1),
        ] {
            let mut scope = context.target_scope(7).expect("scope").clone();
            scope.stock_in_hand = amount;
            scope.stock_hoard_limit = limit;
            context.set_target_scope(7, scope).expect("scope update");
            let order = (reader.read)(
                context.row(7).expect("person"),
                &context,
                &resources,
                reader.name,
            )
            .expect("order");
            assert_eq!(order, expected);
            let selected = selected_node_from(
                LEGACY_GATHER,
                LifeStage::Adult,
                &[
                    ("idle", 1),
                    ("worker", 1),
                    ("inventory_room", 10),
                    ("gather_available", 1),
                    ("stock_hoard_order", order),
                ],
            );
            assert_eq!(
                selected == "gather_idle",
                order < 0,
                "stock {amount}, limit {limit:?}: {selected}"
            );
        }
    }

    #[test]
    fn exact_meal_water_boundary_and_social_preemption_are_data_driven() {
        let mut world = hecs::World::new();
        world.spawn((Agent { id: 7 }, Position::new(1, 1), AgentState::Idle));
        let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        resources.rules.agent.critical_preemption = Some(75.0);
        let context = prepare_context(&world, &resources).expect("context");
        for (value, expected) in [
            (75.0, "meal_idle_water_not_critical"),
            (75.0001, "camp_critical_water_idle"),
        ] {
            let order = critical_order(
                Some(value),
                context.row(7).expect("person"),
                &resources,
                "thirst",
            )
            .expect("raw order");
            let selected = selected_node(
                LifeStage::Adult,
                &[
                    ("idle", 1),
                    ("hungry", 1),
                    ("hunger_actionable", 1),
                    ("thirst_actionable", 1),
                    ("stocked_camp_in_radius", 1),
                    ("water_available", 1),
                    ("thirst_critical_order", order),
                    ("thirst_tenths", 750),
                ],
            );
            if value == 75.0 {
                assert_eq!(selected, expected);
            } else {
                assert!(selected.starts_with("camp_critical_water"), "{selected}");
            }
        }
        for drive in ["hunger", "thirst", "fatigue"] {
            let fact = format!("{drive}_actionable");
            let selected = selected_node(
                LifeStage::Adult,
                &[
                    ("seeking", 1),
                    ("target_social", 1),
                    ("social_seek_enabled", 1),
                    ("social_partner_present", 1),
                    (&fact, 1),
                ],
            );
            assert_eq!(selected, format!("finish_social_need_{drive}"));
        }
    }
    #[test]
    fn idle_with_retained_job_can_choose_needs_and_physical_food() {
        use sim_core::colony::{ItemKind, Job};
        let mut world = hecs::World::new();
        world.spawn((
            Agent { id: 7 },
            Position::new(1, 1),
            AgentState::Idle,
            Job {
                kind: JobKind::Chop { tree: (3, 3) },
                stage: JobStage::ToSource,
                taken_at: 0,
                amount: 0,
                item: ItemKind::Log,
                material: None,
                fetch: None,
                best_distance: 2,
                progress_tick: 0,
            },
        ));
        let resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
        let context = prepare_context(&world, &resources).expect("actual retained job");
        let mut ownership = Vec::new();
        for name in ["idle", "has_job", "job_owned_state"] {
            let provider = BUILTIN_PROVIDERS
                .iter()
                .find(|p| p.name == name)
                .expect("provider");
            let value =
                (provider.read)(context.row(7).expect("person"), &context, &resources, name)
                    .expect("actual ownership fact");
            ownership.push((name, value));
        }
        assert_eq!(
            ownership,
            [("idle", 1), ("has_job", 1), ("job_owned_state", 0)]
        );
        let cases: &[(&str, &[(&str, i64)])] = &[
            (
                "need_idle_hunger",
                &[("hunger_actionable", 1), ("hunger_tenths", 600)],
            ),
            (
                "need_idle_thirst",
                &[("thirst_actionable", 1), ("thirst_tenths", 600)],
            ),
            (
                "critical_thirst_idle_hunger",
                &[
                    ("hunger_actionable", 1),
                    ("hunger_tenths", 600),
                    ("thirst_tenths", 770),
                ],
            ),
            (
                "meal_idle_no_water",
                &[("hungry", 1), ("stocked_camp_in_radius", 1)],
            ),
            (
                "camp_critical_water_idle",
                &[
                    ("hungry", 1),
                    ("stocked_camp_in_radius", 1),
                    ("water_available", 1),
                    ("thirst_critical_order", 1),
                ],
            ),
            (
                "need_idle_hunger_cooked_novel",
                &[
                    ("hunger_actionable", 1),
                    ("hunger_tenths", 600),
                    ("food_cooked_novel_available", 1),
                    ("food_cooked_available", 1),
                    ("food_novel_available", 1),
                ],
            ),
            (
                "need_idle_hunger_cooked",
                &[
                    ("hunger_actionable", 1),
                    ("hunger_tenths", 600),
                    ("food_cooked_available", 1),
                    ("food_novel_available", 1),
                ],
            ),
            (
                "need_idle_hunger_novel",
                &[
                    ("hunger_actionable", 1),
                    ("hunger_tenths", 600),
                    ("food_novel_available", 1),
                ],
            ),
        ];
        for (expected, extra) in cases {
            let mut values = ownership.clone();
            values.extend_from_slice(extra);
            assert_eq!(selected_node(LifeStage::Adult, &values), *expected);
        }
    }
    #[test]
    fn ordinary_social_need_exit_keeps_need_band_and_owner_priorities() {
        use sim_core::colony::mind::FinishMode;
        let program = crate::runtime::mind::MindProgram::load(include_str!(
            "../../../../sim-core/data/colony/mind.ron"
        ))
        .expect("shipped policy");
        for drive in ["hunger", "thirst", "fatigue"] {
            let actionable = format!("{drive}_actionable");
            let expected = format!("finish_social_need_{drive}");
            let mut facts = vec![
                ("seeking", 1),
                ("target_social", 1),
                ("social_seek_enabled", 1),
                ("social_partner_present", 1),
                ("has_job", 1),
                ("job_owned_state", 0),
                ("hunger_critical_order", -1),
                ("thirst_critical_order", -1),
                ("fatigue_critical_order", -1),
                (actionable.as_str(), 1),
            ];
            assert_eq!(selected_node(LifeStage::Adult, &facts), expected);
            let node = program
                .policy
                .node(program.policy.node_id(&expected).expect("id"))
                .expect("node");
            assert_eq!(node.finish, Some(FinishMode::Interrupted));
            assert_eq!(program.policy.band_name(node.band), Some("need"), "{drive}");
            facts.push(("drafted", 1));
            assert_eq!(selected_node(LifeStage::Adult, &facts), "draft_owned");
            facts.push(("fleeing", 1));
            assert_eq!(selected_node(LifeStage::Adult, &facts), "flee_owned");
        }
        // A retained job while socialising is not job-owned execution. Actual job
        // execution still holds below the seam and admits critical need at 75.
        for (hunger, expected) in [
            (749, "job_finish_craft"),
            (750, "critical_hunger_job_hunger"),
        ] {
            assert_eq!(
                selected_node(
                    LifeStage::Adult,
                    &[
                        ("job_owned_state", 1),
                        ("has_job", 1),
                        ("craft_working", 1),
                        ("hunger_actionable", 1),
                        ("hunger_tenths", hunger),
                    ]
                ),
                expected
            );
        }
    }
    #[test]
    fn drafted_policy_acknowledges_owner_without_an_unresolved_target() {
        let program = crate::runtime::mind::MindProgram::load(include_str!(
            "../../../../sim-core/data/colony/mind.ron"
        ))
        .expect("shipped policy");
        let node = program
            .policy
            .node(program.policy.node_id("draft_owned").expect("draft node"))
            .expect("node");
        assert!(node.target.is_none());
        assert_eq!(
            selected_node(LifeStage::Adult, &[("drafted", 1)]),
            "draft_owned"
        );
    }
    #[test]
    fn routine_cooldown_keeps_the_legacy_wander_action() {
        let program = crate::runtime::mind::MindProgram::load(include_str!(
            "../../../../sim-core/data/colony/mind.ron"
        ))
        .expect("shipped policy");
        // The cooldown fact may cease to be referenced when the redundant node is removed.
        let mut values = vec![("idle", 1)];
        if program.policy.fact_id("activity_resting").is_some() {
            values.push(("activity_resting", 1));
        }
        let selected = selected_node(LifeStage::Adult, &values);
        let node = program
            .policy
            .node(program.policy.node_id(&selected).expect("node id"))
            .expect("selected node");
        let behaviour = program.policy.behaviour(node.behaviour).expect("behaviour");
        assert_eq!(
            behaviour.name, "wander",
            "routine cooldown blocks routine admission, not idle wandering"
        );
        assert!(node.target.is_none());
    }
    #[test]
    fn construction_job_ownership_matches_legacy_hold_independent_of_occupancy() {
        use sim_core::colony::{ItemKind, Job, JobKind, JobStage, Occupant};
        let provider = BUILTIN_PROVIDERS
            .iter()
            .find(|p| p.name == "job_owned_state")
            .expect("provider");
        for has_job in [false, true] {
            for occupied in [false, true] {
                for state in [
                    AgentState::Idle,
                    AgentState::Seeking {
                        target: TargetKind::ConstructionSite,
                    },
                    AgentState::Consuming {
                        target: TargetKind::ConstructionSite,
                    },
                    AgentState::Seeking {
                        target: TargetKind::Food,
                    },
                ] {
                    let mut world = hecs::World::new();
                    let entity = world.spawn((Agent { id: 7 }, Position::new(1, 1), state));
                    if has_job {
                        world
                            .insert_one(
                                entity,
                                Job {
                                    kind: JobKind::Chop { tree: (3, 3) },
                                    stage: JobStage::ToSource,
                                    taken_at: 0,
                                    amount: 0,
                                    item: ItemKind::Log,
                                    material: None,
                                    fetch: None,
                                    best_distance: 0,
                                    progress_tick: 0,
                                },
                            )
                            .expect("job");
                    }
                    let mut resources = SimResources::new(8, 8, sim_core::MaterialRegistry::new());
                    if occupied {
                        resources
                            .colony
                            .occupied
                            .insert((1, 1), Occupant::Site((1, 1)));
                    }
                    let holders = crate::runtime::jobs::holders(&world, &resources);
                    let expected = crate::runtime::jobs::hold_for_decision(entity, state, &holders);
                    let context = prepare_context(&world, &resources).expect("facts");
                    let actual = (provider.read)(
                        context.row(7).expect("row"),
                        &context,
                        &resources,
                        provider.name,
                    )
                    .expect("fact");
                    assert_eq!(
                        actual != 0,
                        expected,
                        "state={state:?}, job={has_job}, occupied={occupied}"
                    );
                }
            }
        }
    }
}

#[cfg(test)]
#[path = "social_bucket_tests.rs"]
mod social_bucket_tests;

#[cfg(test)]
#[path = "physical_food_fact_tests.rs"]
mod physical_food_fact_tests;
