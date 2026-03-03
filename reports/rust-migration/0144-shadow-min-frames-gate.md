# 0144 - shadow min frames gate

## Commit
- `[rust-r0-244] Add minimum-frame gate to shadow auto cutover`

## 변경 파일
- `scripts/core/simulation/runtime_shadow_reporter.gd`
  - `min_frames_for_cutover` 조건을 추가해 충분한 검증 프레임 전에는 승인되지 않도록 변경.
  - 신규 필드/메서드:
    - `_min_frames_for_cutover`
    - `get_report_snapshot()`
    - 리포트 payload: `min_frames_for_cutover`, `frames_ready_for_cutover`
  - 기존 event-delta 필드는 alias로 유지.
- `scripts/core/simulation/game_config.gd`
  - `RUST_SHADOW_ALLOWED_MAX_WORK_DELTA` 상수 추가.
  - `RUST_SHADOW_MIN_FRAMES_FOR_CUTOVER=10000` 상수 추가.
  - 기존 `RUST_SHADOW_ALLOWED_MAX_EVENT_DELTA`는 호환 alias로 유지.
- `scripts/core/simulation/simulation_engine.gd`
  - shadow reporter setup에 `allowed_max_work_delta`, `min_frames_for_cutover` 전달.
  - auto cutover 승인 로그에 frame/mismatch_ratio 요약 포함.
- `reports/rust-migration/README.md`
  - 0144 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 공개 함수 시그니처 변경 없음.
- shadow report JSON 확장:
  - 추가: `min_frames_for_cutover`, `frames_ready_for_cutover`
  - 유지: `allowed_max_event_delta` (alias)

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 자동 컷오버는 이제 임계치 일치 + 최소 프레임(기본 10,000) 모두 만족해야 승인된다.
