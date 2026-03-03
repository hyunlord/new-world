# 0003 - Phase G fluent loader scaffold

## Summary
기존 JSON 로컬라이제이션 파이프라인을 유지하면서, Fluent `.ftl` 소스 기반 런타임 로딩 경로를 추가했다.

## Files Changed
- `tools/localization_compile_fluent.py` (new)
  - `localization/compiled/<locale>.json`의 `strings`를 `localization/fluent/<locale>/messages.ftl`로 컴파일.
- `localization/manifest.json`
  - `fluent_dir`, `use_fluent_runtime` 설정 추가.
- `scripts/core/simulation/locale.gd`
  - Fluent 런타임 설정 필드 추가.
  - `load_locale()`에서 Fluent 우선 로드 분기 추가.
  - `_load_fluent_locale()` 추가(기본 key=value FTL 로딩).
  - manifest에서 fluent 관련 설정 읽기 추가.
- `localization/fluent/ko/messages.ftl` (new, generated)
- `localization/fluent/en/messages.ftl` (new, generated)

## API / Signal / Schema Changes
### Localization runtime
- Added manifest keys:
  - `fluent_dir: "fluent"`
  - `use_fluent_runtime: true`
- Locale load order:
  1. Fluent (`localization/fluent/<locale>/messages.ftl`)
  2. Compiled JSON (`localization/compiled/<locale>.json`)
  3. Category JSON fallback (`localization/<locale>/*.json`)

## Verification
- `python3 tools/localization_compile_fluent.py --project-root . --locales ko en` : PASS
- `python3 tools/localization_compile.py --project-root .` : PASS
- Godot headless check: 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 42% complete / 58% remaining
- Current: 50% complete / 50% remaining
- Delta: +8%

## Notes
- 현재 Fluent 로더는 key/value 메시지 로딩 중심이다.
- ICU plural/select 고급 규칙 및 Rust Fluent bundle 완전 이관은 후속 단계에서 확장한다.
