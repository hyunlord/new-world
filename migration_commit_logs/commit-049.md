# Commit 049 - stress tick scratch buffer 재사용 최적화

## 커밋 요약
- `StressSystem`에서 엔티티마다 새 `Packed*Array`를 생성하던 경로를 scratch buffer 재사용 방식으로 변경.
- 단일 `stress_tick_step` 호출 구조는 유지하면서 GDScript 할당/GC 오버헤드를 줄이도록 정리.

## 상세 변경
- `scripts/systems/psychology/stress_system.gd`
  - 신규 상수:
    - `_TICK_SCALAR_LEN = 40`
    - `_TICK_FLAG_LEN = 3`
  - 신규 멤버 scratch 버퍼:
    - `_tick_scalar_inputs: PackedFloat32Array`
    - `_tick_flags: PackedByteArray`
    - `_tick_trace_per_tick: PackedFloat32Array`
    - `_tick_trace_decay: PackedFloat32Array`
  - `_update_entity_stress` 변경:
    - trace 입력 배열과 scalar/flag 입력 배열을 매 tick 엔티티마다 새로 생성하지 않고 resize + 인덱스 갱신 방식으로 재사용
    - `StatCurveScript.stress_tick_step(...)` 호출은 동일하되 입력 버퍼는 재사용 객체 사용
    - trace breakdown/active 갱신/감정 breakdown/상태 반영 로직은 기존 의미 유지

## 기능 영향
- stress 틱 루프에서 PackedArray 생성 횟수 감소로 GDScript 메모리 할당 부담 완화.
- 수식/상태 전이/breakdown 동작은 기존과 동일.

## 검증
- `cd rust && cargo test -q -p sim-systems` 통과 (37 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=240.9`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
