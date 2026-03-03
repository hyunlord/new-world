# Commit 161 - stress support score 수식 Rust 이관

## 커밋 요약
- `stress_system._calc_support_score`의 핵심 지원 점수 수식을 Rust로 이관하고, GDScript는 Rust 우선 + 기존 fallback 구조로 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `stress_support_score(strengths: &[f32]) -> f32` 추가.
    - strongest tie + weak ties 누적 포화(`1 - exp(-weak_sum / 1.5)`) 조합 수식 이관.
    - 입력 strength를 `[0, 1]`로 clamp.
    - 빈 입력 baseline `0.3` 유지.
  - unit test 2개 추가:
    - empty 입력 baseline 검증
    - strong tie 우선 반영 shape 검증

- `rust/crates/sim-bridge/src/lib.rs`
  - export 추가: `body_stress_support_score(strengths: PackedFloat32Array) -> f32`

- `scripts/core/simulation/sim_bridge.gd`
  - wrapper 추가: `body_stress_support_score(strengths: PackedFloat32Array)`

- `scripts/systems/psychology/stress_system.gd`
  - `_calc_support_score`에서 관계 strength를 `PackedFloat32Array`로 구성 후 Rust bridge 우선 호출.
  - bridge 미사용 시 기존 GDScript 수식 fallback 유지.

- `rust/crates/sim-test/src/main.rs`
  - `--bench-stress-math`에 `body::stress_support_score(...)` 호출 및 checksum 합산 항목 추가.

## 기능 영향
- support score 계산식의 수치 의미(강한 유대 + 약한 유대 포화 결합)는 유지.
- bridge 사용 가능 환경에서 support score 계산이 Rust 경로를 사용.
- bridge 미사용 환경에서도 기존 fallback으로 동일 동작 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 91 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=646.7`, `checksum=13767388.00000` (support score 항목 포함으로 기준 업데이트)
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=445.1`, `checksum=33378700.00000`
