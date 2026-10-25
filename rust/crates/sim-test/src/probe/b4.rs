//! Opt-in, read-only B4 observer: pre-award inputs, persistent tallies and raw transitions.
mod craft;
mod daily;
mod hauls;
mod manifest;
mod trade;
mod types;
mod visits;

pub use manifest::Config;
use serde::Serialize;
use sim_core::colony::{content, Command, CommandResult, Job, RecipeKind};
use sim_core::components::Agent;
use sim_engine::SimEngine;
use std::collections::{BTreeMap, BTreeSet};
use types::*;
use visits::{CommandReceipt, CommandSnapshot, PurchaseReceipt, WaveState, WaveTransition};

#[derive(Debug, Serialize)]
pub struct Report {
    pub schema_version: u32,
    pub manifest_sha256: String,
    pub arm: manifest::Arm,
    pub seed: u64,
    pub ticks_per_day: u64,
    pub horizon_ticks: u64,
    pub status: &'static str,
    pub invalid_reasons: Vec<InvalidReason>,
    pub days: Vec<DayRow>,
    pub craft_events: Vec<CraftEvent>,
    pub tally_initial: Vec<TallyRow>,
    pub tally_final: Vec<TallyRow>,
    pub command_receipts: Vec<CommandReceipt>,
    pub wave_transitions: Vec<WaveTransition>,
    pub caravan_generations: Vec<trade::CaravanGeneration>,
    pub purchase_receipts: Vec<PurchaseReceipt>,
    pub haul_starts: Vec<hauls::HaulStart>,
    pub haul_terminals: Vec<hauls::HaulTerminal>,
    pub conservation: Conservation,
}

pub struct B4Watch {
    config: Config,
    report: Report,
    previous_tallies: craft::Tallies,
    candidates: BTreeMap<u64, CraftEvent>,
    previous_jobs: BTreeMap<u64, Job>,
    previous_waves: BTreeMap<u32, WaveState>,
    seen_caravans: BTreeSet<u32>,
    active_hauls: BTreeMap<u64, hauls::HaulStart>,
    population_before: u64,
    meat_hauls_before: u64,
    cold: ColdCounts,
    upcoming_tick: Option<u64>,
}

impl B4Watch {
    pub fn new(e: &SimEngine, config: Config, horizon_ticks: u64) -> Result<Self, String> {
        let catalog = content::installed().map_err(|e| e.to_string())?;
        if catalog.revision() != config.manifest.catalog_revision {
            return Err("B4 catalog revision mismatch".into());
        }
        let recipe = RecipeKind::KnapBlade
            .recipe()
            .ok_or("B4 primary recipe absent")?;
        if recipe.id.code() != 0 || recipe.identity.id != "knap_blade" || recipe.output.1 != 1 {
            return Err("B4 primary recipe identity/output mismatch".into());
        }
        if e.resources.ticks_per_day != 1440 {
            return Err("B4 requires 1440 ticks/day".into());
        }
        let tallies = e.resources.colony.tallies.craft_quality.clone();
        let report = Report {
            schema_version: 1,
            manifest_sha256: config.manifest_sha256.clone(),
            arm: config.manifest.arm,
            seed: e.resources.world_seed,
            ticks_per_day: e.resources.ticks_per_day,
            horizon_ticks,
            status: "VALID",
            invalid_reasons: Vec::new(),
            days: Vec::new(),
            craft_events: Vec::new(),
            tally_initial: craft::rows(&tallies),
            tally_final: Vec::new(),
            command_receipts: Vec::new(),
            wave_transitions: Vec::new(),
            caravan_generations: Vec::new(),
            purchase_receipts: Vec::new(),
            haul_starts: Vec::new(),
            haul_terminals: Vec::new(),
            conservation: Conservation::default(),
        };
        Ok(Self {
            config,
            report,
            previous_tallies: tallies,
            candidates: BTreeMap::new(),
            previous_jobs: BTreeMap::new(),
            previous_waves: visits::waves(e),
            seen_caravans: BTreeSet::new(),
            active_hauls: BTreeMap::new(),
            population_before: daily::population(e),
            meat_hauls_before: e.resources.colony.wildlife.tallies.meat_hauls,
            cold: ColdCounts::default(),
            upcoming_tick: None,
        })
    }

    pub fn before_command(&self, e: &SimEngine, command: &Command) -> CommandSnapshot {
        visits::before_command(e, command)
    }

    pub fn after_command(
        &mut self,
        e: &SimEngine,
        before: CommandSnapshot,
        result: &CommandResult,
    ) {
        let ordinal = self.report.command_receipts.iter().fold(0, |n, _| n + 1);
        let (receipt, purchase) = visits::after_command(e, before, result, ordinal);
        self.report.command_receipts.push(receipt);
        if let Some(purchase) = purchase {
            self.report.purchase_receipts.push(purchase);
        }
    }

