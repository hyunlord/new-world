# Commit 272 - sim-data R-1 로더 확장 (stressor/coping/mental_break/traits)

## 커밋 요약
- Rust `sim-data`를 확장해 기존 3종(emotion/tech/value)에서 7종 데이터 번들로 확장.
- placeholder였던 통합 데이터 로딩 테스트를 실제 검증 테스트로 교체.

## 상세 변경
- `rust/crates/sim-data/src/lib.rs`
  - 신규 모듈 export:
    - `stressor_events`
    - `coping`
    - `mental_breaks`
    - `trait_defs`
  - `DataBundle` 구조체 추가:
    - `emotions`, `tech`, `values`, `stressors`, `coping`, `mental_breaks`, `traits`
  - `load_all`이 위 7개 데이터셋을 로드하도록 확장.

- 신규 로더 파일 추가
  - `rust/crates/sim-data/src/stressor_events.rs`
    - `data/stressor_events.json` 로딩
    - `_comment*` 키 자동 제외
  - `rust/crates/sim-data/src/coping.rs`
    - `data/coping_definitions.json` 로딩
  - `rust/crates/sim-data/src/mental_breaks.rs`
    - `data/mental_breaks.json` 로딩
  - `rust/crates/sim-data/src/trait_defs.rs`
    - `data/personality/trait_definitions_fixed.json` 로딩

- `rust/crates/sim-test/src/main.rs`
  - `sim_data::load_all`의 `DataBundle`을 받아 신규 데이터셋 카운트까지 로그하도록 확장.

- `rust/tests/data_loading_test.rs`
  - placeholder 테스트 제거.
  - 실제 통합 테스트 2개 추가:
    - `load_all_contains_r1_core_datasets`
    - `stressor_loader_skips_comment_keys`

## 기능 영향
- Rust side에서 R-1 핵심 데이터(스트레서/코핑/멘탈브레이크/트레잇)를 직접 파싱/검증 가능.
- 향후 시스템 이식 시 GDScript 데이터 의존도를 단계적으로 제거할 수 있는 기반 강화.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS` 확인).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**:
  - 이전: `3/9` 완료 (emotion, tech, value 중심)
  - 현재: `7/9` 완료 (stressor, coping, mental_break, trait 추가)
  - 잔여: `2/9` (species, mortality/building 계열 로더 정리)
- **전체 전환(엔진/30+ 시스템/Godot 런타임 분리 포함)**:
  - 여전히 하이브리드 단계. 메인 tick 오케스트레이션은 GDScript 주도.
