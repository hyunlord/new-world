pub mod chronicle;
pub mod command;
pub mod engine;
pub mod event_bus;
pub mod event_store;
/// sim-engine: Tick loop, event system, and system registry.
///
/// This crate is the execution heart of the simulation. It wires together:
/// - `SimEngine` — owns the ECS world and drives tick execution
/// - `SimResources` — shared non-component data (calendar, map, settlements, RNG, events)
/// - `SimSystem` — the trait every simulation system implements
/// - `EventBus` — collect-then-drain event dispatcher
/// - `GameEvent` — the full event enum
/// - `CommandQueue` — deferred world mutations (despawn, emit)
/// - `EngineSnapshot` — lightweight diagnostic/save snapshot
///
/// # Typical setup
/// ```ignore
/// use sim_engine::{SimEngine, SimResources, GameEvent, EventBus};
/// use sim_core::{GameCalendar, WorldMap, config::GameConfig};
///
/// let config = GameConfig::default();
/// let resources = SimResources::new(
///     GameCalendar::new(&config),
///     WorldMap::new(256, 256, 42),
///     /* seed */ 42,
/// );
/// let mut engine = SimEngine::new(resources);
/// engine.register(MySystem::new());
/// engine.run_until(4380); // one in-game year
/// ```
pub mod events;
pub mod explain_log;
pub mod frame_snapshot;
pub mod llm_prompt;
pub mod llm_server;
pub mod llm_validator;
pub mod llm_worker;
pub mod notification;
pub mod perf_tracker;
pub mod snapshot;
pub mod system_trait;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use chronicle::{
    ChronicleCapsule, ChronicleCluster, ChronicleDossierStub, ChronicleEntryDetailSnapshot,
    ChronicleEntityRefState, ChronicleEntryId, ChronicleEntryLite, ChronicleEntryStatus,
    ChronicleEvent, ChronicleEventCause, ChronicleEventMagnitude, ChronicleEventType,
    ChronicleFeedItemSnapshot, ChronicleFeedRenderHint, ChronicleFeedResponse, ChronicleHeadline,
    ChronicleHistorySliceResponse, ChronicleLocationRefLite, ChronicleLog, ChronicleQueueBucket,
    ChronicleQueueKind, ChronicleQueueTransition, ChronicleRecallItemSnapshot,
    ChronicleRecallSliceResponse, ChronicleRouteResult, ChronicleSignificanceCategory,
    ChronicleSignificanceMeta, ChronicleSnapshotRevision, ChronicleSubjectRefLite,
    ChronicleTelemetry,
    ChronicleSummary, ChronicleThreadListResponse, ChronicleThreadSnapshot, ChronicleTimeline,
};
pub use command::{Command, CommandQueue};
pub use engine::{
    AgentNeedDiagnostics, ConstructionDiagnostics, DiagnosticDelta, RuntimeStatsSnapshot,
    SimEngine, SimResources,
};
pub use event_bus::{EventBus, Subscriber};
pub use event_store::{EventStore, SimEvent, SimEventType};
pub use events::{GameEvent, LlmEvent};
pub use explain_log::{ExplainEntry, ExplainLog};
pub use frame_snapshot::{build_agent_multimesh_buffer, build_agent_snapshots, AgentSnapshot};
pub use llm_prompt::{
    ActionOption, HexacoDescriptors, LlmPromptContext, LlmPromptTemplates, PromptPayload,
    SpeechRegister,
};
pub use llm_server::{LlmConfig, LlmRuntime, LlmRuntimeError, LlmStatusSnapshot};
pub use llm_validator::{
    load_forbidden_word_list, validate_korean_output, ForbiddenWord, ForbiddenWordList,
    ValidationResult, Violation,
};
pub use llm_worker::{
    generate_fallback_content, LlmPromptVariant, LlmRequest, LlmRequestMeta, LlmResponse,
};
pub use notification::{NotificationTier, SimNotification};
pub use perf_tracker::PerfTracker;
pub use snapshot::EngineSnapshot;
pub use system_trait::SimSystem;

#[cfg(test)]
mod tests {
    use super::load_forbidden_word_list;

    #[test]
    fn llm_validator_exports_project_word_table() {
        assert!(!load_forbidden_word_list().forbidden.is_empty());
    }
}
