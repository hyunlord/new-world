# Commit 303 - PersonalityMaturation linear target Rust 브리지 이관

## 커밋 요약
- `personality_maturation`의 선형 성숙 타깃 계산(`_linear_target`)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `personality_linear_target(age, max_shift, start_age, end_age) -> f32`
  - 단위 테스트 추가:
    - `personality_linear_target_clamps_by_age_window`
    - `personality_linear_target_interpolates_linearly`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_personality_linear_target(...)`

- `scripts/systems/psychology/personality_maturation.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `_linear_target`를 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 성격 성숙 타깃 선형 보간 계산이 Rust 경로로 이동.
- 브리지 실패 시 기존 GDScript 계산 경로를 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `32/56` 적용, 잔여 `24/56`.
