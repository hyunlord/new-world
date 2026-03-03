# Commit 037 - Stress state/emotion-meta snapshot Rust 이관

## 커밋 요약
- stress 상태 버킷 계산과 stress→emotion meta 계산을 Rust snapshot 함수로 통합 이관.
- GDScript는 snapshot 1회 호출 결과를 `stress_state`와 meta 필드에 반영.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 타입:
    - `StressStateSnapshot`
  - 신규 함수:
    - `stress_state_snapshot(stress, allostatic)`
  - 포함 계산:
    - `stress_state`(0~3 bucket)
    - `stress_mu_*` 5종
    - `stress_neg_gain_mult`, `stress_pos_gain_mult`
    - `stress_blunt_mult`
  - 단위 테스트 2개 추가:
    - threshold bucket 검증
    - allostatic 증가 시 blunt multiplier 감소 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_state_snapshot(stress, allostatic) -> VarDictionary`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_state_snapshot(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_state_snapshot(...)`
  - Rust 우선 + 기존 GDScript 수식 fallback 제공
- `scripts/systems/psychology/stress_system.gd`
  - `update_entity_stress`에서 snapshot 1회 계산 후 전달
  - `_update_stress_state`/`_apply_stress_to_emotions`를 snapshot 기반으로 변경
  - 사용 종료된 threshold 상수 제거

## 기능 영향
- stress 상태/메타 계산 경로가 통합 네이티브 호출로 정리되어 호출 수와 스크립트 연산량 감소.
- 기존 메타 키(`stress_mu_*`, `stress_*_mult`)와 의미는 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (23 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
