# Commit 141 - needs 임계치 stress severity 계산 Rust 배치화

## 커밋 요약
- 갈증/체온/안전감 임계치 stress severity 계산을 Rust 배치 함수로 이관하고, needs stressor 구간에서 단일 결과를 재사용하도록 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `critical_severity(value, critical_threshold)` 추가.
  - `needs_critical_severity_step(thirst, warmth, safety, thirst_critical, warmth_critical, safety_critical)` 추가.
    - 반환 순서: `[thirst_severity, warmth_severity, safety_severity]`
  - 테스트 추가:
    - threshold 상회/invalid threshold에서 0 반환
    - 배치 결과가 개별 `critical_severity`와 일치함 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_needs_critical_severity_step(...) -> PackedFloat32Array` export 함수 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_needs_critical_severity_step(...)` wrapper 추가.

- `scripts/systems/psychology/needs_system.gd`
  - 욕구 미충족 stressor 블록 시작 시 severity 배치 계산을 1회 호출.
  - dehydration/hypothermia/constant_threat severity에 배치 결과를 우선 사용.
  - bridge 미지원 시 기존 수식 fallback 유지.

## 기능 영향
- stress severity 수치 의미는 기존과 동일.
- bridge 지원 환경에서 needs stressor severity 계산이 단일 Rust 호출 결과 재사용 구조로 정리.
- bridge 미지원 환경은 기존 GDScript 계산 fallback 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 73 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=450.3`, `checksum=13761358.00000`
