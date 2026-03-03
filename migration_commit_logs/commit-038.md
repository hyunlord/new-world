# Commit 038 - Stress trace batch 처리 Rust 이관

## 커밋 요약
- `stress_traces`의 per-tick decay/유지 판정을 Rust batch step으로 이관.
- GDScript는 trace 데이터를 packed 배열로 전달하고, 결과(mask/updated)를 반영하는 역할만 수행.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 타입:
    - `StressTraceBatchStep`
  - 신규 함수:
    - `stress_trace_batch_step(per_tick, decay_rate, min_keep)`
  - 동작:
    - 현재 contribution 총합 계산
    - decay 적용 후 갱신값 산출
    - `min_keep` 기준 active mask 생성
  - 단위 테스트 2개 추가:
    - 합계/active 판정 검증
    - 입력 길이 불일치 시 min 길이 처리 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_trace_batch_step(...) -> VarDictionary`
  - 반환:
    - `total_contribution`
    - `updated_per_tick` (PackedFloat32Array)
    - `active_mask` (PackedByteArray)
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_trace_batch_step(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_trace_batch_step(...)`
  - Rust 우선 + GDScript fallback 구현
- `scripts/systems/psychology/stress_system.gd`
  - `_process_stress_traces(...)`를 batch step 기반으로 전환
  - 기존 semantics 유지:
    - total은 모든 trace contribution 합
    - breakdown은 active trace만 기록
    - inactive trace는 제거

## 기능 영향
- trace 처리 루프의 핵심 수치 연산이 네이티브 batch 경로로 이동.
- stress trace 동작/의미는 기존과 동일.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (25 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
