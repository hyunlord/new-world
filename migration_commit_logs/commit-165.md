# Commit 165 - stress event 주입 후처리 Rust 이관

## 커밋 요약
- `stress_system`의 이벤트 주입 후처리(즉시 stress clamp + trace append 판정) 공통 수식을 Rust로 이관하고, 두 이벤트 경로에 Rust 우선 적용.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `stress_injection_apply_step(...) -> [next_stress, append_trace_flag]` 추가.
    - `stress + final_instant` clamp
    - `abs(final_per_tick) > trace_threshold` 판정
  - unit test 2개 추가:
    - threshold 초과 시 trace flag 설정
    - threshold 이하 시 trace flag 미설정

- `rust/crates/sim-bridge/src/lib.rs`
  - export 추가: `body_stress_injection_apply_step(...)`

- `scripts/core/simulation/sim_bridge.gd`
  - wrapper 추가: `body_stress_injection_apply_step`

- `scripts/systems/psychology/stress_system.gd`
  - `inject_stress_event(...)`에서 주입 후처리를 Rust 우선 사용.
  - `inject_event(...)`(stressor defs 기반 이벤트)에서도 동일하게 Rust 우선 사용.
  - bridge 미사용 시 기존 clamp/threshold fallback 유지.

- `rust/crates/sim-test/src/main.rs`
  - `--bench-stress-math`에 `stress_injection_apply_step` 호출 및 checksum 합산 항목 추가.

## 기능 영향
- 이벤트 주입 수치 의미(즉시 반영량 clamp, trace 생성 임계치)는 유지.
- 이벤트 경로 두 곳 모두에서 동일한 후처리 수식을 Rust 경로로 통일.
- fallback을 유지해 bridge 미사용 환경 동작 호환성 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=632.4`, `checksum=24032652.00000` (event injection apply 항목 포함으로 기준 업데이트)
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=418.4`, `checksum=38457848.00000`
