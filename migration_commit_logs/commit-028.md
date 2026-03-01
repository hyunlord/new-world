# Commit 028 - SimBridge pathfinding backend 동기화 호출 캐시

## 커밋 요약
- `SimBridge`에서 Rust backend 모드 동기화 호출을 매 요청마다 수행하지 않고, 모드 변경 시에만 호출하도록 캐시를 추가.

## 상세 변경
- `scripts/core/simulation/sim_bridge.gd`
  - 신규 캐시 변수:
    - `_last_synced_pathfind_backend_mode`
  - `_sync_pathfinding_backend_mode(...)` 개선:
    - `desired_mode`가 마지막 동기화 모드와 같으면 bridge 호출 생략
    - `set_pathfinding_backend(...)` 호출 결과가 `false`인 경우 캐시 갱신하지 않음

## 기능 영향
- Pathfinding 호출 빈도가 높은 구간에서 불필요한 bridge method 호출 감소.
- 모드 전환(`cpu/gpu_auto/gpu_force`) 시에는 기존처럼 즉시 동기화 동작 유지.

## 검증
- `tools/migration_verify.sh` 통과
  - extraction: `entries=0`, `keys=437`, `preserved=437`
  - strict audit: inline localized fields 0 유지
