# WorldSim 코드베이스 감사 보고서

기준일: 2026-03-08  
범위: 읽기 전용 감사만 수행. 코드 수정 없음.

## 1. 디렉토리 구조 요약

### Rust 크레이트

| 크레이트 | `src/*.rs` 파일 수 | 상태 요약 |
|---------|:------------------:|-----------|
| `sim-core` | 32 | 컴포넌트/ID/월드/설정 중심. v3.1용 신규 기반 파일은 아직 없음. |
| `sim-data` | 30 | 기존 JSON 로더 + 신규 RON 로더가 공존. |
| `sim-systems` | 23 | 도메인별 런타임 시스템이 이미 큰 규모로 분리되어 있음. |
| `sim-engine` | 16 | 우선순위+인터벌 기반 스케줄링. Hot/Warm/Cold는 아직 없음. |
| `sim-bridge` | 15 | Rust 시스템 등록, Godot 브리지, 런타임 상태 관리. |
| `sim-test` | 1 | 헤드리스 진입점이자 런타임 등록 기준점 역할. |

### GDScript

- 전체 `.gd` 파일: 161
- `scripts/ui/` 전용 UI 파일: 41
- 시뮬 로직 성격 파일(`tick`/`update`/`simulate`/`process` 기반, UI 제외): 69
- `SimBridge` 호출 파일: 74

디렉토리별 분포:

| 디렉토리 | `.gd` 파일 수 |
|---------|:-------------:|
| `scripts/core` | 49 |
| `scripts/systems` | 58 |
| `scripts/ui` | 41 |
| `scripts/debug` | 11 |
| `scripts/ai` | 1 |
| `scripts/rendering` | 1 |

시뮬 로직 성격 파일 분포:

| 디렉토리 | 파일 수 | 판정 |
|---------|:------:|------|
| `scripts/systems` | 38 | 장기적으로 Rust 이관/삭제 대상 |
| `scripts/core` | 21 | 핵심 레거시/쉐도우 계층 |
| `scripts/debug` | 8 | 읽기 전용 디버그로 축소 권장 |
| `scripts/ai` | 1 | Rust 권위 로직과 중복 가능성 높음 |
| `scripts/rendering` | 1 | 렌더 경계만 남기고 시뮬 로직 제거 필요 |

### 데이터 계층

- 루트 `data/` 아래 레거시 JSON 파일: 212
- 신규 RON 파일: 13
- 신규 RON 파일 위치: `rust/crates/sim-data/data/`

### 구조적 결론

- **살릴 것**: Rust 크레이트 분리, `sim-data`의 새 RON 레지스트리, `scripts/ui/` UI 전용 계층
- **고쳐 쓸 것**: 대부분의 `sim-core` 컴포넌트와 `sim-systems` 런타임 시스템
- **버릴 것**: 시뮬레이션 권위 로직을 담고 있는 GDScript 쉐도우/레거시 구현

## 2. sim-core 컴포넌트

관찰 결과:

- `sim-core/src/components`에는 **20개 모듈**, **31개 struct**가 있음
- `31`개 struct 중 `30`개는 `Serialize`/`Deserialize`를 derive
- 예외는 `Values` 하나이며, derive 대신 수동 `Serialize`/`Deserialize` 구현을 사용
- 전용 `Temperament` 컴포넌트는 **없음**
- v3.1 필수 기반 파일도 아직 없음:
  - `influence_grid.rs`: 없음
  - `effect.rs`: 없음
  - `causal_log.rs`: 없음
  - `tile_grid.rs`: 없음
  - `room.rs`: 없음

### 2.1 주요 컴포넌트 판정

