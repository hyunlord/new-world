# Commit 282 - StratificationMonitor 수식 Rust 브리지 이관

## 커밋 요약
- `stratification_monitor`의 Gini 계산과 status score 합성 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `stratification_gini(values: &[f32]) -> f32`
    - `stratification_status_score(...) -> f32`
  - 단위 테스트 추가:
    - 균등 분포/편중 분포에 대한 Gini 동작 검증
    - status score 가중합 및 clamp 동작 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_stratification_gini(values)`
    - `body_stratification_status_score(scalar_inputs)`

- `scripts/systems/social/stratification_monitor.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - `_compute_gini`를 Rust-first 호출로 전환(fallback 유지).
  - `_compute_entity_status`의 status score 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 계층화 모니터의 반복 수식 계산(Gini, status)이 Rust 경로로 이동해 틱 연산 부담을 낮출 기반 확보.
- 브리지 비가용/실패 시 기존 GDScript 계산 경로를 유지해 동작 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `11/56` 적용, 잔여 `45/56`.
