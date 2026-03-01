# Commit 013 - Localization 감사 지표 확장(키 전환 추적)

## 커밋 요약
- `localization_audit`에 inline 다국어의 `*_key` 전환 커버리지 지표를 추가.
- data 생성 산출물(`localization_*.json`)은 감사 대상에서 제외해 노이즈를 제거.

## 상세 변경
- `tools/localization_audit.py`
  - 신규 집계:
    - `inline_localized_group_count`
    - `inline_group_with_key_count`
    - `inline_group_without_key_count`
  - inline 그룹(`base_field + path`) 단위로 `*_key` 존재 여부를 판단.
  - `data/localization_*.json` 파일은 스캔에서 제외.
  - 리포트 출력에 `*_key` 미연결 그룹 샘플 표시 추가.

## 현재 지표(실행 결과)
- `inline_groups: 439`
- `inline_groups_with_key: 437`
- `inline_groups_without_key: 2`
  - 미연결 항목: `intensity_labels`, `labels` (`emotion_definition.json` 루트)

## 검증
- `python3 tools/localization_audit.py --project-root .` 실행 성공
