# Commit 270 - audit conflict 프리뷰 개수 환경변수화

## 커밋 요약
- duplicate conflict 프리뷰 개수를 `MIGRATION_VERIFY_AUDIT_CONFLICT_PREVIEW_LIMIT`로 제어 가능하게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 env 설정 추가:
    - `MIGRATION_VERIFY_AUDIT_CONFLICT_PREVIEW_LIMIT` (기본 `10`)
  - 입력 검증 추가:
    - 비음수 정수가 아니면 즉시 실패
  - verify report `config`에 설정 반영:
    - `config.audit_conflict_preview_limit`
  - `audit_conflict_preview.limit/count/keys` 계산에 env 기반 limit 적용.

## 기능 영향
- 운영/CI에서 리포트 크기와 진단 깊이를 상황별로 조정 가능.
- 기본 동작(10개 프리뷰)은 기존과 동일하게 유지.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts32 MIGRATION_VERIFY_AUDIT_CONFLICT_PREVIEW_LIMIT=3 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts32/migration_verify_report.json`에서:
  - `config.audit_conflict_preview_limit=3`
  - `audit_conflict_preview.limit=3`
  - `audit_conflict_preview.count=3`
  - `audit_conflict_preview.keys` 길이 `3`
  확인.
