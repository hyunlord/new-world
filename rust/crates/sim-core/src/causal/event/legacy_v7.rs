//! Frozen pre-format9 causal event wire; every historical event is retained.
use super::*;

/// Frozen historical event variant order and payloads.
#[derive(serde::Deserialize)]
pub enum LegacyCausalEvent {
    /// A building placement event arrived from FFI and was accepted by BSS
    /// (in-bounds, non-zero radius). Recorded once per accepted event onto
    /// the centre tile so the "왜?" UI can attribute downstream stamps.
    ///
    /// Chain root: `parent == None`.
    BuildingPlaced {
        /// This event's unique id (P3β-1).
        id: EventId,
        /// Parent event id — always `None` for a root building placement.
        parent: Option<EventId>,
        /// Building origin tile (top-left, in tiles).
        position: (u32, u32),
        /// Chebyshev influence radius in tiles (inclusive).
        radius: u32,
        /// Simulation tick at which BSS observed the event.
        tick: u64,
    },

    /// BSS marked a dirty region on `channel` after processing a building
    /// event. Recorded onto the region centre once per channel per event.
    ///
    /// Chain link: `parent = Some(<id of the originating BuildingPlaced>)`.
    StampDirty {
        /// This event's unique id (P3β-1).
        id: EventId,
        /// Parent event id — the `BuildingPlaced` that triggered this stamp.
        parent: Option<EventId>,
        /// Influence channel that received the dirty mark.
        channel: InfluenceChannel,
        /// Region drained by IUS in the same tick.
        region: DirtyRegion,
        /// Simulation tick at which the dirty mark was emitted.
        tick: u64,
    },

    /// IUS observed a non-empty dirty region for `channel` and propagated
    /// influence from `position` (region centre). `old`/`new` capture the
    /// current vs pending intensity at the centre after the propagation
    /// primitive ran — minimal sample per V7 Phase 3-α (the full per-cell
    /// delta would explode the ring; the centre sample suffices for the
    /// "왜?" UI which traces source ⇒ destination, not every tile in
    /// between).
    ///
    /// Chain link: `parent = Some(<id of the most recent same-channel
    /// StampDirty on this tile>)`. The lookup may return `None` if the
    /// matching `StampDirty` was already evicted from the centre tile's
    /// ring buffer — in that case the chain terminates here, which the
    /// "왜?" UI reports as "cause evicted".
    InfluenceChanged {
        /// This event's unique id (P3β-1).
        id: EventId,
        /// Parent event id — the matching-channel `StampDirty`, or `None`
        /// when that stamp has already been evicted from the ring.
        parent: Option<EventId>,
        /// Influence channel that propagated.
        channel: InfluenceChannel,
        /// Region centre tile (same sample IUS uses for the propagation
        /// primitive's source).
        position: (u32, u32),
        /// Intensity at `position` BEFORE this tick's propagation (pulled
        /// from `current[channel]`).
        old: f32,
        /// Intensity at `position` AFTER this tick's propagation (pulled
        /// from `pending[channel]`, before the IUS swap).
        new: f32,
        /// Simulation tick at which IUS recorded the change.
        tick: u64,
    },

    /// First agent-originated event (V7 Phase 5-β / P5β-3).
    ///
    /// Emitted by `AgentDecisionSystem` (priority 125) the moment an
    /// agent's [`Hunger`]/[`Thirst`] crosses its threshold and the
    /// agent transitions `Idle → Seeking { target }`. The variant is
    /// recorded on the agent's current tile so the "왜?" UI can surface
    /// "이 agent가 왜 이 방향으로 움직였나?" right where the agent stood
    /// when the decision was taken.
    ///
    /// Chain semantics: `AgentDecision { parent: Some(<recent same-tile
    /// InfluenceChanged>) }` when an influence event on the agent's tile
    /// preceded the breach (allowing the chain `BuildingPlaced →
    /// StampDirty → InfluenceChanged → AgentDecision` to close); `None`
    /// when no such causal antecedent exists.
    ///
    /// [`Hunger`]: crate::components::Hunger
    /// [`Thirst`]: crate::components::Thirst
    AgentDecision {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — most recent same-tile `InfluenceChanged`,
        /// or `None` for need-only breaches with no influence trigger.
        parent: Option<EventId>,
        /// The deciding agent's `AgentId` (P5α-2 surface).
        agent: AgentId,
        /// Agent tile at decision time.
        position: (u32, u32),
        /// Why the FSM transitioned out of `Idle`.
        reason: DecisionReason,
        /// Simulation tick at which the decision was made.
        tick: u64,
    },

