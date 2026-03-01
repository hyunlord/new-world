# Commit 197 - localization_audit 다국어 중복 충돌 집계 일관화

## 커밋 요약
- `localization_audit.py`의 중복 키/충돌 집계를 `en` 단일 기준에서 `supported_locales` 다국어 기준으로 확장해, compile 단계와 동일한 관점(로케일 최대 충돌)을 반영하도록 정리.

## 상세 변경
- `tools/localization_audit.py`
  - `manifest.json`의 `supported_locales`를 읽어 중복 집계 대상 로케일을 동적으로 결정.
  - 로케일별 중복 요약(`duplicate_locale_summary`) 계산:
    - `duplicate_key_count`
    - `duplicate_conflict_count`
  - 충돌이 가장 큰 로케일을 `duplicate_report_locale`로 선택하고 상세 충돌 목록(`duplicate_details`)을 그 로케일 기준으로 출력.
  - 상위 요약 수치(`duplicate_key_count`, `duplicate_conflict_count`)를 로케일 최대치 기준으로 표준화.
  - 콘솔 출력에 `duplicate_report_locale` 및 `duplicate summary by locale` 섹션 추가.

## 기능 영향
- audit 출력이 compile 게이트(`max_duplicate_conflict_count`)와 같은 기준으로 정렬됨.
- 현재 기준:
  - `en: conflicts=24`
  - `ko: conflicts=35`
  - audit top-level `duplicate_conflicts=35`, `duplicate_report_locale=ko`

## 검증
- `python3 tools/localization_audit.py --project-root .` 실행 확인.
  - `duplicate_conflicts: 35`
  - `duplicate_report_locale: ko`
- `tools/migration_verify.sh --with-benches` 통과.
