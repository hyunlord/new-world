# Commit 005 - GPU bridge 메서드 스텁 추가 (CPU fallback)

## 커밋 요약
- `SimBridge`의 GPU 우선 라우팅이 실제 호출 가능한 형태가 되도록 Rust `WorldSimBridge`에 GPU 메서드 시그니처를 추가.
- 현재 GPU 연산 구현은 미완이므로 CPU 경로를 재사용하는 fallback으로 연결.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - `#[func] pathfind_grid_gpu(...) -> PackedVector2Array` 추가
  - `#[func] pathfind_grid_gpu_batch(...) -> Array<PackedVector2Array>` 추가
  - 두 메서드는 당분간 각각 `pathfind_grid`, `pathfind_grid_batch`를 내부 호출

## 기능 영향
- `ComputeBackend.resolve_mode() == "gpu"`일 때도 메서드 미존재로 인한 실패 없이 bridge 호출 가능.
- 실제 GPU 커널/compute 구현 전까지는 결과 일관성을 유지하면서 CPU 로직으로 처리.

## 검증
- `cargo test -q` 통과
