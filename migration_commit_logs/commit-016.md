# Commit 016 - Localization 감사의 keyable/non-keyable 분리

## 커밋 요약
- inline 다국어 감사에서 `*_key` 전환 대상 여부를 타입 기반으로 분리해 지표 정확도를 개선.
- 문자열 기반 그룹만 `keyable`로 계산하고, 객체/배열 기반 그룹은 `non-keyable`로 별도 분류.

## 상세 변경
- `tools/localization_audit.py`
  - `inline_localized_group`에 `value_types`, `keyable_group` 정보 추가.
  - 집계 지표 분리:
    - `inline_keyable_group_count`
    - `inline_non_keyable_group_count`
    - `inline_keyable_group_with_key_count`
    - `inline_keyable_group_without_key_count`
  - 출력 섹션 추가:
    - keyable 누락 그룹 목록
    - non-keyable 그룹 목록(타입 포함)
  - `--strict` 판정 변경:
    - `parity_issues` 또는 `inline_keyable_group_without_key_count > 0`일 때만 실패.

## 결과
- 현재 기준:
  - `inline_keyable_groups: 437`
  - `inline_keyable_without_key: 0`
  - `inline_non_keyable_groups: 2` (`object` 타입)
- `--strict`가 0으로 통과되어 keyable 전환 완료 상태를 정확히 반영.

## 검증
- `python3 tools/localization_audit.py --project-root .` 실행 성공
- `python3 tools/localization_audit.py --project-root . --strict` 실행 성공(exit=0)
