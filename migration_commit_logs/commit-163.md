# Commit 163 - stress 후처리 수식 Rust 이관 (rebound/shaken)

## 커밋 요약
- `stress_system` tick 후처리의 잔여 수식(rebound 반영, shaken 카운트다운)을 Rust 함수로 이관하고, GDScript는 Rust 우선 + fallback 구조로 정리.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `stress_rebound_apply_step(...) -> [next_stress, next_hidden_threat]` 추가.
    - rebound stress 누적 + hidden_threat 감소 수식 이관.
  - `stress_shaken_countdown_step(...) -> [next_remaining, clear_penalty_flag]` 추가.
    - shaken 카운트다운 및 penalty 해제 플래그 계산 이관.
  - unit test 2개 추가:
    - rebound 적용 결과 검증
    - shaken countdown 종료/clear 플래그 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - export 추가:
    - `body_stress_rebound_apply_step(...)`
    - `body_stress_shaken_countdown_step(...)`

- `scripts/core/simulation/sim_bridge.gd`
  - wrapper 추가:
    - `body_stress_rebound_apply_step`
    - `body_stress_shaken_countdown_step`

- `scripts/systems/psychology/stress_system.gd`
  - `_process_rebound_queue(...)`에서 rebound 적용을 Rust 우선 사용.
  - `_update_entity_stress(...)`의 shaken countdown 처리에서 Rust step 우선 사용.
  - bridge 미사용 시 기존 GDScript fallback 유지.

- `rust/crates/sim-test/src/main.rs`
  - `--bench-stress-math`에 호출 추가:
    - `stress_rebound_apply_step`
    - `stress_shaken_countdown_step`
  - checksum 합산 항목 확장.

## 기능 영향
- stress 후처리 수식의 의미(리바운드 반영, hidden 감소, shaken 페널티 해제 시점)는 유지.
- bridge 가능 환경에서 해당 계산이 Rust 경로를 사용.
- bridge 미사용 환경에서도 fallback으로 동일 동작 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 95 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=602.5`, `checksum=20039734.00000` (rebound/shaken 항목 포함으로 기준 업데이트)
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=421.7`, `checksum=38434752.00000`
