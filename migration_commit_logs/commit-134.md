# Commit 134 - training gain 배치 결과 PackedArray 경로 정리

## 커밋 요약
- `calc_training_gain_batch`의 내부 경로를 `PackedInt32Array` 기반으로 분리(`calc_training_gain_packed`)해 fallback 경로의 Dictionary 생성/키 조회를 줄이고, realized fallback 계산도 packed 결과를 직접 사용하도록 정리.

## 상세 변경
- `scripts/core/entity/body_attributes.gd`
  - `calc_training_gain_packed() -> PackedInt32Array` 추가.
    - bridge 성공 시 Rust 배치 결과 packed 배열 반환.
    - bridge 미지원/실패 시 단건 `calc_training_gain(axis)` 결과를 packed 배열로 생성.
  - 기존 `calc_training_gain_batch()`는 호환용 wrapper로 유지하고 packed 결과를 Dictionary로 변환.
  - `calc_realized_values_packed()` fallback 경로에서 gain 조회를 Dictionary 대신 `calc_training_gain_packed()` 인덱스 접근으로 변경.

## 기능 영향
- training gain/realized 계산 의미와 결과는 기존과 동일.
- bridge fallback 경로에서도 딕셔너리 생성/문자열 키 조회 비용을 줄인 packed 처리로 정리.
- 기존 Dictionary API는 유지되어 호환성 보존.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 65 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=447.4`, `checksum=13761358.00000`