| 컴포넌트 | 필드 수 | serde | v3.1 판정 | 비고 |
|---------|:------:|:----:|:---------:|------|
| `Identity` | 15 | derive | 수정 | 성장/선호/음성 관련 자유 문자열이 많고 `Age`와 책임이 겹침 |
| `Personality` | 2 | derive | 수정 | HEXACO 저장소로는 유용하나 TCI/Temperament 전단이 없음 |
| `Body` | 30 | derive | 수정 | 구조는 재사용 가능하지만 `f32` 필드가 남아 있음 |
| `Intelligence` | 4 | derive | 유지 | `f64` 기반의 compact 상태. 큰 구조 충돌 없음 |
| `Needs` | 5 | derive | 유지 | v3.1 필요/에너지 상태 저장소로 재사용 가능 |
| `Emotion` | 2 | derive | 유지 | compact `f64` 배열 기반. 유지 가능 |
| `Values` | 1 | 수동 구현 | 수정 | 직렬화는 되지만 shared component 전부 derive 규칙과 어긋남 |
| `Stress` | 16 | derive | 수정 | `StressTrace` 포함 다수 운영 필드가 아직 `f32` |
| `Traits` | 2 | derive | 유지 | 가벼운 ID 기반 상태. 유지 가능 |
| `Skills` | 1 | derive | 수정 | `HashMap` 대신 결정적 `BTreeMap` 전환 필요 |
| `Social` | 12 | derive | 수정 | `Vec<RelationshipEdge>`는 v3.1의 sparse capped `BTreeMap`과 충돌 |
| `Memory` | 4 | derive | 수정 | `CausalLog`/ring buffer와 책임 정리 필요 |
| `Economic` | 6 | derive | 유지 | `f64` 기반 경향치 저장소로 사용 가능 |
| `Behavior` | 12 | derive | 수정 | `job`/`occupation` 문자열, `action_target_entity`, `f32` 필드가 hot-path 규칙과 충돌 |
| `Age` | 4 | derive | 유지 | authoritative tick 기반 상태로 쓸 만함 |
| `Position` | 5 | derive | 유지 | `f64` 기반 좌표/속도. v3.1과 양립 가능 |
| `SteeringParams` | 17 | derive | 유지 | 조향 파라미터 저장소로 유지 가능하나 기본값은 data화 필요 |
| `LLM state` | 3/4/4/5/3 | derive | 수정/삭제 혼합 | `LlmCapable`/`Pending`/`Result`는 유지 가능, `NarrativeCache`는 sim-core 밖으로 이동 권장 |
| `Coping` | 12 | derive | 수정 | `HashMap` + `f32` 조합으로 결정성/자료형 규칙 위반 |
| `Faith` | 5 | derive | 수정 | Oracle 기억과 전통 ID가 문자열 중심. 구조화 필요 |

### 2.2 helper struct 관찰

- `RelationshipEdge`, `SkillEntry`, `MemoryEntry`, `TraumaScar`, `OracleMemoryEntry`, `CopingRebound` 같은 helper struct는 대부분 재사용 가능
- 다만 아래 문제는 공통적이다:
  - 직접 `EntityId` 참조 다수
  - `String` 기반 ID/이벤트 타입 다수
  - `HashMap`/`Vec` 사용으로 순서 비결정성 존재
  - 일부 `f32` 연산 필드 잔존

### 2.3 핵심 결론

- **Temperament 파이프라인은 sim-core에 아직 없음**
- **Influence Grid / Tile Grid / Room / Causal Log / Effect Primitive 기반은 아직 없음**
- **현재 컴포넌트층은 “완전 폐기”보다 “schema refactor”가 맞다**

## 3. sim-systems

관찰 결과:

- `sim-systems/src/runtime/mod.rs` 기준 **58개 runtime system**이 re-export 됨
- `sim-test/src/main.rs`에서도 동일하게 **58개 system registration**이 확인됨
- 현재 스케줄링 방식은 **priority + fixed tick interval**
- `sim-engine`는 system registry를 priority 순으로 정렬한 뒤, tick마다 interval divisibility로 전체 registry를 검사함
- v3.1 필수인 **Hot/Warm/Cold 분류 메타데이터는 아직 없음**
- `sim-systems` 전반에 `world.query`, entity-to-entity coupling, `config::` 참조가 매우 많음
- `Influence Grid` 기반 상호작용은 아직 보이지 않음

