# Commit 281 - EconomicTendencySystem 수식 Rust 브리지 이관

## 커밋 요약
- `economic_tendency_system`의 핵심 경제 성향 수식 4종(`saving/risk/generosity/materialism`)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `economic_tendencies_step(...) -> [f32; 4]`
    - 내부 보조 함수 `bipolar(...)`
  - 단위 테스트 추가:
    - 출력 범위 `[-1.0, 1.0]` bounded 검증
    - 성별 risk 보정 및 고자산 generosity penalty 적용 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_economic_tendencies_step(scalar_inputs, is_male, wealth_generosity_penalty)`

- `scripts/systems/social/economic_tendency_system.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - `_compute_tendencies`를 Rust-first 호출로 전환하고, 브리지 실패 시 기존 GDScript 수식 fallback 유지.

## 기능 영향
- 경제 성향 연산의 반복 계산을 Rust 경로로 이전해 tick 당 연산 부담을 낮출 기반 확보.
- 브리지 비가용/호출 실패 시 기존 경로로 안전하게 복귀해 런타임 안정성을 유지.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `10/56` 적용, 잔여 `46/56`.
