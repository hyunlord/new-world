# Commit 249 - 아티팩트 JSON 스키마 버전/생성시각 메타 추가

## 커밋 요약
- compile/bench/verify 리포트 JSON에 공통 메타(`schema_version`, `generated_at_utc`)를 추가해 파서 호환성과 타임라인 추적성을 강화.

## 상세 변경
- `tools/localization_compile.py`
  - compile summary report(`--report-json`)에 메타 추가:
    - `schema_version: 1`
    - `generated_at_utc` (UTC ISO-8601, `Z`)
  - Python `datetime/timezone` 기반 UTC 타임스탬프 생성.

- `tools/migration_verify.sh`
  - `bench_report.json` 메타 추가:
    - `schema_version: 1`
    - `generated_at_utc`
  - `migration_verify_report.json` 메타 추가:
    - `schema_version: 1` (기존 `generated_at_utc` 유지)

## 기능 영향
- 리포트 소비 측에서 스키마 변경 대비 버전 분기 처리가 가능해짐.
- 생성 시각 메타로 동일 러너/다중 실행 결과의 시간순 추적과 비교 자동화가 쉬워짐.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts11 MIGRATION_BENCH_PATH_BACKEND_SMOKE=true MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS=5 MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_HAS_GPU=false MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED=cpu MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED=cpu tools/migration_verify.sh --with-benches` 통과.
- 산출물 확인:
  - `/tmp/worldsim_audit_artifacts11/compile_report.json`
  - `/tmp/worldsim_audit_artifacts11/bench_report.json`
  - `/tmp/worldsim_audit_artifacts11/migration_verify_report.json`
  에서 `schema_version`/`generated_at_utc` 필드 존재 확인.
