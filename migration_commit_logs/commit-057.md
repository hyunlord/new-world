# Commit 057 - rebound queue 처리 Rust batch step 이관

## 커밋 요약
- C05 Denial 종료 후 delayed rebound 처리 루프를 Rust batch step으로 이관.
- `StressSystem._process_rebound_queue`는 입력 배열 구성 + 결과 반영만 수행하도록 정리.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressReboundQueueStep { total_rebound, remaining_amounts, remaining_delays }`
  - 신규 함수:
    - `stress_rebound_queue_step(amounts, delays, decay_per_tick)`
  - 동작:
    - delay 1틱 감소
    - amount decay 적용(0..1 clamp)
    - 만료(delay<=0) 합산, 잔여 항목 분리
  - 단위 테스트 2개 추가:
    - 만료/유지 분기 검증
    - decay 적용 및 decay rate clamp 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_rebound_queue_step(...) -> Dictionary`
  - `Vec<i32> -> PackedInt32Array` 변환 helper 추가
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_rebound_queue_step(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_rebound_queue_step(...)`
  - Rust 우선 + GDScript fallback 제공
- `scripts/systems/psychology/stress_system.gd`
  - scratch buffer 추가:
    - `_rebound_amounts`, `_rebound_delays`
  - `_process_rebound_queue(...)` 변경:
    - queue를 Packed 배열로 변환
    - `StatCurveScript.stress_rebound_queue_step(...)` 호출
    - 반환된 잔여 항목으로 rebound_queue 재구성
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에 `stress_rebound_queue_step` 호출 추가

## 기능 영향
- rebound queue 만료/합산 의미는 유지하면서 루프 계산이 Rust 수식 경로로 통일됨.
- 현재 설정(`REBOUND_DECAY_PER_TICK = 0.0`)에서는 기존과 동일 동작을 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (46 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=353.3`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
