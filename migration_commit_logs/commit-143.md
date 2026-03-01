# Commit 143 - needs critical severity 입력 packed scratch 재사용

## 커밋 요약
- needs 임계치 severity 계산 호출도 packed scratch 재사용 패턴으로 전환해 엔티티 루프에서 인자 구성 할당을 줄임.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - `body_needs_critical_severity_step_packed(scalar_inputs)` 추가.
    - packed input(6개 scalar)을 decode해 기존 Rust 수학 함수 호출.
  - 기존 `body_needs_critical_severity_step(...)`는 호환용으로 유지하고 내부에서 packed 경로로 위임.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_needs_critical_severity_step_packed(scalar_inputs)` wrapper 추가.
  - 기존 scalar wrapper는 packed 배열 구성 후 packed wrapper를 호출하도록 변경.

- `scripts/systems/psychology/needs_system.gd`
  - scratch 상수/버퍼 추가:
    - `_CRITICAL_SEVERITY_SCALAR_COUNT = 6`
    - `_critical_severity_scalar_inputs`
  - `execute_tick` 시작 시 severity scratch 배열 `resize` 보장.
  - 욕구 미충족 stressor 구간에서 인덱스 갱신 후 `body_needs_critical_severity_step_packed` 호출.

## 기능 영향
- dehydration/hypothermia/constant_threat severity 수식 의미는 기존과 동일.
- needs tick에서 severity 입력 구성 시 임시 배열/인자 생성 오버헤드 감소.
- 기존 scalar API는 유지되어 호출 호환성 보존.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 72 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=464.8`, `checksum=13761358.00000`