    pub fn before_tick(&mut self, e: &SimEngine) {
        self.upcoming_tick = Some(e.current_tick());
        self.candidates = craft::capture(e, &self.config);
        // Full persistent maps, never a surviving-agent or primary-recipe projection.
        if self.previous_tallies != e.resources.colony.tallies.craft_quality {
            self.invalid(
                "TALLY_CHANGED_OUTSIDE_TICK",
                e.current_tick(),
                "command interval altered craft tallies".into(),
            );
        }
        self.previous_tallies = e.resources.colony.tallies.craft_quality.clone();
        self.previous_jobs = hauls::jobs(e);
        self.previous_waves = visits::waves(e);
        self.population_before = daily::population(e);
        self.meat_hauls_before = e.resources.colony.wildlife.tallies.meat_hauls;
    }

    pub fn after_tick(&mut self, e: &SimEngine) {
        let r = &e.resources;
        if self.upcoming_tick.take() != Some(r.current_tick) {
            self.invalid(
                "TICK_SEAM",
                r.current_tick,
                "missing or mismatched before_tick".into(),
            );
        }
        let alive = e
            .world
            .query::<&Agent>()
            .iter()
            .map(|(_, a)| a.id)
            .collect();
        let (events, errors) = craft::reconcile(
            &self.previous_tallies,
            &r.colony.tallies.craft_quality,
            &self.candidates,
            &alive,
            r.current_tick,
        );
        self.report.craft_events.extend(events);
        self.report.invalid_reasons.extend(errors);
        self.previous_tallies = r.colony.tallies.craft_quality.clone();
        self.report.wave_transitions.extend(visits::transitions(
            e,
            &self.previous_waves,
            self.population_before,
        ));
        for c in &r.colony.neighbours.caravans {
            if self.seen_caravans.insert(c.id) {
                match trade::capture(e, c.id, &self.config) {
                    Ok(row) => self.report.caravan_generations.push(row),
                    Err(error) => self.invalid("CARAVAN_INPUTS", r.current_tick, error),
                }
            }
        }
        self.observe_hauls(e);
        daily::sample_cold(e, &mut self.cold);
        if let Some(day) = daily::day_end(r.current_tick) {
            self.report
                .days
                .push(daily::row(e, day, &self.cold, &self.report.conservation));
            self.check_primary(e);
        }
    }

    /// Reuse the ordinary probe's invariant result; no second simulation scan or hidden mutation.
    pub fn conservation(&mut self, ledger_bad: u64, reservation_bad: u64) {
        self.report.conservation.ledger_violation_ticks = ledger_bad;
        self.report.conservation.reservation_violation_ticks = reservation_bad;
    }

    fn invalid(&mut self, code: &str, tick: u64, detail: String) {
        self.report.invalid_reasons.push(InvalidReason {
            code: code.into(),
            executed_tick: Some(tick),
            agent_id: None,
            detail,
        });
    }

    fn observe_hauls(&mut self, e: &SimEngine) {
        let terminal = hauls::terminals(e, &mut self.active_hauls);
        self.report.conservation.unresolved_terminal_count += terminal.iter().fold(0, |n, _| n + 1);
        self.report.haul_terminals.extend(terminal);
        let starts = hauls::starts(e, &self.previous_jobs);
        let observed = starts.iter().fold(0u64, |n, _| n + 1);
        match e
            .resources
            .colony
            .wildlife
            .tallies
            .meat_hauls
            .checked_sub(self.meat_hauls_before)
        {
            Some(delta) if delta >= observed => {
                self.report.conservation.unresolved_start_count += delta - observed
            }
            _ => self.invalid(
                "HAUL_TALLY_MISMATCH",
                e.resources.current_tick,
                "observed starts exceed tally delta or tally regressed".into(),
            ),
        }
        for row in starts {
            self.active_hauls.insert(row.agent_id, row.clone());
            self.report.haul_starts.push(row);
        }
    }

    fn check_primary(&mut self, e: &SimEngine) {
        let mut expected: craft::Tallies = self
            .report
            .tally_initial
            .iter()
            .filter(|r| r.recipe_code == 0)
            .map(|r| {
                (
                    (r.agent_id, r.recipe_code),
                    (r.completed_count, r.quality_sum),
                )
            })
            .collect();
        for event in &self.report.craft_events {
            let v = expected
                .entry((event.agent_id, event.recipe_code))
                .or_default();
            v.0 += event.completion_count_delta;
            v.1 += u64::from(event.actual_quality);
        }
        let actual: craft::Tallies = e
            .resources
            .colony
            .tallies
            .craft_quality
            .iter()
            .filter(|((_, recipe), _)| *recipe == 0)
            .map(|(k, v)| (*k, *v))
            .collect();
        if expected != actual {
            self.invalid(
                "PRIMARY_COVERAGE",
                e.resources.current_tick,
                "event sums differ from full persistent primary tallies".into(),
            );
        }
    }

    pub fn report(mut self, e: &SimEngine) -> Report {
        self.check_primary(e);
        self.report.tally_final = craft::rows(&e.resources.colony.tallies.craft_quality);
        if e.current_tick() != self.report.horizon_ticks
            || u64::try_from(self.report.days.len()).ok() != Some(self.report.horizon_ticks / 1440)
        {
            self.invalid(
                "INCOMPLETE_HORIZON",
                e.resources.current_tick,
                "missing ticks or daily rows".into(),
            );
        }
        if !self.report.invalid_reasons.is_empty() {
            self.report.status = "OBSERVER_INVALID";
        }
        self.report
    }
}

#[cfg(test)]
mod tests;
