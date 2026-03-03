# Commit 216 - Pathfinding backend 디스패처 레이어 연결

## 커밋 요약
- `sim-bridge`에 backend 디스패처 레이어를 도입해 `auto/cpu/gpu` 모드 설정이 실제 pathfinding 실행 경로에 반영되도록 정리.
- GPU 구현 전 단계에서는 동일 결과를 보장하기 위해 GPU 엔트리포인트를 CPU 경로로 안전 폴백.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - backend 선택 공통화:
    - `resolve_backend_mode_code(mode: u8)` 추가
    - `resolve_backend_mode(...)`가 코드 기반 해석을 재사용하도록 정리
  - pathfinding 호출 디스패처 추가:
    - `dispatch_pathfind_grid_bytes(...)`
    - `dispatch_pathfind_grid_batch_vec2_bytes(...)`
    - `dispatch_pathfind_grid_batch_xy_bytes(...)`
  - GPU 전용 엔트리포인트 스텁 추가(현재 CPU 폴백):
    - `pathfind_grid_gpu_bytes(...)`
    - `pathfind_grid_batch_vec2_gpu_bytes(...)`
    - `pathfind_grid_batch_xy_gpu_bytes(...)`
  - `WorldSimBridge` 메서드 연결:
    - `pathfind_grid` / `pathfind_grid_xy` / `pathfind_grid_batch` / `pathfind_grid_batch_xy`가 전역 backend 모드를 읽어 디스패처 경유 실행.
    - `pathfind_grid_gpu*` 계열은 강제 GPU 모드로 동일 디스패처를 타도록 정리.
  - 중복된 스텝 기본값 처리 제거를 위해 `normalize_max_steps(max_steps: i32)` 도입.
  - 테스트 추가:
    - `backend_dispatch_single_matches_cpu_path`
    - `backend_dispatch_batch_modes_match_cpu_path`

## 기능 영향
- 기존에는 backend 설정값을 바꿔도 일반 pathfinding 호출은 CPU 경로 고정이었는데, 이제 설정값이 실제 실행 경로 선택에 반영됨.
- GPU 실구현 전에도 API/호출 계약을 먼저 고정해, 이후 GPU 커널 통합 시 디스패처 내부만 교체하면 되도록 기반 구조를 마련.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과.
- `tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
