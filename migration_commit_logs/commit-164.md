# Commit 164 - child stage 판정 수식 Rust 이관

## 커밋 요약
- `child_stress_processor.get_current_stage`의 연령 구간(stage) 판정 수식을 Rust로 이관하고, stage cutoff 캐시를 도입해 반복 조회 비용을 줄임.

## 상세 변경
- `rust/crates/sim-systems/src/body.rs`
  - `child_stage_code_from_age_ticks(...) -> i32` 추가.
    - 입력: age_ticks + 4개 stage cutoff(유아/유년/아동/청소년)
    - 출력: `0=infant, 1=toddler, 2=child, 3=teen, 4=adult`
  - unit test 1개 추가(구간별 코드 판정 검증).

- `rust/crates/sim-bridge/src/lib.rs`
  - export 추가: `body_child_stage_code_from_age_ticks(...)`

- `scripts/core/simulation/sim_bridge.gd`
  - wrapper 추가: `body_child_stage_code_from_age_ticks`

- `scripts/systems/development/child_stress_processor.gd`
  - `_stage_age_cutoffs` 캐시(`PackedFloat32Array`) 추가.
  - `_refresh_stage_age_cutoffs()` 추가:
    - `developmental_stages.json`의 `age_range` 최대값을 stage cutoff 캐시에 반영.
    - 데이터가 없으면 기본값(2/5/12/18) 유지.
  - `_load_stages()`에서 cutoff refresh 호출.
  - `get_current_stage()`에서 Rust bridge(stage code) 우선 사용, fallback은 기존 임계 비교 유지.
  - helper 추가: `_stage_name_from_code(...)`.

- `rust/crates/sim-test/src/main.rs`
  - `--bench-needs-math`에 `child_stage_code_from_age_ticks` 호출 및 checksum 합산 항목 추가.

## 기능 영향
- stage 판정 의미(연령 구간 매핑)는 유지.
- stage cutoff를 캐시해 stage 데이터 파싱/순회 비용을 감소.
- bridge 미사용 시에도 fallback으로 기존 구간 비교 동작 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 96 tests)
  - localization compile `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=804.9`, `checksum=20039734.00000`
- `cd rust && cargo run -q -p sim-test -- --bench-needs-math --iters 10000`
  - `ns_per_iter=602.6`, `checksum=38457848.00000` (stage code 항목 포함으로 기준 업데이트)
