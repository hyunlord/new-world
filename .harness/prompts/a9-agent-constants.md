# A-9: World Rules — Agent Constants를 런타임 시스템에 연결

## Section 1: Implementation Intent

### 문제
`apply_world_rules()`가 agent_constants(mortality_mul, fertility_mul, skill_xp_mul, lifespan_mul, move_speed_mul, body_potential_mul)를 `SimResources`에 저장하지만, **어떤 RuntimeSystem에서도 이 값을 읽지 않음.**

코드 주석: "stored only — consumer integration out of scope"

즉 `eternal_winter.ron`에서 `mortality_mul: 1.3`을 설정해도 **실제로 사망률이 안 바뀜.**
Global constants(hunger_decay_mul 등)는 이미 연결되어 작동하지만, agent constants는 끊겨있음.

### 해결
6개 agent_constants를 해당 RuntimeSystem에서 실제로 읽어서 적용:

| Constant | 소비 시스템 | 적용 방식 |
|----------|-----------|----------|
| mortality_mul | MortalityRuntimeSystem (biology.rs) | hazard × mortality_mul |
| fertility_mul | PopulationRuntimeSystem (biology.rs) | birth chance × fertility_mul |
| skill_xp_mul | IntelligenceRuntimeSystem (cognition.rs) | xp gain × skill_xp_mul |
| lifespan_mul | MortalityRuntimeSystem (biology.rs) | age threshold × lifespan_mul |
| move_speed_mul | steering.rs 또는 movement | speed × move_speed_mul |
| body_potential_mul | PersonalityGeneratorRuntimeSystem | body potential × mul |

### 참조
프로젝트 지식: WorldSim_WorldRules_3자통합분석.md
— "기존 시스템의 수정 레이어 (새 시스템 아니라 초기값 수정)"
— "성능: 초기화 ~50ms, 틱 비용 0"

---

## Section 2: What to Build

### Part A: mortality_mul 연결

**File: `rust/crates/sim-systems/src/runtime/biology.rs`**

MortalityRuntimeSystem에서 hazard 확률에 mortality_mul 적용:

현재 코드 (line ~975):
```rust
let q_check = hazards[5].clamp(0.0, 0.999);
```

수정:
```rust
let mortality_mul = resources.mortality_mul as f32;
let q_check = (hazards[5] * mortality_mul).clamp(0.0, 0.999);
```

### Part B: fertility_mul 연결

**File: `rust/crates/sim-systems/src/runtime/biology.rs`**

PopulationRuntimeSystem에서 birth 확률에 fertility_mul 적용.
`population_birth_block_code()` 또는 birth 로직에서:

현재: birth chance 계산
수정: `birth_chance *= resources.fertility_mul`

### Part C: skill_xp_mul 연결

**File: `rust/crates/sim-systems/src/runtime/cognition.rs`**

IntelligenceRuntimeSystem에서 XP 획득에 skill_xp_mul 적용:

XP 획득 시: `xp_gain *= resources.skill_xp_mul as f32`

### Part D: lifespan_mul 연결

**File: `rust/crates/sim-systems/src/runtime/biology.rs`**

Mortality에서 노화 관련 threshold에 lifespan_mul 적용.
예: 최대 수명 = base_lifespan × lifespan_mul

### Part E: move_speed_mul 연결

**File: `rust/crates/sim-systems/src/runtime/steering.rs`**

이동 속도에 move_speed_mul 적용:
`speed *= resources.move_speed_mul`

### Part F: body_potential_mul 연결

**File: `rust/crates/sim-systems/src/runtime/biology.rs` 또는 `entity_spawner.rs`**

에이전트 생성 시 body potential에 mul 적용.

---

## Section 3: How to Implement

### 수정 순서
1. 각 RuntimeSystem의 `run()` 함수에서 `resources.{constant_mul}`을 읽음
2. 해당 계산에 곱셈 적용
3. harness 테스트로 검증: `eternal_winter.ron` 로드 시 mortality가 실제로 증가하는지

### 핵심 원칙
- **틱 비용 0**: 매 틱마다 f64 곱셈 1회 추가일 뿐. 성능 영향 없음.
- **기존 시스템 구조 변경 없음**: resources에서 값을 읽어 곱하기만.
- **기본값 1.0**: mul이 1.0이면 기존 동작과 동일.

---

## Section 4: Dispatch Plan

| # | Ticket | File | Language | Mode | Depends On |
|---|--------|------|----------|:----:|:----------:|
| T1 | mortality_mul + lifespan_mul | sim-systems/src/runtime/biology.rs | Rust | DISPATCH | — |
| T2 | fertility_mul | sim-systems/src/runtime/biology.rs | Rust | DISPATCH | — |
| T3 | skill_xp_mul | sim-systems/src/runtime/cognition.rs | Rust | DISPATCH | — |
| T4 | move_speed_mul | sim-systems/src/runtime/steering.rs | Rust | DISPATCH | — |
| T5 | body_potential_mul | sim-systems/src/entity_spawner.rs | Rust | DISPATCH | — |
| T6 | harness tests | sim-test/src/main.rs | Rust | DIRECT | T1-T5 |

**Dispatch ratio**: 5/6 = 83%

---

## Section 5: Localization Checklist

No new localization keys.

---

## Section 6: Verification & Harness

### Harness execution
```bash
bash tools/harness/harness_pipeline.sh a9-agent-constants .harness/prompts/a9-agent-constants.md --full
```

### Core assertions

1. **mortality_mul works**: mortality_mul=2.0 loaded -> more deaths than same seed baseline
2. **fertility_mul works**: fertility_mul=0.5 loaded -> fewer births
3. **skill_xp_mul works**: skill_xp_mul=2.0 loaded -> higher skill levels
4. **Default regression**: mortality_mul=1.0 (base_rules) -> identical behavior
5. **Anti-circular**: resources.mortality_mul is actually read in biology.rs (grep check)
6. **eternal_winter scenario**: eternal_winter.ron loaded -> mortality 1.3x + fertility 0.7x confirmed

---

## Section 7: In-game verification

- eternal_winter scenario load shows `[WorldRules] agent constants applied` log
- Winter scenario has higher mortality than base (faster population decline)
- Base scenario behaves identically to before

### Post-implementation report
```
## Implementation Report
### Intent
Connect World Rules agent_constants from "stored only" to actual runtime systems.
### Changes
6 agent_constants read by their respective systems. Tick cost 0.
### Pipeline Results
(table)
```