### 3.1 도메인별 요약

| 도메인 | 시스템 수 | 대표 시스템 | 하드코딩 수준 | v3.1 판정 | 비고 |
|--------|:---------:|-------------|:-------------:|:---------:|------|
| `biology` | 9 | `Age`, `Mortality`, `Parenting` | 높음 | 리팩토링 | 생애주기 기반은 살리되 temperament/유전자 파이프라인과 재결합 필요 |
| `cognition` | 3 | `Behavior`, `Memory`, `Intelligence` | 높음 | 리팩토링 | 행동 결정이 직접 ECS/문자열에 강하게 묶여 있음 |
| `economy` | 6 | `Gathering`, `Construction`, `BuildingEffect` | 높음 | 리팩토링 | material/tag recipe/building 2-layer 기준으로 다시 엮어야 함 |
| `llm` | 3 | `LlmRequest`, `LlmResponse`, `LlmTimeout` | 중간 | 리팩토링 | Oracle/해석기 계층으로 경계 재정의 필요 |
| `needs` | 3 | `Needs`, `UpperNeeds`, `ChildStressProcessor` | 중간 | 유지+리팩토링 | core loop는 살릴 수 있으나 tiering과 data화 필요 |
| `psychology` | 10 | `Stress`, `Emotion`, `Coping`, `Trait` | 높음 | 리팩토링 | HEXACO/ACE 기반에서 TCI 파이프라인으로 확장 필요 |
| `record` | 4 | `Chronicle`, `StatsRecorder`, `StatSync` | 중간 | 유지 | 관측/기록 계층으로 재사용 가치 높음 |
| `social` | 11 | `Network`, `Leader`, `Family`, `Value` | 높음 | 리팩토링 | sparse relation/BTreeMap/World Rules 사회 슬롯 반영 필요 |
| `steering` | 2 | `Steering`, `StorySifter` | 중간 | 유지+리팩토링 | 조향은 유지 가능, StorySifter는 Chronicle/Oracle와 경계 정리 필요 |
| `world` | 7 | `Movement`, `Migration`, `Tech*`, `Tension` | 높음 | 리팩토링 | World Rules slot/compile/runtime 모델로 옮겨야 함 |

### 3.2 전체 시스템 인벤토리

틱 인터벌 기준:

- `Hot`: interval `1~3`
- `Warm`: interval `4~30`
- `Cold`: interval `50+` 또는 사실상 event-driven 후보

관측된 시스템 목록:

