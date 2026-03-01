# Commit 010 - GPU 옵션 라우팅 능력 질의 추가

## 커밋 요약
- Rust 브리지에 `gpu` feature 토글과 GPU 가능 여부 질의 API를 추가.
- GDScript 라우팅이 사용자 모드(`ComputeBackend`)뿐 아니라 네이티브 지원 여부까지 확인한 뒤 GPU 경로를 선택하도록 보강.

## 상세 변경
- `rust/crates/sim-bridge/Cargo.toml`
  - feature 정의 추가:
    - `default = []`
    - `gpu = []`
- `rust/crates/sim-bridge/src/lib.rs`
  - `WorldSimBridge.has_gpu_pathfinding()` 추가.
  - 반환값은 `cfg!(feature = "gpu")` 기반으로 현재 빌드 capability를 노출.
- `scripts/core/simulation/sim_bridge.gd`
  - `_prefer_gpu()`에서 다음 조건을 모두 만족할 때만 GPU 라우팅:
    1) `ComputeBackend.resolve_mode() == "gpu"`
    2) native bridge 존재
    3) `has_gpu_pathfinding()`가 true (없으면 `pathfind_grid_gpu` 메서드 존재 여부로 fallback)

## 기능 영향
- `gpu_force/gpu_auto` 모드에서도 실제 네이티브 GPU 빌드가 아니면 CPU 경로로 안정 fallback.
- 옵션/빌드 capability 불일치로 인한 의미 없는 GPU 경로 호출을 방지.

## 검증
- `cd rust && cargo fmt -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-bridge` 통과
- `cd rust && cargo build -q -p sim-bridge --features gpu` 통과
- `cd rust && cargo test -q` 통과
