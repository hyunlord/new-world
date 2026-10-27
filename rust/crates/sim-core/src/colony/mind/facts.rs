//! Tick-local columnar provider output, with explicit writes and structural stages.

use super::{CompiledMind, FactId, FactKind, LifeStage, MindEvalError, MindRevision};
use crate::components::AgentId;

/// One table allocation per mind tick; providers may apply ordered same-tick deltas.
#[derive(Debug, Clone)]
pub struct PersonFacts {
    revision: MindRevision,
    agents: Vec<AgentId>,
    tick: u64,
    kinds: Vec<FactKind>,
    columns: Vec<Vec<i64>>,
    written: Vec<Vec<bool>>,
    stages: Vec<Option<LifeStage>>,
}
impl PersonFacts {
    /// Allocate policy-referenced columns; no cell or life stage is implicitly valid.
    pub fn new(
        policy: &CompiledMind,
        agents: Vec<AgentId>,
        tick: u64,
    ) -> Result<Self, MindEvalError> {
        if agents.windows(2).any(|pair| pair[0] >= pair[1]) {
            return Err(MindEvalError::AgentOrder);
        }
        let count = agents.len();
        Ok(Self {
            revision: policy.revision(),
            agents,
            tick,
            kinds: policy.facts().iter().map(|fact| fact.kind).collect(),
            columns: vec![vec![0; count]; policy.facts().len()],
            written: vec![vec![false; count]; policy.facts().len()],
            stages: vec![None; count],
        })
    }
    /// Sorted agent IDs for provider loops.
    pub fn agents(&self) -> &[AgentId] {
        &self.agents
    }
    /// Observation tick shared by every row.
    pub const fn tick(&self) -> u64 {
        self.tick
    }
    /// Binary-search the row of a living person.
    pub fn row_index(&self, agent: AgentId) -> Option<usize> {
        self.agents.binary_search(&agent).ok()
    }
    /// Set one required fact or deterministic overlay update.
    pub fn set(&mut self, fact: FactId, row: usize, value: i64) -> Result<(), MindEvalError> {
        let kind = self
            .kinds
            .get(fact.index())
            .ok_or(MindEvalError::InvalidIndex("fact"))?;
        if *kind == FactKind::Bool && !matches!(value, 0 | 1) {
            return Err(MindEvalError::InvalidBoolean(fact));
        }
        let cell = self
            .columns
            .get_mut(fact.index())
            .and_then(|column| column.get_mut(row))
            .ok_or(MindEvalError::InvalidIndex("row"))?;
        *cell = value;
        self.written[fact.index()][row] = true;
        Ok(())
    }
    /// Read only a written provider value; missing cells never masquerade as zero.
    pub fn get(&self, fact: FactId, row: usize) -> Result<i64, MindEvalError> {
        let written = self
            .written
            .get(fact.index())
            .and_then(|column| column.get(row))
            .ok_or(MindEvalError::InvalidIndex("fact/row"))?;
        if !written {
            return Err(MindEvalError::UnwrittenFact { fact, row });
        }
        Ok(self.columns[fact.index()][row])
    }
    /// Write the structural life stage, independently of registered numerical facts.
    pub fn set_life_stage(&mut self, row: usize, stage: LifeStage) -> Result<(), MindEvalError> {
        let slot = self
            .stages
            .get_mut(row)
            .ok_or(MindEvalError::InvalidIndex("row"))?;
        *slot = Some(stage);
        Ok(())
    }
    /// Structural stage for provider/executor use.
    pub fn life_stage(&self, row: usize) -> Result<LifeStage, MindEvalError> {
        self.stages
            .get(row)
            .ok_or(MindEvalError::InvalidIndex("row"))?
            .ok_or(MindEvalError::MissingLifeStage(row))
    }
    /// Borrow a fully supplied row by agent identity.
    pub fn row(&self, agent: AgentId) -> Result<FactRow<'_>, MindEvalError> {
        self.row_at(
            self.row_index(agent)
                .ok_or(MindEvalError::InvalidIndex("agent"))?,
        )
    }
    /// Borrow a fully supplied row by index after provider fill/delta repair.
    pub fn row_at(&self, index: usize) -> Result<FactRow<'_>, MindEvalError> {
        let agent = *self
            .agents
            .get(index)
            .ok_or(MindEvalError::InvalidIndex("row"))?;
        let stage = self.life_stage(index)?;
        for (fact, column) in self.written.iter().enumerate() {
            if !column[index] {
                return Err(MindEvalError::UnwrittenFact {
                    fact: FactId(fact),
                    row: index,
                });
            }
        }
        Ok(FactRow {
            table: self,
            index,
            agent,
            stage,
        })
    }
}
/// Immutable view valid until executor releases it before same-tick mutations.
#[derive(Debug, Clone, Copy)]
pub struct FactRow<'a> {
    table: &'a PersonFacts,
    index: usize,
    agent: AgentId,
    stage: LifeStage,
}
impl FactRow<'_> {
    /// Provider row index.
    pub const fn index(self) -> usize {
        self.index
    }
    /// Person identity.
    pub const fn agent(self) -> AgentId {
        self.agent
    }
    /// Observation tick.
    pub const fn tick(self) -> u64 {
        self.table.tick
    }
    /// Structural person stage.
    pub const fn life_stage(self) -> LifeStage {
        self.stage
    }
    /// Compiled dictionary owning these columns.
    pub const fn revision(self) -> MindRevision {
        self.table.revision
    }
    /// Read a validated filled column without allocating.
    pub fn get(self, fact: FactId) -> Result<i64, MindEvalError> {
        self.table.get(fact, self.index)
    }
}

/// Evaluation-local fact source; implementations may memoize reads without changing values.
pub trait FactReader {
    /// Opt in only when, within one selection, every successful fact read remains
    /// the same value and cannot later fail, and omitting repeat reads has no
    /// observable side effects. The selector may reuse successful comparisons.
    /// Errors on the first requested read are never cached or bypassed.
    const STABLE_READS: bool = false;
    /// Provider failures retain their runtime-specific detail.
    type Error: From<MindEvalError>;
    /// Person identity at this evaluation boundary.
    fn agent(&self) -> AgentId;
    /// Observation tick.
    fn tick(&self) -> u64;
    /// Structural life stage.
    fn life_stage(&self) -> LifeStage;
    /// Compiled policy owning these facts.
    fn revision(&self) -> MindRevision;
    /// Read a requested value; absence must never fabricate zero.
    fn get(&mut self, fact: FactId) -> Result<i64, Self::Error>;
}
impl FactReader for FactRow<'_> {
    const STABLE_READS: bool = true;
    type Error = MindEvalError;
    fn agent(&self) -> AgentId {
        (*self).agent()
    }
    fn tick(&self) -> u64 {
        (*self).tick()
    }
    fn life_stage(&self) -> LifeStage {
        (*self).life_stage()
    }
    fn revision(&self) -> MindRevision {
        (*self).revision()
    }
    fn get(&mut self, fact: FactId) -> Result<i64, Self::Error> {
        (*self).get(fact)
    }
}