| 시스템 | 현재 interval | v3.1 판정 | 제안 tier |
|--------|:-------------:|:---------:|:---------:|
| `StatSyncRuntimeSystem` | 10 | 유지 | Warm |
| `ResourceRegenSystem` | 1 | 유지 | Hot |
| `ChildcareRuntimeSystem` | 2 | 리팩토링 | Hot |
| `JobAssignmentRuntimeSystem` | 1 | 리팩토링 | Hot |
| `NeedsRuntimeSystem` | 1 | 유지+리팩토링 | Hot |
| `StatThresholdRuntimeSystem` | 5 | 유지 | Warm |
| `UpperNeedsRuntimeSystem` | 1 | 유지+리팩토링 | Hot |
| `BuildingEffectRuntimeSystem` | 1 | 리팩토링 | Hot |
| `IntelligenceRuntimeSystem` | 50 | 유지 | Cold |
| `MemoryRuntimeSystem` | 1 | 유지+리팩토링 | Hot |
| `BehaviorRuntimeSystem` | 1 | 리팩토링 | Hot |
| `GatheringRuntimeSystem` | 1 | 리팩토링 | Hot |
| `ConstructionRuntimeSystem` | 1 | 리팩토링 | Hot |
| `SteeringRuntimeSystem` | config | 유지+리팩토링 | Hot |
| `MovementRuntimeSystem` | 1 | 유지 | Hot |
| `EmotionRuntimeSystem` | 12 | 유지+리팩토링 | Warm |
| `ChildStressProcessorRuntimeSystem` | 2 | 리팩토링 | Hot |
| `StressRuntimeSystem` | 50 | 리팩토링 | Cold |
| `MentalBreakRuntimeSystem` | 1 | 리팩토링 | Hot |
| `OccupationRuntimeSystem` | 1 | 리팩토링 | Hot |
| `TraumaScarRuntimeSystem` | 10 | 유지+리팩토링 | Warm |
| `TitleRuntimeSystem` | 1 | 리팩토링 | Hot |
| `TraitViolationRuntimeSystem` | 1 | 리팩토링 | Hot |
| `SocialEventRuntimeSystem` | 30 | 리팩토링 | Warm |
| `ContagionRuntimeSystem` | 3 | 리팩토링 | Hot |
| `ReputationRuntimeSystem` | 1 | 유지+리팩토링 | Hot |
| `EconomicTendencyRuntimeSystem` | 1 | 유지+리팩토링 | Hot |
| `MoraleRuntimeSystem` | 5 | 유지+리팩토링 | Warm |
| `JobSatisfactionRuntimeSystem` | 1 | 리팩토링 | Hot |
| `CopingRuntimeSystem` | 30 | 리팩토링 | Warm |
| `IntergenerationalRuntimeSystem` | 240 | 리팩토링 | Cold |
| `ParentingRuntimeSystem` | 240 | 리팩토링 | Cold |
| `AgeRuntimeSystem` | 50 | 유지 | Cold |
| `MortalityRuntimeSystem` | 1 | 유지+리팩토링 | Hot |
| `PopulationRuntimeSystem` | 1 | 유지+리팩토링 | Hot |
| `FamilyRuntimeSystem` | 365 | 유지+리팩토링 | Cold |
| `LeaderRuntimeSystem` | 1 | 리팩토링 | Hot |
| `ValueRuntimeSystem` | 200 | 유지+리팩토링 | Cold |
| `LlmResponseRuntimeSystem` | config | 유지+리팩토링 | Warm |
| `LlmTimeoutRuntimeSystem` | config | 유지+리팩토링 | Warm |
| `NetworkRuntimeSystem` | 1 | 리팩토링 | Hot |
| `MigrationRuntimeSystem` | 1 | 리팩토링 | Hot |
| `TechDiscoveryRuntimeSystem` | 1 | 리팩토링 | Hot |
| `TechPropagationRuntimeSystem` | 1 | 리팩토링 | Hot |
| `TechMaintenanceRuntimeSystem` | 1 | 리팩토링 | Hot |
| `TensionRuntimeSystem` | 1 | 리팩토링 | Hot |
| `TechUtilizationRuntimeSystem` | 1 | 리팩토링 | Hot |
| `StratificationMonitorRuntimeSystem` | 1 | 유지+리팩토링 | Hot |
| `StatsRecorderRuntimeSystem` | 200 | 유지 | Cold |
| `StorySifterRuntimeSystem` | config | 유지+리팩토링 | Cold |
| `LlmRequestRuntimeSystem` | config | 유지+리팩토링 | Warm |
| `SettlementCultureRuntimeSystem` | 100 | 리팩토링 | Cold |
| `PersonalityMaturationRuntimeSystem` | 100 | 리팩토링 | Cold |
| `PersonalityGeneratorRuntimeSystem` | 100 | 리팩토링 | Cold |
| `AttachmentRuntimeSystem` | 100 | 유지+리팩토링 | Cold |
| `AceTrackerRuntimeSystem` | 100 | 유지+리팩토링 | Cold |
| `TraitRuntimeSystem` | 10 | 리팩토링 | Warm |
| `ChronicleRuntimeSystem` | 1 | 유지 | Hot |

