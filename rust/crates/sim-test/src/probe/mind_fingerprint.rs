//! Domain-separated hidden mind state for trajectory determinism checks.
//! RON streams every serialized field, including tuple-keyed provenance maps,
//! directly into FNV; no full-state Debug string or encoded byte buffer is built.
use sim_core::colony::mind_state::MindState;
use sim_engine::SimResources;
use std::io::{self, Write};
use std::sync::atomic::Ordering;

fn populated(state: &MindState) -> bool {
    state.policy_source.is_some()
        || state.history.is_some()
        || !state.running.is_empty()
        || !state.fact_origins.is_empty()
        || !state.order_origins.is_empty()
        || !state.job_origins.is_empty()
        || !state.draft_origins.is_empty()
        || !state.fact_source_events.is_empty()
}
struct HashWriter<'a>(&'a mut u64);
impl Write for HashWriter<'_> {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        super::fingerprint::feed(self.0, bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
/// Preserve the legacy byte stream exactly when mind is empty. A populated mind
/// binds its entire persisted state and the next causal ID, which affects future reasons.
/// Serialization failure is fatal: a partial hash must never claim deterministic equality.
pub(super) fn append(hash: &mut u64, resources: &SimResources) {
    if !populated(&resources.mind) {
        return;
    }
    super::fingerprint::feed(hash, b"\0M-CORE-mind-state-v1\0");
    ron::ser::to_writer(
        HashWriter(hash),
        &(
            &resources.mind,
            resources.next_event_id.load(Ordering::Relaxed),
        ),
    )
    .expect("mind determinism fingerprint serialization failed");
}

#[cfg(test)]
#[path = "mind_fingerprint_tests.rs"]
mod tests;
