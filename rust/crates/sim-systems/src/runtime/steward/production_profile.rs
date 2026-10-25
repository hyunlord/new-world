//! Opt-in input-planner phase timing and counters; no simulation decisions or persistent state.
use sim_core::{colony::ItemKind, nav::NavState};
use std::time::Instant;

#[derive(Clone, Copy)]
pub(super) enum Phase {
    Other,
    Recipes,
    Knowledge,
    Materials,
    Marked,
    Count,
    Filter,
    Sort,
    Emit,
}

#[derive(Default)]
struct KindRow {
    kind: u32,
    wanted: u32,
    marked: u32,
    available: u32,
    deficit: u32,
    scanned: usize,
    reaches: usize,
    candidates: usize,
    emitted: usize,
    region_before: u64,
    region_builds: u64,
    phases: [u64; 9],
}

struct Sample {
    tick: u64,
    start: Instant,
    last: Instant,
    phase: Phase,
    phases: [u64; 9],
    recipes: usize,
    nodes: usize,
    known: usize,
    material_entities: usize,
    material_lines: usize,
    rows: Vec<KindRow>,
    active: bool,
}

pub(super) struct InputProfile(Option<Sample>);
impl InputProfile {
    pub(super) fn new(tick: u64, recipes: usize, nodes: usize) -> Self {
        if std::env::var("WORLDSIM_MUX2_PROFILE").as_deref() != Ok("1") {
            return Self(None);
        }
        let now = Instant::now();
        Self(Some(Sample {
            tick,
            start: now,
            last: now,
            phase: Phase::Other,
            phases: [0; 9],
            recipes,
            nodes,
            known: 0,
            material_entities: 0,
            material_lines: 0,
            rows: Vec::new(),
            active: false,
        }))
    }
    pub(super) fn enabled(&self) -> bool {
        self.0.is_some()
    }
    pub(super) fn phase(&mut self, phase: Phase) {
        if let Some(s) = &mut self.0 {
            s.close();
            s.phase = phase;
        }
    }
    pub(super) fn known(&mut self) {
        if let Some(s) = &mut self.0 {
            s.known += 1;
        }
    }
    pub(super) fn material_entity(&mut self) {
        if let Some(s) = &mut self.0 {
            s.material_entities += 1;
        }
    }
    pub(super) fn material_line(&mut self) {
        if let Some(s) = &mut self.0 {
            s.material_lines += 1;
        }
    }
    pub(super) fn begin_kind(&mut self, kind: ItemKind, wanted: u32, nav: &NavState) {
        if let Some(s) = &mut self.0 {
            s.close();
            s.rows.push(KindRow {
                kind: kind.code(),
                wanted,
                region_before: nav.stats.borrow().region_builds,
                ..KindRow::default()
            });
            s.active = true;
        }
    }
    pub(super) fn demand(&mut self, marked: u32, available: u32, deficit: u32) {
        if let Some(row) = self.0.as_mut().and_then(|s| s.rows.last_mut()) {
            row.marked = marked;
            row.available = available;
            row.deficit = deficit;
        }
    }
    pub(super) fn finish_kind(&mut self, counts: [usize; 4], nav: &NavState) {
        if let Some(s) = &mut self.0 {
            s.close();
            if let Some(row) = s.rows.last_mut() {
                [row.scanned, row.reaches, row.candidates, row.emitted] = counts;
                row.region_builds = nav
                    .stats
                    .borrow()
                    .region_builds
                    .saturating_sub(row.region_before);
            }
            s.active = false;
            s.phase = Phase::Other;
        }
    }
}
impl Sample {
    fn close(&mut self) {
        let now = Instant::now();
        let elapsed = u64::try_from(now.duration_since(self.last).as_nanos()).unwrap_or(u64::MAX);
        let phase = self.phase as usize;
        self.phases[phase] = self.phases[phase].saturating_add(elapsed);
        if self.active {
            if let Some(row) = self.rows.last_mut() {
                row.phases[phase] = row.phases[phase].saturating_add(elapsed);
            }
        }
        self.last = now;
    }
}
impl Drop for InputProfile {
    fn drop(&mut self) {
        let Some(s) = &mut self.0 else {
            return;
        };
        s.close();
        let total = u64::try_from(s.last.duration_since(s.start).as_nanos()).unwrap_or(u64::MAX);
        eprintln!("MUX2_INPUTS_PROFILE {{\"tick\":{},\"total_ns\":{},\"phases_ns\":{:?},\"recipes\":{},\"known\":{},\"material_entities\":{},\"material_lines\":{},\"kinds\":{},\"nodes\":{},\"phase_order\":[\"other\",\"recipes\",\"knowledge\",\"materials\",\"marked\",\"count\",\"filter_reach\",\"sort\",\"emit\"]}}",
            s.tick,total,s.phases,s.recipes,s.known,s.material_entities,s.material_lines,s.rows.len(),s.nodes);
        for r in &s.rows {
            eprintln!("MUX2_INPUT_KIND_PROFILE {{\"tick\":{},\"kind\":{},\"wanted\":{},\"marked\":{},\"available\":{},\"deficit\":{},\"scanned\":{},\"reaches\":{},\"candidates\":{},\"emitted\":{},\"region_builds\":{},\"phases_ns\":{:?}}}",
                s.tick,r.kind,r.wanted,r.marked,r.available,r.deficit,r.scanned,r.reaches,r.candidates,r.emitted,r.region_builds,r.phases);
        }
    }
}
