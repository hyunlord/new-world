# Commit 011 - Data inline 다국어 추출 파이프라인 추가

## 커밋 요약
- `data/**/*.json`의 인라인 `*_en/*_ko/*_kr` 텍스트를 로컬라이제이션 키 구조로 옮기기 위한 자동 추출 파이프라인을 추가.
- 런타임 `Locale`이 `field_key` 패턴을 일반화해 읽을 수 있도록 확장.

## 상세 변경
- `tools/data_localization_extract.py` (신규)
  - data JSON 스캔 후 인라인 문자열 다국어 필드 추출.
  - 생성물:
    - `localization/en/data_generated.json`
    - `localization/ko/data_generated.json`
    - `data/localization_extraction_map.json` (원본 위치↔생성 키 매핑)
  - 비파괴 방식: 현재 단계에서는 원본 data 파일 수정 없음.
- `localization/en/data_generated.json`, `localization/ko/data_generated.json` (신규 생성물)
  - 추출 키 437개 수록.
- `localization/manifest.json`
  - `categories_order`에 `data_generated` 추가.
- `tools/localization_compile.py`
  - 기본 카테고리에 `data_generated` 반영.
- `scripts/core/simulation/locale.gd`
  - 기본 카테고리에 `data_generated` 추가.
  - `tr_data()`에서 `field + "_key"` 일반 패턴 지원 추가.
- `localization/compiled/en.json`, `localization/compiled/ko.json`
  - 재컴파일 반영(총 key 4030).

## 기능 영향
- 인라인 다국어 데이터를 key-value localization으로 점진 이관할 수 있는 자동화 기반 확보.
- `name_key/desc_key` 외에도 `*_key`를 공통 처리해 데이터 모델 확장 시 코드 수정량 감소.
- 기존 인라인 필드는 유지되어 하위 호환 동작 보장.

## 검증
- `python3 tools/data_localization_extract.py --project-root .` 실행 성공
- 출력: `entries=437, keys=437`
- `python3 tools/localization_compile.py --project-root .` 실행 성공
- 출력: `ko/en strings=4030, duplicates=248`
- `python3 tools/localization_audit.py --project-root .` 실행 성공
