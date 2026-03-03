# Commit 268 - verify 리포트에 compile 임계치 게이트 추가

## 커밋 요약
- `migration_verify_report.json`에 compile 임계치(`compile_thresholds`)와 임계치 충족 상태(`compile_threshold_status`)를 추가.

## 상세 변경
- `tools/migration_verify.sh`
  - 중첩 JSON 경로 추출 헬퍼 추가:
    - `to_json_opt_int_from_json_file_path(path, key_path)`
    - `compile_report.json`의 `thresholds.*` 값을 안전 추출
  - 정수 비교/집계 헬퍼 추가:
    - `to_json_leq_ints(lhs, rhs)` (`lhs <= rhs`)
    - `to_json_bool_and(...)` (여러 bool의 AND 집계)
  - verify report 확장:
    - `compile_thresholds`
      - `max_duplicate_key_count`
      - `max_duplicate_conflict_count`
      - `max_missing_key_fill_count`
      - `max_owner_rule_miss_count`
      - `max_duplicate_owner_missing_count`
      - `max_owner_unused_count`
    - `compile_threshold_status`
      - 각 임계치별 `*_ok` + `all_ok`
    - `verification_status.compile_thresholds_all_ok` 추가

## 기능 영향
- compile 품질 임계치가 report에 명시적으로 포함되어 운영/CI에서 별도 계산 없이 통과 여부를 즉시 판정 가능.
- compile report 누락 시 관련 필드는 `null`로 직렬화되어 파서 안정성 유지.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts30 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts30/migration_verify_report.json`에서:
  - `compile_thresholds.max_duplicate_conflict_count=35`
  - `compile_threshold_status.all_ok=true`
  - `verification_status.compile_thresholds_all_ok=true`
  반영 확인.
