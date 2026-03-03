# Commit 266 - verify 리포트에 bench 요약 흡수

## 커밋 요약
- `migration_verify_report.json`이 `bench_report.json`의 핵심 벤치 설정/체크섬을 직접 포함하도록 `bench_summary`를 추가.

## 상세 변경
- `tools/migration_verify.sh`
  - JSON 파일 bool 추출 헬퍼 추가:
    - `to_json_opt_bool_from_json_file_key(path, key)`
  - verify report에 `bench_summary` 객체 추가:
    - `path_iters`
    - `stress_iters`
    - `needs_iters`
    - `path_backend`
    - `path_split_enabled`
    - `path_smoke_enabled`
    - `path_smoke_expect_has_gpu`
    - `stress_checksum`
    - `needs_checksum`
  - `verification_status.bench_report_present_when_enabled`와 결합해 bench 실행/아티팩트 상태를 단일 리포트에서 판단 가능하게 유지.

## 기능 영향
- `--with-benches` 실행 시 verify report 하나만으로 벤치 파라미터/결과 핵심값을 확인 가능.
- bench 미실행 또는 아티팩트 누락 환경에서는 각 필드가 `null`로 직렬화되어 파서 호환성 유지.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts28 tools/migration_verify.sh --with-benches` 통과.
- `/tmp/worldsim_audit_artifacts28/migration_verify_report.json`에서:
  - `bench_summary.path_iters=100`
  - `bench_summary.stress_checksum=24032652.00000`
  - `bench_summary.needs_checksum=38457848.00000`
  - `verification_status.bench_report_present_when_enabled=true`
  반영 확인.
