# Commit 302 - StatsRecorder 리소스 delta 계산 Rust 브리지 이관

## 커밋 요약
- `stats_recorder`의 리소스 변화율 계산(`get_resource_deltas`)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `stats_resource_deltas_per_100(latest_food, latest_wood, latest_stone, older_food, older_wood, older_stone, tick_diff) -> [f32; 3]`
  - 단위 테스트 추가:
    - `stats_resource_deltas_per_100_scales_by_tick_diff`
    - `stats_resource_deltas_per_100_returns_zero_when_tick_diff_invalid`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_stats_resource_deltas_per_100(...)`

- `scripts/systems/record/stats_recorder.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `get_resource_deltas`를 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 통계 기록 시스템의 리소스 변화율 계산이 Rust 경로로 이동.
- 브리지 실패/결과 비정상 시 기존 GDScript 계산 경로로 fallback 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `31/56` 적용, 잔여 `25/56`.
