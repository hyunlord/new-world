# Commit 009 - Localization 컴파일 인덱스 구조 도입

## 커밋 요약
- 카테고리 분할 JSON 작성 방식은 유지하면서, 런타임은 컴파일된 단일 인덱스(`compiled/<locale>.json`)를 우선 로드하도록 구조 개선.
- Manifest 기반으로 locale/category/compiled 디렉토리를 선언해 확장성과 유지보수성을 높임.

## 상세 변경
- `localization/manifest.json` (신규)
  - `default_locale`, `supported_locales`, `categories_order`, `compiled_dir` 선언.
- `tools/localization_compile.py` (신규)
  - 카테고리 JSON들을 locale별 flat map으로 컴파일.
  - 기존 로더와 동일한 first-wins 중복 키 처리 유지.
  - 결과 파일에 key source 메타를 기록해 디버깅/정리 작업 지원.
- `localization/compiled/en.json`, `localization/compiled/ko.json` (신규 생성물)
  - 각 locale 3593 key 단일 인덱스 파일.
- `scripts/core/simulation/locale.gd`
  - startup 시 `manifest.json` 로드.
  - `load_locale()`에서 compiled 파일 우선 로드, 미존재/파싱 실패 시 기존 카테고리 방식 fallback.
  - manifest 값으로 `supported_locales`, `default_locale`, `categories`를 runtime 반영.

## 기능 영향
- 런타임 초기 로딩에서 파일 open/parse 횟수를 줄여 로딩 비용을 낮춤.
- 카테고리 추가/순서 변경을 manifest에서 제어할 수 있어 확장시 코드 변경량 감소.
- 기존 JSON 카테고리 파일은 fallback 경로로 유지되어 안전하게 점진 이행 가능.

## 검증
- `python3 tools/localization_compile.py --project-root .` 실행 성공
- 출력: `ko/en strings=3593, duplicates=248`
- `python3 tools/localization_audit.py --project-root .` 실행 성공
- `cd rust && cargo test -q` 통과
