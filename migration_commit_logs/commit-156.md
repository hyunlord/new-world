# Commit 156 - child simultaneous ACE burst 계산 Rust 이관

## 커밋 요약
- `child_stress_processor`의 동시 ACE 이벤트 누적 계산(`effective_damage/max_severity/kindling`)을 Rust 함수로 이관하고, GDScript는 Rust 우선 + 기존 fallback 구조로 연결.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `child_simultaneous_ace_step(severities, prev_residual)` 추가.
    - 출력: `[effective_damage, max_severity_index, kindling_bonus]`
    - severity clamp, burst 결합, residual 반영, 최종 clamp(0~1.25) 수행.
  - unit test 2개 추가(정상 입력 shape/empty 입력 처리).

- `rust/crates/sim-bridge/src/lib.rs`
  - `body_child_simultaneous_ace_step(prev_residual, severities)` export 추가.

- `scripts/core/simulation/sim_bridge.gd`
  - `body_child_simultaneous_ace_step(...)` wrapper 추가.

- `scripts/systems/development/child_stress_processor.gd`
  - scratch 버퍼 추가:
    - `_simultaneous_ace_severities`
    - `_simultaneous_ace_scar_candidates`
  - `simultaneous_ace_events` 처리 구간을 Rust 우선 경로로 전환:
    - 이벤트 severity/scar 후보를 packed/array로 구성
    - Rust 결과로 `effective_damage`, `kindling_bonus`, `scar_candidate` 복원
    - Rust 미지원 시 기존 `_handle_simultaneous_ace_events` fallback 유지

## 기능 영향
- 동시 ACE 누적 의미(버스트 기반 피해, scar 후보, kindling bonus)는 기존과 동일.
- child stress tick의 해당 수학 경로가 Rust 실행으로 이동.
- bridge 미지원 환경은 기존 fallback으로 동작 보장.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 83 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=684.9`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=186.9`, `checksum=29743414.00000`
