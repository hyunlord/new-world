# Commit 245 - migration_verify 메타 리포트(JSON) 추가

## 커밋 요약
- `migration_verify` 실행 결과 메타 정보를 단일 JSON으로 저장하는 `MIGRATION_VERIFY_REPORT_JSON` 출력을 추가해, 생성 아티팩트 경로와 실행 옵션을 한 파일에서 추적 가능하게 확장.

## 상세 변경
- `tools/migration_verify.sh`
  - 신규 환경변수:
    - `MIGRATION_VERIFY_REPORT_JSON`
  - `MIGRATION_AUDIT_REPORT_DIR` 사용 시 기본 파일 자동 할당:
    - `migration_verify_report.json`
  - verify report 출력 로직 추가:
    - 메타:
      - `generated_at_utc`
      - `root_dir`
      - `with_benches`
      - `apply_key_fields`
      - `strip_inline_fields`
    - 아티팩트 경로 맵:
      - `compile_report_json`
      - `audit_report_json`
      - `audit_duplicate_report_json`
      - `audit_conflict_markdown`
      - `audit_key_owner_policy_json`
      - `audit_owner_policy_markdown`
      - `audit_owner_policy_compare_report_json`
      - `bench_report_json`
    - 미생성/미설정 경로는 `null`로 출력.
  - 상대 경로 입력을 절대 경로로 정규화해 report에 기록.

## 기능 영향
- verify 파이프라인의 결과물 위치와 실행 조건을 단일 메타 파일로 제공해 CI 수집/후처리 자동화가 단순해짐.
- report dir 기반 운영에서 compile/audit/bench 산출물 인덱스 역할을 수행.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts7 tools/migration_verify.sh` 통과.
  - `/tmp/worldsim_audit_artifacts7/migration_verify_report.json` 생성 확인.
  - `python3 -m json.tool`로 JSON 유효성 확인.
  - `with_benches=false` 실행에서 `bench_report_json=null` 반영 확인.
