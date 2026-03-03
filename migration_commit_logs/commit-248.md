# Commit 248 - verify 산출물 존재 강제 검증 옵션 추가

## 커밋 요약
- `migration_verify`에 `MIGRATION_VERIFY_ASSERT_ARTIFACTS`를 추가해, 생성되어야 하는 report 아티팩트 파일 존재를 검증 단계에서 강제할 수 있게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 환경변수:
    - `MIGRATION_VERIFY_ASSERT_ARTIFACTS` (`true|false`, 기본 `false`)
  - 입력값 검증:
    - `true|false` 외 값이면 즉시 실패.
  - verify report 작성 구간 확장:
    - `assert_artifact_exists` 헬퍼 추가(상대 경로 절대화 포함).
    - `assert=true`일 때 아래 아티팩트의 파일 존재를 강제:
      - `compile_report_json`
      - `audit_report_json`
      - `audit_duplicate_report_json`
      - `audit_conflict_markdown`
      - `audit_key_owner_policy_json`
      - `audit_owner_policy_markdown`
      - `audit_owner_policy_compare_report_json`
      - (`WITH_BENCHES=true`일 때) `bench_report_json`
    - 모두 통과 시 `artifact existence checks passed` 로그 출력.
  - `migration_verify_report.json` 메타 확장:
    - `assert_artifacts` 필드 추가.

## 기능 영향
- CI/자동화에서 “경로만 출력되고 파일이 누락”되는 종류의 회귀를 즉시 검출 가능.
- verify 메타 리포트에 assert 실행 여부가 남아, 아티팩트 신뢰 수준을 사후 추적하기 쉬워짐.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts10 MIGRATION_VERIFY_ASSERT_ARTIFACTS=true tools/migration_verify.sh` 통과.
  - 로그에서 `artifact existence checks passed` 확인.
  - `/tmp/worldsim_audit_artifacts10/migration_verify_report.json`에 `assert_artifacts=true` 반영 확인.
