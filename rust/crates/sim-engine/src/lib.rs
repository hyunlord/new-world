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
pub mod event_bus;
pub mod event_store;
pub mod explain_log;
pub mod llm_prompt;
pub mod llm_server;
pub mod llm_worker;
pub mod system_trait;
pub mod engine;
pub mod command;
pub mod frame_snapshot;
pub mod notification;
pub mod snapshot;
pub mod perf_tracker;

// ── Convenience re-exports ────────────────────────────────────────────────────

pub use events::{GameEvent, LlmEvent};
pub use event_bus::{EventBus, Subscriber};
pub use event_store::{EventStore, SimEvent, SimEventType};
pub use explain_log::{ExplainEntry, ExplainLog};
pub use llm_prompt::{LlmPromptContext, LlmPromptTemplates, RenderedPrompt};
pub use llm_server::{LlmConfig, LlmRuntime, LlmRuntimeError, LlmStatusSnapshot};
pub use llm_worker::{
    generate_fallback_content, LlmPromptVariant, LlmRequest, LlmRequestMeta, LlmResponse,
};
pub use system_trait::SimSystem;
pub use engine::{ChronicleEvent, RuntimeStatsSnapshot, SimEngine, SimResources};
pub use command::{Command, CommandQueue};
pub use frame_snapshot::{build_agent_snapshots, AgentSnapshot};
pub use notification::{NotificationTier, SimNotification};
pub use snapshot::EngineSnapshot;
pub use perf_tracker::PerfTracker;
