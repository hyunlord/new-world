# Commit 244 - Localization compile summary JSON 아티팩트 추가

## 커밋 요약
- `localization_compile`에 compile 요약 JSON 출력 옵션을 추가하고, `migration_verify`에서 report dir 사용 시 `compile_report.json`을 자동 생성하도록 연동.

## 상세 변경
- `tools/localization_compile.py`
  - `run(...)` 시그니처 확장:
    - `report_json` 인자 추가.
  - 신규 CLI 옵션:
    - `--report-json <path>`
  - compile 완료 후 요약 리포트 출력 추가:
    - 전역 요약:
      - `default_locale`, `supported_locales`, `categories_order`
      - `registry_key_count`, `active_key_count`
      - `max_locale_duplicates`, `max_locale_duplicate_conflicts`
      - `max_locale_missing_filled`, `max_locale_owner_rule_misses`
      - owner policy 지표(엔트리/누락/미사용 + 키 목록)
    - locale별 요약:
      - `string_count`
      - duplicate/owner/missing-fill 지표
      - compiled output path
  - 리포트 작성 로그:
    - `[localization_compile] report written: ...`

- `tools/migration_verify.sh`
  - 신규 환경변수:
    - `MIGRATION_COMPILE_REPORT_JSON`
  - compile 단계를 명령 배열(`compile_cmd`)로 전환해 optional `--report-json` 전달 지원.
  - `MIGRATION_AUDIT_REPORT_DIR`가 설정되고 compile report 경로가 비어 있으면 기본 파일 `compile_report.json` 자동 할당.

## 기능 영향
- compile 단계 결과를 구조화된 JSON으로 저장할 수 있어 localization 구조 변화 추적/대시보드 집계가 쉬워짐.
- audit 아티팩트와 compile 아티팩트를 같은 디렉터리로 수집해 검증 산출물 관리가 단순화됨.

## 검증
- `bash -n tools/migration_verify.sh` 통과.
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts6 tools/migration_verify.sh` 통과.
  - `/tmp/worldsim_audit_artifacts6/compile_report.json` 생성 확인.
  - `python3 -m json.tool`로 JSON 유효성 확인.
  - 리포트에 locale별 지표(`en`, `ko`)와 owner policy 요약 지표 반영 확인.
