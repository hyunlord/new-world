# Commit 110 - Map Editor 스폰 카운트 포맷 경량화

## 커밋 요약
- `brush_palette`의 스폰 총합 라벨 포맷 호출을 `Locale.trf1`로 전환.

## 상세 변경
- `scripts/ui/map_editor/brush_palette.gd`
  - 초기화 시 `UI_MAP_SPAWN_TOTAL` 호출:
    - `trf` → `trf1("count", 0)`
  - 합계 갱신 시 `UI_MAP_SPAWN_TOTAL` 호출:
    - `trf` → `trf1("count", total)`

## 기능 영향
- 맵 에디터 스폰 합계 라벨 출력은 기존과 동일.
- 반복 라벨 갱신 경로에서 단일 placeholder 포맷 호출 임시 params Dictionary 생성을 줄여 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=450.8`, `checksum=13761358.00000`
