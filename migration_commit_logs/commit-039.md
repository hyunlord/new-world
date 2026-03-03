# Commit 039 - Resilience 업데이트 수식 Rust 이관

## 커밋 요약
- `StressSystem._update_resilience`의 핵심 수식을 Rust로 이관.
- trauma scar 모디파이어는 기존처럼 GDScript에서 계산해 Rust 함수 입력으로 전달.

## 상세 변경
- `rust/crates/sim-systems/src/stat_curve.rs`
  - 신규 함수:
    - `stress_resilience_value(...) -> f32`
  - 입력:
    - HEXACO 축 6개, support, allostatic, hunger/energy, scar 모디파이어
  - 기존 규칙 반영:
    - 축 가중치 합산
    - allostatic 패널티
    - hunger/energy fatigue 패널티
    - scar 모디파이어 가산
    - clamp `[0.05, 1.0]`
  - 단위 테스트 2개 추가:
    - clamp 범위 검증
    - fatigue 상태에서 resilience 하락 검증
- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 Godot 함수:
    - `stat_stress_resilience_value(...) -> f32`
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 프록시:
    - `stat_stress_resilience_value(...)`
- `scripts/core/stats/stat_curve.gd`
  - 신규 함수:
    - `stress_resilience_value(...)`
  - Rust 우선 + 기존 GDScript fallback 수식 제공
- `scripts/systems/psychology/stress_system.gd`
  - `_update_resilience(...)`가 `StatCurveScript.stress_resilience_value(...)` 호출로 전환

## 기능 영향
- stress 파이프라인에서 resilience 계산도 네이티브 수학 경로로 이동.
- scar 기반 보정은 기존 흐름과 동일하게 유지.

## 검증
- `cd rust && cargo fmt -p sim-systems -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-systems` 통과 (27 tests)
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
  - strict audit: inline localized fields 0 유지
