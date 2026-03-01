# Commit 154 - upper_needs Rust 경로 fallback 지연 계산

## 커밋 요약
- `upper_needs_system`에서 Rust bridge가 사용 가능한 경우에도 fallback 계산을 선행하던 비효율을 제거하고, fallback은 bridge 미지원일 때만 계산하도록 정리.

## 상세 변경
- `scripts/systems/psychology/upper_needs_system.gd`
  - `_apply_upper_needs_rust_step`에서:
    - `best_skill_norm` 초기값을 `0.0`으로 두고,
      - Rust 결과가 있으면 즉시 사용
      - 없을 때만 `_get_best_skill_normalized(entity)` fallback 계산
    - `alignment`도 동일하게,
      - Rust 결과가 있으면 사용
      - 없을 때만 `_get_job_value_alignment(entity)` fallback 계산

## 기능 영향
- upper needs 수치 결과는 기존과 동일.
- Rust path 활성 시 불필요한 GDScript 계산(최고 스킬 스캔/가치정합 계산) 제거로 tick당 오버헤드 감소.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 79 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=532.1`, `checksum=13761358.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=196.4`, `checksum=29743414.00000`
