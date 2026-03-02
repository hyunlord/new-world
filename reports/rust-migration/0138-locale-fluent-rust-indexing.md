# 0138 - locale fluent rust indexing

## Commit
- `[rust-r0-238] Prefer Rust Fluent formatting when building locale key index`

## 변경 파일
- `scripts/core/simulation/locale.gd`
  - `_load_fluent_locale()`가 단순 FTL 라인 파서 결과를 바로 사용하는 대신, Rust Fluent 런타임(`SimBridge.locale_format_fluent`)으로 key registry 전수를 포맷해 `_flat_strings`를 구성하도록 변경.
  - Rust Fluent 준비 시(`_rust_fluent_ready`) registry 기반 인덱스를 우선 구축하고, Rust 경로를 사용할 수 없을 때만 기존 단순 파서를 fallback으로 사용.
  - helper 추가:
    - `_build_flat_strings_from_rust_fluent(locale)`
    - `_parse_fluent_source_basic(source)`
- `reports/rust-migration/README.md`
  - 0138 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 공개 시그니처 변경 없음.
- Locale 로딩 동작 변경:
  - Fluent 활성 시 key/value 인덱스 생성 경로가 Rust Fluent 포맷터 기반으로 우선 전환됨.
  - multiline/select/plural 메시지의 GDScript 단순 파싱 의존도를 낮춤.

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-engine` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 현재 JSON locale 파일은 fallback/마이그레이션 원본으로 남아 있으며, Fluent 경로 우선 사용성을 강화한 단계다.
