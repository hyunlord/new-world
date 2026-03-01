# Commit 096 - HUD 건물 상태 포맷 경량 경로 적용

## 커밋 요약
- `HUD`의 건물 패널 갱신 경로에서 1-파라미터 포맷 호출을 `Locale.trf1`로 전환.

## 상세 변경
- `scripts/ui/hud.gd`
  - `_update_building_panel()`:
    - `UI_UNDER_CONSTRUCTION_FMT` (stockpile/shelter/campfire 분기) `trf` → `trf1`
    - `UI_BUILDING_WIP_FMT` `trf` → `trf1`
  - `_on_speed_changed()`:
    - `UI_SPEED_MULT_FMT` `trf` → `trf1`

## 기능 영향
- 건물 상태/속도 라벨 출력은 기존과 동일.
- 반복 갱신 경로에서 임시 params Dictionary 생성량을 줄여 HUD 포맷 호출 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=459.1`, `checksum=13761358.00000`
