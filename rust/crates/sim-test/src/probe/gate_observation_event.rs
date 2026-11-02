//! Explicit event-conditioned endpoints for the selected-pack M6a fixture only.
use super::GateObservation;
use serde::Serialize;
use sim_engine::SimEngine;
use std::path::Path;

#[derive(Serialize)]
pub(super) struct EventHorizon {
    condition: &'static str,
    condition_met: Option<bool>,
    reached_limit: Option<bool>,
}

impl EventHorizon {
    pub(super) fn complete(&mut self, now: u64, initial: u64, maximum: u64) -> bool {
        self.reached_limit = Some(now == maximum);
        now >= initial && now <= maximum && (self.condition_met == Some(true) || now == maximum)
    }
}

impl GateObservation {
    /// Start schema v2 with a maximum duration, never a guessed realized horizon.
    pub(crate) fn begin_food_chain(
        engine: &SimEngine,
        out: &Path,
        maximum_ticks: u64,
    ) -> Result<Self, String> {
        let mut watch = Self::begin(engine, out, "m6a-data-only", 0, "primary", maximum_ticks)?;
        watch.observation.schema_version = 2;
        watch.observation.event_horizon = Some(EventHorizon {
            condition: "m6a_food_chain_complete",
            condition_met: None,
            reached_limit: None,
        });
        watch.persist()?;
        Ok(watch)
    }

    /// Bind the producer's observed condition; the wrapper must check its normal report.
    pub(crate) fn finish_food_chain(
        mut self,
        engine: &SimEngine,
        loop_complete: bool,
        condition_met: bool,
    ) -> Result<(), String> {
        let event = self
            .observation
            .event_horizon
            .as_mut()
            .ok_or("not a food-chain observation")?;
        event.condition_met = Some(condition_met);
        // Record limit even when loop_complete=false (finish short-circuits then).
        event.reached_limit =
            Some(engine.current_tick() == self.observation.requested_terminal_tick);
        self.finish(engine, loop_complete)
    }
}
