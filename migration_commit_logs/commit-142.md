# Commit 142 - 중복된 needs temp-decay API 정리

## 커밋 요약
- `needs_base_decay_step` 통합 이후 중복이 된 `needs_temp_decay_step` 단독 API 경로를 제거하고, needs 시스템은 base decay 결과 또는 GDScript fallback만 사용하도록 정리.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `needs_temp_decay_step` 함수 제거.
  - 관련 테스트(`combined_temp_decay_matches_individual_functions`) 제거.
  - 테스트 import 정리.

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_needs_temp_decay_step` export 함수 제거.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_needs_temp_decay_step(...)` wrapper 제거.

- `scripts/systems/psychology/needs_system.gd`
  - base decay 실패 시 추가로 `body_needs_temp_decay_step`를 재호출하던 분기 제거.
  - 현재 경로:
    - base decay 성공 시 thirst/warmth 결과 재사용
    - 실패 시 기존 GDScript 수식 fallback

## 기능 영향
- thirst/warmth 소모 수식 의미는 유지.
- 중복 bridge API/호출 경로 제거로 코드 복잡도 감소.
- base decay 통합 경로 중심으로 needs 소모 계산 구조가 단순화됨.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 72 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=497.0`, `checksum=13761358.00000`
