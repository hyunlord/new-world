# Commit 223 - Pathfinding backend dispatch 계측(카운터) 추가

## 커밋 요약
- pathfinding backend 레이어에 CPU/GPU dispatch 카운터를 추가하고, 브리지에서 조회/리셋 가능한 API를 제공.
- GPU 전환 단계에서 실제 호출 분포를 런타임에서 관측할 수 있게 확장.

## 상세 변경
- `rust/crates/sim-bridge/src/pathfinding_backend.rs`
  - dispatch 카운터 원자 변수 추가:
    - `CPU_DISPATCH_COUNT`
    - `GPU_DISPATCH_COUNT`
  - API 추가:
    - `record_dispatch(resolved_mode)`
    - `read_dispatch_counts()`
    - `reset_dispatch_counts()`
- `rust/crates/sim-bridge/src/lib.rs`
  - `dispatch_pathfind_grid_bytes` / `dispatch_pathfind_grid_batch_vec2_bytes` / `dispatch_pathfind_grid_batch_xy_bytes`에서
    - resolved backend 계산 후 `record_dispatch` 호출.
  - 브리지 함수 추가:
    - `get_pathfinding_backend_stats()`
      - `configured`, `resolved`, `cpu_dispatches`, `gpu_dispatches`, `total_dispatches` 반환
    - `reset_pathfinding_backend_stats()`
  - 테스트 추가:
    - `backend_dispatch_counters_track_resolved_modes`
      - 병렬 테스트 간섭을 고려해 절대값이 아닌 증가 델타 기반 검증으로 작성.

## 기능 영향
- 성능/운영 측면에서 backend 사용 분포(실제 CPU/GPU dispatch 횟수)를 즉시 확인 가능.
- GPU 기능이 점진 도입될 때 auto 모드 분배 동작을 수치로 검증하는 기반 확보.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과 (20 tests).
- `tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
