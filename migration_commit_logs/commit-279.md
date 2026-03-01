# Commit 279 - NetworkSystem 핵심 수식 Rust 브리지 이관

## 커밋 요약
- `network_system`의 핵심 수식(사회자본 정규화, 혁명 위험 합성)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `network_social_capital_norm(...)`
    - `revolution_risk_score(...)`
  - 단위 테스트 추가:
    - social capital 수식/정규화 검증
    - revolution risk 평균 합성 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_network_social_capital_norm(...)`
    - `body_revolution_risk_score(...)`

- `scripts/systems/social/network_system.gd`
  - SimBridge 연결 캐시 추가.
  - `_compute_entity_social_capital`를 Rust-first 호출로 전환(fallback 유지).
  - `_compute_revolution_risk`를 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 사회자본/혁명위험의 반복 계산 수식이 Rust 경로로 이동.
- 브리지 부재/호출 실패 시 기존 GDScript 수식을 유지해 런타임 안정성을 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `8/56` 적용, 잔여 `48/56`.
