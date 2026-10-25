//! Optional M-UX2 house-planner timing; measurement never feeds simulation decisions.
//! Emits one stderr JSON row per call, including early returns, only when profiling is enabled.
use std::time::Instant;

struct Sample {
    tick: u64,
    reserve_only: bool,
    start: Instant,
    last: Instant,
    phase: usize,
    phases: [u64; 5],
    candidates: usize,
    visited: usize,
}

/// A call-local diagnostic; no simulation state or persistent timing cache.
pub(super) struct HouseProfile(Option<Sample>);

impl HouseProfile {
    pub(super) fn new(tick: u64, reserve_only: bool) -> Self {
        if std::env::var("WORLDSIM_MUX2_PROFILE").as_deref() != Ok("1") {
            return Self(None);
        }
        let now = Instant::now();
        Self(Some(Sample {
            tick,
            reserve_only,
            start: now,
            last: now,
            phase: 0,
            phases: [0; 5],
            candidates: 0,
            visited: 0,
        }))
    }

    /// Begin capacity (1), candidate generation (2), ranking (3), or search (4).
    pub(super) fn phase(&mut self, phase: usize) {
        if let Some(sample) = self.0.as_mut() {
            sample.close_phase();
            sample.phase = phase.min(sample.phases.len() - 1);
        }
    }

    pub(super) fn candidates(&mut self, count: usize) {
        if let Some(sample) = self.0.as_mut() {
            sample.candidates = count;
        }
    }

    pub(super) fn visit(&mut self) {
        if let Some(sample) = self.0.as_mut() {
            sample.visited += 1;
        }
    }
}

impl Sample {
    fn close_phase(&mut self) {
        let now = Instant::now();
        let elapsed = u64::try_from(now.duration_since(self.last).as_nanos()).unwrap_or(u64::MAX);
        self.phases[self.phase] = self.phases[self.phase].saturating_add(elapsed);
        self.last = now;
    }
}

impl Drop for HouseProfile {
    fn drop(&mut self) {
        let Some(sample) = self.0.as_mut() else {
            return;
        };
        sample.close_phase();
        // Total excludes formatting/write time; native StewardSystem timing includes that overhead.
        let total =
            u64::try_from(sample.last.duration_since(sample.start).as_nanos()).unwrap_or(u64::MAX);
        eprintln!(
            "MUX2_HOUSE_PROFILE {{\"tick\":{},\"reserve_only\":{},\"total_ns\":{},\"preflight_ns\":{},\"capacity_ns\":{},\"generation_ns\":{},\"ranking_ns\":{},\"search_ns\":{},\"candidates\":{},\"visited\":{}}}",
            sample.tick, sample.reserve_only, total, sample.phases[0], sample.phases[1],
            sample.phases[2], sample.phases[3], sample.phases[4], sample.candidates, sample.visited,
        );
    }
}
