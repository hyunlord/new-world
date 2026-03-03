# 0122 - owner transfer allowlist phase 32

## Commit
- `[rust-r0-222] Expand owner-ready allowlist with construction system`

## 변경 파일
- `scripts/core/simulation/simulation_engine.gd`
  - `_RUST_OWNER_READY_SYSTEM_KEYS`에 `construction_system` 추가.
  - Rust-primary 하이브리드 모드에서 해당 시스템의 GD fallback skip 가능 상태 반영.
- `reports/rust-migration/data/runtime-registered-systems-v2.csv`
  - `construction_system`의 `exec_owner=gdscript -> rust` 반영.
- `reports/rust-migration/data/tracking-metadata.json`
  - `exec_owner_rule`에 `construction_system` 추가.
  - `generated_at` 갱신.
- `reports/rust-migration/README.md`
  - 0122 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- owner-ready allowlist 추가: `construction_system`
- 삭제: 없음

## 변경 API / 시그널 / 스키마
- 공개 API/시그니처 변경 없음.
- Runtime ownership 변경:
  - `construction_system` 실행 소유자 `rust`로 전환.

## 검증 결과
- `cd rust && cargo check -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `39 / 46 = 84.78%`
- Owner transfer 완료율 (`exec_owner=rust`): `39 / 46 = 84.78%`
- State-write 잔여율: `15.22%`
- Owner transfer 잔여율: `15.22%`

## 메모
- `construction_system`은 이제 Rust active-write + owner transfer가 모두 완료된 상태다.
