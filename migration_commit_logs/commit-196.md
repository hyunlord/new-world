# Commit 196 - localization_compile에 중복 충돌 회귀 게이트 추가

## 커밋 요약
- `localization_compile.py`에 중복 키 값 충돌(`duplicate_conflict`) 계산/출력을 추가하고, manifest 기준선(`max_duplicate_conflict_count`)으로 회귀를 차단하도록 확장.

## 상세 변경
- `tools/localization_compile.py`
  - manifest 기본 필드에 `max_duplicate_conflict_count` 추가.
  - `_compile_locale()`에서 duplicate key 발생 시 값이 다른 중복을 `duplicate_conflict_keys`로 별도 추적.
  - locale meta에 `duplicate_conflict_count` 포함.
  - 컴파일 로그에 `duplicate_conflicts=` 출력 추가.
  - 회귀 게이트 추가:
    - `max_locale_duplicate_conflicts > max_duplicate_conflict_count`일 때 실패.
- `localization/manifest.json`
  - `max_duplicate_conflict_count` 기준선 추가 (`35`).
- `localization/compiled/en.json`, `localization/compiled/ko.json`
  - meta에 `duplicate_conflict_count` 반영.

## 기능 영향
- 기존 duplicate key 총량 게이트와 별도로, 값 불일치 중복 증가를 자동 차단.
- 현재 기준:
  - `en duplicate_conflicts=24`
  - `ko duplicate_conflicts=35`

## 검증
- `tools/migration_verify.sh --with-benches` 통과.
  - localization compile 단계에서 `duplicate_conflicts` 출력 확인.
  - strict audit/bench checksum 모두 통과.
