## Implementation Intent

Chronicle baseline is the first step toward making emergent simulation explainable instead of merely observable. The influence steering layer already computes a dominant causal reason for movement, but before this ticket that reason only lived ephemerally in runtime behavior and low-level causal logs.

This ticket adds a bounded, significance-filtered chronicle layer so the simulation can answer why an agent moved in a structured way. The scope stayed narrow on purpose: record important influence-driven decisions, store them cheaply, and expose them through the bridge. It does not attempt full narrative text generation or a broad history system.

## How It Was Implemented

The implementation adds a typed chronicle model in `/Users/rexxa/github/new-world-wt/codex-ws-exp-001-chronicle-hook/rust/crates/sim-engine/src/chronicle.rs`. The new types are:

- `ChronicleEventType`
- `ChronicleEventCause`
- `ChronicleEventMagnitude`
- `ChronicleEvent`
- `ChronicleLog`

`ChronicleLog` is a bounded ring-buffer-style store:

- global world events are stored in a `VecDeque`
- per-entity events are stored in a `BTreeMap<EntityId, VecDeque<ChronicleEvent>>`
- capacity is limited by:
  - `CHRONICLE_LOG_MAX_EVENTS`
  - `CHRONICLE_LOG_MAX_PER_ENTITY`

`SimResources` now owns a single `chronicle_log` instead of the older loose world/personal chronicle vectors. This keeps storage centralized and makes pruning behavior consistent.

The significance filter is implemented through two layers:

1. event creation in steering only happens when the computed significance is at least `CHRONICLE_SIGNIFICANCE_THRESHOLD`
2. `ChronicleRuntimeSystem` periodically prunes old low-significance and medium-significance entries through `ChronicleLog::prune_by_significance(...)`

The pruning path was also corrected so negative medium cutoffs are clamped to `0` before conversion to `u64`. Without that clamp, medium-significance historical events could be incorrectly dropped early due to signed-to-unsigned wraparound.

The steering integration lives in `/Users/rexxa/github/new-world-wt/codex-ws-exp-001-chronicle-hook/rust/crates/sim-systems/src/runtime/steering.rs`.

When unified influence steering resolves a dominant influence, it now:

1. writes the existing low-level `CausalEvent` into `CausalLog`
2. derives a typed chronicle event from the dominant influence channel
3. appends the chronicle event only if significance is high enough

Channel-to-event mapping is intentionally explicit:

- `Food` → `InfluenceAttraction`
- `Danger` → `InfluenceAvoidance`
- `Warmth` / shelter bias → `ShelterSeeking`
- `Social` → `GatheringFormation`

Bridge exposure was added in `/Users/rexxa/github/new-world-wt/codex-ws-exp-001-chronicle-hook/rust/crates/sim-bridge/src/lib.rs`. `runtime_get_entity_detail()` now includes:

- `recent_chronicle_events`
- `recent_explains`
- `recent_dominant_cause_key`
- `recent_influence_channel_id`

This keeps the UI side read-only and lets Godot render explanations without owning any simulation logic.

## What Feature It Adds

The simulation can now retain structured explanations for important influence-driven movement instead of only raw state changes.

Concretely, this enables:

- observable movement reasons such as food attraction, danger avoidance, shelter seeking, and gathering
- bounded recent-event history per agent
- future UI/debug surfaces that can show why an entity moved without recomputing the reason
- a direct foundation for later chronicle summarization or narrative systems

In practical terms, WorldSim can now answer questions like:

- this agent moved because food influence dominated
- this agent fled because danger influence won
- this agent gathered because social pull was strongest

## Verification After Implementation

Commands run:

- `cd rust && cargo check --workspace` — PASS
- `cd rust && cargo test --workspace` — PASS
- `cd rust && cargo clippy --workspace -- -D warnings` — PASS
- `cd rust && cargo build -p sim-bridge` — PASS
- `git diff --check` — PASS

Focused behavioral evidence:

- `steering_runtime_system_appends_food_chronicle_event` proves food-driven steering writes a chronicle event
- `steering_runtime_system_records_danger_as_dominant_chronicle_cause` proves danger-driven steering is tagged correctly
- `chronicle_event_for_decision_filters_low_significance_force` proves noise is filtered before storage
- `chronicle_runtime_system_prunes_old_low_importance_events` proves bounded pruning works and now preserves valid medium-significance history correctly
- `chronicle_log_keeps_recent_world_and_personal_entries_bounded` proves ring buffer behavior under capacity pressure

The resulting system is deterministic, bounded, and does not mutate simulation behavior itself. It only records already-resolved influence decisions.

## Remaining Risks

- Chronicle currently records influence-driven movement only. Other important simulation events such as construction milestones, births, deaths, and social relationship transitions are still outside this baseline.
- Bridge exposure currently returns summary keys and typed cause IDs, but downstream GDScript presentation may still need locale-aware rendering polish.
- Significance thresholds are intentionally conservative but still heuristic; they may need tuning once more channels and crowded scenarios are observed in live play.
- Event append still allocates when an event is actually logged because summary/effect keys are stored as `String`. That is acceptable under this ticket’s hot-path rule because no allocation occurs unless an event crosses the significance threshold.

## In-Game Checks (한국어)

- 에이전트 이동 이유가 실제로 기록되는지 본다.
- 음식 때문에 이동한 경우 `Food` 계열 cause가 남는지 확인한다.
- 위험 때문에 도망간 경우 `Danger` cause가 남는지 확인한다.
- 캠프파이어 주변 모임이 `Social` 계열 event로 남는지 확인한다.
- shelter 쪽 이동이 `Warmth` 또는 shelter 관련 cause로 남는지 확인한다.
- Chronicle 로그가 너무 많이 쌓여 화면이나 디버그 확인이 어려워지지 않는지 본다.
- 시간이 지나도 로그가 무한정 증가하지 않고 최근 중요 이벤트 위주로 유지되는지 확인한다.
- 시뮬레이션 성능이 체감상 떨어지지 않는지 본다.
