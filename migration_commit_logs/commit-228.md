# Commit 228 - sim-bridge 공개 backend helper API 회귀 테스트 추가

## 커밋 요약
- 최근 추가된 non-Godot backend 공개 helper API(`set/get/resolve`, dispatch counter helper)에 대한 단위 테스트를 추가해 계약을 고정.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs` 테스트 모듈 확장:
  - import 추가:
    - `set_pathfind_backend_mode`
    - `get_pathfind_backend_mode`
    - `resolve_pathfind_backend_mode`
    - `pathfind_backend_dispatch_counts`
    - `reset_pathfind_backend_dispatch_counts`
    - `pathfind_grid_batch_dispatch_bytes`
    - `pathfind_grid_batch_xy_dispatch_bytes`
  - 신규 테스트 2개:
    - `public_backend_mode_helpers_roundtrip_and_validate`
      - `cpu`/`auto` 설정 roundtrip
      - invalid mode 거부 검증
    - `public_dispatch_counter_helpers_track_dispatch_paths`
      - 공개 dispatch API 호출 후 counter 증가 검증

## 기능 영향
- backend helper API의 동작이 테스트로 고정되어, 이후 GPU 실구현/리팩터링 시 외부 호출 계약 회귀를 빠르게 탐지 가능.
- runtime 동작/checksum 변화 없음.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과 (22 tests).
- `tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
