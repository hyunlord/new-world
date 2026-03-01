# Commit 251 - Compile report에 manifest 임계치 컨텍스트 추가

## 커밋 요약
- `compile_report.json`에 localization manifest의 품질 게이트 임계치들을 `thresholds` 필드로 추가해, 리포트 단독으로 검증 기준을 해석할 수 있게 확장.

## 상세 변경
- `tools/localization_compile.py`
  - compile summary report payload에 `thresholds` 객체 추가:
    - `max_duplicate_key_count`
    - `max_duplicate_conflict_count`
    - `max_missing_key_fill_count`
    - `max_owner_rule_miss_count`
    - `max_owner_unused_count`
    - `max_duplicate_owner_missing_count`
  - 값은 manifest에서 읽힌 파싱 결과(없으면 `null`)를 그대로 기록.

## 기능 영향
- compile 결과 지표(`max_locale_*`)와 정책 기준(`thresholds`)을 한 파일에서 함께 비교 가능.
- CI 아티팩트만으로도 “어떤 기준으로 pass/fail 되었는지” 추적이 쉬워짐.

## 검증
- `MIGRATION_AUDIT_REPORT_DIR=/tmp/worldsim_audit_artifacts13 tools/migration_verify.sh` 통과.
- `/tmp/worldsim_audit_artifacts13/compile_report.json`에서 `thresholds` 필드와 값 반영 확인:
  - `max_duplicate_key_count=248`
  - `max_duplicate_conflict_count=35`
  - `max_missing_key_fill_count=0`
  - `max_owner_rule_miss_count=0`
  - `max_owner_unused_count=0`
  - `max_duplicate_owner_missing_count=0`
