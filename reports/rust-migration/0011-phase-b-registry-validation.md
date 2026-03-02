# 0011 - Phase B runtime registry upsert and validation

## Summary
Rust runtime 시스템 레지스트리를 `append`에서 `upsert` 구조로 보강하고, GDScript 등록 정보와 Rust registry 스냅샷의 정합성을 부팅 시점에 검증하도록 연결했다.

## Files Changed
- `rust/crates/sim-bridge/src/lib.rs`
  - `runtime_apply_commands_v2`의 `register_system` 처리 로직 개선
    - 동일 name 재등록 시 기존 엔트리 갱신(upsert)
    - 등록 후 priority 기준 정렬 유지
- `scripts/core/simulation/simulation_engine.gd`
  - 시스템 등록 메타데이터 로컬 추적 추가
    - `_registered_system_count`
    - `_registered_system_payloads`
  - `register_system()`에서 runtime payload를 공통 생성/저장 후 전송
  - `validate_runtime_registry()` 추가
    - runtime snapshot과 expected registry count/order 비교
    - mismatch 시 warning 출력
- `scenes/main/main.gd`
  - 전체 시스템 등록 직후 `sim_engine.validate_runtime_registry()` 호출 추가

## API / Signal / Schema Changes
### SimulationEngine API
- Added: `validate_runtime_registry() -> Dictionary`
  - 반환: `runtime_available`, `expected_count`, `runtime_count`, `count_match`, `order_match`

### Runtime command semantics
- `register_system`
  - 기존: 동일 name도 중복 추가
  - 변경: 동일 name upsert + priority 정렬

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `cd rust && cargo test -p sim-bridge --lib` : PASS (26 passed)
- `cd rust && cargo test -p sim-engine --lib` : PASS (21 passed)
- Godot headless check: 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 84% complete / 16% remaining
- Current: 87% complete / 13% remaining
- Delta: +3%

## Notes
- 이번 단계는 "등록 정합성" 검증/보강이다.
- 실제 시스템 실행 로직 자체를 Rust 단일 경로로 완전히 이전하는 작업은 후속 단계(실행 컷오버)로 남아 있다.
