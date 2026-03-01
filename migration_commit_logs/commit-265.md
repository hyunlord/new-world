# Commit 265 - verify 리포트에 해석용 상태 플래그 추가

## 커밋 요약
- `migration_verify_report.json`에 카운트 기반 상태 플래그 `verification_status`를 추가해 pass/fail 해석을 단순화.

## 상세 변경
- `tools/migration_verify.sh`
  - 상태 계산 헬퍼 추가:
    - `to_json_zero_is_true(value)` (0이면 `true`, 양수면 `false`, 비정수면 `null`)
    - `to_json_three_zeros_is_true(a,b,c)` (세 값이 모두 0이면 `true`)
  - verify report에 `verification_status` 객체 추가:
    - `artifacts_complete`
    - `audit_parity_clean`
    - `audit_duplicate_conflict_free`
    - `audit_owner_policy_missing_duplicate_clean`
    - `audit_owner_policy_unused_clean`
    - `owner_policy_compare_clean`
    - `bench_report_present_when_enabled`

## 기능 영향
- 리포트 소비자가 여러 count 필드를 조합하지 않고도 핵심 품질 상태를 빠르게 판정 가능.
- `WITH_BENCHES` 비활성 시 bench 상태를 `null`로 유지해 의미 없는 false 판정을 방지.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts27 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts27/migration_verify_report.json`에서:
  - `verification_status.audit_parity_clean=true`
  - `verification_status.audit_duplicate_conflict_free=false`
  - `verification_status.owner_policy_compare_clean=true`
  반영 확인.
