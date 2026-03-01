# Commit 283 - ValueSystem plasticity 수식 Rust 브리지 이관

## 커밋 요약
- `value_system`의 연령별 plasticity 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `value_plasticity(age_years: f32) -> f32`
  - 단위 테스트 추가:
    - 연령 구간별 plasticity 값(5/15/25/55세) 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_value_plasticity(age_years)`

- `scripts/systems/social/value_system.gd`
  - static SimBridge 캐시/조회 로직 추가.
  - `get_plasticity`를 Rust-first 호출로 전환하고, 브리지 실패 시 기존 GDScript 수식 fallback 유지.

## 기능 영향
- 가치관 변화 계수(plasticity) 계산을 Rust 경로로 이동해 반복 계산 비용을 낮출 기반 확보.
- 브리지 실패 시 기존 경로를 유지해 기존 시뮬레이션 동작과 호환성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `12/56` 적용, 잔여 `44/56`.
