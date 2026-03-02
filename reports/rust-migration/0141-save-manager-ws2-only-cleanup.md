# 0141 - save manager ws2 only cleanup

## Commit
- `[rust-r0-241] Simplify SaveManager to ws2-only runtime path`

## 변경 파일
- `scripts/core/simulation/save_manager.gd`
  - 레거시 binary save/load 보조 루틴(`entities/buildings/relationships/settlements/world/stats`) 제거.
  - `save_game` / `load_game`는 ws2 runtime 경로만 유지하도록 정리.
  - `get_slot_info()`를 `ws2 파일 존재` 기준으로 엄격화.
  - slot 디렉터리에 레거시 아티팩트가 남아 있고 ws2가 없으면
    `unsupported_legacy=true`, `legacy_files` 목록을 반환하도록 추가.
  - 저장 시 runtime snapshot(`runtime_get_snapshot`)에서 `population`, `settlement_count`를 meta에 기록.
  - 외부 호출 호환을 위해 공개 메서드 시그니처는 유지.
- `reports/rust-migration/README.md`
  - 0141 항목 및 누적 전환률 반영.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 공개 함수 시그니처 변경 없음.
- save/load 동작 변경:
  - ws2 미존재 슬롯은 meta 존재 여부와 무관하게 로드 가능 슬롯으로 표시하지 않음.
  - 레거시 파일만 있는 슬롯은 unsupported 상태로 명시.

## 검증 결과
- `cd rust && cargo check -p sim-engine -p sim-systems -p sim-bridge` ✅
- `cd rust && cargo test -p sim-bridge` ✅
- GDScript 정적 참조 확인:
  - `main.gd` / `pause_menu.gd`의 SaveManager 호출 시그니처 유지 확인 ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- 정책상 구세이브 미지원은 유지하며, UI에서 레거시 슬롯을 명확히 구분할 수 있는 데이터만 제공한다.
