# 0005 - Phase F domain compute modes

## Summary
ComputeBackend를 전역 단일 모드에서 도메인별 모드 체계로 확장해, GPU 활용 범위를 시스템 단위로 제어할 수 있게 했다.

## Files Changed
- `scripts/core/simulation/compute_backend.gd`
  - 도메인 모드 신호 추가: `compute_domain_mode_changed(domain, new_mode, resolved_mode)`
  - 도메인 목록 추가: `pathfinding`, `needs`, `stress`, `emotion`, `orchestration`
  - 도메인별 설정/조회 API 추가:
    - `get_mode_for_domain(domain)`
    - `set_mode_for_domain(domain, mode)`
    - `resolve_mode_for_domain(domain)`
    - `get_domain_modes_snapshot()`
  - settings 저장 구조 확장:
    - `compute_mode`
    - `compute_domain_modes`
  - pathfinding backend sync를 도메인 설정 기반으로 변경.

## API / Signal / Schema Changes
### Compute backend API
- Added signal: `compute_domain_mode_changed(domain: String, new_mode: String, resolved_mode: String)`
- Added methods:
  - `get_mode_for_domain(domain: String) -> String`
  - `set_mode_for_domain(domain: String, new_mode: String) -> void`
  - `resolve_mode_for_domain(domain: String) -> String`
  - `get_domain_modes_snapshot() -> Dictionary`

### Settings schema
- Added `compute_domain_modes` object in `user://settings.json`.

## Verification
- 정적 코드 점검: PASS (GDScript 변경 범위 수동 검토)
- Godot headless check: 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 57% complete / 43% remaining
- Current: 62% complete / 38% remaining
- Delta: +5%

## Notes
- 현재 실제 GPU 경로가 완성된 도메인은 pathfinding 우선이며,
  needs/stress/emotion/orchestration 도메인은 라우팅 기반만 선반영한 상태다.