### 3.3 하드코딩 결합도가 높은 지점

`config::` 참조 상위 파일:

| 파일 | `config::` 참조 수 |
|------|:------------------:|
| `sim-systems/src/runtime/mod.rs` | 127 |
| `sim-systems/src/runtime/social.rs` | 61 |
| `sim-engine/src/llm_server.rs` | 48 |
| `sim-systems/src/runtime/needs.rs` | 39 |
| `sim-systems/src/runtime/world.rs` | 37 |
| `sim-systems/src/runtime/cognition.rs` | 34 |
| `sim-systems/src/runtime/biology.rs` | 17 |
| `sim-systems/src/runtime/story_sifter.rs` | 16 |
| `sim-systems/src/runtime/steering.rs` | 13 |
| `sim-systems/src/runtime/economy.rs` | 13 |

### 3.4 핵심 결론

- 시스템은 **버릴 것보다 살릴 것이 더 많다**
- 하지만 현재 결합 방식은 v3.1과 정면 충돌한다:
  - 직접 `world.query`
  - 직접 `EntityId` 추적
  - 문자열 기반 분기
  - `config.rs` 단일 상수 저장소 의존
  - 건물/재료/기질/세계규칙의 data-driven 경로 미사용
- 특히 **삭제/대체 후보가 비교적 선명한 시스템**은 아래와 같다:
  - `IntergenerationalRuntimeSystem`
  - `ParentingRuntimeSystem`
  - `PersonalityGeneratorRuntimeSystem`
  - `AttachmentRuntimeSystem`
  - `AceTrackerRuntimeSystem`
  - `BuildingEffectRuntimeSystem`
  - `ChronicleRuntimeSystem`의 현재 legacy pruning 경로

## 4. sim-data 상태

### 4.1 현재 상태

- 크레이트 존재: **Yes**
- A-1 RON 로더 구현 여부: **Yes**
- Hot-reload watcher 구현 여부: **No**
- `notify` 의존성 여부: **No**

### 4.2 신규 RON 데이터 현황

| 범주 | 파일 수 | 정의 수 |
|------|:------:|:------:|
| `materials/` | 5 | 10 |
| `furniture/` | 2 | 5 |
| `recipes/` | 2 | 4 |
| `structures/` | 1 | 1 |
| `actions/` | 1 | 5 |
| `world_rules/` | 1 | 1 |
| `temperament/` | 1 | 1 |

### 4.3 실제 로딩 경로

현재 공존 상태:

- **레거시 경로**: `sim_data::load_all(base_dir)`가 루트 `data/` JSON을 읽음
- **신규 경로**: `DataRegistry::load_from_directory()`가 `rust/crates/sim-data/data/` RON을 읽음

실제 런타임 연결 상태:

- `rust/tests/data_loading_test.rs` -> 아직 `load_all()`
- `rust/crates/sim-test/src/main.rs` -> 아직 `load_all()`
- `rust/crates/sim-bridge/src/lib.rs` -> 아직 `load_all()`
- `rust/crates/sim-data/tests/ron_registry_test.rs` -> `DataRegistry::load_from_directory()`

즉, **A-1은 구현됐지만 아직 런타임 default path가 아니다.**

### 4.4 스키마 수준 구현/미구현

이미 있음:

- `MaterialDef`
- `FurnitureDef`
- `ActionDef`
- `RecipeDef`
- `StructureDef`
- `WorldRuleset`
- `TemperamentRules`

아직 없음:

- World Rules compile/runtime lifecycle
- Temperament runtime derivation
- RON content를 실제 시스템이 소비하는 생산 경로

### 4.5 핵심 결론

- `sim-data`는 v3.1 전환의 **가장 앞서 있는 부분**
- 하지만 현재 상태는 **가산적(additive)** 이다
- 아직은 “JSON 시대 위에 RON이 하나 더 올라간 상태”이며, authoritative runtime path 전환이 필요하다

