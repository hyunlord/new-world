# Commit 309 - Contagion 핵심 수식 Rust 브리지 이관

## 커밋 요약
- `contagion_system`의 AoE/Network/Spiral 핵심 계산 경로를 Rust-first 호출로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `contagion_aoe_total_susceptibility(...) -> f32`
    - `contagion_stress_delta(...) -> f32`
    - `contagion_network_delta(...) -> f32`
    - `contagion_spiral_increment(...) -> f32`
  - 단위 테스트 추가:
    - `contagion_aoe_total_susceptibility_drops_with_larger_crowd`
    - `contagion_stress_delta_applies_threshold_and_cap`
    - `contagion_network_delta_is_clamped_to_band`
    - `contagion_spiral_increment_respects_conditions`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_contagion_aoe_total_susceptibility(...)`
    - `body_contagion_stress_delta(...)`
    - `body_contagion_network_delta(...)`
    - `body_contagion_spiral_increment(...)`

- `scripts/systems/psychology/contagion_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `_run_aoe_contagion`:
    - 총 감염 민감도 계산을 Rust-first 호출로 전환.
    - stress contagion delta 계산을 Rust-first 호출로 전환.
  - `_run_network_contagion`:
    - network valence delta 계산을 Rust-first 호출로 전환.
  - `_run_spiral_dampener`:
    - spiral increment 계산을 Rust-first 호출로 전환.
  - 브리지 실패 시 기존 GDScript 수식 fallback 유지.
  - 기존 오탈자 수준으로 보이던 spiral warning 블록 들여쓰기 정렬.

## 기능 영향
- 감염 시스템의 반복 계산 경로(민감도/전파량/나선 증폭량)가 Rust 경로로 이동.
- 기존 contagion 이벤트/메타 처리 흐름은 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `42/56` 적용, 잔여 `14/56`.
- **잔여 주요 파일(14)**:
  - `scripts/systems/biology/population_system.gd`
  - `scripts/systems/development/ace_tracker.gd`
  - `scripts/systems/development/attachment_system.gd`
  - `scripts/systems/development/childcare_system.gd`
  - `scripts/systems/development/intergenerational_system.gd`
  - `scripts/systems/development/parenting_system.gd`
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
  - `scripts/systems/psychology/mental_break_system.gd`
  - `scripts/systems/psychology/psychology_coordinator.gd`
  - `scripts/systems/psychology/trait_violation_system.gd`
  - `scripts/systems/psychology/trauma_scar_system.gd`
  - `scripts/systems/record/chronicle_system.gd`
  - `scripts/systems/record/memory_system.gd`
