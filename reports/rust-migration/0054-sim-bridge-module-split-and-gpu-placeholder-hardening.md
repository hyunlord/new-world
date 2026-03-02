# 0054 - sim-bridge module split + GPU placeholder hardening

## Commit
- `[rust-r0-154] Split sim-bridge modules and harden non-path GPU placeholders`

## 변경 파일
- `rust/crates/sim-bridge/src/lib.rs`
  - `body_bindings`, `pathfinding_bindings`, `locale_bindings`, `runtime_bindings` 모듈을 사용하도록 분리.
  - 기존 helper 구현(배열 변환/경로 인코딩/locale helper/runtime compute helper)을 모듈로 이관.
  - 외부 GDExtension 메서드 시그니처는 유지.
  - runtime compute command 처리에서 도메인별 mode 정규화 적용.
- `rust/crates/sim-bridge/src/body_bindings.rs` (신규)
  - PackedArray <-> Vec 변환 및 step pair 구성 helper 분리.
- `rust/crates/sim-bridge/src/pathfinding_bindings.rs` (신규)
  - path encode/decode, backend mode 파싱/해석 helper 분리.
- `rust/crates/sim-bridge/src/locale_bindings.rs` (신규)
  - Fluent source 저장/삭제/format helper 분리.
- `rust/crates/sim-bridge/src/runtime_bindings.rs` (신규)
  - runtime compute domain default mode 및 mode 정규화 helper 분리.
- `scripts/core/simulation/compute_backend.gd`
  - GPU 활성 도메인을 `pathfinding`으로 제한.
  - `needs/stress/emotion/orchestration` 도메인은 저장/명령/해석 전 구간에서 `cpu` 강제.

## 변경 API / 시그널 / 스키마
- GDExtension 공개 API 시그니처 변경 없음.
- Runtime compute 동작 변경:
  - pathfinding 외 도메인은 `gpu_auto/gpu_force` 요청이 와도 runtime에서 `cpu`로 정규화.
  - `set_compute_mode_all`도 동일 규칙 적용.
- Tracking CSV/metadata 스키마 변경 없음.

## 검증 결과
- `cd rust && cargo check -p sim-bridge -p sim-systems` ✅
- `cd rust && cargo test -p sim-bridge` ✅
- `cd rust && cargo test -p sim-systems` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `3 / 46 = 6.52%`
- Owner transfer 완료율 (`exec_owner=rust`): `0 / 46 = 0.00%`
- 잔여율: `93.48%`

## 메모
- 본 커밋은 구조 분리 및 GPU 노출 정리 단계다.
- 다음 단계(Phase 5)는 핵심 루프 실포팅 순서(`stress -> emotion -> reputation -> social_event -> morale`)로 진행한다.
