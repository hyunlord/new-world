# 0140 - disable gpu placeholder resolution

## Commit
- `[rust-r0-240] Disable pathfinding GPU placeholder resolution`

## 변경 파일
- `rust/crates/sim-bridge/src/pathfinding_backend.rs`
  - `GPU_BACKEND_ACTIVE=false` 게이트 추가.
  - `resolve_backend_mode_code()`가 현재 단계에서 `gpu/auto`를 모두 `cpu`로 해석하도록 정리.
  - `has_gpu_backend()`를 런타임 활성 상태 기반으로 반환하도록 수정.
- `rust/crates/sim-bridge/src/lib.rs`
  - pathfinding backend 관련 테스트 기대값을 placeholder 비활성 정책에 맞게 업데이트.
  - `resolves_pathfinding_backend_with_runtime_gate` 테스트명으로 의미 정렬.
- `scripts/core/simulation/compute_backend.gd`
  - `is_gpu_capable()`에서 `SimBridge.has_gpu_pathfinding()`를 우선 확인하도록 변경.
  - 렌더러가 GPU 가능이어도 bridge backend가 비활성 상태면 CPU로 고정.
- `reports/rust-migration/README.md`
  - 0140 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 공개 시그니처 변경 없음.
- 동작 변경:
  - pathfinding backend `auto/gpu` 설정이 현재 빌드에서 실제로 `cpu`로 resolve됨.
  - GPU dispatch 카운터가 placeholder 경로에서 증가하지 않도록 동작 정합성 고정.

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-engine` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 실제 GPU 커널이 연결되는 시점에 `GPU_BACKEND_ACTIVE`를 true로 전환하고 관련 parity/perf 검증을 재활성화하면 된다.