    /// Agent transitioned from `Seeking { ConstructionSite }` to
    /// `Consuming { ConstructionSite }` (V7 Phase 6-β / P6β-3). Emitted by
    /// `AgentDecisionSystem` the moment an agent reaches an active
    /// construction site. `parent` links to the originating
    /// `AgentDecision { reason: ConstructionReason, agent: <this agent> }`
    /// on the same tile.
    ///
    /// `position` follows the `(u32, u32)` tuple precedent shared with
    /// `BuildingPlaced` and `AgentDecision`. `blueprint` snapshots the
    /// site's blueprint at start time (the site entity may be despawned
    /// before this record is consumed).
    ConstructionStarted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the originating `AgentDecision { ConstructionReason }`.
        parent: Option<EventId>,
        /// Immutable design spec of the building being constructed.
        blueprint: crate::components::BuildingBlueprint,
        /// Construction tile (matches `ConstructionSite.position`).
        position: (u32, u32),
        /// Simulation tick at which the transition occurred.
        tick: u64,
    },

    /// Construction progress reached `required_progress` on this tick
    /// (V7 Phase 6-β / P6β-4). Emitted by `ConstructionSystem` BEFORE the
    /// closing `BuildingPlaced` so the parent chain
    /// `BuildingPlaced → ConstructionCompleted → ConstructionStarted →
    /// AgentDecision { ConstructionReason }` walks correctly.
    ///
    /// The site entity is despawned in the same tick — `position` is the
    /// durable identifier (the recycled hecs handle would be misleading
    /// after despawn), and the embedded `blueprint` mirrors
    /// `ConstructionStarted` exactly.
    ConstructionCompleted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the originating `ConstructionStarted`.
        parent: Option<EventId>,
        /// Immutable design spec of the completed building.
        blueprint: crate::components::BuildingBlueprint,
        /// Construction tile (matches `ConstructionSite.position`).
        position: (u32, u32),
        /// Simulation tick at which completion fired.
        tick: u64,
    },

    /// Mutual social handshake closed: both agents transitioned from
    /// Seeking{Agent(other)} to Consuming{Agent(other)} on the same tile.
    /// Emitted ONCE per handshake by `AgentDecisionSystem` (deduplicated by
    /// emitting only from the smaller-AgentId agent's transition).
    /// `parent` links to the same-tile `AgentDecision{SocialReason,
    /// agent: emitter}`. V7 Phase 7-β / P7β-3.
    SocialInteractionStarted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the smaller-AgentId agent's `AgentDecision{
        /// SocialReason}`.
        parent: Option<EventId>,
        /// Canonical pair `(smaller AgentId, larger AgentId)`.
        agents: (AgentId, AgentId),
        /// Shared tile coordinate at handshake time.
        position: (u32, u32),
        /// Simulation tick at which the handshake closed.
        tick: u64,
    },

    /// Mutual social interaction reached `REQUIRED_INTERACTION_PROGRESS`.
    /// Emitted by `SocialInteractionSystem` before the agent state
    /// transitions back to Idle. `familiarity_after` snapshots the
    /// post-bump `RelationshipState::familiarity` value (saturating at
    /// 1.0). `parent` links to the originating `SocialInteractionStarted`.
    /// V7 Phase 7-β / P7β-4.
    SocialInteractionCompleted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the originating `SocialInteractionStarted`.
        parent: Option<EventId>,
        /// Canonical pair `(smaller AgentId, larger AgentId)`.
        agents: (AgentId, AgentId),
        /// Shared tile coordinate at completion.
        position: (u32, u32),
        /// Familiarity scalar after `FAMILIARITY_BUMP` applied (clamped
        /// to `[0.0, 1.0]`).
        familiarity_after: f64,
        /// Simulation tick at which completion fired.
        tick: u64,
    },

    /// V7 Phase 8-β / P8Plan-6. Emitted by `AgentDecisionSystem` when a
    /// cascade-bias memory weight delta flips the cascade's natural
    /// winner. The `recalled_event` field references the top-contributor
    /// memory entry's `event_id`; `parent` carries the recalled event's
    /// own parent so the lineage walk continues through the recall.
    MemoryRecalled {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — typically the recalled event's parent, or
        /// `None` when the recalled event has no parent or has been
        /// evicted from the ring buffer.
        parent: Option<EventId>,
        /// The agent whose cascade was flipped.
        agent: AgentId,
        /// `event_id` of the top-contributor [`MemoryEntry`] driving the
        /// flip.
        ///
        /// [`MemoryEntry`]: crate::components::MemoryEntry
        recalled_event: EventId,
        /// Why the recall fired. Phase 8-β only emits `CascadeBias`.
        triggered_by: MemoryRecallTrigger,
        /// Simulation tick at which the recall was emitted.
        tick: u64,
    },

    /// Agent transitioned to `Consuming { Agent(defender) }` via the
    /// combat cascade arm. Emitted ONCE per pair by `AgentDecisionSystem`
    /// from the smaller-`AgentId` agent's evaluation (deduplication,
    /// mirrors `SocialInteractionStarted` pattern).
    /// `parent` links to the emitting agent's `AgentDecision{CombatReason}`.
    /// V7 Phase 9-β / P9β-1.
    CombatStarted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the attacker's `AgentDecision{CombatReason}`.
        parent: Option<EventId>,
        /// Agent initiating combat (the one whose cascade triggered).
        attacker: AgentId,
        /// Agent being attacked.
        defender: AgentId,
        /// Shared tile coordinate at combat start.
        position: (u32, u32),
        /// Simulation tick at which combat started.
        tick: u64,
    },

    /// `CombatSystem` applied `DAMAGE_PER_COMBAT_TICK` to the defender.
    /// `parent` links to the originating `CombatStarted`. `hp_after`
    /// snapshots the defender's HP after damage (saturates at 0.0).
    /// NOTE: field is `hp_after: f64`, NOT `defender_died: bool` — P9β-3.
    /// V7 Phase 9-β / P9β-1.
    CombatCompleted {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the originating `CombatStarted`.
        parent: Option<EventId>,
        /// Agent that initiated combat.
        attacker: AgentId,
        /// Agent that received damage.
        defender: AgentId,
        /// Shared tile coordinate at completion.
        position: (u32, u32),
        /// Defender HP after `apply_damage(DAMAGE_PER_COMBAT_TICK)`
        /// (saturated at 0.0). P9β-3: this is NOT `defender_died: bool`.
        hp_after: f64,
        /// Settlement link tag (V7 Phase 10-γ / P10γ-A13). `Some(sid)`
        /// when either `attacker` or `defender` is a member of settlement
        /// `sid` at emission time; `None` otherwise. Populated by
        /// [`CombatSystem`] (priority 137) before [`SettlementSystem`]
        /// (priority 138) runs in the same tick. Used by
        /// [`SettlementSystem`] step 6 to gate community_history routing:
        /// only events whose `settlement_link == Some(sid)` are routed to
        /// `sid`'s history. Member-involved combat WITHOUT this tag (e.g.
        /// hand-constructed test events) is NOT routed, distinguishing
        /// the explicit-link routing predicate from a pure membership
        /// check.
        settlement_link: Option<SettlementId>,
        /// Simulation tick at which damage was applied.
        tick: u64,
    },

    /// Agent was born into the simulation (V7 Phase 10-α / P10Plan-5).
    ///
    /// Emitted by `SettlementSystem` (Phase 10-β) when population growth
    /// conditions are met. Replaces the absent `AgentSpawned` event —
    /// only `spawn_agent()` engine method existed prior to Phase 10.
    ///
    /// `parent` links to the `SettlementFormed` event of the birth settlement
    /// when available; `None` for the initial world population seeding.
    ///
    /// Chain note: `AgentBorn` is a causal root for the agent's lifetime —
    /// subsequent `AgentDecision`, `SocialInteractionStarted`, and
    /// `CombatStarted` events chain from this birth event via the agent's
    /// decision cascade.
    AgentBorn {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — `SettlementFormed` id when the birth occurs
        /// inside an established settlement; `None` for world-seed spawns.
        parent: Option<EventId>,
        /// The newly spawned agent's `AgentId`.
        agent: AgentId,
        /// Parent agents in lineage/inheritance order. Family-enabled births
        /// use mother then father; otherwise the two lowest-id distinct
        /// settlement members remain the fallback. Empty for world-seed spawns.
        parent_ids: Vec<AgentId>,
        /// Simulation tick at which the agent was born.
        tick: u64,
    },

    /// Automatic cluster detection formed a new settlement (V7 Phase 10-β /
    /// P10Plan-2). Emitted by `SettlementSystem` (priority 138) when the
    /// agent + building density within `SETTLEMENT_PROXIMITY_RADIUS`
    /// crosses both formation thresholds.
    ///
    /// Chain root: `parent == None` (auto-formation is causally a root).
    SettlementFormed {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — always `None` for auto-formation root.
        parent: Option<EventId>,
        /// The newly formed settlement's stable id.
        settlement_id: SettlementId,
        /// All agents within `SETTLEMENT_PROXIMITY_RADIUS` of the formation
        /// tile at the formation tick.
        founding_members: Vec<AgentId>,
        /// Simulation tick at which formation was detected.
        tick: u64,
    },

    /// Settlement dissolved — population fell to zero and no buildings
    /// remain (V7 Phase 10-β / P10Plan-3). Emitted by `SettlementSystem`
    /// when both `population_stats.current == 0` AND
    /// `member_buildings.is_empty()` hold for the same tick.
    SettlementDissolved {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — the originating `SettlementFormed` event id,
        /// or `None` if the chain root has been evicted from the ring.
        parent: Option<EventId>,
        /// The dissolved settlement's stable id.
        settlement_id: SettlementId,
        /// Population at dissolution time (always 0 by precondition).
        final_population: u32,
        /// Typed dissolution cause (V7 Phase 10-γ / P10γ-A17). At present
        /// the only canonical cause is
        /// [`DissolutionCause::EmptyMembersAndBuildings`].
        cause: DissolutionCause,
        /// `AgentId` of the LAST member observed inside the settlement
        /// prior to dissolution (the lowest-id member captured by the
        /// pre-clearing snapshot). `None` if the settlement never held a
        /// member after formation.
        last_member_id: Option<AgentId>,
        /// `BuildingId` of the LAST building observed inside the settlement
        /// prior to dissolution (the lowest-id building captured by the
        /// pre-clearing snapshot). `None` if the settlement never held a
        /// building after formation.
        last_building_id: Option<BuildingId>,
        /// Simulation tick at which dissolution was detected.
        tick: u64,
    },

    /// An agent died (V7 feature `add-starvation-death`). Emitted by the
    /// shared `survival::despawn_agent` helper — invoked by both
    /// `StarvationSystem` (needs death) and `CombatSystem` (combat death) —
    /// at the dead agent's tile so the chronicle can answer "왜 죽었나?".
    ///
    /// Mirrors the `AgentBorn` field shape. A causal leaf for the agent's
    /// lifetime: `parent == None` (no upstream cause is currently linked).
    AgentDied {
        /// This event's unique id.
        id: EventId,
        /// Parent event id — always `None` (death is a recorded leaf).
        parent: Option<EventId>,
        /// The dead agent's `AgentId`.
        agent: AgentId,
        /// Tile at which the agent died (its last `Position`).
        position: (u32, u32),
        /// Typed cause of death.
        reason: DeathReason,
        /// Simulation tick at which the death occurred.
        tick: u64,
    },

    /// M5 (ADR 0027): a pressure touched a person — "why?" names it. Emitted
    /// at the person's tile, at most once per person per pressure and effect
    /// (the first cold hour of a snap, the first wolf bite, the onset of an
    /// illness). A causal leaf (`parent == None`).
    PressureCause {
        /// This event's unique id.
        id: EventId,
        /// Always `None`.
        parent: Option<EventId>,
        /// The person.
        agent: AgentId,
        /// The person's tile.
        position: (u32, u32),
        /// The pressure's `PressureKind::code` (0 cold snap, 1 drought, 2 beasts, 3 illness).
        kind: u8,
        /// What it did: 0 cold, 1 a wound, 2 sickness, 3 thirst or empty forage.
        effect: u8,
        /// Simulation tick.
        tick: u64,
    },
}

