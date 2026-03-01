# Commit 264 - verify 리포트에 audit 요약 지표 흡수

## 커밋 요약
- `migration_verify_report.json`이 `audit.json`/`owner_policy_compare.json`의 핵심 카운트를 직접 포함하도록 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - JSON 파일에서 값을 안전 추출하는 헬퍼 추가:
    - `to_json_opt_int_from_json_file_key(path, key)`
    - `to_json_opt_array_len_from_json_file_key(path, key)`
    - `to_json_opt_string_from_json_file_key(path, key)`
  - verify report에 `audit_summary` 객체 추가:
    - `parity_issue_count`
    - `duplicate_key_count`
    - `duplicate_conflict_count`
    - `duplicate_consistent_count`
    - `duplicate_report_key_count`
    - `duplicate_report_conflict_count`
    - `duplicate_report_locale`
    - `inline_localized_field_count`
    - `owner_policy_entry_count`
    - `owner_policy_category_count`
    - `owner_policy_missing_duplicate_count`
    - `owner_policy_unused_count`
  - verify report에 `owner_policy_compare_summary` 객체 추가:
    - `missing_count`
    - `extra_count`
    - `changed_count`

## 기능 영향
- verify report 단일 파일만으로 localization 품질 핵심 지표를 조회 가능.
- 별도 `audit.json` 재파싱 없이도 CI/대시보드에서 실패 원인 분류가 쉬워짐.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts26 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts26/migration_verify_report.json`에서:
  - `audit_summary.duplicate_conflict_count=35`
  - `audit_summary.owner_policy_category_count=2`
  - `owner_policy_compare_summary.missing_count=0`
  - `owner_policy_compare_summary.extra_count=0`
  - `owner_policy_compare_summary.changed_count=0`
  반영 확인.