## 5. GDScript 레거시

### 5.1 수량 요약

- 시뮬 로직 성격 `.gd`: 69
- UI 전용 `.gd`: 41
- `SimBridge` 호출 `.gd`: 74

`SimBridge` 호출 분포:

| 디렉토리 | 파일 수 |
|---------|:------:|
| `scripts/systems` | 52 |
| `scripts/core` | 10 |
| `scripts/ui` | 9 |
| `scripts/debug` | 3 |

### 5.2 판정

| 영역 | 판정 | 이유 |
|------|:----:|------|
| `scripts/ui/` | 유지 | UI/표현 계층으로 남길 가치가 높음 |
| `scripts/debug/` | 유지+정리 | 읽기 전용 inspector/telemetry로 축소 권장 |
| `scripts/core/` | 리팩토링/삭제 | Rust 권위 로직과 중복되는 쉐도우 시뮬 계층이 많음 |
| `scripts/systems/` | 리팩토링/삭제 | 시스템성 로직의 주요 레거시 적재 위치 |
| `scripts/ai/` | 삭제 또는 Rust 이관 | 의사결정 로직은 Rust authoritative path로 이동 필요 |
| `scripts/rendering/` | 유지+정리 | 렌더 디코더만 남기고 시뮬 계산 제거 권장 |

### 5.3 핵심 결론

- 현재 프로젝트는 **Rust-first 문서**와 **혼합 실행 현실** 사이에 있다
- 가장 큰 “버릴 코드”는 Rust와 기능이 겹치는 GDScript 시뮬레이션 레이어다
- UI는 유지하되, 시뮬레이션 권위는 Rust로 수렴시켜야 한다

## 6. config.rs 하드코딩 상수

관찰 결과:

- `rust/crates/sim-core/src/config.rs`의 `pub const`: **501개**
- 파일 주석도 현재 상태를 직접 설명함: `scripts/core/simulation/game_config.gd`의 Rust port
- 즉, 현 구조는 v3.1 data-driven 철학 이전의 **중앙 상수 파일**이다

### 6.1 대표 상수와 이관 대상

| 상수 | 현재 값 | v3.1 이관 대상 | 이관 위치 |
|------|--------:|:--------------:|-----------|
| `WORLD_WIDTH` | 256 | Yes | `world_rules/base_rules.ron` 또는 scenario settings |
| `WORLD_HEIGHT` | 256 | Yes | `world_rules/base_rules.ron` 또는 scenario settings |
| `TICKS_PER_SECOND` | 10 | Yes | Global Constants slot / runtime settings |
| `MAX_ENTITIES` | 500 | Yes | scenario/world bootstrap data |
| `INITIAL_SPAWN_COUNT` | 20 | Yes | scenario/world bootstrap data |
| `MAX_TICKS_PER_FRAME` | 5 | Partial | engine runtime config |
| `TICK_HOURS` | 2 | Yes | Global Constants slot |
| `TICKS_PER_DAY` | 12 | Yes | Global Constants slot |
| `MEMORY_WORKING_MAX` | 100 | Yes | psych/memory data definition |
| `NEEDS_TICK_INTERVAL` | 4 | Yes | Hot/Warm/Cold scheduler metadata |
| `STRESS_SYSTEM_TICK_INTERVAL` | 4 | Yes | Hot/Warm/Cold scheduler metadata |
| `BEHAVIOR_TICK_INTERVAL` | 10 | Yes | Hot/Warm/Cold scheduler metadata |
| `STEERING_NEIGHBOR_RADIUS` | 80.0 | Yes | movement/agent tuning data |
| `STEERING_MAX_FORCE` | 100.0 | Yes | movement/agent tuning data |
| `STEERING_MAX_SPEED` | 120.0 | Yes | movement/agent tuning data |
| `NETWORK_TIE_WEAK_MIN` | 5.0 | Yes | society/social rules data |
| `NETWORK_TIE_STRONG_MIN` | 60.0 | Yes | society/social rules data |
| `REVOLUTION_RISK_THRESHOLD` | 0.70 | Yes | society/global world rules |

