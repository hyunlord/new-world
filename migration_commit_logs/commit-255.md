# Commit 255 - verify 리포트에 단계별 실행 시간 메트릭 추가

## 커밋 요약
- `migration_verify_report.json`에 전체 실행 시간과 단계별 소요 시간을 추가해 검증 파이프라인의 성능 추이를 함께 추적 가능하게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 스크립트 시작 시각과 단계별 duration 변수 초기화 추가.
  - 각 단계 전후로 epoch timestamp를 측정해 소요 시간(초) 집계:
    - `rust_tests`
    - `data_localization_extract`
    - `localization_compile`
    - `localization_audit`
    - `rust_bench` (`WITH_BENCHES=true`일 때)
  - verify report 확장:
    - `total_duration_seconds`
    - `timings_seconds` 객체

## 기능 영향
- 검증이 성공했더라도 특정 단계만 느려지는 회귀를 아티팩트 기반으로 감시 가능.
- CI에서 실행 시간 기준의 경고/임계치 자동화를 붙이기 쉬워짐.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts17 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts17/migration_verify_report.json`에서:
  - `total_duration_seconds` 존재
  - `timings_seconds.rust_tests/data_localization_extract/localization_compile/localization_audit/rust_bench` 존재
  확인.
