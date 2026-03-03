# Commit 300 - JobAssignment 선택/재균형 수식 Rust 브리지 이관

## 커밋 요약
- `job_assignment_system`의 직업 deficit 선택 및 재균형 대상 계산을 Rust-first 경로로 이관.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - 신규 함수 추가:
    - `job_assignment_best_job_code(ratios, counts, alive_count) -> i32`
    - `job_assignment_rebalance_codes(ratios, counts, alive_count, threshold) -> [i32; 2]`
  - 단위 테스트 추가:
    - `job_assignment_best_job_code_picks_largest_deficit`
    - `job_assignment_rebalance_codes_respects_threshold`

- `rust/crates/sim-bridge/src/lib.rs`
  - 신규 GDExtension 메서드 추가:
    - `body_job_assignment_best_job_code(...)`
    - `body_job_assignment_rebalance_codes(...)`

- `scripts/systems/work/job_assignment_system.gd`
  - SimBridge 캐시/조회 로직 추가.
  - `_find_most_needed_job`를 Rust-first 호출로 전환(fallback 유지).
  - `_rebalance_jobs`의 surplus/deficit 선택을 Rust-first 호출로 전환(fallback 유지).

## 기능 영향
- 인구/직업 분배 반복 루프에서 deficit/surplus 선택 계산이 Rust 경로로 이동.
- 브리지 호출 실패 시 기존 GDScript 계산 경로를 유지해 동작 안정성 보장.

## 검증
- `cd rust && cargo test -q` 통과.
- `cd rust && cargo run -q -p sim-test` 통과 (`[sim-test] PASS`).

## Rust 전환 잔여량(이번 청크 기준)
- **데이터 로더 축(R-1 core loader set)**: `9/9` 완료, 잔여 `0/9`.
- **시스템 실행 축(브리지 적용, scripts/systems 기준)**: `29/56` 적용, 잔여 `27/56`.
