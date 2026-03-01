# Commit 012 - Data JSON에 *_key 참조 자동 주입(하위호환 유지)

## 커밋 요약
- 인라인 다국어가 있는 원본 data JSON 4개 파일에 `*_key` 참조를 자동 주입.
- 기존 `*_en/*_ko/*_kr` 텍스트는 그대로 유지하여 즉시 회귀 없이 점진 이관 가능 상태로 전환.

## 상세 변경
- `tools/data_localization_extract.py`
  - `--apply-key-fields` 옵션 추가.
  - 옵션 사용 시 추출된 key를 원본 node에 `<field>_key` 형태로 기록.
  - 비파괴 원칙 유지: 기존 인라인 텍스트 필드는 삭제하지 않음.
- key 주입 적용 파일(4개)
  - `data/stressor_events.json`
  - `data/species/human/species_definition.json`
  - `data/species/human/emotions/emotion_definition.json`
  - `data/species/human/personality/trait_definitions.json`

## 기능 영향
- 데이터 소비 코드가 `Locale.tr_data(..., field)` 호출 시 `field_key`를 우선 사용할 수 있어 key-value 구조로 점진 전환 가능.
- 인라인 텍스트가 남아있어 기존 경로와 동시 운영 가능.

## 검증
- `python3 tools/data_localization_extract.py --project-root . --apply-key-fields` 실행 성공
  - 출력: `entries=437, keys=437, changed_files=4`
- `cd rust && cargo test -q` 통과
