# 0007 - Phase F pathfinding domain route

## Summary
`SimBridge`의 pathfinding GPU 판정을 전역 compute mode가 아닌 domain 기반(`pathfinding`)으로 전환했다.

## Files Changed
- `scripts/core/simulation/sim_bridge.gd`
  - `_prefer_gpu()`:
    - `ComputeBackend.resolve_mode_for_domain("pathfinding")` 우선 사용.
    - fallback으로 기존 `resolve_mode()` 유지.
  - `_resolve_desired_pathfinding_backend_mode()`:
    - `ComputeBackend.get_mode_for_domain("pathfinding")` 우선 사용.
    - fallback으로 기존 `get_mode()` 유지.

## API / Signal / Schema Changes
- 새 API 추가 없음.
- 기존 API 사용 경로만 domain-aware로 변경.

## Verification
- 정적 코드 점검: PASS (GDScript 변경 범위 수동 검토)
- Godot headless check: 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 68% complete / 32% remaining
- Current: 72% complete / 28% remaining
- Delta: +4%

## Notes
- 경로탐색은 domain 기반 모드로 실제 연결 완료.
- needs/stress/emotion 도메인의 실제 kernel 라우팅 적용은 후속 단계에서 진행.
