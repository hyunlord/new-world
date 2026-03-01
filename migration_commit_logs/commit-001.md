# Commit 001 - Rust 기반 안정화 + Pathfinding Rust 이관 기반

## 커밋 요약
- Rust 워크스페이스가 `cargo check/test`로 안정 동작하도록 기존 깨진 포인트를 수정.
- `sim-systems`에 A* 경로탐색 모듈을 추가.
- `sim-bridge`에 경로탐색 입력/검증/호출 경계 로직과 Godot GDExtension 클래스 초안을 추가.
- Rust GDExtension 산출물(`cdylib`)과 `.gdextension` 로더 파일을 연결 가능 상태로 구성.

## 상세 변경

### 1) Rust 빌드/테스트 안정화
- `rust/Cargo.toml`
  - `rand`에 `small_rng` feature 추가.
  - workspace dependency에 `godot = "0.4.5"` 추가.
- `rust/Cargo.lock`
  - 실제 의존성 잠금 파일 생성/추가.
- `rust/crates/sim-core/src/world/mod.rs`
  - 중복 import 제거로 컴파일 에러 해소.
- `rust/crates/sim-core/src/components/values.rs`
  - `[f64; 33]` 직렬화 한계를 우회하기 위해 `Values`에 수동 `Serialize/Deserialize` 구현.
- `rust/crates/sim-core/src/enums.rs`
  - `GrowthStage` 경계 테스트 기대값을 구현 경계와 일치하도록 조정.
- `rust/crates/sim-core/Cargo.toml`
  - `sim-core` 테스트에서 사용하는 `serde_json`를 dev-dependency에 추가.
- `rust/crates/sim-data/src/tech.rs`
  - 실제 데이터의 `prereq_logic.any_of` 형식(평면 배열/배열의 배열) 모두 처리하도록 `TechAnyOf`(untagged enum) 도입.
- `rust/crates/sim-engine/src/event_bus.rs`
  - 테스트용 미사용 import 제거.
- `rust/crates/sim-test/src/main.rs`
  - 미사용 import 제거.

### 2) Pathfinding Rust 모듈 추가
- `rust/crates/sim-systems/src/lib.rs`
  - `pathfinding` 모듈 export 추가.
- `rust/crates/sim-systems/src/pathfinding.rs` (신규)
  - 8방향 이동 + Chebyshev heuristic 기반 A* 구현.
  - `GridPos`, `GridCostMap` 타입 추가.
  - `find_path()` 구현(기존 GDScript 동작과 동일한 제약: target walkable 체크, max_steps 제한, from==to 처리).
  - 단위테스트 4개 추가(기본 경로, 차단 타일, 벽 우회, from==to).

### 3) Bridge 경계/노출 준비
- `rust/crates/sim-bridge/Cargo.toml`
  - `crate-type = ["cdylib", "rlib"]` 설정.
  - `godot` 의존성 활성화.
- `rust/crates/sim-bridge/src/lib.rs`
  - `PathfindInput`, `PathfindError`, `pathfind_from_flat()` 구현.
  - `pathfind_grid_bytes()` 추가(바이트 walkable + float cost 입력).
  - `WorldSimBridge` Godot 클래스(`#[class(base=Object, singleton)]`) 추가.
  - `#[func] pathfind_grid(...) -> PackedVector2Array` 메서드 노출.
  - `#[gdextension(entry_symbol = worldsim_rust_init)]` 확장 엔트리 추가.
  - 단위테스트 추가/보강.
- `rust/worldsim.gdextension` (신규)
  - 플랫폼별 라이브러리 경로와 엔트리 심볼 구성.

### 4) 저장소 위생
- `.gitignore`
  - `rust/target/` 무시 추가.

## 검증
- `cargo check -q` 통과
- `cargo test -q` 통과
- `cargo build -q -p sim-bridge` 통과
- `cargo build -q --release -p sim-bridge` 통과

## 산출물
- debug dylib: `rust/target/debug/libsim_bridge.dylib`
- release dylib: `rust/target/release/libsim_bridge.dylib`
