# Commit 176 - sim_bridge pathfinding backend resolve 캐시

## 커밋 요약
- pathfinding 호출 시 반복되던 backend resolve bridge 호출을 캐시해 GPU/CPU 모드 판정 오버헤드를 줄임.

## 상세 변경
- `scripts/core/simulation/sim_bridge.gd`
  - 캐시 필드 추가:
    - `_resolved_pathfind_backend_cache`
    - `_resolved_pathfind_backend_cached`
  - `set_pathfinding_backend(...)` 성공 시 resolve 캐시 무효화.
  - `resolve_pathfinding_backend()`가 `_resolve_pathfinding_backend_cached(...)` 경로를 사용하도록 변경.
  - `_prefer_gpu()`가 매 호출 direct resolve 대신 캐시 경로를 사용하도록 변경.
  - `_get_native_bridge()`에서 bridge 인스턴스 결정 시 resolve 캐시를 초기화.
  - `_sync_pathfinding_backend_mode(...)`에서 mode 변경 성공 시 resolve 캐시를 무효화.
  - `_resolve_pathfinding_backend_cached(bridge)` 헬퍼 추가:
    - sync 후 캐시 hit 시 즉시 반환
    - cache miss 시 bridge resolve 1회 수행 후 결과 저장

## 기능 영향
- pathfinding backend 선택 의미(`cpu/auto/gpu`)와 fallback 정책은 동일.
- pathfinding hot path에서 backend resolve 관련 bridge call 빈도를 줄여 분기 비용을 완화.

## 검증
- `tools/migration_verify.sh` 통과
  - rust workspace tests 통과 (`sim-systems` 98 tests)
  - localization compile `filled=0`, `updated=0`
  - localization strict audit: inline localized fields 0 유지
- `cd rust && cargo run -p sim-test --release -- --bench-stress-math --iters 10000`
  - `ns_per_iter=383.6`, `checksum=24032652.00000`
- `cd rust && cargo run -p sim-test --release -- --bench-needs-math --iters 10000`
  - `ns_per_iter=147.0`, `checksum=38457848.00000`
