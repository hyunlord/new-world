# Commit 065 - relationship method code 기반 event step 경로 전환

## 커밋 요약
- inject_event 경로의 relationship method 전달을 문자열 대신 method code(enum) 기반으로 전환.
- 브리지 경계에서 문자열 변환/비교 비용을 줄이고 code-path API를 추가.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 관계 method 코드 상수 추가:
    - `RELATIONSHIP_METHOD_NONE = 0`
    - `RELATIONSHIP_METHOD_BOND_STRENGTH = 1`
  - 신규 함수:
    - `stress_relationship_scale_code(...)`
    - `stress_event_scale_step_code(...)`
    - `stress_event_inject_step_code(...)`
  - 기존 string 기반 함수는 유지(호환성)
  - 단위 테스트 3개 추가:
    - relationship code/string 동등성
    - event scale step code/string 동등성
    - event inject step code/string 동등성
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_event_scale_step_code(...)`
    - `stat_stress_event_inject_step_code(...)`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_event_scale_step_code(...)`
    - `stat_stress_event_inject_step_code(...)`
- `scripts/core/stats/stat_curve.gd`
  - 관계 method code 상수 추가
  - 신규 함수:
    - `stress_event_scale_step_code(...)`
    - `stress_event_inject_step_code(...)`
  - Rust 우선 + fallback(string 경로 재사용) 제공
- `scripts/systems/psychology/stress_system.gd`
  - relationship 컴파일 결과에 `_r_method_code` 저장
  - `inject_event(...)`가 code 기반 event step API를 사용하도록 전환
- `rust/crates/sim-test/src/main.rs`
  - stress 벤치에 code 기반 step 호출 추가

## 기능 영향
- relationship scaling 의미는 동일 유지.
- 이벤트 주입 경로에서 문자열 기반 method 전달 오버헤드를 완화.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge -p sim-test` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (51 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `cd rust && cargo test -q -p sim-test` 통과
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000` 통과
  - 예시 출력: `ns_per_iter=455.8`
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
