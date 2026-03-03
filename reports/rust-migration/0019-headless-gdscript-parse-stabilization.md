# 0019 - Headless GDScript parse stabilization

## Summary
Rust shadow 검증을 반복 실행하는 과정에서 확인된 GDScript 들여쓰기 붕괴를 복구해, headless 검사 경로가 문법 오류 없이 동작하도록 정리했다.

## Files Changed
- `scripts/systems/world/tech_maintenance_system.gd`
  - `_chronicle` 로깅 블록의 과다 들여쓰기 복구
  - `tech_stabilized`, `tech_fallback`, `tech_lost` 이벤트 로그 블록 정렬
- `scripts/systems/world/tech_propagation_system.gd`
  - teaching/tech_import 로깅 블록 들여쓰기 복구
  - `match channel` 하위에 잘못 들어간 `_chronicle.log_event(...)`를 `if _chronicle` 블록으로 정렬
- `scripts/ui/hud.gd`
  - `_on_simulation_event`의 `match event_type` 케이스 들여쓰기 복구
  - `_on_tech_state_changed` 블록 들여쓰기 정리
- `scripts/ui/panels/list_panel.gd`
  - 엔티티/건물 리스트 렌더 블록 들여쓰기 붕괴 복구
  - `_draw_building_list` 루프 문법 복구

## API / Signal / Schema Changes
- 없음 (문법/정렬 복구)

## Verification
- `Godot --headless --check-only --quit-after 1` : PASS (문법 오류 없음)
  - 잔여 로그: registry order mismatch warning / ObjectDB leak warning
- `Godot --headless --script tools/rust_shadow_smoke.gd` : PASS
  - `SHADOW_REPORT_PATH=/Users/rexxa/Library/Application Support/Godot/app_userdata/WorldSim/reports/rust_shadow/latest.json`
- `python3 tools/rust_shadow_cutover_check.py --report <latest.json>` : PASS
  - `approved_for_cutover=True`
  - `frames=800 mismatch_frames=0`

## Rust Migration Progress
- Previous: 100% complete / 0% remaining
- Current: 100% complete / 0% remaining
- Delta: +0%

## Notes
- 본 커밋은 Rust 전환 범위를 넓히는 변경이 아니라, 전환 완료 상태에서 검증 자동화가 막히지 않도록 GDScript 문법 상태를 안정화한 유지보수 단계다.
