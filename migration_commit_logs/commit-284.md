# Commit 284 - Stratification wealth/SettlementCulture plasticity Rust 브리지 이관

## 커밋 요약
- `stratification_monitor`의 wealth score 수식과 `settlement_culture`의 연령 plasticity 계산을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `stratification_wealth_score(...) -> f32`
  - 단위 테스트 추가:
    - 리소스 증가 시 wealth score 증가 동작 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_stratification_wealth_score(...)`

- `scripts/systems/social/stratification_monitor.gd`
  - SimBridge 메서드 상수 확장 (`body_stratification_wealth_score`).
  - `_compute_wealth_scores`를 Rust-first 호출로 전환(fallback 유지).

- `scripts/systems/social/settlement_culture.gd`
  - static SimBridge 캐시/조회 로직 추가.
  - `apply_conformity_pressure`의 plasticity 계산을 Rust-first(`body_value_plasticity`) 호출로 전환(fallback 유지).

## 기능 영향
- 정착지 계층화(wealth score)와 문화 동조 압력(plasticity) 연산 일부가 Rust 경로로 이전되어 반복 수식 계산 비용을 낮출 기반 확보.
- 브리지 실패 시 기존 GDScript 계산 경로를 유지해 동작 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `13/56` 적용, 잔여 `43/56`.
