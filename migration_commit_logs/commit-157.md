# Commit 157 - needs-math 벤치에 child 수식 커버리지 확장

## 커밋 요약
- `sim-test --bench-needs-math`에 최근 이관한 child stress 수학 함수 호출을 추가해 성능/회귀 추적 범위를 확장.

## 상세 변경
- `rust/crates/sim-test/src/main.rs`
  - `run_needs_math_bench` 루프에 아래 호출 추가:
    - `body::child_parent_stress_transfer(...)`
    - `body::child_simultaneous_ace_step(...)`
  - checksum 합산 항목에 child 계산 결과를 포함.

## 기능 영향
- 런타임 시뮬레이션 로직 영향 없음(벤치 전용).
- needs/body 벤치가 child stress 이관 수식까지 포함해 회귀 감지 범위가 확대.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 83 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=532.2`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=190.7`, `checksum=29781070.00000` (child 수식 포함으로 checksum 기준 업데이트)
