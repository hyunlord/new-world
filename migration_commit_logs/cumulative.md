# Rust Migration Commit Cumulative Log

## Commit 001
- Rust 워크스페이스 컴파일/테스트 안정화.
- `sim-systems`에 A* pathfinding 모듈 추가.
- `sim-bridge`에 pathfinding 브리지 API + Godot 클래스 노출(`WorldSimBridge`) 추가.
- `sim-bridge`를 `cdylib`로 빌드 가능하게 구성하고 `.gdextension` 파일 추가.
- 데이터 로더와 enum/serde 테스트 호환성 문제 수정.

## Commit 002
- `Pathfinder`에 Rust 우선 + GDScript fallback 경로탐색 분기 추가.
- `SimBridge` autoload shim 추가 및 `project.godot`에 등록.
- `Locale.ltr()`를 flat dictionary lookup으로 최적화하고 `tech` 카테고리 로딩 추가.

## Commit 003
- Rust bridge에 `pathfind_grid_batch` 계열 배치 API 추가.
- GDScript `Pathfinder`/`SimBridge`에 배치 프록시 경로 추가.
- `MovementSystem`이 재계산 엔티티를 모아 배치 호출하도록 최적화.

## Commit 004
- `ComputeBackend` autoload 추가(`cpu/gpu_auto/gpu_force`).
- `SimBridge`에 GPU 메서드 우선 탐색 경로 추가(없으면 CPU fallback).
- `project.godot` autoload에 `ComputeBackend` 등록.

## Commit 005
- Rust `WorldSimBridge`에 `pathfind_grid_gpu` / `pathfind_grid_gpu_batch` 메서드 추가.
- 현재 구현은 CPU 경로 재사용 fallback으로 안정 동작 보장.

## Commit 006
- `tools/localization_audit.py` 추가.
- en/ko parity, 중복 키, data 내 inline localized field 자동 감사 가능.

## Commit 007
- `sim-systems::stat_curve` 모듈 신규 추가.
- 기존 GDScript stat curve 수학 함수들을 Rust 순수 함수로 1차 이관.
- 단위 테스트 추가로 수학 로직 회귀 방지.
