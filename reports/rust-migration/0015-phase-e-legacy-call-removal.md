# 0015 - Phase E legacy migration call removal

## Summary
ws2-only 저장 정책에 맞춰 초기화 경로에서 legacy 마이그레이션 호출을 제거했다.

## Files Changed
- `scenes/main/main.gd`
  - `save_manager.migrate_legacy_save()` 호출 제거

## API / Signal / Schema Changes
- 없음

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `godot --headless --check-only` : 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 95% complete / 5% remaining
- Current: 96% complete / 4% remaining
- Delta: +1%

## Notes
- `SaveManager.migrate_legacy_save()` 함수는 남아 있지만 실행 경로에서 호출되지 않는다.
