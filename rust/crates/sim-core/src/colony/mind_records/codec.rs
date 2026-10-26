//! Canonical unsigned varints and signed zigzag; allocation is bounded by input bytes.
use super::reader::Reader;
use super::{ChoiceCandidate, ChoiceFact, MindChoiceRecord, MindRecordError};
use crate::colony::mind::{CompiledMind, FactKind};

pub(super) struct Raw {
    pub revision: u64,
    pub id: u64,
    pub tick: u64,
    pub parent: Option<u64>,
    pub agent: u64,
    pub node: usize,
    pub band: usize,
    pub score: i64,
    pub top3: Vec<(usize, i64)>,
    pub facts: Vec<(usize, i64, Option<u64>)>,
}
fn unsigned(out: &mut Vec<u8>, mut value: u64) {
    while value >= 128 {
        out.push((value as u8 & 127) | 128);
        value >>= 7;
    }
    out.push(value as u8);
}
fn signed(out: &mut Vec<u8>, value: i64) {
    unsigned(
        out,
        value
            .unsigned_abs()
            .wrapping_mul(2)
            .wrapping_sub(u64::from(value < 0)),
    );
}
fn optional(out: &mut Vec<u8>, value: Option<u64>) {
    match value {
        None => out.push(0),
        Some(value) => {
            out.push(1);
            unsigned(out, value);
        }
    }
}
fn slot(out: &mut Vec<u8>, value: usize) -> Result<(), MindRecordError> {
    unsigned(
        out,
        u64::try_from(value).map_err(|_| MindRecordError::Invalid("slot overflow"))?,
    );
    Ok(())
}
pub(super) fn encode(
    policy: &CompiledMind,
    record: &MindChoiceRecord,
) -> Result<Vec<u8>, MindRecordError> {
    if record.revision != policy.revision() {
        return Err(MindRecordError::PolicyMismatch);
    }
    let raw = Raw {
        revision: policy.revision().code(),
        id: record.id,
        tick: record.tick,
        parent: record.parent,
        agent: record.agent,
        node: record.node.index(),
        band: record.band.index(),
        score: record.score,
        top3: record
            .top3
            .iter()
            .map(|c| (c.node.index(), c.score))
            .collect(),
        facts: record
            .facts
            .iter()
            .map(|f| (f.fact.index(), f.value, f.cause))
            .collect(),
    };
    validate(&raw)?;
    let _ = resolve(raw, policy)?;
    let mut out = vec![1];
    for value in [policy.revision().code(), record.id, record.tick] {
        unsigned(&mut out, value);
    }
    optional(&mut out, record.parent);
    unsigned(&mut out, record.agent);
    slot(&mut out, record.node.index())?;
    slot(&mut out, record.band.index())?;
    signed(&mut out, record.score);
    slot(&mut out, record.top3.len())?;
    for candidate in &record.top3 {
        slot(&mut out, candidate.node.index())?;
        signed(&mut out, candidate.score);
    }
    slot(&mut out, record.facts.len())?;
    for fact in &record.facts {
        slot(&mut out, fact.fact.index())?;
        signed(&mut out, fact.value);
        optional(&mut out, fact.cause);
    }
    Ok(out)
}
pub(super) fn parse(bytes: &[u8]) -> Result<Raw, MindRecordError> {
    let mut r = Reader { bytes, at: 0 };
    if r.byte()? != 1 {
        return Err(MindRecordError::Invalid("unsupported schema"));
    }
    let revision = r.unsigned()?;
    let id = r.unsigned()?;
    let tick = r.unsigned()?;
    let parent = r.optional()?;
    let agent = r.unsigned()?;
    let node = r.slot()?;
    let band = r.slot()?;
    let score = r.signed()?;
    let count = r.slot()?;
    if count > 3 {
        return Err(MindRecordError::Invalid("more than three candidates"));
    }
    let mut top3 = Vec::with_capacity(count);
    for _ in 0..count {
        top3.push((r.slot()?, r.signed()?));
    }
    let count = r.slot()?;
    if count > (bytes.len() - r.at) / 3 {
        return Err(MindRecordError::Invalid("fact count exceeds payload"));
    }
    let mut facts = Vec::with_capacity(count);
    for _ in 0..count {
        facts.push((r.slot()?, r.signed()?, r.optional()?));
    }
    if r.at != bytes.len() {
        return Err(MindRecordError::Invalid("trailing bytes"));
    }
    let raw = Raw {
        revision,
        id,
        tick,
        parent,
        agent,
        node,
        band,
        score,
        top3,
        facts,
    };
    validate(&raw)?;
    Ok(raw)
}
fn validate(raw: &Raw) -> Result<(), MindRecordError> {
    if raw.top3.is_empty() {
        return Err(MindRecordError::Invalid("missing ranked candidates"));
    }
    if raw.top3.len() > 3 {
        return Err(MindRecordError::Invalid("more than three candidates"));
    }
    if raw
        .top3
        .iter()
        .enumerate()
        .any(|(i, c)| raw.top3[..i].iter().any(|p| p.0 == c.0))
    {
        return Err(MindRecordError::Invalid("duplicate candidate"));
    }
    if raw
        .top3
        .windows(2)
        .any(|w| w[0].1 < w[1].1 || (w[0].1 == w[1].1 && w[0].0 > w[1].0))
    {
        return Err(MindRecordError::Invalid("candidate ranking"));
    }
    if raw.facts.windows(2).any(|w| w[0].0 >= w[1].0) {
        return Err(MindRecordError::Invalid("facts not uniquely sorted"));
    }
    if raw
        .top3
        .iter()
        .any(|candidate| candidate.0 == raw.node && candidate.1 != raw.score)
    {
        return Err(MindRecordError::Invalid(
            "selected candidate score mismatch",
        ));
    }
    if raw.parent.is_some() && !raw.facts.iter().any(|fact| fact.2 == raw.parent) {
        return Err(MindRecordError::Invalid(
            "parent is not a deciding fact cause",
        ));
    }
    if raw.parent == Some(raw.id) || raw.facts.iter().any(|f| f.2 == Some(raw.id)) {
        return Err(MindRecordError::Invalid("self cause"));
    }
    Ok(())
}
pub(super) fn resolve(
    raw: Raw,
    policy: &CompiledMind,
) -> Result<MindChoiceRecord, MindRecordError> {
    if raw.revision != policy.revision().code() {
        return Err(MindRecordError::PolicyMismatch);
    }
    let node = policy
        .node_id_at(raw.node)
        .ok_or(MindRecordError::UnknownSlot("node"))?;
    let band = policy
        .band_id_at(raw.band)
        .ok_or(MindRecordError::UnknownSlot("band"))?;
    if policy.node(node).map(|n| n.band) != Some(band) {
        return Err(MindRecordError::Invalid("node band mismatch"));
    }
    let mut top3 = Vec::with_capacity(raw.top3.len());
    for (index, score) in raw.top3 {
        let node = policy
            .node_id_at(index)
            .ok_or(MindRecordError::UnknownSlot("candidate"))?;
        if policy.node(node).map(|n| n.band) != Some(band) {
            return Err(MindRecordError::Invalid("candidate band mismatch"));
        }
        top3.push(ChoiceCandidate { node, score });
    }
    let mut facts = Vec::with_capacity(raw.facts.len());
    for (index, value, cause) in raw.facts {
        let fact = policy
            .fact_id_at(index)
            .ok_or(MindRecordError::UnknownSlot("fact"))?;
        if matches!(policy.facts()[index].kind, FactKind::Bool) && value != 0 && value != 1 {
            return Err(MindRecordError::Invalid("boolean fact value"));
        }
        facts.push(ChoiceFact { fact, value, cause });
    }
    Ok(MindChoiceRecord {
        revision: policy.revision(),
        id: raw.id,
        parent: raw.parent,
        agent: raw.agent,
        tick: raw.tick,
        node,
        band,
        score: raw.score,
        top3,
        facts,
    })
}
