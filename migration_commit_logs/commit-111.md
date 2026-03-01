# Commit 111 - HUD 알림/팔로우 포맷 경량화

## 커밋 요약
- `HUD`의 알림/라벨 포맷 호출을 `Locale.trf1/trf2/trf3` 경량 경로로 전환.

## 상세 변경
- `scripts/ui/hud.gd`
  - `UI_BUILDING_STORAGE_FMT`:
    - `trf` + Dictionary → `trf3("food","wood","stone")`
  - 시뮬레이션 이벤트 알림 포맷:
    - `UI_NOTIF_BUILDING_BUILT_FMT` → `trf1`
    - `UI_NOTIF_DIED_STARVED_FMT` → `trf1`
    - `UI_NOTIF_DIED_CAUSE_AGE_FMT` → `trf3`
    - `UI_NOTIF_MATERNAL_FMT` → `trf1`
    - `UI_NOTIF_STILLBORN_FMT` → `trf1`
    - `UI_NOTIF_LEADER_ELECTED_FMT` → `trf2`
    - `UI_NOTIF_LEADER_LOST_FMT` → `trf1`
    - `UI_NOTIF_TECH_DISCOVERED_FMT` → `trf2`
    - `UI_NOTIF_ERA_ADVANCED_FMT` → `trf1`
    - `UI_NOTIF_TECH_ATROPHY_FMT` → `trf3`
    - `UI_NOTIF_TECH_FALLBACK_FMT` → `trf2`
  - 기술 상태 이벤트 알림 포맷:
    - `UI_NOTIF_TECH_STABILIZED_FMT` → `trf1`
    - `UI_NOTIF_TECH_REGRESSION_FMT` → `trf1`
    - `UI_NOTIF_TECH_LOST_FMT` → `trf1`
  - 기타 HUD 라벨:
    - `UI_NOTIF_WORLDSIM_STARTED_FMT` → `trf1`
    - `UI_FOLLOWING_FMT`(2곳) → `trf1`

## 기능 영향
- HUD 알림/팔로우 라벨의 출력 텍스트 의미는 기존과 동일.
- 이벤트/갱신 경로의 params Dictionary 생성 비용을 줄여 미세 오버헤드를 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과
  - localization compile `filled=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -q -p sim-test -- --bench-stress-math --iters 10000`
  - `ns_per_iter=460.3`, `checksum=13761358.00000`
