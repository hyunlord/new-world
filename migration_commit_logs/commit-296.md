# Commit 296 - IntelligenceCurves 보정 수식 Rust 브리지 이관

## 커밋 요약
- `intelligence_curves`의 activity/ACE 보정 수식을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `cognition_activity_modifier(active_skill_count, activity_buffer, inactivity_accel) -> f32`
    - `cognition_ace_fluid_decline_mult(ace_penalty, ace_penalty_minor, ace_fluid_decline_mult) -> f32`
  - 단위 테스트 추가:
    - activity count 기반 분기 검증
    - ACE threshold 게이트 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_cognition_activity_modifier(...)`
    - `body_cognition_ace_fluid_decline_mult(...)`

- `scripts/systems/cognition/intelligence_curves.gd`
  - static SimBridge 캐시/조회 로직 추가.
  - `get_activity_modifier`를 Rust-first 호출로 전환(fallback 유지).
  - `get_ace_fluid_decline_mult`를 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 인지 곡선 보정 수식 일부가 Rust 경로로 이동해 반복 계산 비용을 낮출 기반 확보.
- 브리지 실패 시 기존 GDScript 경로를 유지해 동작 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `25/56` 적용, 잔여 `31/56`.
