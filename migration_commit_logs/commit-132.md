# Commit 132 - realized 배치 결과를 PackedArray 경로로 최적화

## 커밋 요약
- `realized` 6축 배치 계산 결과를 Dictionary가 아닌 `PackedInt32Array`로 직접 사용하는 경로를 추가해 문자열 키 조회/딕셔너리 할당 오버헤드를 줄임.

## 상세 변경
- `scripts/core/entity/body_attributes.gd`
  - `calc_realized_values_packed(age_years) -> PackedInt32Array` 추가.
    - bridge 성공 시 Rust 결과 packed 배열을 그대로 반환.
    - bridge 미지원/실패 시 기존 fallback 수식으로 packed 배열(6축) 구성.
  - 기존 `calc_realized_values_batch(age_years)`는 호환용 wrapper로 유지하고, 내부에서 `calc_realized_values_packed` 결과를 Dictionary로 변환하도록 변경.

- `scripts/systems/biology/age_system.gd`
  - 연간 `realized` 갱신 경로를 `calc_realized_values_batch`(Dictionary)에서 `calc_realized_values_packed`로 전환.
  - 5축 루프에서 axis 배열 인덱스로 packed 값을 읽고, `dr`는 인덱스 5를 직접 사용.

- `scripts/core/entity/entity_manager.gd`
  - 스폰 초기 `realized` 반영 경로를 packed 기반 인덱스 접근으로 전환.

## 기능 영향
- `realized` 계산 값/클램프/변경 이벤트 조건은 기존과 동일.
- `age_system`/`entity_manager`에서 딕셔너리 생성 및 문자열 키 조회 비용이 줄어든 경로로 실행.
- 기존 Dictionary API(`calc_realized_values_batch`)는 유지되어 호환성 보존.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 65 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=500.1`, `checksum=13761358.00000`