impl From<LegacyCausalEvent> for CausalEvent {
    fn from(old: LegacyCausalEvent) -> Self {
        match old {
            LegacyCausalEvent::BuildingPlaced {
                id,
                parent,
                position,
                radius,
                tick,
            } => Self::BuildingPlaced {
                id,
                parent,
                position,
                radius,
                tick,
            },
            LegacyCausalEvent::StampDirty {
                id,
                parent,
                channel,
                region,
                tick,
            } => Self::StampDirty {
                id,
                parent,
                channel,
                region,
                tick,
            },
            LegacyCausalEvent::InfluenceChanged {
                id,
                parent,
                channel,
                position,
                old,
                new,
                tick,
            } => Self::InfluenceChanged {
                id,
                parent,
                channel,
                position,
                old,
                new,
                tick,
            },
            LegacyCausalEvent::AgentDecision {
                id,
                parent,
                agent,
                position,
                reason,
                tick,
            } => Self::AgentDecision {
                id,
                parent,
                agent,
                position,
                reason,
                tick,
            },
            LegacyCausalEvent::ConstructionStarted {
                id,
                parent,
                blueprint,
                position,
                tick,
            } => Self::ConstructionStarted {
                id,
                parent,
                blueprint,
                position,
                tick,
            },
            LegacyCausalEvent::ConstructionCompleted {
                id,
                parent,
                blueprint,
                position,
                tick,
            } => Self::ConstructionCompleted {
                id,
                parent,
                blueprint,
                position,
                tick,
            },
            LegacyCausalEvent::SocialInteractionStarted {
                id,
                parent,
                agents,
                position,
                tick,
            } => Self::SocialInteractionStarted {
                id,
                parent,
                agents,
                position,
                tick,
            },
            LegacyCausalEvent::SocialInteractionCompleted {
                id,
                parent,
                agents,
                position,
                familiarity_after,
                tick,
            } => Self::SocialInteractionCompleted {
                id,
                parent,
                agents,
                position,
                familiarity_after,
                tick,
            },
            LegacyCausalEvent::MemoryRecalled {
                id,
                parent,
                agent,
                recalled_event,
                triggered_by,
                tick,
            } => Self::MemoryRecalled {
                id,
                parent,
                agent,
                recalled_event,
                triggered_by,
                tick,
            },
            LegacyCausalEvent::CombatStarted {
                id,
                parent,
                attacker,
                defender,
                position,
                tick,
            } => Self::CombatStarted {
                id,
                parent,
                attacker,
                defender,
                position,
                tick,
            },
            LegacyCausalEvent::CombatCompleted {
                id,
                parent,
                attacker,
                defender,
                position,
                hp_after,
                settlement_link,
                tick,
            } => Self::CombatCompleted {
                id,
                parent,
                attacker,
                defender,
                position,
                hp_after,
                settlement_link,
                tick,
            },
            LegacyCausalEvent::AgentBorn {
                id,
                parent,
                agent,
                parent_ids,
                tick,
            } => Self::AgentBorn {
                id,
                parent,
                agent,
                parent_ids,
                tick,
            },
            LegacyCausalEvent::SettlementFormed {
                id,
                parent,
                settlement_id,
                founding_members,
                tick,
            } => Self::SettlementFormed {
                id,
                parent,
                settlement_id,
                founding_members,
                tick,
            },
            LegacyCausalEvent::SettlementDissolved {
                id,
                parent,
                settlement_id,
                final_population,
                cause,
                last_member_id,
                last_building_id,
                tick,
            } => Self::SettlementDissolved {
                id,
                parent,
                settlement_id,
                final_population,
                cause,
                last_member_id,
                last_building_id,
                tick,
            },
            LegacyCausalEvent::AgentDied {
                id,
                parent,
                agent,
                position,
                reason,
                tick,
            } => Self::AgentDied {
                id,
                parent,
                agent,
                position,
                reason,
                tick,
            },
            LegacyCausalEvent::PressureCause {
                id,
                parent,
                agent,
                position,
                kind,
                effect,
                tick,
            } => Self::PressureCause {
                id,
                parent,
                agent,
                position,
                kind: u32::from(kind),
                effect,
                tick,
            },
        }
    }
}
