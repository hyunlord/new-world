# Commit 187 - world tile batch update revision coalescing

## 커밋 요약
- 월드 생성/프리셋 생성 시 타일별 revision 증가를 배치 단위 1회로 coalescing해 불필요한 revision churn을 줄임.

## 상세 변경
- `scripts/core/world/world_data.gd`
  - 배치 업데이트 상태 필드 추가:
    - `_tile_update_depth`
    - `_tile_update_changed`
  - `set_tile(...)`
    - 변경 감지 시 배치 모드에서는 즉시 revision 증가 대신 changed 플래그 설정.
    - 배치 모드가 아닐 때만 즉시 `terrain_revision += 1`.
  - `begin_tile_update()` / `end_tile_update()` 추가:
    - 중첩 depth 지원.
    - 최외곽 `end`에서 변경 발생 시 revision 1회 증가.
  - `init_world(...)`에서 배치 상태 초기화.
- `scripts/core/world/world_generator.gd`
  - 전체 타일 생성 루프를 `begin_tile_update()`/`end_tile_update()`로 감싸 revision 증가를 1회로 집약.
- `scripts/core/world/preset_map_generator.gd`
  - 프리셋 생성 진입점(`generate_preset`)을 배치 업데이트로 감싸 내부 다중 `set_tile` 호출의 revision 증가를 1회로 집약.

## 기능 영향
- 지형/바이옴 결과는 동일.
- 생성 단계에서 revision 폭증을 줄여 terrain revision 관리 비용을 완화.
- pathfinder는 revision 기반 무효화를 유지하면서도 생성 완료 후 한 번만 캐시 재검증하면 됨.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=391.3`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=145.9`, `checksum=38457848.00000`
