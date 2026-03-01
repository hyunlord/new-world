# Commit 316 - Parenting HPA/Bandura 수식 Rust 브리지 이관

## 커밋 요약
- `parenting_system`의 HPA 보정 스트레스 배수와 Bandura 관찰 base_rate 계산을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `parenting_hpa_adjusted_stress_gain(...) -> f32`
    - `parenting_bandura_base_rate(...) -> f32`
  - 단위 테스트 추가:
    - `parenting_hpa_adjusted_stress_gain_tracks_epigenetic_load`
    - `parenting_bandura_base_rate_applies_maladaptive_multiplier`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_parenting_hpa_adjusted_stress_gain(...)`
    - `body_parenting_bandura_base_rate(...)`

- `scripts/systems/development/parenting_system.gd`
  - SimBridge 캐시/조회 로직(`_get_sim_bridge`) 추가.
  - `_apply_adulthood_transition`:
    - HPA 보정 스트레스 배수를 Rust-first 호출로 전환.
  - `_apply_bandura_modeling`:
    - coping 관찰 base_rate 계산을 Rust-first 호출로 전환.
    - maladaptive 분기 상수 배열(`_MALADAPTIVE_COPING_IDS`) 정리.
  - 브리지 실패 시 기존 GDScript 계산 fallback 유지.

## 기능 영향
- Parenting 경로의 반복 산식(HPA 배수, 관찰 학습률)이 Rust 경로로 이동.
- 성인 전환/메타 업데이트/Bandura familiarity 누적 로직 의미는 기존 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(bridged 대상 56개 기준)**: `49/56` 적용, 잔여 `7/56`.
- **잔여 주요 파일(7)**:
  - `scripts/systems/biology/population_system.gd`
  - `scripts/systems/development/ace_tracker.gd`
  - `scripts/systems/development/childcare_system.gd`
  - `scripts/systems/psychology/coping_system.gd`
  - `scripts/systems/psychology/emotion_system.gd`
  - `scripts/systems/psychology/psychology_coordinator.gd`
  - `scripts/systems/record/chronicle_system.gd`
