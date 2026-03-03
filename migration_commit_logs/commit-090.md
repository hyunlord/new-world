# Commit 090 - HUD needs 정규화 배치 조회 전환

## 커밋 요약
- `HUD` 엔티티 패널의 needs 표시 갱신에서 개별 `StatQuery.get_normalized` 반복 호출을 배치 조회 1회로 통합.

## 상세 변경
- `scripts/ui/hud.gd`
  - `_ENTITY_NEED_STAT_IDS` 상수 추가:
    - `NEED_HUNGER/ENERGY/SOCIAL/THIRST/WARMTH/SAFETY` 고정 조회 목록 정의.
  - `_entity_need_norm_values` scratch `PackedFloat32Array` 추가:
    - 엔티티 패널 갱신 시 정규화 결과 버퍼 재사용.
  - `_update_entity_panel()` 변경:
    - `StatQuery.get_normalized_batch_into(entity, _ENTITY_NEED_STAT_IDS, _entity_need_norm_values, true)` 호출로 needs 6종 정규화 값을 한 번에 수집.
    - 각 progress bar/percent 라벨은 버퍼 인덱스 값을 사용하도록 전환.
    - 저배고픔 깜빡임 판정은 추가 조회 없이 `hunger_norm` 재사용.

## 기능 영향
- HUD 엔티티 패널의 시각 출력(수치/바/깜빡임)은 기존 의미와 동일.
- 선택 엔티티 갱신 루프에서 `StatQuery` 호출 수를 줄이고 임시 할당을 재사용해 UI tick 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=447.2`, `checksum=13761358.00000`
