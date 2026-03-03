# Commit 238 - Audit 아티팩트 디렉터리 일괄 출력 옵션 추가

## 커밋 요약
- `migration_verify`에 `MIGRATION_AUDIT_REPORT_DIR`를 추가해 localization audit 관련 JSON/Markdown 아티팩트를 단일 디렉터리 기준으로 자동 생성할 수 있게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 환경변수 추가:
    - `MIGRATION_AUDIT_REPORT_DIR`
  - 동작:
    - 값이 주어지면 trailing slash를 정규화한 뒤, 개별 경로 env가 비어 있는 항목에 기본 경로를 자동 할당.
    - 기본 파일 매핑:
      - `audit.json`
      - `duplicate.json`
      - `duplicate_conflicts.md`
      - `key_owner_policy.generated.json`
      - `owner_policy.md`
    - 이미 개별 env(`MIGRATION_AUDIT_REPORT_JSON` 등)가 설정되어 있으면 해당 값 우선(override 없음).
  - 실행 로그에 `audit artifact dir=...`를 출력해 아티팩트 수집 위치를 명시.

## 기능 영향
- CI/로컬 검증에서 audit 산출물 수집 설정이 단순화됨(환경변수 1개).
- duplicate/owner-policy 관련 구조 분석 결과를 동일 디렉터리에 모아 보관해 회귀 비교와 리뷰 생산성이 개선됨.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts tools/migration_verify.sh` 통과.
  - 생성 파일 확인:
    - `/tmp/worldsim_audit_artifacts/audit.json`
    - `/tmp/worldsim_audit_artifacts/duplicate.json`
    - `/tmp/worldsim_audit_artifacts/duplicate_conflicts.md`
    - `/tmp/worldsim_audit_artifacts/key_owner_policy.generated.json`
    - `/tmp/worldsim_audit_artifacts/owner_policy.md`
  - 기존 검증 결과 유지:
    - parity issues 0
    - owner policy missing/unused 0
