# Commit 200 - migration_verify 벤치 반복 수 환경변수 지원

## 커밋 요약
- `tools/migration_verify.sh --with-benches`가 벤치 반복 수를 환경변수로 조정할 수 있게 확장하고, 기본/비기본 반복 수에 따라 checksum 검증/관측 모드를 자동 분기하도록 개선.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 환경변수:
    - `MIGRATION_BENCH_PATH_ITERS` (기본 `100`)
    - `MIGRATION_BENCH_STRESS_ITERS` (기본 `10000`)
    - `MIGRATION_BENCH_NEEDS_ITERS` (기본 `10000`)
  - 값 유효성 검사 추가(양의 정수만 허용).
  - 벤치 실행 전 현재 반복 수 구성 출력 추가.
  - 비기본 반복 수에서는 strict checksum 비교 대신 `run_bench_observe`로 checksum 관측값만 출력하도록 분기.
  - 기본 반복 수에서는 기존과 동일하게 checksum 기준선 검증 유지.

## 기능 영향
- 기본 설정에서는 기존 회귀 게이트 의미를 그대로 유지.
- CI/로컬 상황에 맞게 반복 수를 조정하면서도 벤치 결과를 안정적으로 수집 가능.

## 검증
- `tools/migration_verify.sh --with-benches` 통과.
  - 기본 반복 수에서 기존 checksum 기준선 검증 통과.
- `MIGRATION_BENCH_PATH_ITERS=10 MIGRATION_BENCH_STRESS_ITERS=100 MIGRATION_BENCH_NEEDS_ITERS=100 tools/migration_verify.sh --with-benches` 통과.
  - 비기본 반복 수에서 checksum 관측 모드로 정상 출력 확인.
