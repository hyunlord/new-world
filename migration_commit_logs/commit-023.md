# Commit 023 - ComputeBackend와 Rust pathfinding backend 모드 동기화

## 커밋 요약
- Rust bridge에 pathfinding backend 모드(`auto/cpu/gpu`) 설정/조회 API를 추가.
- `SimBridge`가 `ComputeBackend` 설정을 Rust bridge backend 모드로 동기화하고, 실제 사용 모드(`resolve`) 기준으로 GPU 경로 선호 여부를 판단하도록 연결.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - pathfinding backend 원자 상태 추가:
    - `PATHFIND_BACKEND_AUTO/CPU/GPU`
    - `PATHFIND_BACKEND_MODE: AtomicU8`
  - 신규 헬퍼:
    - `parse_pathfind_backend(...)`
    - `backend_mode_to_str(...)`
    - `resolve_backend_mode(...)`
  - 신규 Godot 함수:
    - `set_pathfinding_backend(mode: GString) -> bool`
    - `get_pathfinding_backend() -> GString`
    - `resolve_pathfinding_backend() -> GString`
  - 단위 테스트 2개 추가:
    - backend mode 파싱 검증
    - `gpu` feature 유무에 따른 resolve 결과 검증
- `scripts/core/simulation/sim_bridge.gd`
  - bridge method 후보군/캐시 추가:
    - `set_pathfinding_backend`
    - `get_pathfinding_backend`
    - `resolve_pathfinding_backend`
  - `_prefer_gpu()`에서 Rust backend 모드를 우선 기준으로 사용:
    - `ComputeBackend.get_mode()` 값을 `cpu/gpu/auto`로 매핑
    - Rust bridge에 모드 동기화 후 `resolve_pathfinding_backend()` 결과가 `gpu`인지 판단
    - 미지원 bridge에서는 기존 `ComputeBackend.resolve_mode()` + capability 체크 fallback 유지

## 기능 영향
- Compute 모드(`cpu/gpu_auto/gpu_force`)와 Rust pathfinding backend 의사결정이 동일한 정책으로 정렬됨.
- GPU feature 미빌드/미지원 환경에서는 자동으로 CPU resolve되어 기존 fallback 안정성 유지.
- 향후 실제 GPU pathfinding 구현 시 호출부 변경 없이 backend resolve 정책만으로 경로 선택 가능.

## 검증
- `cd rust && cargo fmt -p sim-bridge` 통과
- `cd rust && cargo test -q -p sim-bridge` 통과 (8 tests)
- `tools/migration_verify.sh` 통과
