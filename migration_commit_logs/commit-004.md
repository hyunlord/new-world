# Commit 004 - GPU 옵션 선택 구조(기반) 추가

## 커밋 요약
- CPU/GPU 실행 모드를 설정 가능한 `ComputeBackend` autoload를 추가.
- `SimBridge`가 런타임 모드에 따라 GPU 메서드를 우선 선택할 수 있는 경로를 추가.

## 상세 변경

### 1) ComputeBackend autoload 추가
- `scripts/core/simulation/compute_backend.gd` (신규)
  - 모드: `cpu`, `gpu_auto`, `gpu_force`
  - 저장/로드: `user://settings.json`의 `compute_mode`
  - `resolve_mode()`에서 런타임 최종 모드(`cpu`/`gpu`) 반환
  - `is_gpu_capable()`:
    - 렌더러 방식(compatibility 여부) 확인
    - `RenderingServer.get_rendering_device()` 존재 여부 확인

### 2) SimBridge GPU 우선 선택 경로
- `scripts/core/simulation/sim_bridge.gd`
  - 메서드 후보 확장:
    - 단건: `pathfind_grid_gpu` 우선, 없으면 기존 CPU 메서드
    - 배치: `pathfind_grid_gpu_batch` 우선, 없으면 기존 CPU 메서드
  - `ComputeBackend.resolve_mode()`가 `gpu`이면 GPU 후보 메서드 탐색
  - 실제 GPU 메서드가 없으면 CPU 메서드로 안전 fallback

### 3) Autoload 등록
- `project.godot`
  - `ComputeBackend` autoload 추가

## 기능 영향
- 현재는 GPU 메서드가 네이티브에 구현되지 않아도 문제 없이 CPU 경로 유지.
- 이후 Rust 측 GPU 메서드 추가 시 GDScript 변경 없이 자동 선택 경로 활용 가능.

## 검증
- Rust 회귀 검증: `cargo test -q` 통과
- Godot 런타임 실행 검증은 현재 환경의 `godot` 바이너리 부재로 미실행
