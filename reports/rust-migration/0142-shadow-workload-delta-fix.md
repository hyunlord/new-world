# 0142 - shadow workload delta fix

## Commit
- `[rust-r0-242] Fix shadow comparison metric to use processed ticks`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - shadow runtime 상태에서 `ticks_processed`를 읽어 `_shadow_reporter.record_frame()`의 Rust 측 비교값으로 전달.
  - mismatch 로그 메시지를 `gd_ticks/rust_ticks/rust_events`로 분리해 의미를 명확화.
- `scripts/core/simulation/runtime_shadow_reporter.gd`
  - 비교 메트릭을 `event_delta` 기반 명칭에서 `work_delta`(실제 처리량) 기반으로 정리.
  - 리포트에 `max_work_delta`, `avg_work_delta`, `allowed_max_work_delta` 추가.
  - 하위 호환을 위해 기존 `*_event_delta` 필드는 alias로 유지.
- `reports/rust-migration/README.md`
  - 0142 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 공개 함수 시그니처 변경 없음.
- shadow 리포트 JSON 필드 확장:
  - 추가: `max_work_delta`, `avg_work_delta`, `allowed_max_work_delta`
  - 유지(호환): `max_event_delta`, `avg_event_delta`, `allowed_max_event_delta`

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 기존 shadow 비교는 서로 다른 의미의 수치(gd ticks vs rust events)를 비교하던 문제가 있었고, 이번 커밋으로 동일 의미의 처리량(ticks processed) 비교로 교정했다.
