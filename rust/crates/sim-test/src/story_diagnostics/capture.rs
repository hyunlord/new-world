//! Separate immutable chronicle graph export for explaining story-yield totals.
use serde::Serialize;
use sim_core::chronicle::{CauseRef, Chronicle, ChronicleEntry, ChronicleKind, NeedSample};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Serialize)]
pub(super) struct YearKinds {
    year: u64,
    pub(super) observed_ticks: u64,
    pub(super) complete_year: bool,
    pub(super) kinds: BTreeMap<&'static str, usize>,
}

#[derive(Serialize)]
pub(super) struct EntryDiagnostic<'a> {
    index: u32,
    kind_name: &'static str,
    entry: &'a ChronicleEntry,
    pub(super) cause: Option<CauseRef>,
}

#[derive(Serialize)]
pub(super) struct Diagnostics<'a> {
    schema_version: u8,
    seed: u64,
    end_tick: u64,
    ticks_per_year: u64,
    fingerprint: &'a str,
    scope: &'static str,
    pub(super) years: Vec<YearKinds>,
    pub(super) entries: Vec<EntryDiagnostic<'a>>,
    referenced_samples: BTreeMap<u32, Option<&'a NeedSample>>,
}

pub(super) fn capture<'a>(
    chronicle: &'a Chronicle,
    seed: u64,
    end_tick: u64,
    ticks_per_year: u64,
    fingerprint: &'a str,
) -> Result<Diagnostics<'a>, String> {
    // Reuse the metric's window validation and annual boundaries.
    let measured = crate::story_yield::measure(chronicle, end_tick, ticks_per_year)?;
    let mut years: Vec<_> = measured
        .into_iter()
        .map(|year| YearKinds {
            year: year.year,
            observed_ticks: year.observed_ticks,
            complete_year: year.complete_year,
            kinds: ChronicleKind::ALL
                .iter()
                .map(|kind| (kind.as_str(), 0))
                .collect(),
        })
        .collect();
    let mut entries = Vec::with_capacity(chronicle.len());
    let mut referenced_samples = BTreeMap::new();
    for (index, entry) in chronicle.entries().iter().enumerate() {
        let index = u32::try_from(index).map_err(|_| "chronicle index exceeds u32")?;
        let year = usize::try_from(entry.tick / ticks_per_year)
            .map_err(|_| "chronicle year exceeds usize")?;
        let counts = years.get_mut(year).ok_or("chronicle year outside window")?;
        *counts.kinds.entry(entry.kind.as_str()).or_default() += 1;
        let cause = chronicle.cause_of(index);
        if let Some(CauseRef::Sample { sample, .. }) = cause {
            referenced_samples
                .entry(sample)
                .or_insert_with(|| chronicle.sample(sample));
        }
        entries.push(EntryDiagnostic {
            index,
            kind_name: entry.kind.as_str(),
            entry,
            cause,
        });
    }
    Ok(Diagnostics {
        schema_version: 1,
        seed,
        end_tick,
        ticks_per_year,
        fingerprint,
        scope: "Chronicle entries and typed cause sidecars only; not the mind EventId graph. Null referenced sample means absent; sample.entry=null is a valid root. Original indices preserved across years.",
        years,
        entries,
        referenced_samples,
    })
}

pub(super) fn write(
    out: &Path,
    chronicle: &Chronicle,
    seed: u64,
    end_tick: u64,
    ticks_per_year: u64,
    fingerprint: &str,
) -> Result<(), String> {
    let report = capture(chronicle, seed, end_tick, ticks_per_year, fingerprint)?;
    let bytes = serde_json::to_vec_pretty(&report).map_err(|error| error.to_string())?;
    std::fs::write(out.join(format!("story_diagnostics_{seed}.json")), bytes)
        .map_err(|error| error.to_string())
}
