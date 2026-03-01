# Commit 145 - sim-test needs/body 수학 벤치 추가

## 커밋 요약
- `sim-test`에 `--bench-needs-math` 모드를 추가해 최근 Rust 이관된 body/needs 수학 경로의 성능/회귀를 독립적으로 측정 가능하게 확장.

## 상세 변경
- `rust/crates/sim-test/src/main.rs`
  - CLI 인자 분기에 `--bench-needs-math` 모드 추가.
  - 공용 반복 횟수 파서 `parse_bench_iterations(args, default)`를 도입해 `--iters` 처리 로직을 공통화.
  - `run_needs_math_bench` 함수 추가:
    - 호출 대상: `compute_age_curves`, `age_trainability_modifiers`, `calc_training_gains`, `calc_realized_values`,
      `action_energy_cost`, `rest_energy_recovery`, `thirst_decay`, `warmth_decay`,
      `needs_base_decay_step`, `needs_critical_severity_step`.
    - 결과 합산용 checksum과 `ns_per_iter` 출력 포맷 제공.
  - 기존 `run_stress_math_bench`는 동일 기능 유지, 반복 횟수 파싱만 공용 함수 사용으로 치환.

## 기능 영향
- 기존 headless 테스트/`--bench-stress-math` 동작은 유지.
- body/needs 수학 경로를 별도 벤치로 추적할 수 있어 Rust 마이그레이션 이후 미세 성능 회귀 확인이 용이.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 72 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=482.6`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=199.7`, `checksum=29719684.00000`
