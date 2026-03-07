/// All simulation events — mirrors SimulationBus signals from GDScript.
///
/// Events are collected during each tick, then drained and published
/// via EventBus at the end of the tick. This avoids borrow issues.
use serde::{Deserialize, Serialize};
use sim_core::components::LlmRequestType;
use sim_core::ids::{BuildingId, EntityId, SettlementId};

/// LLM lifecycle and request/response events emitted through the engine event bus.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmEvent {
    RequestSubmitted {
        entity_id: u64,
        request_type: LlmRequestType,
    },
    ResponseReceived {
        entity_id: u64,
        generation_ms: u32,
    },
    RequestTimedOut {
        entity_id: u64,
    },
    ServerStarted,
    ServerStopped,
    ServerHealthCheckFailed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    // ── Entity Lifecycle ──────────────────────────────────────────────────────
    EntitySpawned {
        entity_id: EntityId,
    },
    EntityDied {
        entity_id: EntityId,
        cause: String,
    },
    EntityRemoved {
        entity_id: EntityId,
    },

    // ── Tick ──────────────────────────────────────────────────────────────────
    TickCompleted {
        tick: u64,
    },
    SimulationPaused,
    SimulationResumed,
    SpeedChanged {
        speed_index: i32,
    },

    // ── Needs ─────────────────────────────────────────────────────────────────
    NeedChanged {
        entity_id: EntityId,
        need: String,
        old_value: f64,
        new_value: f64,
    },
    NeedCritical {
        entity_id: EntityId,
        need: String,
    },

    // ── Emotions ─────────────────────────────────────────────────────────────
    EmotionChanged {
        entity_id: EntityId,
        emotion: String,
        intensity: f64,
    },
    SteeringParamsUpdated {
        entity_id: EntityId,
    },

    // ── Stress / Mental ───────────────────────────────────────────────────────
    StressChanged {
        entity_id: EntityId,
        stress: f64,
    },
    MentalBreakTriggered {
        entity_id: EntityId,
        break_type: String,
    },
    TraumaRecorded {
        entity_id: EntityId,
        trauma_type: String,
    },

    // ── Social ────────────────────────────────────────────────────────────────
    RelationshipChanged {
        entity_a: EntityId,
        entity_b: EntityId,
        affinity: f64,
    },
    SocialEventOccurred {
        event_type: String,
        participants: Vec<EntityId>,
    },

    // ── Economy / Jobs ────────────────────────────────────────────────────────
    JobAssigned {
        entity_id: EntityId,
        job: String,
    },
    BuildingConstructed {
        building_id: BuildingId,
        building_type: String,
    },
    ResourceGathered {
        entity_id: EntityId,
        resource: String,
        amount: f64,
    },

    // ── Settlement ────────────────────────────────────────────────────────────
    SettlementFounded {
        settlement_id: SettlementId,
    },
    MigrationOccurred {
        entity_id: EntityId,
        from_settlement: SettlementId,
        to_settlement: SettlementId,
    },

    // ── Population ────────────────────────────────────────────────────────────
    BirthOccurred {
        parent_a: EntityId,
        parent_b: EntityId,
        child_id: EntityId,
    },
    FamilyFormed {
        entity_a: EntityId,
        entity_b: EntityId,
    },

    // ── Technology ────────────────────────────────────────────────────────────
    TechDiscovered {
        settlement_id: SettlementId,
        tech_id: String,
    },
    EraAdvanced {
        settlement_id: SettlementId,
        new_era: String,
    },
    Llm(LlmEvent),
}

impl GameEvent {
    /// Returns a static string name for the event (for logging/metrics).
    pub fn name(&self) -> &'static str {
        match self {
            GameEvent::EntitySpawned { .. } => "entity_spawned",
            GameEvent::EntityDied { .. } => "entity_died",
            GameEvent::EntityRemoved { .. } => "entity_removed",
            GameEvent::TickCompleted { .. } => "tick_completed",
            GameEvent::SimulationPaused => "simulation_paused",
            GameEvent::SimulationResumed => "simulation_resumed",
            GameEvent::SpeedChanged { .. } => "speed_changed",
            GameEvent::NeedChanged { .. } => "need_changed",
            GameEvent::NeedCritical { .. } => "need_critical",
            GameEvent::EmotionChanged { .. } => "emotion_changed",
            GameEvent::SteeringParamsUpdated { .. } => "steering_params_updated",
            GameEvent::StressChanged { .. } => "stress_changed",
            GameEvent::MentalBreakTriggered { .. } => "mental_break_triggered",
            GameEvent::TraumaRecorded { .. } => "trauma_recorded",
            GameEvent::RelationshipChanged { .. } => "relationship_changed",
            GameEvent::SocialEventOccurred { .. } => "social_event_occurred",
            GameEvent::JobAssigned { .. } => "job_assigned",
            GameEvent::BuildingConstructed { .. } => "building_constructed",
            GameEvent::ResourceGathered { .. } => "resource_gathered",
            GameEvent::SettlementFounded { .. } => "settlement_founded",
            GameEvent::MigrationOccurred { .. } => "migration_occurred",
            GameEvent::BirthOccurred { .. } => "birth_occurred",
            GameEvent::FamilyFormed { .. } => "family_formed",
            GameEvent::TechDiscovered { .. } => "tech_discovered",
            GameEvent::EraAdvanced { .. } => "era_advanced",
            GameEvent::Llm(event) => match event {
                LlmEvent::RequestSubmitted { .. } => "llm_request_submitted",
                LlmEvent::ResponseReceived { .. } => "llm_response_received",
                LlmEvent::RequestTimedOut { .. } => "llm_request_timed_out",
                LlmEvent::ServerStarted => "llm_server_started",
                LlmEvent::ServerStopped => "llm_server_stopped",
                LlmEvent::ServerHealthCheckFailed => "llm_server_health_check_failed",
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_name_is_stable() {
        let e = GameEvent::TickCompleted { tick: 42 };
        assert_eq!(e.name(), "tick_completed");
    }

    #[test]
    fn speed_changed_name_is_stable() {
        let e = GameEvent::SpeedChanged { speed_index: 3 };
        assert_eq!(e.name(), "speed_changed");
    }

    #[test]
    fn event_is_cloneable() {
        let e = GameEvent::EntitySpawned {
            entity_id: EntityId(1),
        };
        let _ = e.clone();
    }

    #[test]
    fn llm_event_name_is_stable() {
        let e = GameEvent::Llm(LlmEvent::ServerStarted);
        assert_eq!(e.name(), "llm_server_started");
    }
}
