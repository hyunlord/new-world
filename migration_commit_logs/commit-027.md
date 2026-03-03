# Commit 027 - 데이터 inline 다국어 필드 전면 제거(key-only 전환)

## 커밋 요약
- `data_localization_extract --apply-key-fields --strip-inline-fields`를 실제 데이터에 적용해 inline 다국어 필드를 제거.
- key-first 구조로 데이터 소스를 정리하고, strict audit 기준에서 inline localized field를 0으로 만들었다.

## 상세 변경
- 변경 데이터 파일(4개):
  - `data/species/human/personality/trait_definitions.json`
  - `data/stressor_events.json`
  - `data/species/human/emotions/emotion_definition.json`
  - `data/species/human/species_definition.json`
- 제거된 필드 유형:
  - `name_en`, `name_kr`
  - `description_en`, `description_kr`
  - `species_name_kr`
- 유지/활용 경로:
  - `*_key` 필드 기반으로 Locale lookup
  - `localization/en|ko/data_generated.json`은 preserve 모드로 유지
- `data/localization_extraction_map.json`
  - 현재 스캔 기준으로 `entries=0`, key 보존 기반 summary로 갱신

## 기능 영향
- 데이터 구조가 key-only로 수렴되어 언어 확장/유지보수 시 중복 관리 비용 감소.
- 코드 측 key-first 조회 경로(이전 커밋에서 정리)와 데이터 구조가 일치.

## 검증
- `python3 tools/data_localization_extract.py --project-root . --apply-key-fields --strip-inline-fields`
  - `changed_files=4`, `stripped_fields=873`
- `tools/migration_verify.sh` 통과
  - extraction: `entries=0`, `keys=437`, `preserved=437`
  - strict audit:
    - `inline_localized_fields: 0`
    - `inline_groups: 0`
    - `inline_keyable_without_key: 0`
