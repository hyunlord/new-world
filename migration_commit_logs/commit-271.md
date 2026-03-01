# Commit 271 - audit conflict 프리뷰 잘림 상태 추가

## 커밋 요약
- `audit_conflict_preview`에 전체 충돌 수와 잘림 여부(`truncated`)를 추가해 프리뷰 범위의 충분성을 판별 가능하게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 정수 비교 헬퍼 추가:
    - `to_json_gt_ints(lhs, rhs)` (`lhs > rhs`)
  - `audit_conflict_preview` 확장:
    - `total_conflict_key_count` (전체 duplicate value conflict 키 수)
    - `truncated` (`total_conflict_key_count > count`)

## 기능 영향
- 프리뷰가 전체 충돌을 모두 담았는지 report에서 즉시 확인 가능.
- 운영자가 limit 값을 늘려야 할지(`truncated=true`) 빠르게 판단 가능.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts33 MIGRATION_VERIFY_AUDIT_CONFLICT_PREVIEW_LIMIT=3 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts33/migration_verify_report.json`에서:
  - `audit_conflict_preview.total_conflict_key_count=35`
  - `audit_conflict_preview.count=3`
  - `audit_conflict_preview.truncated=true`
  반영 확인.
