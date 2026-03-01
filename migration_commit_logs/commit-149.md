# Commit 149 - upper_needs 시스템 Rust 배치 스텝 도입

## 커밋 요약
- `upper_needs_system`의 decay/fulfillment/clamp hot path를 Rust 배치 스텝으로 이관하고, GDScript는 Rust 우선 + 기존 함수 fallback 구조로 전환.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 상위욕구 관련 함수 추가:
    - `upper_needs_best_skill_normalized(skill_levels, max_level)`
    - `upper_needs_job_alignment(job_code, craftsmanship, skill, hard_work, nature, independence)`
    - `upper_needs_step(...)` (8개 상위욕구 decay/충족/clamp 통합)
  - 보조 함수 `maxf32` 추가.
  - unit test 3개 추가(정규화/정합도/통합 스텝).

- `rust/crates/sim-bridge/src/lib.rs`
  - bridge export 추가:
    - `body_upper_needs_best_skill_normalized(PackedInt32Array, max_level)`
    - `body_upper_needs_job_alignment(...)`
    - `body_upper_needs_step_packed(scalar_inputs, flag_inputs)`
  - `body_upper_needs_step_packed`는 packed scalar/flag를 디코딩해 Rust 통합 스텝 호출 후 `PackedFloat32Array` 반환.

- `scripts/core/simulation/sim_bridge.gd`
  - 대응 wrapper 3개 추가:
    - `body_upper_needs_best_skill_normalized`
    - `body_upper_needs_job_alignment`
    - `body_upper_needs_step_packed`

- `scripts/systems/psychology/upper_needs_system.gd`
  - scratch packed 버퍼 추가/재사용:
    - `_upper_needs_scalar_inputs`(29), `_upper_needs_flag_inputs`(3), `_upper_needs_skill_levels`(5)
  - `execute_tick`가 `_apply_upper_needs_rust_step(entity)`를 먼저 시도하고 실패 시 기존 `_apply_decay/_apply_fulfillment/_clamp_upper_needs` fallback 사용.
  - `_apply_upper_needs_rust_step` 추가:
    - best skill norm, job alignment Rust 우선 계산
    - 통합 upper-needs step Rust 호출
    - 결과 8축을 entity 필드에 반영
  - `_get_job_code(job)` helper 추가(직업 문자열 → Rust 정합도 코드).

- `rust/crates/sim-test/src/main.rs`
  - `--bench-needs-math`에 upper-needs 함수 호출(`best_skill_normalized/job_alignment/upper_needs_step`)을 포함해 회귀 추적 범위를 확장.

## 기능 영향
- upper needs의 수치 의미(decay/충족/clamp)는 유지.
- 엔티티당 상위욕구 처리 경로가 Rust 배치 호출 중심으로 전환되어 스크립트 계산/분기 비용을 절감.
- bridge 미지원 시 기존 GDScript 로직으로 동일 동작 보장.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 79 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=489.7`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=179.7`, `checksum=29743414.00000` (upper-needs 벤치 항목 추가로 checksum 기준 업데이트)
