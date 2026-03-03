# Commit 226 - Non-Godot backend 모드 공개 API + sim-test `--backend` 옵션 추가

## 커밋 요약
- `sim-bridge`에 non-Godot 경로에서 backend 모드를 제어/조회할 수 있는 공개 API를 추가.
- `sim-test` pathfinding 벤치에 `--backend auto|cpu|gpu` 옵션을 연결해 모드별 관측을 가능하게 확장.

## 상세 변경
- `rust/crates/sim-bridge/src/lib.rs`
  - 공개 helper 추가:
    - `set_pathfind_backend_mode(mode: &str) -> bool`
    - `get_pathfind_backend_mode() -> &'static str`
    - `resolve_pathfind_backend_mode() -> &'static str`
  - 기존 `WorldSimBridge`의 `set/get/resolve_pathfinding_backend`가 위 helper를 재사용하도록 정리.
- `rust/crates/sim-test/src/main.rs`
  - `--backend` 인자 파서 추가(`parse_pathfind_backend_arg`).
  - pathfinding 벤치 실행 전 `set_pathfind_backend_mode` 적용.
  - 잘못된 값 입력 시 명시적 에러 후 종료 코드 `2` 반환.
  - backend 출력에 `configured`/`resolved` 정보를 포함.

## 기능 영향
- Godot 엔진 외부(headless/CI 벤치)에서도 backend 모드 제어가 가능해져 CPU/GPU/auto 비교 실험이 쉬워짐.
- 기본 검증(`migration_verify`)은 기존과 동일하게 `auto -> cpu` 경로에서 checksum 유지.

## 검증
- `cd rust && cargo test -p sim-bridge --release` 통과 (20 tests).
- `cd rust && cargo run -q -p sim-test --release -- --bench-pathfind-bridge --iters 10 --backend cpu` 확인.
  - checksum `7080.00000`
  - backend `configured=cpu resolved=cpu`
- `tools/migration_verify.sh --with-benches` 통과.
  - pathfind checksum: `70800.00000` (@100)
  - stress checksum: `24032652.00000` (@10000)
  - needs checksum: `38457848.00000` (@10000)
