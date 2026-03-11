## Implementation Intent

WS-SYS-001 exists because Food, Danger, Shelter, and Social slices were all already influencing movement, but they were still represented as independent calculations inside the steering runtime instead of an explicit unified steering architecture.

The goal of this ticket was to make influence-driven movement legible and extensible:

- sample all active influence channels into one steering context
- derive need-driven weights in one place
- resolve a single influence steering decision
- apply explicit danger arbitration before the final movement vector is blended into velocity

Weighted blending was chosen because it preserves the existing slice behavior while providing one deterministic composition point for future channels.

## How It Was Implemented

The work was centered in [rust/crates/sim-systems/src/runtime/steering.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/steering.rs).

- Added `InfluenceSteeringSystem` as the unified steering runtime type and kept `SteeringRuntimeSystem` as a compatibility alias.
- Added `SteeringWeights` to derive:
  - hunger weight
  - fear weight
  - cold weight
  - loneliness weight
  - temperament modifiers used by those weights
- Added `SteeringSignalSample` and `AgentSteeringContext` so Food, Warmth, Shelter, Social, and Danger are sampled and stored in one structure before composition.
- Added `InfluenceSteeringDecision` and `resolve_influence_steering(...)` so the final influence force is resolved in one place.
- Added explicit danger arbitration:
  - if local `Danger` signal exceeds `STEERING_DANGER_PRIORITY_SIGNAL_THRESHOLD`
  - and the weighted danger force is meaningful
  - danger avoidance overrides the other influence terms
- Carried the arbitration result into the final movement blend so that, when danger override is active, both desired-action pull and direct action-target blending are suppressed for that tick and the final velocity still resolves away from threat.
- Refactored channel sampling through `weighted_channel_sample_with_sign(...)` so attraction and avoidance use the same typed path.
- Converted the room warmth bias into `room_shelter_sample(...)` so shelter preference is part of the same context/decision flow instead of a one-off extra force.
- Updated [rust/crates/sim-bridge/src/runtime_system.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-bridge/src/runtime_system.rs) and [rust/crates/sim-systems/src/runtime/mod.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-systems/src/runtime/mod.rs) to register/export the unified `InfluenceSteeringSystem`.
- Added one new shared config constant in [rust/crates/sim-core/src/config.rs](/Users/rexxa/github/new-world-wt/codex-ws-sys-001-steering-layer/rust/crates/sim-core/src/config.rs):
  - `STEERING_DANGER_PRIORITY_SIGNAL_THRESHOLD`

The final movement path is still:

1. Behavior picks an action/target
2. Unified steering samples influence channels into `AgentSteeringContext`
3. Unified steering resolves one influence decision with priority arbitration
4. Desired action force, influence decision, separation, cohesion, and direct target blend are combined
5. Velocity is normalized/clamped and consumed by `MovementRuntimeSystem`

## What Feature It Adds

This adds a true unified influence-driven steering layer instead of four loosely accumulated behavior slices.

As a gameplay result:

- food attraction, danger avoidance, shelter seeking, and social gathering now share one steering solver
- danger can explicitly dominate the final movement decision under threat, even when an action target exists
- clustered behavior is more stable because all channel contributions are evaluated through one consistent path
- future channels can plug into the same context/solver architecture without scattering more ad-hoc force code

## Verification After Implementation

- `cd rust && cargo test -p sim-systems steering -- --nocapture` — PASS
- `cd rust && cargo check --workspace` — PASS
- `cd rust && cargo test --workspace` — PASS
- `cd rust && cargo clippy --workspace -- -D warnings` — PASS
- `git diff --check` — PASS

Focused behavioral evidence:

- `danger_overrides_food_when_fear_is_high` verifies danger arbitration beats food attraction
- `danger_override_blocks_direct_target_blend_in_runtime` verifies direct action targeting does not pull an agent back toward danger during an override tick
- `danger_outweighs_social_gathering_when_fear_is_high` verifies danger beats social gathering
- `cold_agent_prefers_room_warmth_more_than_comfortable_agent` verifies shelter bias still responds to cold
- `lonely_agents_cluster_toward_shared_social_anchor` verifies social clustering remains active
- `steering_runtime_system_clamps_velocity_under_combined_influences` verifies the final movement vector stays normalized/clamped

## Remaining Risks

- `BehaviorRuntimeSystem` still owns action selection and direct target assignment, so movement is unified but planning is not.
- Food target choice in cognition remains a separate influence-assisted decision path and was not collapsed into steering.
- The solver still blends desired-action force with influence steering rather than making influence steering the only motion term.
- This ticket does not add a fully generic steering registry or pluggable channel pipeline; it unifies the existing four slices only.

## In-Game Checks (한국어)

- 에이전트가 음식과 위험 사이에서 합리적인 선택을 하는지 본다.
- 위험이 가까울 때는 음식보다 회피 행동이 우선되는지 본다.
- 추울 때는 shelter 방향으로 이동하는지 본다.
- 외로운 에이전트가 군집으로 이동하는지 본다.
- 여러 영향이 동시에 있을 때 행동이 자연스러운지 본다.
- 이동 벡터가 갑자기 튀지 않는지 본다.
- 시뮬레이션이 안정적으로 계속 진행되는지 본다.