### 6.2 결론

- `config.rs` 전체를 한 번에 없애는 것은 비현실적
- 하지만 v3.1 전환의 핵심은:
  1. 월드/시간/사회/이동/경제처럼 **세계가 바뀌면 달라지는 값**을 밖으로 빼고
  2. 엔진 내부 불변 기계 상수만 `config.rs`에 남기는 것

## 7. v3.1 전환을 위한 작업 목록 (우선순위 순)

1. [ ] **A-1 authoritative path 전환**: `sim-bridge`, `sim-test`, 통합 테스트에서 `load_all(JSON)` 대신 `DataRegistry(RON)`를 기본 경로로 연결
2. [ ] **sim-core 기반 추가**: `InfluenceGrid`, `Effect Primitive`, `CausalLog`, `TileGrid`, `Room`, `Temperament`를 새 파일로 추가
3. [ ] **컴포넌트 schema 정리**: `f32 -> f64`, `HashMap/Vec -> BTreeMap` 전환, `NarrativeCache` 같은 비권위 캐시를 sim-core 밖으로 이동
4. [ ] **Behavior/Social refactor**: 직접 `EntityId`/문자열/직접 관계 추적을 줄이고 v3.1 sparse relation + Influence 기반으로 수렴
5. [ ] **건물/economy 재정의**: `BuildingEffect`/`Construction`을 2-layer building + room recognition + GOAP blueprint 기준으로 재설계
6. [ ] **material/tag recipe 연결**: `Gathering`/`Construction`/제작 경로가 `MaterialDef`, `TagRequirement`, 자동 파생 스탯을 실제로 소비하게 연결
7. [ ] **World Rules lifecycle 구현**: Settings -> Compile -> Runtime 구성과 event-driven patch만 허용하는 적용 계층 추가
8. [ ] **Temperament runtime 구현**: 유전자 -> TCI 4축 -> HEXACO bias -> 행동까지 잇는 파이프라인 추가
9. [ ] **스케줄러 tiering 도입**: 현재 interval 기반 registration을 Hot/Warm/Cold 분류와 연결
10. [ ] **GDScript 시뮬 로직 제거 계획 수립**: `scripts/core/`와 `scripts/systems/`의 시뮬레이션 권위 코드를 단계적으로 Rust 쪽으로 정리
11. [ ] **config.rs 분할 이전**: 501개 상수 중 World Rules/Scenario/System tuning으로 옮길 항목부터 우선 분리
12. [ ] **관측/디버그 계층 유지**: `Chronicle`, `StatsRecorder`, `scripts/debug`는 읽기 전용 관측 계층으로 살리고, 권위 로직은 넣지 않기

## 8. 최종 판단

현재 코드베이스는 v2에서 v3.1로 넘어가기 위한 **뼈대는 이미 존재**한다.

- 유지 가치가 큰 것:
  - Rust 크레이트 분할
  - `sim-core`의 주요 상태 컴포넌트
  - `sim-systems`의 도메인별 분리
  - `sim-data`의 신규 RON 레지스트리
  - `scripts/ui` 기반 UI 계층

- 반드시 바꿔야 하는 것:
  - `config.rs` 단일 상수 체계
  - 직접 `world.query`/직접 `EntityId` 결합
  - `f32`, `HashMap`, 자유 문자열 hot-path
  - GDScript 시뮬레이션 쉐도우 레이어

- 사실상 새로 만들어야 하는 것:
  - Influence Grid
  - Effect Primitive
  - Causal Log
  - Tile Grid / Room
  - Temperament runtime
  - World Rules compile/runtime 계층

요약하면, **전면 재작성보다 “핵심 기반 추가 + 기존 시스템 대규모 리팩토링 + GDScript 권위 로직 제거”가 맞는 코드베이스**다.
