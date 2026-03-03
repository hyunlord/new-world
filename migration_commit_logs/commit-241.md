# Commit 241 - Owner-policy compare 결과 JSON 아티팩트 추가

## 커밋 요약
- `localization_audit`에 owner-policy 비교 결과(JSON) 출력 옵션을 추가하고, `migration_verify`에서 report dir 기반으로 자동 생성되도록 연동.

## 상세 변경
- `tools/localization_audit.py`
  - 신규 CLI 옵션:
    - `--owner-policy-compare-report-json <path>`
  - owner policy compare(`--compare-key-owner-policy` 또는 `--compare-key-owner-policy-auto`) 결과를 파일로 출력:
    - `compare_enabled`
    - `compare_path`
    - `missing_keys`, `extra_keys`, `changed_keys`
    - `missing_count`, `extra_count`, `changed_count`
  - compare 미실행 상태에서도 기본 구조(0 count, 빈 목록)로 출력 가능.

- `tools/migration_verify.sh`
  - 신규 환경변수:
    - `MIGRATION_AUDIT_OWNER_POLICY_COMPARE_REPORT_JSON`
  - `MIGRATION_AUDIT_REPORT_DIR` 사용 시 기본 파일명 자동 할당:
    - `owner_policy_compare.json`
  - 기본 compare 경로/override compare 경로 모두에서 해당 아티팩트 옵션 전달.

## 기능 영향
- owner-policy compare 결과가 콘솔 출력뿐 아니라 구조화된 JSON으로 남아 CI/리포팅/회귀 비교 자동화가 쉬워짐.
- report dir 기반 아티팩트 수집 시 owner-policy compare 상태까지 단일 디렉터리로 완결 수집 가능.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts3 tools/migration_verify.sh` 통과.
  - `/tmp/worldsim_audit_artifacts3/owner_policy_compare.json` 생성 확인.
  - 현재 결과:
    - `compare_enabled=true`
    - `compare_path=.../localization/key_owners.json`
    - `missing/extra/changed=0`
