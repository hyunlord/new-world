# Commit 220 - Pathfinding backend 상태/해석 모듈 분리

## 커밋 요약
- `sim-bridge`의 pathfinding backend 상태 관리(모드 저장, 파싱, 해석)를 `pathfinding_backend.rs`로 분리해 GPU 구현 확장 지점을 명확히 정리.
- 기존 브리지 API(`set/get/resolve_pathfinding_backend`, dispatch 경로)는 동일 동작을 유지.

## 상세 변경
- 신규 파일: `rust/crates/sim-bridge/src/pathfinding_backend.rs`
  - 상수/상태:
    - `PATHFIND_BACKEND_AUTO`, `PATHFIND_BACKEND_CPU`, `PATHFIND_BACKEND_GPU`
    - 내부 `AtomicU8` 모드 저장소
  - API:
    - `set_backend_mode`, `get_backend_mode`
    - `parse_backend_mode`
    - `backend_mode_to_str`
    - `resolve_backend_mode_code`, `resolve_backend_mode_str`
    - `has_gpu_backend`
- `rust/crates/sim-bridge/src/lib.rs`
  - 기존에 `lib.rs` 내부에 있던 backend atomic/상수/해석 로직을 모듈 호출로 치환.
  - `WorldSimBridge` pathfinding backend 관련 함수가 모듈 API를 사용하도록 변경.
  - pathfinding 일반/배치 호출에서 backend 모드 load 경로를 모듈 getter로 통일.
  - 테스트 import를 새 모듈 경로로 정리.

## 기능 영향
- 런타임 동작/체크섬은 유지하면서, backend 제어 코드가 독립 모듈화되어 GPU 실구현 시 교체 범위를 축소.
- `lib.rs`의 결합도를 낮춰 유지보수성과 확장성(backend 추가, 실험 모드 도입) 개선.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과.
- `MIGRATION_AUDIT_COMPARE_KEY_OWNER_POLICY=localization/key_owners.json tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
