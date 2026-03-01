# Commit 267 - verify 리포트에 compile 요약 흡수

## 커밋 요약
- `migration_verify_report.json`에 `compile_report.json` 핵심 지표를 반영하는 `compile_summary`를 추가.

## 상세 변경
- `tools/migration_verify.sh`
  - `compile_report.json`에서 주요 값을 추출해 verify report에 포함:
    - `generated_at_utc`
    - `default_locale`
    - `supported_locale_count`
    - `active_key_count`
    - `registry_key_count`
    - `max_locale_duplicates`
    - `max_locale_duplicate_conflicts`
    - `max_locale_missing_filled`
    - `max_locale_owner_rule_misses`
    - `owner_policy_entry_count`
    - `owner_policy_missing_duplicate_count`
    - `owner_policy_unused_count`
    - `strict_duplicates`

## 기능 영향
- compile/audit/bench 결과가 verify report 한 파일에 집약되어 후처리/대시보드 연결이 단순화.
- compile report 부재 시 각 필드는 `null`로 직렬화되어 기존 파서 호환성 유지.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts29 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts29/migration_verify_report.json`에서:
  - `compile_summary.active_key_count=4030`
  - `compile_summary.max_locale_duplicate_conflicts=35`
  - `compile_summary.strict_duplicates=false`
  반영 확인.
