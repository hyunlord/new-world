# Commit 066 - stress_tick_step packed output 경로 도입

## 커밋 요약
- stress tick 핵심 경로에서 dictionary key 기반 결과 파싱을 packed scalar/int 인덱스 기반으로 전환.
- tick hot path의 GDScript 문자열 조회 오버헤드를 완화.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_tick_step_packed(...)`
  - 반환 형식:
    - `scalars: PackedFloat32Array`
    - `ints: PackedInt32Array`
    - `updated_per_tick: PackedFloat32Array`
    - `active_mask: PackedByteArray`
  - 기존 `stat_stress_tick_step(...)`는 유지(호환성)
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_tick_step_packed(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_tick_step_packed(...)`
  - Rust 우선 + fallback(기존 dictionary 결과를 packed 형태로 변환) 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress(...)`
    - `stress_tick_step(...)` 대신 `stress_tick_step_packed(...)` 호출
    - packed 결과를 인덱스 상수(`_TICK_OUT_SC_*`, `_TICK_OUT_INT_*`)로 해석
    - stress state/meta 반영을 직접 packed 값으로 적용
  - 신규 helper:
    - `_packed_scalar(...)`
    - `_packed_int(...)`
  - `_update_stress_state`, `_apply_stress_to_emotions` 제거
  - emotion 기여 인덱스 매핑 상수 `_EMOTION_SCALAR_INDEX` 추가

## 기능 영향
- stress tick 결과 의미/클램프/상태 반영 동작은 유지.
- tick당 dictionary key lookup 감소로 stress hot path의 해석 비용을 완화.

## 검증
- `cd rust && cargo fmt -p sim-bridge -p sim-systems -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (51 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=455.1`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
