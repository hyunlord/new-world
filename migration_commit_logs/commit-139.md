# Commit 139 - needs base decay 입력 packed scratch 재사용

## 커밋 요약
- `needs_system`에서 base decay bridge 호출 입력을 엔티티마다 새로 생성하지 않고 scratch packed 배열을 재사용하도록 최적화.

## 상세 변경
- `scripts/core/simulation/sim_bridge.gd`
  - `body_needs_base_decay_step_packed(scalar_inputs, flag_inputs)` 추가.
  - 기존 `body_needs_base_decay_step(...)`는 호환용 wrapper로 유지하고, 내부에서 packed 호출로 위임.

- `scripts/systems/psychology/needs_system.gd`
  - scratch 상수/버퍼 추가:
    - `_BASE_DECAY_SCALAR_COUNT = 13`
    - `_BASE_DECAY_FLAG_COUNT = 2`
    - `_base_decay_scalar_inputs`, `_base_decay_flag_inputs`
  - `execute_tick` 시작 시 scratch 배열 `resize` 보장.
  - 엔티티 루프에서 scalar/flag 인덱스만 갱신한 뒤 `body_needs_base_decay_step_packed` 호출.
  - 기존 엔티티별 임시 배열 생성/append 방식 제거.

## 기능 영향
- base decay 계산 수식/결과는 기존과 동일.
- needs tick에서 base decay bridge 입력 구성 시 메모리 할당/append 오버헤드 감소.
- 기존 공개 wrapper API는 유지되어 호출 호환성 보존.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 71 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=449.2`, `checksum=13761358.00000`
