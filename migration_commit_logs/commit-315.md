# Commit 315 - Intergenerational 핵심 수식 Rust 브리지 이관

## 커밋 요약
- `intergenerational_system`의 전이율/부하/HPA/Meaney repair 핵심 계산을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `intergen_scar_index(...) -> f32`
    - `intergen_child_epigenetic_step(inputs: &[f32]) -> [f32; 2]`
    - `intergen_hpa_sensitivity(...) -> f32`
    - `intergen_meaney_repair_load(...) -> f32`
  - 단위 테스트 추가:
    - `intergen_scar_index_is_normalized`
    - `intergen_child_epigenetic_step_returns_load_and_t`
    - `intergen_hpa_sensitivity_scales_with_load`
    - `intergen_meaney_repair_load_applies_threshold_and_floor`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_intergen_scar_index(...)`
    - `body_intergen_child_epigenetic_step(...)`
    - `body_intergen_hpa_sensitivity(...)`
    - `body_intergen_meaney_repair_load(...)`

- `scripts/systems/development/intergenerational_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `calculate_child_epigenetic_load`:
    - 입력 스칼라를 `PackedFloat32Array`로 구성해 Rust step 우선 호출.
    - 결과 `[child_load, transmission_rate]`를 반영하고 fallback 유지.
  - `_scar_index`:
    - scar count 정규화를 Rust helper 우선 호출로 전환.
  - `get_hpa_sensitivity`:
    - HPA 민감도 계산을 Rust helper 우선 호출로 전환.
  - `apply_meaney_repair`:
    - repair load 업데이트를 Rust helper 우선 호출로 전환.
  - 브리지 실패 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- 세대 간 전달 핵심 수식 경로가 Rust로 이동하면서 반복 계산 비용을 축소.
- 정착지 수렴/붕괴 이벤트 흐름 및 기존 설정값 의미는 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `48/56` 적용, 잔여 `8/56`.
- **잔여 주요 파일(8)**:
  - `scripts/systems/biology/population_system.gd`
  - `scripts/systems/development/ace_tracker.gd`
  - `scripts/systems/development/childcare_system.gd`
  - `scripts/systems/development/parenting_system.gd`
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
  - `scripts/systems/psychology/psychology_coordinator.gd`
  - `scripts/systems/record/chronicle_system.gd`
