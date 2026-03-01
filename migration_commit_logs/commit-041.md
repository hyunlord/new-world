# Commit 041 - stress delta/denial step Rust 이관

## 커밋 요약
- `StressSystem` 틱 핫패스에서 최종 delta 계산 + denial redirect 처리 로직을 Rust로 이관.
- 기존 stress 파이프라인(continuous/trace/emotion/recovery) 결과를 받아 최종 적용값만 네이티브 계산하도록 연결.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 구조체:
    - `StressDeltaStep { delta, hidden_threat_accumulator }`
  - 신규 함수:
    - `stress_delta_step(...) -> StressDeltaStep`
  - 반영 규칙:
    - `delta = (continuous + trace + emotion) * ace_mult * trait_mult - recovery`
    - `abs(delta) < epsilon`이면 0 처리
    - denial 활성 + `delta > 0`이면 redirect fraction만큼 hidden accumulator로 이동
    - hidden accumulator는 `denial_max_accumulator`로 clamp
  - 단위 테스트 2개 추가:
    - epsilon 기반 0 처리 검증
    - denial redirect + cap 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_delta_step(...) -> Dictionary`
  - 반환 키:
    - `delta`
    - `hidden_threat_accumulator`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_delta_step(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_delta_step(...)`
  - Rust 우선 + 기존 GDScript fallback 수식 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_update_entity_stress`의 delta/denial 블록을 `StatCurveScript.stress_delta_step(...)` 호출로 전환
  - denial 활성 시 반환된 `hidden_threat_accumulator`를 meta에 반영
- `rust/crates/sim-test/src/main.rs`
  - `--bench-stress-math` 루프에 `stress_delta_step` 호출을 추가해 신규 네이티브 경로도 측정 범위에 포함

## 기능 영향
- entity 당 매 tick 수행되는 stress 최종 갱신 분기(ACE/trait/recovery/denial)가 네이티브 수학 경로로 이동.
- 숨김 위협 누적치(hidden accumulator)의 clamp/redirect 의미는 기존과 동일하게 유지.
- stress 벤치에서 delta step까지 포함한 end-to-end 연산량 측정 가능.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (29 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=81.3`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
