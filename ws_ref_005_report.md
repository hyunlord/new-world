# WS-REF-005 — Influence Grid Runtime Implementation

## Implementation Intent

Influence Grid는 월드 개체들이 직접 서로를 참조하지 않고도 공간을 통해 원인과 결과를 전달하기 위한 핵심 매체다. 이번 구현의 목표는 기존 scaffold 상태였던 `InfluenceGrid`, `TileGrid`, `Room`, `CausalLog`, `Temperament`, `InfluenceEmitter`를 실제 runtime 시스템으로 연결해, 에이전트가 음식, 위험, 온기 신호를 공간적으로 감지하고 반응하도록 만드는 것이었다.

이 구조는 `entity -> field emission -> spatial propagation -> per-agent sampling -> steering decision` 순서를 강제한다. 그 결과, 개체 간 직접 포인터 없이도 음식원 주변으로 모이거나, 위험 지형을 피하거나, 따뜻한 장소를 향해 움직이는 emergent behavior를 만들 수 있다.

## How It Was Implemented

### Channel system

- `sim-core::ChannelId`를 `Food`, `Danger`, `Warmth`, `Social`, `Authority`, `Noise`, `Disease` 중심의 typed channel set으로 확장했다.
- `ChannelMeta`는 채널 이름, decay, 기본 반경, 최대 반경, wall blocking sensitivity, clamp policy를 보유한다.
- `sim-data::WorldRuleset.influence_channels`를 추가해 RON registry에서 채널 메타데이터를 override할 수 있게 했고, `sim-bridge::runtime_init`가 그 값을 읽어 `InfluenceGrid`에 주입한다.

### Emitter runtime

- `InfluenceEmitter` / `EmitterRecord`를 `base_intensity`, `radius`, `falloff`, `decay_rate`, `tags` 구조로 확장했다.
- 새 `InfluenceRuntimeSystem`가 매 tick에 runtime emitter set을 재구성한다.
  - 맵 food tile -> `Food`
  - 위험 terrain/deep water/impassable edge -> `Danger`
  - 완성된 building -> registry-defined influence or safe fallback
  - ECS component `InfluenceEmitter` -> direct spatial emitter
- emitter 수집은 map scan 순서, building id 순서, entity id 순서로 정렬해 deterministic하게 유지했다.

### Propagation algorithm

- `InfluenceGrid`는 double buffer를 유지하며 `current` read / `pending` write / swap 구조로 동작한다.
- 지원 falloff:
  - `Linear`
  - `Exponential`
  - `Gaussian`
  - 기존 scaffold와 호환되는 `InverseSquare`, `Constant`
- 각 tick에서:
  1. 기존 field decay
  2. emitter stamp
  3. per-channel normalization
  4. sigmoid or unit clamp
  5. buffer swap
- sampling은 `sample`, `sample_gradient`, `sample_weighted_sum`으로 제공되고 allocation 없이 상수 시간에 동작한다.

### Wall blocking integration

- `InfluenceRuntimeSystem`가 완료된 shelter를 structural `TileGrid`에 stamp하고, `detect_rooms` / `assign_room_ids`를 통해 room context를 계산한다.
- wall material blocking coefficient를 `WallBlockingMask`로 반영한 뒤, `InfluenceGrid`의 path attenuation이 emitter -> target 경로의 blocking을 누적 적용한다.
- shelter는 문 개구부를 남긴 perimeter wall ring으로 stamp되므로, 문 방향 open path와 벽 반대편 blocked path가 실제로 다른 warmth field를 만든다.

### Agent steering integration

- `SteeringRuntimeSystem`가 `InfluenceGrid` gradient를 직접 샘플링한다.
- hunger / safety-fear / warmth deficit를 need state에서 계산하고, temperament axis를 통해 gradient weight를 조정한다.
- vertical slice behaviors:
  - Hunger -> `Food` gradient attraction
  - Fear / low safety -> `Danger` gradient avoidance
  - Cold / low warmth -> `Warmth` gradient seeking
- 기존 direct action target seek와 blend되므로, 행동 목적지는 유지하되 공간 신호가 steering force를 보정한다.

### Causal logging

- steering이 influence gradient를 실제로 사용해 velocity를 만들면 `CausalLog`에 기록한다.
- 현재 기록되는 cause key:
  - `CAUSE_INFLUENCE_FOOD_GRADIENT`
  - `CAUSE_INFLUENCE_WARMTH_GRADIENT`
  - `CAUSE_INFLUENCE_DANGER_GRADIENT`

## What Feature It Adds

이 구현으로 WorldSim은 공간 자체를 원인 매체로 쓰는 첫 번째 실제 runtime slice를 갖게 됐다.

- 음식이 있는 타일은 `Food` field를 방출한다.
- 위험 지형은 `Danger` field를 만든다.
- campfire와 shelter는 `Warmth` field를 만든다.
- 에이전트는 그 field를 샘플링해서 직접 steering에 반영한다.
- shelter wall과 doorway는 warmth 전파를 실제로 다르게 만든다.
- influence 기반 steering은 `CausalLog`로 기록되므로, 왜 그 방향으로 움직였는지 추적 가능하다.

결과적으로 settlement attraction, food clustering, danger avoidance, warmth seeking 같은 emergent movement pattern의 기반이 생겼다.

## Verification After Implementation

실행 검증:

- `cd rust && cargo check --workspace`
- `cd rust && cargo test --workspace`
- `cd rust && cargo clippy --workspace -- -D warnings`

확인한 핵심 사항:

- InfluenceGrid propagation이 정상적으로 동작한다.
- `sample_gradient`와 `sample_weighted_sum`이 allocation 없이 동작한다.
- campfire + shelter wall scenario에서 inside / doorway / blocked warmth 차이가 난다.
- hungry agent steering test에서 food gradient 쪽으로 힘이 형성된다.
- low-safety agent steering test에서 danger gradient를 회피한다.
- registry-defined influence emission과 ECS `InfluenceEmitter` 둘 다 runtime에 반영된다.
- workspace 전체 build/test/lint가 통과했고 ECS corruption이나 bridge breakage는 발생하지 않았다.

## In-Game Checks (한국어)

- 에이전트가 배고프면 음식이 있는 방향으로 실제로 이동하는지 본다.
- 물가나 위험 지형 근처에서 에이전트가 자동으로 그 방향을 피하는지 본다.
- 화덕 근처에서 `warmth`가 높아지고, 추운 에이전트가 더 따뜻한 쪽으로 기울어지는지 본다.
- shelter 벽이 있는 방향과 문이 열린 방향에서 warmth 전파가 다르게 느껴지는지 본다.
- 실내 쪽은 더 따뜻하고, 벽 반대편 바깥은 warmth가 약해지는지 확인한다.
- 여러 에이전트가 음식 타일 주변으로 자연스럽게 모이는 움직임이 생기는지 본다.
- 디버그 detail/causal view에서 `food_gradient`, `warmth_gradient`, `danger_gradient` 같은 원인 기록이 남는지 확인한다.
- 시뮬레이션이 계속 진행되는 동안 movement jitter나 멈춤 없이 안정적으로 tick되는지 본다.
