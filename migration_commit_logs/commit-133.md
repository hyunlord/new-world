# Commit 133 - trainability 배치 결과 PackedArray 경로 최적화

## 커밋 요약
- 나이 기반 trainability 배치 결과를 Dictionary 대신 `PackedFloat32Array`로 직접 사용하는 경로를 추가하고, 채집/건설 XP 누적 경로를 packed 인덱스 조회로 전환.

## 상세 변경
- `scripts/core/entity/body_attributes.gd`
  - `get_age_trainability_modifier_packed(age_years) -> PackedFloat32Array` 추가.
    - bridge 성공 시 Rust 결과 packed 배열 반환.
    - bridge 미지원/실패 시 기존 단건 함수로 fallback packed 배열 생성.
  - 기존 `get_age_trainability_modifier_batch(age_years)`는 호환용 wrapper로 유지하고 packed 결과를 Dictionary로 변환하도록 변경.

- `scripts/systems/work/gathering_system.gd`
  - trainability 배치를 Dictionary에서 PackedFloat32Array로 변경.
  - 축 인덱스 매핑(`str=0, agi=1, end=2, tou=3, rec=4`) 기반으로 XP 배수 조회.

- `scripts/systems/work/construction_system.gd`
  - 완공 보너스 trainability 조회를 packed 인덱스 접근으로 전환(`str=0`, `agi=1`).

## 기능 영향
- trainability 배수 수치 및 XP 누적 의미는 기존과 동일.
- work 시스템 경로에서 딕셔너리 생성/키 조회 비용을 줄인 packed 처리로 전환.
- 기존 Dictionary API는 유지되어 호환성 보존.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 65 tests)
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=458.3`, `checksum=13761358.00000`
