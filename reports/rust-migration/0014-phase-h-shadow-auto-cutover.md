# 0014 - Phase H shadow auto-cutover hook

## Summary
Shadow 검증 결과가 승인 조건을 만족할 때 `rust_primary`로 자동 전환할 수 있는 훅을 추가했다. 기본값은 비활성화이며, 명시적으로 플래그를 켰을 때만 동작한다.

## Files Changed
- `scripts/core/simulation/game_config.gd`
  - Added: `RUST_SHADOW_AUTO_CUTOVER_ENABLED: bool = false`
- `scripts/core/simulation/runtime_shadow_reporter.gd`
  - Added state: `_last_approved_for_cutover`
  - Added method: `is_approved_for_cutover() -> bool`
  - `_flush_report()` 계산 결과를 `_last_approved_for_cutover`에 반영
- `scripts/core/simulation/simulation_engine.gd`
  - Added method: `_try_shadow_auto_cutover()`
  - shadow 프레임 기록 후 승인 상태 확인
  - 승인 + 플래그 활성화 시 `_runtime_mode`를 `rust_primary`로 전환

## API / Signal / Schema Changes
### Shadow reporter API
- Added: `is_approved_for_cutover() -> bool`

### Runtime behavior
- `RUST_SHADOW_AUTO_CUTOVER_ENABLED=true`인 경우에만 자동 전환 실행
- 기본값 `false`로 기존 동작 영향 없음

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `godot --headless --check-only` : 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 93% complete / 7% remaining
- Current: 95% complete / 5% remaining
- Delta: +2%

## Notes
- 승인 상태는 shadow report flush 주기(`RUST_SHADOW_REPORT_INTERVAL_TICKS`) 기준으로 갱신된다.
- 안정화 확인 후 플래그를 true로 전환하면 런타임 기본 전환 자동화를 사용할 수 있다.
