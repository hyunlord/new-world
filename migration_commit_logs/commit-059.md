# Commit 059 - inject_event 스케일 계산 결합 Rust step 도입

## 커밋 요약
- `inject_event`에서 분리 호출되던 relationship/context/event scaling 계산을 단일 Rust step으로 결합.
- 이벤트 경로의 bridge round-trip을 추가로 축소.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressEventScaleStep`
  - 신규 함수:
    - `stress_event_scale_step(...)`
  - 동작:
    - relationship scale 계산
    - context scale 계산
    - event final scale/instant/per_tick 계산
  - 단위 테스트 1개 추가:
    - 분리 함수 조합 결과와 결합 step 결과 일치 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_event_scale_step(...) -> Dictionary`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_event_scale_step(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_event_scale_step(...)`
  - Rust 우선 + fallback(relationship/context/event_scaled 조합) 제공
- `scripts/systems/psychology/stress_system.gd`
  - `inject_event(...)` 변경:
    - 기존 `_calc_relationship_scale` + `_calc_context_scale` + `stress_event_scaled` 분리 경로 제거
    - `stress_event_scale_step(...)` 단일 호출로 치환
    - context는 `_collect_active_context_multipliers(...)`로 활성 multiplier만 수집
  - `_calc_relationship_scale`, `_calc_context_scale` 제거
  - 신규 helper:
    - `_collect_active_context_multipliers(...)`
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에 `stress_event_scale_step` 호출 추가

## 기능 영향
- 이벤트 최종 스케일 계산 의미는 유지하면서 inject_event 경로의 네이티브 호출 결합도가 높아짐.
- 디버그 출력에 쓰는 relationship/context scale 값도 결합 step 결과를 사용.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (47 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=342.5`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
