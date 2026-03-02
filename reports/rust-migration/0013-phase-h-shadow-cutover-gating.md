# 0013 - Phase H shadow cutover gating metrics

## Summary
Rust shadow 비교 리포트에 컷오버 판정 기준을 추가했다. 이제 shadow report JSON에서 mismatch ratio와 허용 임계치, 최종 `approved_for_cutover` 여부를 함께 기록한다.

## Files Changed
- `scripts/core/simulation/game_config.gd`
  - shadow 승인 임계치 상수 추가
    - `RUST_SHADOW_ALLOWED_MAX_TICK_DELTA`
    - `RUST_SHADOW_ALLOWED_MAX_EVENT_DELTA`
    - `RUST_SHADOW_ALLOWED_MISMATCH_RATIO`
- `scripts/core/simulation/runtime_shadow_reporter.gd`
  - `setup()` 확장: 승인 임계치 파라미터 수신
  - 리포트 계산 항목 확장
    - `mismatch_ratio`
    - `allowed_*` threshold fields
    - `approved_for_cutover`
- `scripts/core/simulation/simulation_engine.gd`
  - shadow reporter setup 시 GameConfig threshold 전달

## API / Signal / Schema Changes
### Shadow report schema (`user://reports/rust_shadow/latest.json`)
- Added fields:
  - `mismatch_ratio: float`
  - `allowed_max_tick_delta: int`
  - `allowed_max_event_delta: int`
  - `allowed_mismatch_ratio: float`
  - `approved_for_cutover: bool`

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `godot --headless --check-only` : 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 90% complete / 10% remaining
- Current: 93% complete / 7% remaining
- Delta: +3%

## Notes
- 이 단계는 컷오버 판단 신뢰도 확보를 위한 계측/판정 강화다.
- 후속 단계에서 `approved_for_cutover=true` 조건을 만족하면 runtime default를 `rust_primary`로 전환할 수 있다.
