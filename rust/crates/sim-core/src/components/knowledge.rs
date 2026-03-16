use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

/// Knowledge transmission channels derived from dual-inheritance theory.
#[repr(u8)]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransmissionSource {
    /// Independently discovered by the agent.
    #[default]
    SelfDiscovered = 0,
    /// Learned through oral instruction.
    Oral = 1,
    /// Learned by observing another practitioner.
    Observed = 2,
    /// Learned through apprenticeship.
    Apprenticed = 3,
    /// Learned from recorded media.
    Recorded = 4,
    /// Learned through formal schooling.
    Schooled = 5,
}

/// A single piece of knowledge owned by an individual agent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KnowledgeEntry {
    /// Stable knowledge identifier, e.g. `TECH_FIRE_MAKING`.
    pub knowledge_id: String,
    /// How well the agent knows the concept, 0.0..=1.0.
    pub proficiency: f64,
    /// Where the knowledge came from.
    pub source: TransmissionSource,
    /// Tick when the knowledge was first acquired.
    pub acquired_tick: u32,
    /// Tick when the knowledge was last practiced or applied.
    pub last_used_tick: u32,
    /// Raw teacher entity id. `0` means self-discovered or no teacher.
    pub teacher_id: u64,
}

/// In-progress learning tracked separately from fully owned knowledge.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LearningState {
    /// Stable knowledge identifier being learned.
    pub knowledge_id: String,
    /// Current learning progress, 0.0..=1.0.
    pub progress: f64,
    /// Learning source channel.
    pub source: TransmissionSource,
    /// Raw teacher entity id. `0` means no current teacher.
    pub teacher_id: u64,
}

/// Individual knowledge ownership for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentKnowledge {
    /// Fully acquired knowledge entries owned by this agent.
    pub known: SmallVec<[KnowledgeEntry; 8]>,
    /// Optional in-progress learning record.
    pub learning: Option<LearningState>,
    /// Current teaching target: student entity id + knowledge id.
    pub teaching_target: Option<(u64, String)>,
    /// Innovation potential derived from intelligence, openness, and variance.
    pub innovation_potential: f64,
}

impl Default for AgentKnowledge {
    fn default() -> Self {
        Self {
            known: SmallVec::new(),
            learning: None,
            teaching_target: None,
            innovation_potential: 0.0,
        }
    }
}

impl AgentKnowledge {
    /// Returns true when the agent already owns the requested knowledge id.
    pub fn has_knowledge(&self, id: &str) -> bool {
        self.known.iter().any(|entry| entry.knowledge_id == id)
    }

    /// Returns the proficiency for a known entry, or 0.0 if absent.
    pub fn proficiency(&self, id: &str) -> f64 {
        self.known
            .iter()
            .find(|entry| entry.knowledge_id == id)
            .map(|entry| entry.proficiency)
            .unwrap_or(0.0)
    }

    /// Returns how many knowledge entries the agent currently owns.
    pub fn known_count(&self) -> usize {
        self.known.len()
    }

    /// Adds a knowledge entry if the id is not already present.
    pub fn learn(&mut self, entry: KnowledgeEntry) {
        if self.has_knowledge(&entry.knowledge_id) {
            return;
        }
        self.known.push(entry);
    }

    /// Removes a knowledge entry by id if present.
    pub fn forget(&mut self, id: &str) {
        self.known.retain(|entry| entry.knowledge_id != id);
    }
}

#[cfg(test)]
mod tests {
    use super::{AgentKnowledge, KnowledgeEntry, LearningState, TransmissionSource};

    fn sample_entry(id: &str, proficiency: f64) -> KnowledgeEntry {
        KnowledgeEntry {
            knowledge_id: id.to_string(),
            proficiency,
            source: TransmissionSource::Oral,
            acquired_tick: 1,
            last_used_tick: 2,
            teacher_id: 7,
        }
    }

    #[test]
    fn default_is_empty() {
        let knowledge = AgentKnowledge::default();
        assert!(knowledge.known.is_empty());
        assert!(knowledge.learning.is_none());
        assert!(knowledge.teaching_target.is_none());
        assert_eq!(knowledge.innovation_potential, 0.0);
    }

    #[test]
    fn learn_and_has() {
        let mut knowledge = AgentKnowledge::default();
        knowledge.learn(sample_entry("TECH_FIRE_MAKING", 0.75));

        assert!(knowledge.has_knowledge("TECH_FIRE_MAKING"));
        assert_eq!(knowledge.known_count(), 1);
    }

    #[test]
    fn learn_duplicate_noop() {
        let mut knowledge = AgentKnowledge::default();
        knowledge.learn(sample_entry("TECH_FIRE_MAKING", 0.75));
        knowledge.learn(sample_entry("TECH_FIRE_MAKING", 0.20));

        assert_eq!(knowledge.known_count(), 1);
        assert!((knowledge.proficiency("TECH_FIRE_MAKING") - 0.75).abs() < f64::EPSILON);
    }

    #[test]
    fn forget_removes() {
        let mut knowledge = AgentKnowledge::default();
        knowledge.learn(sample_entry("TECH_FIRE_MAKING", 0.75));
        knowledge.learn(sample_entry("TECH_FORAGING", 0.90));
        knowledge.forget("TECH_FIRE_MAKING");

        assert!(!knowledge.has_knowledge("TECH_FIRE_MAKING"));
        assert!(knowledge.has_knowledge("TECH_FORAGING"));
        assert_eq!(knowledge.known_count(), 1);
    }

    #[test]
    fn proficiency_returns_correct() {
        let mut knowledge = AgentKnowledge::default();
        knowledge.learning = Some(LearningState {
            knowledge_id: "TECH_SHELTER".to_string(),
            progress: 0.5,
            source: TransmissionSource::Observed,
            teacher_id: 11,
        });
        knowledge.learn(sample_entry("TECH_STONE_KNAPPING", 0.63));

        assert!((knowledge.proficiency("TECH_STONE_KNAPPING") - 0.63).abs() < f64::EPSILON);
        assert_eq!(knowledge.proficiency("TECH_UNKNOWN"), 0.0);
    }
}
