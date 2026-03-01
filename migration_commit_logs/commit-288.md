# Commit 288 - TensionSystem 수식 Rust 브리지 이관

## 커밋 요약
- `tension_system`의 scarcity pressure 계산과 tension 누적/감쇠 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `tension_scarcity_pressure(s1_deficit, s2_deficit, per_shared_resource) -> f32`
    - `tension_next_value(current, scarcity_pressure, decay_per_year, dt_years) -> f32`
  - 단위 테스트 추가:
    - deficit 조합별 pressure 계산 검증
    - decay/clamp 포함 next tension 계산 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_tension_scarcity_pressure(...)`
    - `body_tension_next_value(...)`

- `scripts/systems/world/tension_system.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - `_update_pair_tension`의 scarcity pressure 계산을 Rust-first 호출로 전환(fallback 유지).
  - `_update_pair_tension`의 다음 tension 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 정착지 간 긴장도 계산의 핵심 수식이 Rust 경로로 이동해 반복 계산 비용을 낮출 기반 확보.
- 브리지 실패 시 기존 GDScript 경로를 유지해 시뮬레이션 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `17/56` 적용, 잔여 `39/56`.
