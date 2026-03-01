# Commit 306 - PersonalityGenerator 자식 축 z 계산 Rust 브리지 이관

## 커밋 요약
- `personality_generator`의 자식 축 z-score 합성 수식(유전/성차/문화 시프트)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `personality_child_axis_z(has_parents, parent_a_axis, parent_b_axis, heritability, random_axis_z, is_female, sex_diff_d, culture_shift) -> f32`
  - 단위 테스트 추가:
    - `personality_child_axis_z_applies_inheritance_and_sex_shift`
    - `personality_child_axis_z_applies_culture_shift`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_personality_child_axis_z(...)`

- `scripts/systems/biology/personality_generator.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `generate_personality`의 axis별 `z_child` 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- HEXACO 축 z-score 합성 루프의 핵심 수식이 Rust 경로로 이동.
- 브리지 실패 시 기존 GDScript 계산 경로 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 실측 기준)**: `39/56` 적용, 잔여 `17/56`.
