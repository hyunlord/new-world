# Commit 093 - Locale 경량 포맷 API 도입 및 HUD 루프 적용

## 커밋 요약
- `Locale`에 단일/이중 placeholder 전용 포맷 API를 추가하고 HUD의 프레임 루프 포맷 호출을 해당 경로로 치환.

## 상세 변경
- `scripts/core/simulation/locale.gd`
  - `trf1(key, param_key, param_value)` 추가:
    - 1개 placeholder 치환 전용 경량 경로.
  - `trf2(key, param_a_key, param_a_value, param_b_key, param_b_value)` 추가:
    - 2개 placeholder 치환 전용 경량 경로.
  - 두 함수 모두 기존 `trf`와 동일하게 `key_id` 캐시 경로를 사용하고, miss 시 `ltr(key)` fallback을 유지.
- `scripts/ui/hud.gd`
  - 프레임 루프/선택 엔티티 갱신의 빈번한 포맷 호출을 `trf1/trf2`로 전환:
    - `UI_POP_FMT`, `UI_BLD_WIP_FMT`, `UI_BLD_FMT`
    - `UI_RES_FOOD_FMT`, `UI_RES_WOOD_FMT`, `UI_RES_STONE_FMT`
    - `UI_POS_FMT`, `UI_ENTITY_STATS_FMT`

## 기능 영향
- 표시되는 텍스트 결과는 기존과 동일.
- HUD 핫패스에서 매 프레임 생성되던 다수 임시 `Dictionary`를 줄여 locale 포맷 호출 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=459.2`, `checksum=13761358.00000`
