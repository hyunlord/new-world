# Commit 256 - verify 리포트에 실행 설정 스냅샷(config) 추가

## 커밋 요약
- `migration_verify_report.json`에 실행 시 사용된 audit/bench 설정 스냅샷(`config`)을 추가해, 결과 아티팩트만으로 실행 조건을 재현 가능하게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - verify report 생성 구간에 JSON 변환 헬퍼 추가:
    - `to_json_opt_bool_literal`
    - `to_json_opt_int`
    - `to_json_opt_string`
  - `config` 객체 추가:
    - audit:
      - `audit_report_dir`
      - `audit_compare_key_owner_policy`
      - `audit_refresh_key_owner_policy`
    - report targets:
      - `compile_report_json`
      - `bench_report_json`
    - bench:
      - `path_iters`, `stress_iters`, `needs_iters`
      - `path_backend`, `path_split`
      - `path_backend_smoke`, `path_backend_smoke_iters`
      - `path_backend_smoke_expect_has_gpu`
      - `path_backend_smoke_expect_auto`, `path_backend_smoke_expect_gpu`
      - `expected_resolved_backend`
  - `WITH_BENCHES=false` 실행에서도 null-safe 값으로 report 생성되도록 변수 접근을 안전 처리.

## 기능 영향
- report를 읽는 쪽에서 “어떤 옵션 조합으로 만들어진 결과인지”를 별도 로그 없이 즉시 확인 가능.
- 실행 환경/옵션 차이에 따른 checksum 및 아티팩트 변화 분석이 쉬워짐.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts18 MIGRATION_BENCH_PATH_BACKEND_SMOKE=true MIGRATION_BENCH_PATH_BACKEND_SMOKE_ITERS=5 MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_HAS_GPU=false MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_AUTO_RESOLVED=cpu MIGRATION_BENCH_PATH_BACKEND_SMOKE_EXPECT_GPU_RESOLVED=cpu MIGRATION_BENCH_EXPECT_RESOLVED_BACKEND=cpu tools/migration_verify.sh --with-benches` 통과.
- `/tmp/worldsim_audit_artifacts18/migration_verify_report.json`에서 `config` 객체와 bench 설정 필드 반영 확인.
