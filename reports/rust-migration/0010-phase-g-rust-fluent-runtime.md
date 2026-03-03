# 0010 - Phase G Rust Fluent runtime bridge

## Summary
Localization Fluent 경로를 Rust 브리지에 연결했다. `Locale`는 Fluent 모드에서 Rust 포맷터를 우선 사용하며, 파라미터 치환/복수형 선택을 Rust `fluent-bundle` 기반으로 처리한다.

## Files Changed
- `rust/Cargo.toml`
  - workspace dependency 추가: `fluent-bundle`, `unic-langid`
- `rust/crates/sim-bridge/Cargo.toml`
  - `sim-bridge`에 Fluent 런타임 의존성 연결
- `rust/crates/sim-bridge/src/lib.rs`
  - Fluent source 캐시(`locale -> ftl text`) 추가
  - Fluent 포맷 헬퍼 추가
    - locale fallback (`xx-YY` -> `xx` -> `en`)
    - VarDictionary params -> Fluent args 변환
    - format 시 isolating 비활성화(`set_use_isolating(false)`)
  - `WorldSimBridge` API 추가
    - `locale_load_fluent(locale, source)`
    - `locale_clear_fluent(locale)`
    - `locale_format_fluent(locale, key, params)`
  - Rust unit test 추가 (named param, plural)
- `scripts/core/simulation/sim_bridge.gd`
  - Rust locale bridge wrapper 추가
    - `locale_load_fluent()`
    - `locale_clear_fluent()`
    - `locale_format_fluent()`
- `scripts/core/simulation/locale.gd`
  - Fluent 로드 시 Rust runtime cache priming 연결
  - `ltr/trf/trf1..trf5`에서 Rust Fluent formatter 우선 사용 경로 추가
  - 미지원/미로딩 시 기존 GDScript fallback 유지
- `rust/Cargo.lock`
  - Fluent/ICU 관련 의존성 잠금 반영

## API / Signal / Schema Changes
### SimBridge localization API
- Added: `locale_load_fluent(locale: String, source: String) -> bool`
- Added: `locale_clear_fluent(locale: String) -> void`
- Added: `locale_format_fluent(locale: String, key: String, params: Dictionary) -> String`

### Locale runtime behavior
- `use_fluent_runtime=true` + Rust bridge available 시
  - `Locale.trf*()` 포맷 경로는 Rust Fluent 우선
  - 실패 시 기존 문자열/placeholder 치환 fallback

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-bridge --lib` : PASS (26 passed)
- `cd rust && cargo test -p sim-engine --lib` : PASS (21 passed)
- Godot headless check: 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 80% complete / 20% remaining
- Current: 84% complete / 16% remaining
- Delta: +4%

## Notes
- 현재는 FTL source를 메모리 캐시에 저장하고 runtime format 시 bundle을 구성하는 방식이다.
- 다음 단계에서 Fluent bundle 캐시 고도화 및 ICU selector 케이스 확장 테스트를 추가할 수 있다.
