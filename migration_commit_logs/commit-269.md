# Commit 269 - verify 리포트에 duplicate 충돌 키 프리뷰 추가

## 커밋 요약
- `migration_verify_report.json`에 `audit_conflict_preview`를 추가해 duplicate value conflict 키를 상위 N개로 빠르게 확인 가능하게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - conflict preview 추출 헬퍼 추가:
    - `to_json_conflict_key_preview(path, max_items)`
    - `to_json_conflict_key_preview_count(path, max_items)`
  - `audit.json`의 `duplicate_details`에서 `value_conflict=true`인 키를 정렬 후 상위 N개 추출.
  - verify report에 `audit_conflict_preview` 객체 추가:
    - `limit`
    - `count`
    - `keys`
  - 기본 프리뷰 제한값은 `10`으로 설정.

## 기능 영향
- duplicate conflict가 많은 상황에서도 report만으로 대표 충돌 키를 즉시 파악 가능.
- 후속 triage(어떤 키/도메인부터 정리할지) 속도 향상.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts31 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts31/migration_verify_report.json`에서:
  - `audit_conflict_preview.limit=10`
  - `audit_conflict_preview.count=10`
  - `audit_conflict_preview.keys`에 상위 충돌 키 10개 반영 확인.
