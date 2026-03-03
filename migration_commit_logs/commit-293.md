# Commit 293 - TechMaintenanceSystem memory decay Rust 브리지 이관

## 커밋 요약
- `tech_maintenance_system`의 cultural_memory 감소 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `tech_cultural_memory_decay(current_memory, base_decay, forgotten_long_multiplier, memory_floor, forgotten_recent) -> f32`
  - 단위 테스트 추가:
    - forgotten_recent/forgotten_long 상태별 decay 차이 검증
    - floor 하한 유지 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_tech_cultural_memory_decay(...)`

- `scripts/systems/world/tech_maintenance_system.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - `_decay_cultural_memory` 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 기술 유지 시스템의 메모리 감쇠 수식이 Rust 경로로 이동해 반복 계산 비용을 낮출 기반 확보.
- 브리지 실패 시 기존 GDScript 경로 유지로 런타임 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `22/56` 적용, 잔여 `34/56`.
