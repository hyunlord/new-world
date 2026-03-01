# Commit 131 - Entity spawn realized 초기화 경로 단일 배치 통합

## 커밋 요약
- 엔티티 스폰 시 `body.realized` 초기화 경로를 `calc_realized_values_batch` 단일 호출로 전환해 연간 갱신 경로와 동일한 Rust 통합 경로를 사용.

## 상세 변경
- `scripts/core/entity/entity_manager.gd`
  - 기존 `compute_age_curve_batch` 결과를 축별로 읽어 계산하던 초기화 루프를 제거.
  - `entity.body.calc_realized_values_batch(_body_age_y)` 1회 호출 결과(`str/agi/end/tou/rec/dr`)를 그대로 반영하도록 변경.

## 기능 영향
- 스폰 직후 realized 산식 의미는 기존과 동일.
- bridge 지원 환경에서 스폰 초기화 시 body 관련 bridge 호출이 단일 realized 호출 경로로 정렬됨.
- bridge 미지원 환경은 `calc_realized_values_batch` 내부 fallback 경로로 기존 계산과 동일하게 동작.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 65 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=453.1`, `checksum=13761358.00000`
