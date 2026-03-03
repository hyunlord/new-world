# 0100 - owner transfer allowlist phase 21

## Commit
- `[rust-r0-200] Expand owner-ready allowlist with title system`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 `title_system` 추가.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `title_system`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`에 `title_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0100 항목 추가 및 누적 전환률 갱신.

## 추가/삭제 시스템 키
- owner transfer 추가: `title_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 GDExtension 시그니처 변경 없음.
- Runtime owner-ready policy 변경:
  - Rust primary 경로에서 `title_system` fallback skip 가능.
- tracking 스키마 변경 없음(값만 갱신).

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `28 / 46 = 60.87%`
- Owner transfer 완료율 (`exec_owner=rust`): `28 / 46 = 60.87%`
- State-write 잔여율: `39.13%`
- Owner transfer 잔여율: `39.13%`

## 메모
- 이번 단계로 `title_system` 실행 소유권이 Rust로 승격됐다.
- 다음 실포팅 우선순위 후보는 데이터 스키마 확장이 작은 영역(예: `stat_sync_system` 정식 write 이관 또는 `tech_*` 최소 스키마 도입)이다.
