# 0012 - Phase E ws2-only save/load cutover

## Summary
`SaveManager`의 실행 경로를 `.ws2` 단일 포맷으로 컷오버했다. 기존 legacy 바이너리(`entities.bin`, `buildings.bin` 등) 저장/로드 경로는 호출되지 않으며, Rust runtime(ws2 API) 준비가 안 된 경우 명시적으로 실패한다.

## Files Changed
- `scripts/core/simulation/save_manager.gd`
  - `save_game()`/`load_game()`을 ws2 전용 경로로 전환
    - 기존 legacy fallback 제거
    - runtime 미준비 시 경고 후 `false` 반환
  - `_use_rust_ws2_backend()`를 `_is_ws2_runtime_ready()`로 변경
    - runtime mode 문자열 조건 제거
    - bridge method availability 기준으로 ws2 readiness 판단
  - ws2 metadata version 상수 추가: `WS2_META_VERSION = 2`
  - ws2 저장 시 meta `version`을 `WS2_META_VERSION`으로 기록
  - `get_slot_info()`에서 `meta.json`이 없어도 `sim.ws2`가 있으면 슬롯 존재로 인식
  - `migrate_legacy_save()` 비활성화 (ws2-only 정책)
  - 파일 상단 주석을 ws2 구조 설명으로 갱신

## API / Signal / Schema Changes
### Save policy
- Runtime save/load path: ws2 only
- Legacy save migration: disabled

### Meta schema
- ws2 meta `version`: `2`
- `save_backend`: `rust_ws2`

## Verification
- `cd rust && cargo check -p sim-bridge` : PASS
- `godot --headless --check-only` : 미실행 (`godot` binary 없음)

## Rust Migration Progress
- Previous: 87% complete / 13% remaining
- Current: 90% complete / 10% remaining
- Delta: +3%

## Notes
- legacy 직렬화 함수 본문은 파일에 남아 있으나 실행 경로에서 사용되지 않는다.
- 후속 단계에서 dead legacy 함수 제거 및 sidecar(`names.json`, `tension.json`) ws2 통합 여부를 결정할 수 있다.
