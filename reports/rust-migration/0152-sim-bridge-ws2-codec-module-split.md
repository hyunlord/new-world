# 0152 - sim-bridge ws2 codec module split

## Commit
- `[rust-r0-252] Extract ws2 save codec into dedicated sim-bridge module`

## 변경 파일
- `rust/crates/sim-bridge/src/lib.rs`
  - `ws2_codec` 모듈 선언 및 re-export(`encode_ws2_blob`, `decode_ws2_blob`) 추가.
  - 기존 인라인 ws2 코덱 상수/함수 제거.
- `rust/crates/sim-bridge/src/ws2_codec.rs`
  - `.ws2` 바이너리 코덱(헤더/체크섬/zstd+bincode) 구현 이동.
  - 기존 동작과 시그니처를 유지한 채 모듈 단위로 분리.

## 추가/삭제 시스템 키
- 없음

## 변경 API / 시그널 / 스키마
- 외부 GDExtension API/시그널 변경 없음.
- `.ws2` 포맷 스키마 변경 없음 (모듈 구조만 분리).

## 검증 결과
- `cargo test -p sim-bridge` ✅
- `bash tools/migration_verify.sh` ✅

## 누적 전환률 (strict 기준)
- State-write 기준 완료율: `46 / 46 = 100.00%`
- Owner transfer 완료율 (`exec_owner=rust`): `46 / 46 = 100.00%`
- State-write 잔여율: `0.00%`
- Owner transfer 잔여율: `0.00%`

## 메모
- `sim-bridge/lib.rs` 단일 파일 리스크를 줄이기 위한 구조 분리 1차 단계.
- 다음 분리 후보는 pathfinding/runtime binding 블록이다.
