# Commit 253 - verify 리포트에 아티팩트 SHA-256 추가

## 커밋 요약
- `migration_verify_report.json`에 아티팩트별 SHA-256 해시(`artifact_sha256`)를 추가해 결과 무결성 비교와 실행 간 diff 추적을 강화.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 헬퍼:
    - `to_json_opt_sha256(path)` 추가
      - 파일이 없으면 `null`
      - macOS(`shasum -a 256`) / Linux(`sha256sum`) 모두 대응
  - verify report 확장:
    - `artifact_sha256` 객체 추가
      - `compile_report_json`
      - `audit_report_json`
      - `audit_duplicate_report_json`
      - `audit_conflict_markdown`
      - `audit_key_owner_policy_json`
      - `audit_owner_policy_markdown`
      - `audit_owner_policy_compare_report_json`
      - `bench_report_json`

## 기능 영향
- 동일 경로에 저장된 아티팩트의 내용 변경을 해시만으로 빠르게 감지 가능.
- CI에서 verify report 하나만 수집해도 아티팩트 무결성 검증 및 캐시 키 생성에 활용 가능.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts15 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts15/migration_verify_report.json`에서
  - `artifact_sha256` 객체 생성 확인.
  - non-bench 실행 기준:
    - compile/audit 관련 해시는 문자열로 채워짐
    - `bench_report_json` 해시는 `null`
    확인.
