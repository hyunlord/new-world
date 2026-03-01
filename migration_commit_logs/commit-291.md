# Commit 291 - TechDiscoverySystem 확률 수식 Rust 브리지 이관

## 커밋 요약
- `tech_discovery_system`의 연간→주기 확률 변환 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `tech_discovery_prob(base, pop_bonus, knowledge_bonus, openness_bonus, logical_bonus, naturalistic_bonus, soft_bonus, rediscovery_bonus, max_bonus, checks_per_year) -> f32`
  - 단위 테스트 추가:
    - annual probability를 per-check probability로 변환하는 경로 검증
    - checks_per_year<=1 구간 clamp 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_tech_discovery_prob(...)`

- `scripts/systems/world/tech_discovery_system.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - `_compute_discovery_prob`의 최종 확률 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 기술 발견 확률의 핵심 수식이 Rust 경로로 이동해 주기별 반복 계산 비용을 낮출 기반 확보.
- 브리지 실패 시 기존 GDScript 계산 경로를 유지해 동작 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `20/56` 적용, 잔여 `36/56`.
