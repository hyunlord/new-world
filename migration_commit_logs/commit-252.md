# Commit 252 - Audit JSON에 스키마/타임스탬프 메타 추가

## 커밋 요약
- `localization_audit` JSON 산출물(`audit.json`, `duplicate.json`)에 `schema_version`과 `generated_at_utc` 메타를 추가해 아티팩트 버전/생성시각 추적을 강화.

## 상세 변경
- `tools/localization_audit.py`
  - `run_audit()` 결과에 메타 필드 추가:
    - `schema_version: 1`
    - `generated_at_utc` (UTC ISO-8601, `Z`)
  - `--duplicate-report-json` 출력 payload에도 동일 메타를 포함.
  - `datetime/timezone` import 추가.

## 기능 영향
- audit/duplicate 리포트 소비 측에서 스키마 버전 분기 처리 가능.
- 동일 경로에 누적 저장되는 리포트의 생성 시각 비교가 쉬워져 회귀 분석 자동화에 유리.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts14 tools/migration_verify.sh` 통과.
- 메타 필드 확인:
  - `/tmp/worldsim_audit_artifacts14/audit.json`
  - `/tmp/worldsim_audit_artifacts14/duplicate.json`
  에서 `schema_version`, `generated_at_utc` 존재 확인.
