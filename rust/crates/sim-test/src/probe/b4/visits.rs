//! Immediate command receipts and explicitly tick-boundary wave observations.
use super::daily::population;
use serde::Serialize;
use sim_core::colony::{
    neighbours::{Goods, VisitState, Wave, WaveAnswer},
    Command, CommandResult,
};
use sim_engine::SimEngine;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WaveState {
    pub state: &'static str,
    pub waiting_since: Option<u64>,
    pub answer: Option<&'static str>,
    pub joined_ids: Vec<u64>,
}
impl From<&Wave> for WaveState {
    fn from(w: &Wave) -> Self {
        let (state, waiting_since) = match w.state {
            VisitState::Arriving => ("arriving", None),
            VisitState::Waiting { since } => ("waiting", Some(since)),
            VisitState::Leaving => ("leaving", None),
            VisitState::Gone => ("gone", None),
        };
        Self {
            state,
            waiting_since,
            answer: w.answer.map(|a| match a {
                WaveAnswer::Accepted => "accepted",
                WaveAnswer::Declined => "declined",
                WaveAnswer::Unanswered => "unanswered",
            }),
            joined_ids: w.joined.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ResultRow {
    pub accepted: bool,
    pub count: u32,
    pub reason: Option<&'static str>,
}
impl From<&CommandResult> for ResultRow {
    fn from(r: &CommandResult) -> Self {
        Self {
            accepted: r.accepted,
            count: r.count,
            reason: r.reason.map(|r| r.name()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct CommandReceipt {
    pub ordinal: u64,
    pub upcoming_tick: u64,
    pub command: Command,
    pub result: ResultRow,
    pub population_before: u64,
    pub population_after: u64,
    pub wave_id: Option<u32>,
    pub wave_size: Option<u64>,
    pub wave_before: Option<WaveState>,
    pub wave_after: Option<WaveState>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GoodsRow {
    pub item_code: u32,
    pub count: u16,
    pub quality: u8,
}
pub fn goods_rows(goods: &[Goods]) -> Vec<GoodsRow> {
    goods
        .iter()
        .map(|g| GoodsRow {
            item_code: g.kind.code(),
            count: g.count,
            quality: g.quality,
        })
        .collect()
}

#[derive(Debug, Serialize)]
pub struct PurchaseReceipt {
    pub command_ordinal: u64,
    pub caravan_id: u32,
    pub executed_tick: u64,
    pub goods_before: Vec<GoodsRow>,
    pub goods_after: Vec<GoodsRow>,
    pub unknown_bought_before: u64,
    pub unknown_bought_after: u64,
    pub eurekas_before: u64,
    pub eurekas_after: u64,
}

pub struct CommandSnapshot {
    tick: u64,
    command: Command,
    population: u64,
    wave_id: Option<u32>,
    wave_size: Option<u64>,
    wave: Option<WaveState>,
    caravan_id: Option<u32>,
    goods: Vec<GoodsRow>,
    unknown: u64,
    eurekas: u64,
}

pub fn before_command(e: &SimEngine, command: &Command) -> CommandSnapshot {
    let wave_id = match command {
        Command::AcceptMigrants { wave } | Command::DeclineMigrants { wave } => Some(*wave),
        _ => None,
    };
    let caravan_id = match command {
        Command::Trade { caravan, .. } => Some(*caravan),
        _ => None,
    };
    let nb = &e.resources.colony.neighbours;
    let wave = wave_id.and_then(|id| nb.waves.iter().find(|w| w.id == id));
    CommandSnapshot {
        tick: e.current_tick(),
        command: command.clone(),
        population: population(e),
        wave_id,
        wave_size: wave.map(|w| w.people.iter().fold(0, |n, _| n + 1)),
        wave: wave.map(WaveState::from),
        caravan_id,
        goods: caravan_id
            .and_then(|id| nb.caravans.iter().find(|c| c.id == id))
            .map_or_else(Vec::new, |c| goods_rows(&c.goods)),
        unknown: nb.tallies.unknown_bought,
        eurekas: nb.tallies.eurekas,
    }
}

pub fn after_command(
    e: &SimEngine,
    before: CommandSnapshot,
    result: &CommandResult,
    ordinal: u64,
) -> (CommandReceipt, Option<PurchaseReceipt>) {
    let nb = &e.resources.colony.neighbours;
    let purchase = before.caravan_id.map(|id| PurchaseReceipt {
        command_ordinal: ordinal,
        caravan_id: id,
        executed_tick: before.tick,
        goods_before: before.goods,
        goods_after: nb
            .caravans
            .iter()
            .find(|c| c.id == id)
            .map_or_else(Vec::new, |c| goods_rows(&c.goods)),
        unknown_bought_before: before.unknown,
        unknown_bought_after: nb.tallies.unknown_bought,
        eurekas_before: before.eurekas,
        eurekas_after: nb.tallies.eurekas,
    });
    (
        CommandReceipt {
            ordinal,
            upcoming_tick: before.tick,
            command: before.command,
            result: result.into(),
            population_before: before.population,
            population_after: population(e),
            wave_id: before.wave_id,
            wave_size: before.wave_size,
            wave_before: before.wave,
            wave_after: before
                .wave_id
                .and_then(|id| nb.waves.iter().find(|w| w.id == id))
                .map(WaveState::from),
        },
        purchase,
    )
}

pub fn waves(e: &SimEngine) -> BTreeMap<u32, WaveState> {
    e.resources
        .colony
        .neighbours
        .waves
        .iter()
        .map(|w| (w.id, w.into()))
        .collect()
}

#[derive(Debug, Serialize)]
pub struct WaveTransition {
    pub wave_id: u32,
    pub executed_tick: u64,
    pub before: Option<WaveState>,
    pub after: WaveState,
    pub population_before: u64,
    pub population_after: u64,
    pub configured_cap: Option<u32>,
    pub outcome: &'static str,
    pub population_observation: &'static str,
}

pub fn transitions(
    e: &SimEngine,
    before: &BTreeMap<u32, WaveState>,
    population_before: u64,
) -> Vec<WaveTransition> {
    let cap = sim_systems::runtime::neighbours::rule(&e.resources).map(|r| r.population_cap);
    waves(e)
        .into_iter()
        .filter_map(|(id, state)| {
            let old = before.get(&id);
            if old == Some(&state) {
                return None;
            }
            let outcome = if old.is_some_and(|o| o.joined_ids.len() < state.joined_ids.len()) {
                "joined"
            } else if old.is_some_and(|o| o.answer == Some("accepted"))
                && state.answer.is_none()
                && state.state == "waiting"
            {
                "acceptance_cleared_waiting"
            } else {
                "state_changed"
            };
            Some(WaveTransition {
                wave_id: id,
                executed_tick: e.resources.current_tick,
                before: old.cloned(),
                after: state,
                population_before,
                population_after: population(e),
                configured_cap: cap,
                outcome,
                population_observation: "tick_boundary_not_arrival_instant",
            })
        })
        .collect()
}
