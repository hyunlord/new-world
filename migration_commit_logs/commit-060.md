# Commit 060 - inject_event scale+emotion 결합 Rust step 도입

## 커밋 요약
- `inject_event` 경로에서 event scale 계산과 emotion layer 반영을 단일 Rust step으로 결합.
- 이벤트 주입 경로 bridge 호출을 추가로 축소.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressEventInjectStep`
  - 신규 함수:
    - `stress_event_inject_step(...)`
  - 동작:
    - `stress_event_scale_step` + `stress_emotion_inject_step` 조합을 단일 step으로 제공
  - 단위 테스트 1개 추가:
    - 결합 step 결과와 분리 step 조합 결과 일치 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_event_inject_step(...) -> Dictionary`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_event_inject_step(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_event_inject_step(...)`
  - Rust 우선 + fallback(결합 전 함수 조합) 제공
- `scripts/systems/psychology/stress_system.gd`
  - `inject_event(...)` 변경:
    - `stress_event_scale_step` + `_inject_emotions` 분리 호출을 `stress_event_inject_step` 단일 호출로 치환
    - 결과 fast/slow 배열을 `_apply_event_emotion_layers(...)`로 반영
  - 신규 helper:
    - `_fill_event_emotion_current(...)`
    - `_apply_event_emotion_layers(...)`
  - 기존 `_inject_emotions(...)` 제거
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에 `stress_event_inject_step` 호출 추가

## 기능 영향
- 이벤트 stress 주입의 최종 scale 및 emotion layer 반영 의미는 유지.
- inject_event 경로의 네이티브 결합도가 높아져 GDScript 호출/조립 비용이 감소.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (48 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=484.3`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
