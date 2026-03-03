# Commit 289 - ResourceRegenSystem 수식 Rust 브리지 이관

## 커밋 요약
- `resource_regen_system`의 리소스 리젠 수식(`min(current + rate, cap)`)을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `resource_regen_next(current, cap, rate) -> f32`
  - 단위 테스트 추가:
    - cap 적용 및 guard(cap/rate/current 조건) 동작 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_resource_regen_next(...)`

- `scripts/systems/world/resource_regen_system.gd`
  - SimBridge 연결 캐시/조회 로직 추가.
  - food/wood 리젠 계산을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 월드 자원 재생의 반복 수식 계산을 Rust 경로로 이동해 타일 순회 루프의 계산 부담을 줄일 기반 확보.
- 브리지 실패 시 기존 GDScript 계산 경로 유지로 런타임 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `18/56` 적용, 잔여 `38/56`.
