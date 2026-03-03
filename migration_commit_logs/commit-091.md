# Commit 091 - HUD 건물 집계 단일 순회 통합

## 커밋 요약
- `HUD._process()`에서 건물 카운트와 stockpile 자원 합계 계산을 단일 루프로 통합해 중복 순회를 제거.

## 상세 변경
- `scripts/ui/hud.gd`
  - `_process()`의 건물 관련 갱신 경로를 재구성:
    - `get_all_buildings()` 결과를 1회만 순회.
    - 같은 루프에서 `built_count`, `wip_count`, `total_food`, `total_wood`, `total_stone`를 동시에 집계.
    - 집계값으로 `UI_BLD_*` 및 `UI_RES_*` 라벨을 즉시 갱신.
  - `_get_stockpile_totals()` 제거:
    - 기존 `get_buildings_by_type("stockpile")` 기반 추가 순회 경로를 삭제.

## 기능 영향
- 건물 수/자원 라벨 표시 의미는 기존과 동일.
- 프레임당 건물 데이터 순회 횟수를 줄여 HUD 업데이트 경로의 CPU 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=451.5`, `checksum=13761358.00000`
