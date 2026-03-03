# Commit 006 - Localization/Data 감사 스크립트 추가

## 커밋 요약
- localization 구조 개선 작업을 지속 가능하게 하기 위해 자동 감사 스크립트를 추가.
- en/ko 키 불일치, 중복 키, data 내부 `_en/_ko/_kr` 인라인 로컬라이즈 필드를 자동 탐지.

## 상세 변경
- `tools/localization_audit.py` (신규)
  - 검사 항목:
    1) `localization/en` vs `localization/ko` 파일별 top-level key parity
    2) `localization/en` 파일 간 duplicate key
    3) `data/**/*.json` 내 `_en/_ko/_kr` 키 탐지
  - 옵션:
    - `--project-root <path>`
    - `--strict` (이슈 발견 시 non-zero exit)

## 실행 결과(현재 코드 기준)
- `parity_issues: 0`
- `duplicate_keys: 248`
- `inline_localized_fields: 876`

## 기능 영향
- 현재 로컬라이제이션/데이터 정규화 진행률을 반복적으로 측정 가능.
- 향후 CI에 strict 모드 연결 시 구조 회귀 방지 가능.

## 검증
- `python3 tools/localization_audit.py --project-root .` 실행 성공
