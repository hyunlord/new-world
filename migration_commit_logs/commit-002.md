# Commit 002 - Godot 런타임 연동 + Locale 조회 최적화

## 커밋 요약
- Godot에서 Rust pathfinding을 우선 호출하고, 실패 시 기존 GDScript 경로탐색으로 즉시 fallback 되도록 연결.
- `SimBridge` autoload shim을 추가해 네이티브 브리지 존재 여부를 런타임에 안전하게 탐색.
- Locale 조회를 카테고리 순회 방식에서 flat lookup 방식으로 개선(O(1)).

## 상세 변경

### 1) Rust pathfinding 호출 경로 연결
- `scripts/core/world/pathfinder.gd`
  - 기존 `find_path()`에 Rust 호출 분기 추가.
  - `_find_path_rust()` 추가:
    - `SimBridge.pathfind_grid(...)` 호출.
    - bridge 부재/메서드 부재/반환 null 시 자동 fallback.
  - `_find_path_gd()`로 기존 A* 구현 분리(기존 로직 보존).
  - world grid를 `PackedByteArray`(walkable), `PackedFloat32Array`(move_cost)로 캐시해 bridge 입력을 생성.
  - Rust 반환 타입(`PackedVector2Array`, `Array[Vector2i|Vector2|Dictionary]`) 정규화 로직 추가.

### 2) SimBridge autoload shim 추가
- `scripts/core/simulation/sim_bridge.gd` (신규)
  - `pathfind_grid(...)` 프록시 메서드 제공.
  - 우선순위 탐색:
    1. Engine singleton(`WorldSimBridge`, `SimBridgeNative`, `RustBridge`)
    2. ClassDB 인스턴스 생성 가능 클래스
  - 사용 가능한 메서드명(`pathfind_grid`, `pathfind`) 탐색.
  - 브리지 미발견 시 null 반환하여 상위 호출부 fallback 유도.

- `project.godot`
  - `[autoload]`에 `SimBridge` 등록.

### 3) Locale 성능 최적화
- `scripts/core/simulation/locale.gd`
  - `_flat_strings` 캐시 딕셔너리 도입.
  - `load_locale()` 시 카테고리 로드와 동시에 flat index 구축.
  - `ltr()`를 카테고리 순회에서 단일 dict 조회로 변경.
  - 누락되어 있던 `tech` 카테고리를 `_categories`에 포함.

## 기능 영향
- Rust 브리지 로딩 시: pathfinding이 Rust 경로를 사용.
- Rust 브리지 미로딩 시: 기존 GDScript A*를 그대로 사용(동작 유지).
- Locale 조회 빈도가 높은 UI에서 문자열 조회 오버헤드 감소.

## 검증
- Rust side 회귀 확인: `cargo test -q` 통과
- Godot 런타임 실행 검증은 현재 환경에서 `godot` 바이너리 부재로 미실행
