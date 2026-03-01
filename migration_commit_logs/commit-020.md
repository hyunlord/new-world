# Commit 020 - Emotion label 경로를 Locale key 기반으로 전환

## 커밋 요약
- `emotion_definition.json`의 남은 object형 inline 다국어 필드(`labels_en`, `labels_kr`, `intensity_labels_kr`) 의존을 제거.
- `EmotionData`의 라벨 조회를 `Locale.ltr("EMO_...")` 키 기반으로 통일.

## 상세 변경
- `scripts/core/entity/emotion_data.gd`
  - `_load_emotion_definitions()`에서 inline label dictionary 로드를 제거.
  - `get_intensity_label()`이 값 구간(약/중/강)에 따라 locale key 생성:
    - `EMO_<ID>_MILD`
    - `EMO_<ID>_BASE`
    - `EMO_<ID>_INTENSE`
  - `get_intensity_label_kr()`는 key 기반 경로로 통합(`get_intensity_label` 위임).
  - 내부 헬퍼 `_get_intensity_locale_key()` 추가.
- `data/species/human/emotions/emotion_definition.json`
  - 제거 필드:
    - `intensity_labels`
    - `intensity_labels_kr`
    - `labels_en`
    - `labels_kr`
  - 감정 이름은 기존 `name_key` 경로 유지.

## 기능 영향
- Emotion label 표현이 data inline 문자열이 아니라 localization 키(`EMO_*`) 단일 체계로 동작.
- locale 변경 시 동일 키 기반으로 일관된 라벨 반환.
- 데이터 구조에서 non-keyable inline 다국어 object 의존 제거.

## 검증
- `tools/migration_verify.sh` 전체 통과
- strict audit 결과:
  - `inline_non_keyable_groups: 0`
  - `inline_keyable_without_key: 0`
